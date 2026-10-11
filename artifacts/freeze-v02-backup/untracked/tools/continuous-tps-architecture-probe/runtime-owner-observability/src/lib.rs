use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use wuxian_horror_ch1::fixed_step::FixedStepScheduler;

const EVIDENCE: &str = "isolated_candidate_tp17b1";
const WORLD_EPOCH: u64 = 7;
const QUEUE_CAPACITY: usize = 8;
const PAYLOAD_LIMIT: usize = 64;
const SOAK_TICKS: u64 = 36_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueError {
    Closed,
    StaleEpoch { expected: u64, actual: u64 },
    Full { capacity: usize },
    PayloadTooLarge { limit: usize, actual: usize },
    DuplicateSequence(u64),
    EmptyEventId,
}

impl QueueError {
    fn code(&self) -> &'static str {
        match self {
            Self::Closed => "E_QUEUE_CLOSED",
            Self::StaleEpoch { .. } => "E_QUEUE_STALE_EPOCH",
            Self::Full { .. } => "E_QUEUE_FULL_REJECT_NEWEST",
            Self::PayloadTooLarge { .. } => "E_QUEUE_PAYLOAD_TOO_LARGE",
            Self::DuplicateSequence(_) => "E_QUEUE_DUPLICATE_SEQUENCE",
            Self::EmptyEventId => "E_QUEUE_EMPTY_EVENT_ID",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueItem<T> {
    pub epoch: u64,
    pub tick: u64,
    pub event_id: String,
    pub enqueue_sequence: u64,
    pub payload_bytes: usize,
    pub payload: T,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueMetrics {
    pub length: usize,
    pub capacity: usize,
    pub payload_limit: usize,
    pub high_water: usize,
    pub dropped: u64,
    pub rejected: u64,
    pub overflow_rejected: u64,
    pub oversize_rejected: u64,
    pub stale_rejected: u64,
}

#[derive(Clone, Debug)]
pub struct BoundedEventQueue<T> {
    epoch: u64,
    capacity: usize,
    payload_limit: usize,
    closed: bool,
    items: VecDeque<QueueItem<T>>,
    sequences: BTreeSet<u64>,
    metrics: QueueMetrics,
}

impl<T> BoundedEventQueue<T> {
    pub fn new(epoch: u64, capacity: usize, payload_limit: usize) -> Self {
        Self {
            epoch,
            capacity,
            payload_limit,
            closed: false,
            items: VecDeque::new(),
            sequences: BTreeSet::new(),
            metrics: QueueMetrics {
                length: 0,
                capacity,
                payload_limit,
                high_water: 0,
                dropped: 0,
                rejected: 0,
                overflow_rejected: 0,
                oversize_rejected: 0,
                stale_rejected: 0,
            },
        }
    }

    pub fn try_enqueue(&mut self, item: QueueItem<T>) -> Result<(), QueueError> {
        if self.closed {
            self.metrics.rejected += 1;
            return Err(QueueError::Closed);
        }
        if item.epoch != self.epoch {
            self.metrics.rejected += 1;
            self.metrics.stale_rejected += 1;
            return Err(QueueError::StaleEpoch {
                expected: self.epoch,
                actual: item.epoch,
            });
        }
        if item.event_id.is_empty() {
            self.metrics.rejected += 1;
            return Err(QueueError::EmptyEventId);
        }
        if item.payload_bytes > self.payload_limit {
            self.metrics.rejected += 1;
            self.metrics.oversize_rejected += 1;
            return Err(QueueError::PayloadTooLarge {
                limit: self.payload_limit,
                actual: item.payload_bytes,
            });
        }
        if !self.sequences.insert(item.enqueue_sequence) {
            self.metrics.rejected += 1;
            return Err(QueueError::DuplicateSequence(item.enqueue_sequence));
        }
        if self.items.len() >= self.capacity {
            self.sequences.remove(&item.enqueue_sequence);
            self.metrics.rejected += 1;
            self.metrics.overflow_rejected += 1;
            return Err(QueueError::Full {
                capacity: self.capacity,
            });
        }
        self.items.push_back(item);
        self.metrics.length = self.items.len();
        self.metrics.high_water = self.metrics.high_water.max(self.items.len());
        Ok(())
    }

    pub fn drain(&mut self) -> Vec<QueueItem<T>> {
        let mut drained: Vec<_> = self.items.drain(..).collect();
        drained.sort_by(|left, right| {
            (left.epoch, left.tick, &left.event_id, left.enqueue_sequence).cmp(&(
                right.epoch,
                right.tick,
                &right.event_id,
                right.enqueue_sequence,
            ))
        });
        self.sequences.clear();
        self.metrics.length = 0;
        drained
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn replace_epoch(&mut self, epoch: u64) {
        self.metrics.dropped += self.items.len() as u64;
        self.items.clear();
        self.sequences.clear();
        self.metrics.length = 0;
        self.epoch = epoch;
        self.closed = false;
    }

    pub fn metrics(&self) -> QueueMetrics {
        self.metrics.clone()
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

pub struct SharedBoundedEventQueue<T> {
    inner: Arc<Mutex<BoundedEventQueue<T>>>,
}

impl<T> Clone for SharedBoundedEventQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> SharedBoundedEventQueue<T> {
    pub fn new(epoch: u64, capacity: usize, payload_limit: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BoundedEventQueue::new(
                epoch,
                capacity,
                payload_limit,
            ))),
        }
    }

    pub fn try_enqueue(&self, item: QueueItem<T>) -> Result<(), QueueError> {
        self.inner.lock().expect("queue mutex").try_enqueue(item)
    }

    pub fn close(&self) {
        self.inner.lock().expect("queue mutex").close();
    }

    pub fn drain_with<F>(&self, callback: F)
    where
        F: FnMut(QueueItem<T>),
    {
        // Drain under the mutex, release it, then invoke callbacks. Reentry can
        // therefore enqueue into the same shared queue without lock recursion.
        let items = {
            let mut guard = self.inner.lock().expect("queue mutex");
            guard.drain()
        };
        drop_reentry_guard(self, items, callback);
    }

    pub fn metrics(&self) -> QueueMetrics {
        self.inner.lock().expect("queue mutex").metrics()
    }
    pub fn replace_epoch(&self, epoch: u64) {
        self.inner.lock().expect("queue mutex").replace_epoch(epoch);
    }
}

fn drop_reentry_guard<T, F>(
    queue: &SharedBoundedEventQueue<T>,
    items: Vec<QueueItem<T>>,
    mut callback: F,
) where
    F: FnMut(QueueItem<T>),
{
    let _ = queue;
    for item in items {
        callback(item);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    Direct,
    WrapperOwned,
    Unobservable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observed<T> {
    pub value: Option<T>,
    pub provenance: Provenance,
}

impl<T> Observed<T> {
    fn direct(value: T) -> Self {
        Self {
            value: Some(value),
            provenance: Provenance::Direct,
        }
    }
    fn owned(value: T) -> Self {
        Self {
            value: Some(value),
            provenance: Provenance::WrapperOwned,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeOwnerSnapshot {
    pub owner_generation: Observed<u64>,
    pub world_epoch: Observed<u64>,
    pub running: Observed<bool>,
    pub world_count: Observed<u64>,
    pub entity_count: Observed<u64>,
    pub queue_count: Observed<u64>,
    pub candidate_owner_registered_count: BTreeMap<String, Observed<u64>>,
    pub processed_event_entries: Observed<u64>,
    pub reward_entries: Observed<u64>,
    pub cache_entries: Observed<u64>,
    pub queue_length: Observed<usize>,
    pub queue_capacity: Observed<usize>,
    pub queue_high_water: Observed<usize>,
    pub queue_dropped: Observed<u64>,
    pub queue_rejected: Observed<u64>,
    pub created_count: Observed<u64>,
    pub disposed_count: Observed<u64>,
    pub dependency_internal_counters: BTreeMap<String, Provenance>,
}

#[derive(Clone, Debug)]
pub struct RuntimeOwner {
    generation: u64,
    epoch: u64,
    running: bool,
    world_count: u64,
    entity_count: u64,
    queue_count: u64,
    registered: BTreeMap<String, u64>,
    processed: BTreeSet<String>,
    rewards: BTreeSet<String>,
    caches: BTreeSet<String>,
    created: u64,
    disposed: u64,
    queue: BoundedEventQueue<String>,
}

impl RuntimeOwner {
    pub fn new(epoch: u64, capacity: usize, payload_limit: usize) -> Self {
        Self {
            generation: 0,
            epoch,
            running: false,
            world_count: 0,
            entity_count: 0,
            queue_count: 0,
            registered: BTreeMap::new(),
            processed: BTreeSet::new(),
            rewards: BTreeSet::new(),
            caches: BTreeSet::new(),
            created: 0,
            disposed: 0,
            queue: BoundedEventQueue::new(epoch, capacity, payload_limit),
        }
    }

    pub fn enter_world(&mut self, epoch: u64, entities: u64) -> Result<(), &'static str> {
        if self.running {
            return Err("E_OWNER_ALREADY_RUNNING");
        }
        self.generation += 1;
        self.epoch = epoch;
        self.running = true;
        self.world_count = 1;
        self.entity_count = entities;
        self.queue_count = 1;
        self.registered = [
            "fixed_step",
            "world_v3",
            "kcc",
            "interaction",
            "nav",
            "combat",
            "campaign",
            "save",
        ]
        .into_iter()
        .map(|id| (id.to_string(), 1))
        .collect();
        self.processed.clear();
        self.rewards.clear();
        self.caches.clear();
        self.created += 1;
        self.queue.replace_epoch(epoch);
        Ok(())
    }

    pub fn enqueue(&mut self, item: QueueItem<String>) -> Result<(), QueueError> {
        self.queue.try_enqueue(item)
    }

    pub fn drain_process(&mut self) -> Vec<String> {
        let items = self.queue.drain();
        let mut ids = Vec::new();
        for item in items {
            self.processed.insert(item.event_id.clone());
            ids.push(item.event_id);
        }
        ids
    }

    pub fn add_reward(&mut self, reward_id: &str) {
        self.rewards.insert(reward_id.to_string());
    }
    pub fn add_cache(&mut self, cache_id: &str) {
        self.caches.insert(cache_id.to_string());
    }

    pub fn exit_world(&mut self) -> Result<(), &'static str> {
        if !self.running {
            return Err("E_OWNER_NOT_RUNNING");
        }
        self.queue.drain();
        self.queue.close();
        self.running = false;
        self.world_count = 0;
        self.entity_count = 0;
        self.queue_count = 0;
        self.registered.clear();
        self.processed.clear();
        self.rewards.clear();
        self.caches.clear();
        self.disposed += 1;
        Ok(())
    }

    pub fn snapshot(&self) -> RuntimeOwnerSnapshot {
        let metrics = self.queue.metrics();
        let mut dependency_internal_counters = BTreeMap::new();
        for owner in [
            "interaction_processed_events",
            "combat_processed_events",
            "nav_cache_entries",
            "formal_world_entities",
            "formal_kcc_colliders",
        ] {
            dependency_internal_counters.insert(owner.to_string(), Provenance::Unobservable);
        }
        RuntimeOwnerSnapshot {
            owner_generation: Observed::owned(self.generation),
            world_epoch: Observed::direct(self.epoch),
            running: Observed::owned(self.running),
            world_count: Observed::owned(self.world_count),
            entity_count: Observed::owned(self.entity_count),
            queue_count: Observed::owned(self.queue_count),
            candidate_owner_registered_count: self
                .registered
                .iter()
                .map(|(k, v)| (k.clone(), Observed::owned(*v)))
                .collect(),
            processed_event_entries: Observed::owned(self.processed.len() as u64),
            reward_entries: Observed::owned(self.rewards.len() as u64),
            cache_entries: Observed::owned(self.caches.len() as u64),
            queue_length: Observed::owned(metrics.length),
            queue_capacity: Observed::owned(metrics.capacity),
            queue_high_water: Observed::owned(metrics.high_water),
            queue_dropped: Observed::owned(metrics.dropped),
            queue_rejected: Observed::owned(metrics.rejected),
            created_count: Observed::owned(self.created),
            disposed_count: Observed::owned(self.disposed),
            dependency_internal_counters,
        }
    }
}

fn item(epoch: u64, tick: u64, id: &str, sequence: u64, bytes: usize) -> QueueItem<String> {
    QueueItem {
        epoch,
        tick,
        event_id: id.to_string(),
        enqueue_sequence: sequence,
        payload_bytes: bytes,
        payload: id.to_string(),
    }
}

fn scenario(
    id: &str,
    input: Value,
    events: Value,
    expected: Value,
    actual: Value,
    pass: bool,
    snapshot: Value,
    stable_error: Option<&str>,
) -> Value {
    json!({"id":id,"input":input,"events":events,"expected":expected,"actual":actual,"pass":pass,"snapshot":snapshot,"queueMetrics":snapshot.get("queue_length"),"stableError":stable_error,"evidenceLevel":EVIDENCE})
}

fn test_scenarios() -> Vec<Value> {
    let mut scenarios = Vec::new();
    let empty: BoundedEventQueue<String> = BoundedEventQueue::new(WORLD_EPOCH, 2, PAYLOAD_LIMIT);
    let empty_metrics = empty.metrics();
    scenarios.push(scenario(
        "empty_queue",
        json!({"capacity":2}),
        json!([]),
        json!({"length":0,"highWater":0}),
        json!(empty_metrics),
        empty_metrics.length == 0 && empty_metrics.high_water == 0,
        json!(empty_metrics),
        None,
    ));

    let mut full = BoundedEventQueue::new(WORLD_EPOCH, 2, PAYLOAD_LIMIT);
    let first = full.try_enqueue(item(WORLD_EPOCH, 1, "a", 1, 4));
    let second = full.try_enqueue(item(WORLD_EPOCH, 1, "b", 2, 4));
    let full_metrics = full.metrics();
    scenarios.push(scenario(
        "full_queue",
        json!({"capacity":2}),
        json!(["a", "b"]),
        json!({"length":2,"bothAccepted":true}),
        json!({"first":first,"second":second,"metrics":full_metrics}),
        first.is_ok() && second.is_ok() && full_metrics.length == 2,
        json!(full_metrics),
        None,
    ));

    let overflow = full.try_enqueue(item(WORLD_EPOCH, 1, "c", 3, 4));
    let overflow_metrics = full.metrics();
    scenarios.push(scenario(
        "overflow_reject_newest",
        json!({"capacity":2,"policy":"reject_newest"}),
        json!(["c"]),
        json!({"error":"E_QUEUE_FULL_REJECT_NEWEST","length":2}),
        json!({"error":overflow.as_ref().err().map(QueueError::code),"metrics":overflow_metrics}),
        overflow
            .as_ref()
            .err()
            .is_some_and(|e| e.code() == "E_QUEUE_FULL_REJECT_NEWEST")
            && overflow_metrics.length == 2,
        json!(overflow_metrics),
        Some("E_QUEUE_FULL_REJECT_NEWEST"),
    ));

    let oversize = full.try_enqueue(item(WORLD_EPOCH, 1, "oversize", 4, PAYLOAD_LIMIT + 1));
    scenarios.push(scenario(
        "oversize_payload_rejected",
        json!({"payloadLimit":PAYLOAD_LIMIT,"bytes":PAYLOAD_LIMIT+1}),
        json!(["oversize"]),
        json!({"error":"E_QUEUE_PAYLOAD_TOO_LARGE"}),
        json!({"error":oversize.as_ref().err().map(QueueError::code)}),
        oversize
            .as_ref()
            .err()
            .is_some_and(|e| e.code() == "E_QUEUE_PAYLOAD_TOO_LARGE"),
        json!(full.metrics()),
        Some("E_QUEUE_PAYLOAD_TOO_LARGE"),
    ));

    let mut ordered = BoundedEventQueue::new(WORLD_EPOCH, 8, PAYLOAD_LIMIT);
    for (id, tick, seq) in [("z", 2, 3), ("b", 1, 2), ("a", 1, 1)] {
        ordered
            .try_enqueue(item(WORLD_EPOCH, tick, id, seq, 4))
            .unwrap();
    }
    let ids: Vec<String> = ordered
        .drain()
        .into_iter()
        .map(|entry| entry.event_id)
        .collect();
    scenarios.push(scenario(
        "fifo_tick_event_tie_break",
        json!({"orderKey":["epoch","tick","eventId","enqueueSequence"]}),
        json!(ids.clone()),
        json!({"order":["a","b","z"]}),
        json!({"order":ids}),
        ids == vec!["a", "b", "z"],
        json!(ordered.metrics()),
        None,
    ));

    let mut duplicate_sequence = BoundedEventQueue::new(WORLD_EPOCH, 8, PAYLOAD_LIMIT);
    duplicate_sequence
        .try_enqueue(item(WORLD_EPOCH, 0, "a", 9, 4))
        .unwrap();
    let duplicate = duplicate_sequence.try_enqueue(item(WORLD_EPOCH, 0, "b", 9, 4));
    scenarios.push(scenario(
        "duplicate_enqueue_sequence",
        json!({"sequence":9}),
        json!(["a", "b"]),
        json!({"error":"E_QUEUE_DUPLICATE_SEQUENCE"}),
        json!({"error":duplicate.as_ref().err().map(QueueError::code)}),
        duplicate
            .as_ref()
            .err()
            .is_some_and(|e| e.code() == "E_QUEUE_DUPLICATE_SEQUENCE"),
        json!(duplicate_sequence.metrics()),
        Some("E_QUEUE_DUPLICATE_SEQUENCE"),
    ));

    let mut stale = BoundedEventQueue::new(WORLD_EPOCH, 8, PAYLOAD_LIMIT);
    let stale_result = stale.try_enqueue(item(WORLD_EPOCH - 1, 0, "old", 1, 4));
    scenarios.push(scenario(
        "stale_epoch_rejected",
        json!({"queueEpoch":WORLD_EPOCH,"eventEpoch":WORLD_EPOCH-1}),
        json!(["old"]),
        json!({"error":"E_QUEUE_STALE_EPOCH"}),
        json!({"error":stale_result.as_ref().err().map(QueueError::code)}),
        stale_result
            .as_ref()
            .err()
            .is_some_and(|e| e.code() == "E_QUEUE_STALE_EPOCH"),
        json!(stale.metrics()),
        Some("E_QUEUE_STALE_EPOCH"),
    ));

    let mut closed = BoundedEventQueue::new(WORLD_EPOCH, 8, PAYLOAD_LIMIT);
    closed.close();
    let closed_result = closed.try_enqueue(item(WORLD_EPOCH, 0, "closed", 1, 4));
    scenarios.push(scenario("close_rejects_enqueue", json!({"closed":true}), json!(["closed"]), json!({"error":"E_QUEUE_CLOSED"}), json!({"error":closed_result.as_ref().err().map(QueueError::code),"closed":closed.is_closed()}), closed_result.as_ref().err().is_some_and(|e| e.code()=="E_QUEUE_CLOSED"), json!(closed.metrics()), Some("E_QUEUE_CLOSED")));

    let mut high_water = BoundedEventQueue::new(WORLD_EPOCH, 4, PAYLOAD_LIMIT);
    for seq in 0..4 {
        high_water
            .try_enqueue(item(WORLD_EPOCH, 0, &format!("h{seq}"), seq, 4))
            .unwrap();
    }
    let _ = high_water.drain();
    let high_metrics = high_water.metrics();
    scenarios.push(scenario(
        "drain_retains_high_water",
        json!({"capacity":4}),
        json!(["h0", "h1", "h2", "h3"]),
        json!({"lengthAfterDrain":0,"highWater":4}),
        json!(high_metrics),
        high_metrics.length == 0 && high_metrics.high_water == 4,
        json!(high_metrics),
        None,
    ));

    let mut replacement = BoundedEventQueue::new(WORLD_EPOCH, 4, PAYLOAD_LIMIT);
    replacement
        .try_enqueue(item(WORLD_EPOCH, 0, "old", 1, 4))
        .unwrap();
    replacement.replace_epoch(WORLD_EPOCH + 1);
    let replacement_metrics = replacement.metrics();
    let fresh = replacement.try_enqueue(item(WORLD_EPOCH + 1, 1, "new", 2, 4));
    scenarios.push(scenario("epoch_replacement_clears_old_events", json!({"oldEpoch":WORLD_EPOCH,"newEpoch":WORLD_EPOCH+1}), json!(["old","new"]), json!({"oldDropped":1,"newAccepted":true}), json!({"epoch":replacement.epoch(),"metrics":replacement_metrics,"newAccepted":fresh.is_ok()}), replacement.epoch()==WORLD_EPOCH+1 && replacement_metrics.length==0 && replacement_metrics.dropped==1 && fresh.is_ok(), json!(replacement_metrics), None));

    let shared = SharedBoundedEventQueue::new(WORLD_EPOCH, 4, PAYLOAD_LIMIT);
    shared
        .try_enqueue(item(WORLD_EPOCH, 0, "reentry", 1, 4))
        .unwrap();
    let reentry_queue = shared.clone();
    let mut callback_ids = Vec::new();
    shared.drain_with(|entry| {
        callback_ids.push(entry.event_id);
        reentry_queue
            .try_enqueue(item(WORLD_EPOCH, 1, "reentered", 2, 4))
            .unwrap();
    });
    let reentry_metrics = shared.metrics();
    scenarios.push(scenario(
        "callback_reentry_outside_lock",
        json!({"callbackEnqueues":true}),
        json!(callback_ids.clone()),
        json!({"callbackIds":["reentry"],"reenteredLength":1}),
        json!({"callbackIds":callback_ids,"metrics":reentry_metrics}),
        callback_ids == vec!["reentry"] && reentry_metrics.length == 1,
        json!(reentry_metrics),
        None,
    ));

    let concurrent = SharedBoundedEventQueue::new(WORLD_EPOCH, 16, PAYLOAD_LIMIT);
    let mut handles = Vec::new();
    for producer in 0..4_u64 {
        let queue = concurrent.clone();
        handles.push(thread::spawn(move || {
            let seq = producer;
            queue
                .try_enqueue(item(WORLD_EPOCH, 1, &format!("p{producer}"), seq, 4))
                .unwrap();
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    let mut concurrent_drain = Vec::new();
    concurrent.drain_with(|entry| concurrent_drain.push(entry.event_id));
    scenarios.push(scenario(
        "concurrent_producers_deterministic_sequence",
        json!({"producers":4,"explicitSequences":[0,1,2,3]}),
        json!(concurrent_drain.clone()),
        json!({"order":["p0","p1","p2","p3"]}),
        json!({"order":concurrent_drain}),
        concurrent_drain == vec!["p0", "p1", "p2", "p3"],
        json!(concurrent.metrics()),
        None,
    ));

    scenarios
}

fn lifecycle_scenario() -> Value {
    let mut owner = RuntimeOwner::new(WORLD_EPOCH, QUEUE_CAPACITY, PAYLOAD_LIMIT);
    let mut baseline: Option<RuntimeOwnerSnapshot> = None;
    let mut all_equal = true;
    for cycle in 0..30_u64 {
        owner.enter_world(WORLD_EPOCH + cycle, 1).unwrap();
        owner
            .enqueue(item(
                WORLD_EPOCH + cycle,
                cycle,
                &format!("cycle-{cycle}"),
                cycle,
                4,
            ))
            .unwrap();
        owner.add_reward(&format!("reward-{cycle}"));
        owner.add_cache(&format!("cache-{cycle}"));
        owner.drain_process();
        owner.exit_world().unwrap();
        let snapshot = owner.snapshot();
        if let Some(base) = &baseline {
            all_equal &= snapshot.running == base.running
                && snapshot.world_count == base.world_count
                && snapshot.entity_count == base.entity_count
                && snapshot.queue_count == base.queue_count
                && snapshot.candidate_owner_registered_count
                    == base.candidate_owner_registered_count
                && snapshot.processed_event_entries == base.processed_event_entries
                && snapshot.reward_entries == base.reward_entries
                && snapshot.cache_entries == base.cache_entries
                && snapshot.queue_length == base.queue_length
                && snapshot.queue_capacity == base.queue_capacity
                && snapshot.queue_high_water == base.queue_high_water
                && snapshot.queue_dropped == base.queue_dropped
                && snapshot.queue_rejected == base.queue_rejected;
        } else {
            baseline = Some(snapshot.clone());
        }
    }
    let final_snapshot = owner.snapshot();
    let pass = all_equal
        && final_snapshot.running.value == Some(false)
        && final_snapshot.world_count.value == Some(0)
        && final_snapshot.entity_count.value == Some(0)
        && final_snapshot.queue_count.value == Some(0)
        && final_snapshot.processed_event_entries.value == Some(0)
        && final_snapshot.reward_entries.value == Some(0)
        && final_snapshot.cache_entries.value == Some(0)
        && final_snapshot.created_count.value == Some(30)
        && final_snapshot.disposed_count.value == Some(30);
    json!({"id":"thirty_enter_exit_wrapper_baseline","input":{"cycles":30,"capacity":QUEUE_CAPACITY},"events":["enter","enqueue","drain","exit"],"expected":{"wrapperOwnedBaseline":true,"created":30,"disposed":30},"actual":{"pass":pass,"snapshot":final_snapshot,"allCyclesEqual":all_equal},"pass":pass,"evidenceLevel":EVIDENCE})
}

fn soak_scenario() -> Value {
    let started = Instant::now();
    let mut scheduler = FixedStepScheduler::with_frequency(60, 0).unwrap();
    let dt = scheduler.config().fixed_dt_nanos();
    let mut owner = RuntimeOwner::new(WORLD_EPOCH, 64, PAYLOAD_LIMIT);
    owner.enter_world(WORLD_EPOCH, 1).unwrap();
    let before = owner.snapshot();
    let mut samples = Vec::with_capacity(SOAK_TICKS as usize);
    let mut peak = before.clone();
    for tick in 0..SOAK_TICKS {
        let tick_started = Instant::now();
        scheduler.advance((tick + 1) * dt).unwrap();
        owner
            .enqueue(item(WORLD_EPOCH, tick, &format!("soak-{tick}"), tick, 4))
            .unwrap();
        owner.add_cache(&format!("cache-{tick}"));
        if tick % 1_000 == 0 {
            owner.add_reward(&format!("reward-{tick}"));
        }
        owner.drain_process();
        let snapshot = owner.snapshot();
        if snapshot.processed_event_entries.value > peak.processed_event_entries.value {
            peak = snapshot;
        }
        samples.push(tick_started.elapsed().as_nanos());
    }
    owner.exit_world().unwrap();
    let after = owner.snapshot();
    samples.sort_unstable();
    let percentile =
        |fraction: f64| samples[((samples.len() as f64 - 1.0) * fraction).round() as usize];
    let pass = scheduler.total_ticks() == SOAK_TICKS
        && after.world_count.value == Some(0)
        && after.entity_count.value == Some(0)
        && after.queue_count.value == Some(0)
        && after.processed_event_entries.value == Some(0)
        && after.reward_entries.value == Some(0)
        && after.cache_entries.value == Some(0);
    json!({"id":"bounded_36000_tick_owner_queue_soak","input":{"ticks":SOAK_TICKS,"capacity":64,"payloadLimit":PAYLOAD_LIMIT},"events":["enqueue","drain","owner_process"],"expected":{"ticks":SOAK_TICKS,"afterBaseline":true},"actual":{"ticks":scheduler.total_ticks(),"before":before,"peak":peak,"after":after,"queue":after.queue_length,"wallTimeMs":started.elapsed().as_secs_f64()*1000.0,"tickLatencyNs":{"count":samples.len(),"p50":percentile(0.50),"p95":percentile(0.95),"p99":percentile(0.99),"max":*samples.last().unwrap()},"memory":{"rss":"unknown","allocator":"unknown","evidenceLevel":"unavailable_public_api"}},"pass":pass,"evidenceLevel":EVIDENCE})
}

pub fn run_probe() -> Value {
    let mut scenarios = test_scenarios();
    scenarios.push(lifecycle_scenario());
    scenarios.push(soak_scenario());
    let pass_count = scenarios
        .iter()
        .filter(|scenario| scenario["pass"] == true)
        .count();
    let all_pass = pass_count == scenarios.len();
    json!({
        "schema":"runtime_owner_observability_tp17b1_v1",
        "status":if all_pass {"passed_candidate_isolated"} else {"blocked_public_api_boundary"},
        "pass":all_pass,
        "candidateOnly":true,
        "formalSceneReady":false,
        "snapshotContract":{
            "fields":["ownerGeneration","worldEpoch","running","worldCount","entityCount","queueCount","candidateOwnerRegisteredCount","processedEventEntries","rewardEntries","cacheEntries","queueLength","queueCapacity","queueHighWater","queueDropped","queueRejected","createdCount","disposedCount"],
            "provenance":["direct","wrapperOwned","unobservable"],
            "dependencyInternalCounters":"unobservable, never reported as zero"
        },
        "queueContract":{
            "capacity":QUEUE_CAPACITY,
            "payloadByteLimit":PAYLOAD_LIMIT,
            "stableEnqueueSequence":true,
            "order":["epoch","tick","eventId","enqueueSequence"],
            "overflowPolicy":"reject_newest",
            "drain":"ordered FIFO with same-tick event tie-break",
            "close":"reject enqueue, drain remains allowed",
            "replacement":"epoch replacement clears old events and increments dropped",
            "callback":"callbacks run after drain mutex release",
            "concurrency":"explicit sequence keys make producer order deterministic"
        },
        "scenarios":scenarios,
        "passCount":pass_count,
        "blockedInterfaces":[
            {"errorCode":"E_UNOBSERVABLE_DEPENDENCY_COUNTER","missing":"TP-17A1 dependency-private ledgers/caches and formal internal owner counters","provenance":"unobservable","policy":"never report zero or infer cleanup"},
            {"errorCode":"E_FORMAL_RUNTIME_NOT_BOUND","missing":"Tauri/WebView/Babylon and production owner wiring","provenance":"unobservable"},
            {"errorCode":"E_MEMORY_METRICS_UNAVAILABLE","missing":"RSS and allocator instrumentation","provenance":"unobservable"}
        ],
        "upgradeWhitelist":[
            "server-rs/src/runtime_owner_observability.rs (future; explicit owner snapshot API)",
            "server-rs/src/bounded_event_queue.rs (future; bounded queue API)",
            "tools/continuous-tps-architecture-probe/runtime-integration-soak (future adapter update only after public APIs exist)",
            "docs/CONTINUOUS_RUNTIME_OWNER_OBSERVABILITY_REVIEW.md"
        ],
        "formalGaps":["formal owner wiring absent","dependency-private counters remain unobservable","RSS/allocator/GPU metrics absent","TP-17 and related Gates remain blocked; Architecture no_go"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_contract_scenarios_pass() {
        let scenarios = test_scenarios();
        assert!(scenarios.iter().all(|scenario| scenario["pass"] == true));
        assert!(scenarios.len() >= 12);
    }

    #[test]
    fn lifecycle_returns_wrapper_owned_baseline() {
        assert_eq!(lifecycle_scenario()["pass"], true);
    }

    #[test]
    fn soak_reaches_exact_tick_budget_and_unknown_memory_is_explicit() {
        let result = soak_scenario();
        assert_eq!(result["pass"], true);
        assert_eq!(result["actual"]["ticks"], SOAK_TICKS);
        assert_eq!(result["actual"]["memory"]["rss"], "unknown");
    }
}
