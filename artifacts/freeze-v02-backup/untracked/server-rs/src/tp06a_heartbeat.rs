//! TP-06A: synthetic, read-only WorldView heartbeat.
//!
//! This module deliberately owns no CampaignStore, input, physics, renderer,
//! save, or gameplay state. It only validates and sends a bounded synthetic
//! WorldView stream through one replaceable Tauri Channel.

use crate::world_v3::{
    Transform, Vec3, WorldStateV3, WorldView, WORLD_VIEW_PROTOCOL, WORLD_VIEW_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;

pub const TP06A_WORLD_ID: &str = "tp06a-synthetic";
pub const TP06A_DEFAULT_CADENCE: Duration = Duration::from_millis(33);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Tp06SessionId(pub u64);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tp06Error {
    AlreadyRunning,
    StaleSession,
    InvalidEpoch,
    InvalidView,
    ChannelSendFailed,
    Closing,
    RuntimeNotInitialized,
}

impl fmt::Display for Tp06Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::AlreadyRunning => "already_running",
            Self::StaleSession => "stale_session",
            Self::InvalidEpoch => "invalid_epoch",
            Self::InvalidView => "invalid_view",
            Self::ChannelSendFailed => "channel_send_failed",
            Self::Closing => "closing",
            Self::RuntimeNotInitialized => "runtime_not_initialized",
        };
        formatter.write_str(text)
    }
}

impl std::error::Error for Tp06Error {}

struct StopSignal {
    stopped: AtomicBool,
    wake: (Mutex<()>, Condvar),
}

impl StopSignal {
    fn new() -> Self {
        Self {
            stopped: AtomicBool::new(false),
            wake: (Mutex::new(()), Condvar::new()),
        }
    }

    fn request_stop(&self) {
        self.stopped.store(true, Ordering::Release);
        self.wake.1.notify_all();
    }

    fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }

    fn wait_for_next_tick(&self, cadence: Duration) -> bool {
        if self.is_stopped() {
            return true;
        }
        let guard = self.wake.0.lock().expect("tp06a stop signal poisoned");
        let _ = self
            .wake
            .1
            .wait_timeout(guard, cadence)
            .expect("tp06a stop signal poisoned");
        self.is_stopped()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SessionToken {
    id: Tp06SessionId,
    generation: u64,
    world_epoch: u64,
}

struct ActiveSession {
    token: SessionToken,
    stop: Arc<StopSignal>,
    join: Option<JoinHandle<()>>,
}

#[derive(Default)]
struct RuntimeState {
    closing: bool,
    replacing: bool,
    next_id: u64,
    next_generation: u64,
    active: Option<ActiveSession>,
}

struct RuntimeRegistry {
    state: Mutex<RuntimeState>,
    state_changed: Condvar,
}

pub type Tp06aTestPreSendHook = Arc<dyn Fn() + Send + Sync>;

pub struct Tp06aRuntime {
    registry: Arc<RuntimeRegistry>,
    pre_send_hook: Mutex<Option<Tp06aTestPreSendHook>>,
}

impl Default for Tp06aRuntime {
    fn default() -> Self {
        Self {
            registry: Arc::new(RuntimeRegistry {
                state: Mutex::new(RuntimeState {
                    next_id: 1,
                    next_generation: 1,
                    ..RuntimeState::default()
                }),
                state_changed: Condvar::new(),
            }),
            pre_send_hook: Mutex::new(None),
        }
    }
}

impl Tp06aRuntime {
    pub fn start(
        &self,
        channel: Channel<WorldView>,
        world_epoch: u64,
    ) -> Result<Tp06SessionId, Tp06Error> {
        self.start_with_cadence(channel, world_epoch, TP06A_DEFAULT_CADENCE)
    }

    pub fn start_with_cadence(
        &self,
        channel: Channel<WorldView>,
        world_epoch: u64,
        cadence: Duration,
    ) -> Result<Tp06SessionId, Tp06Error> {
        if world_epoch == 0 || cadence.is_zero() {
            return Err(if world_epoch == 0 {
                Tp06Error::InvalidEpoch
            } else {
                Tp06Error::InvalidView
            });
        }
        let pre_send_hook = self
            .pre_send_hook
            .lock()
            .map_err(|_| Tp06Error::RuntimeNotInitialized)?
            .clone();

        let old = {
            let mut state = self
                .registry
                .state
                .lock()
                .map_err(|_| Tp06Error::RuntimeNotInitialized)?;
            if state.closing {
                return Err(Tp06Error::Closing);
            }
            if state.replacing {
                return Err(Tp06Error::AlreadyRunning);
            }
            state.replacing = true;
            state.active.take()
        };
        stop_and_join(old);

        let (token, stop, send_gate) = {
            let mut state = self
                .registry
                .state
                .lock()
                .map_err(|_| Tp06Error::RuntimeNotInitialized)?;
            if state.closing {
                state.replacing = false;
                self.registry.state_changed.notify_all();
                return Err(Tp06Error::Closing);
            }
            let id = Tp06SessionId(state.next_id);
            state.next_id = state.next_id.saturating_add(1);
            let generation = state.next_generation;
            state.next_generation = state.next_generation.saturating_add(1);
            let token = SessionToken {
                id,
                generation,
                world_epoch,
            };
            let stop = Arc::new(StopSignal::new());
            let send_gate = Arc::new(Mutex::new(()));
            state.active = Some(ActiveSession {
                token,
                stop: Arc::clone(&stop),
                join: None,
            });
            (token, stop, send_gate)
        };

        let first = match synthetic_view(world_epoch, 0, 0) {
            Ok(view) => view,
            Err(error) => {
                self.clear_starting_session(token);
                return Err(error);
            }
        };
        if let Err(error) = send_for_token(
            &self.registry,
            token,
            &stop,
            &send_gate,
            &pre_send_hook,
            &channel,
            first,
        ) {
            self.clear_starting_session(token);
            return Err(error);
        }

        let worker_stop = Arc::clone(&stop);
        let worker_registry = Arc::clone(&self.registry);
        let worker_send_gate = Arc::clone(&send_gate);
        let worker = thread::Builder::new()
            .name("tp06a-heartbeat".to_owned())
            .spawn(move || {
                heartbeat_loop(
                    channel,
                    token,
                    cadence,
                    worker_stop,
                    worker_registry,
                    worker_send_gate,
                    pre_send_hook,
                )
            })
            .map_err(|_| {
                self.clear_starting_session(token);
                Tp06Error::RuntimeNotInitialized
            })?;

        let mut state = self
            .registry
            .state
            .lock()
            .map_err(|_| Tp06Error::RuntimeNotInitialized)?;
        let Some(active) = state.active.as_mut() else {
            state.replacing = false;
            self.registry.state_changed.notify_all();
            drop(state);
            stop_and_join(Some(ActiveSession {
                token,
                stop,
                join: Some(worker),
            }));
            return Err(Tp06Error::StaleSession);
        };
        if active.token != token {
            state.replacing = false;
            self.registry.state_changed.notify_all();
            drop(state);
            stop_and_join(Some(ActiveSession {
                token,
                stop,
                join: Some(worker),
            }));
            return Err(Tp06Error::StaleSession);
        }
        active.join = Some(worker);
        state.replacing = false;
        self.registry.state_changed.notify_all();
        Ok(token.id)
    }

    pub fn stop(&self, session_id: Tp06SessionId) -> Result<(), Tp06Error> {
        let active = {
            let mut state = self
                .registry
                .state
                .lock()
                .map_err(|_| Tp06Error::RuntimeNotInitialized)?;
            if state.replacing {
                return Err(Tp06Error::AlreadyRunning);
            }
            match state.active.as_ref().map(|active| active.token.id) {
                Some(id) if id == session_id => state.active.take(),
                _ => return Err(Tp06Error::StaleSession),
            }
        };
        stop_and_join(active);
        Ok(())
    }

    pub fn close(&self) {
        let active = {
            let mut state = self.registry.state.lock().expect("tp06a runtime poisoned");
            state.closing = true;
            while state.replacing {
                state = self
                    .registry
                    .state_changed
                    .wait(state)
                    .expect("tp06a runtime poisoned");
            }
            state.active.take()
        };
        stop_and_join(active);
    }

    pub fn is_active(&self, session_id: Tp06SessionId) -> bool {
        self.registry
            .state
            .lock()
            .ok()
            .and_then(|state| {
                state
                    .active
                    .as_ref()
                    .map(|active| active.token.id == session_id)
            })
            .unwrap_or(false)
    }

    #[doc(hidden)]
    pub fn set_test_pre_send_hook(&self, hook: Option<Tp06aTestPreSendHook>) {
        if let Ok(mut current) = self.pre_send_hook.lock() {
            *current = hook;
        }
    }

    fn clear_starting_session(&self, token: SessionToken) {
        if let Ok(mut state) = self.registry.state.lock() {
            if state
                .active
                .as_ref()
                .is_some_and(|active| active.token == token)
            {
                state.active = None;
            }
            state.replacing = false;
            self.registry.state_changed.notify_all();
        }
    }
}

impl Drop for Tp06aRuntime {
    fn drop(&mut self) {
        self.close();
    }
}

fn stop_and_join(active: Option<ActiveSession>) {
    let Some(mut active) = active else { return };
    active.stop.request_stop();
    if let Some(join) = active.join.take() {
        let _ = join.join();
    }
}

fn heartbeat_loop(
    channel: Channel<WorldView>,
    token: SessionToken,
    cadence: Duration,
    stop: Arc<StopSignal>,
    registry: Arc<RuntimeRegistry>,
    send_gate: Arc<Mutex<()>>,
    pre_send_hook: Option<Tp06aTestPreSendHook>,
) {
    let started = Instant::now();
    let mut tick = 1_u64;
    loop {
        if stop.is_stopped() {
            return;
        }
        let server_time_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        match synthetic_view(token.world_epoch, tick, server_time_ms).and_then(|view| {
            send_for_token(
                &registry,
                token,
                &stop,
                &send_gate,
                &pre_send_hook,
                &channel,
                view,
            )
        }) {
            Ok(()) => tick = tick.saturating_add(1),
            Err(Tp06Error::StaleSession) => return,
            Err(error) => {
                eprintln!("[tp06a] heartbeat stopped: {error}");
                return;
            }
        }
        if stop.wait_for_next_tick(cadence) {
            return;
        }
    }
}

fn send_for_token(
    registry: &Arc<RuntimeRegistry>,
    token: SessionToken,
    stop: &Arc<StopSignal>,
    send_gate: &Arc<Mutex<()>>,
    pre_send_hook: &Option<Tp06aTestPreSendHook>,
    channel: &Channel<WorldView>,
    view: WorldView,
) -> Result<(), Tp06Error> {
    if let Some(hook) = pre_send_hook {
        hook();
    }
    let _send_guard = send_gate
        .lock()
        .map_err(|_| Tp06Error::RuntimeNotInitialized)?;
    if stop.is_stopped() || !token_is_current(registry, token) {
        return Err(Tp06Error::StaleSession);
    }
    send_view(channel, view, token.world_epoch)
}

fn token_is_current(registry: &Arc<RuntimeRegistry>, token: SessionToken) -> bool {
    registry.state.lock().ok().is_some_and(|state| {
        !state.closing
            && state
                .active
                .as_ref()
                .is_some_and(|active| active.token == token)
    })
}

fn synthetic_view(
    world_epoch: u64,
    tick: u64,
    server_time_ms: u64,
) -> Result<WorldView, Tp06Error> {
    let transform = Transform::new(Vec3::zero(), 0.0).map_err(|_| Tp06Error::InvalidView)?;
    let velocity = Vec3::zero();
    let state = WorldStateV3::new(
        TP06A_WORLD_ID,
        world_epoch,
        tick,
        server_time_ms,
        transform,
        velocity,
        Vec::new(),
        Vec::new(),
    )
    .map_err(|_| Tp06Error::InvalidView)?;
    let view = state.view(0);
    validate_view(&view, world_epoch)?;
    Ok(view)
}

pub fn validate_view(view: &WorldView, expected_epoch: u64) -> Result<(), Tp06Error> {
    if view.protocol() != WORLD_VIEW_PROTOCOL
        || view.version() != WORLD_VIEW_VERSION
        || view.world_epoch() != expected_epoch
        || view.world_id() != TP06A_WORLD_ID
        || view.ack_seq() != 0
    {
        return Err(Tp06Error::InvalidView);
    }
    let encoded = serde_json::to_value(view).map_err(|_| Tp06Error::InvalidView)?;
    if !all_numbers_finite(&encoded) {
        return Err(Tp06Error::InvalidView);
    }
    Ok(())
}

pub fn send_view(
    channel: &Channel<WorldView>,
    view: WorldView,
    expected_epoch: u64,
) -> Result<(), Tp06Error> {
    validate_view(&view, expected_epoch)?;
    channel.send(view).map_err(|_| Tp06Error::ChannelSendFailed)
}

fn all_numbers_finite(value: &Value) -> bool {
    match value {
        Value::Array(values) => values.iter().all(all_numbers_finite),
        Value::Object(values) => values.values().all(all_numbers_finite),
        Value::Number(number) => number.as_f64().is_some_and(f64::is_finite),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn capture_channel() -> (Channel<WorldView>, Arc<Mutex<Vec<WorldView>>>) {
        let received = Arc::new(Mutex::new(Vec::new()));
        let clone = Arc::clone(&received);
        let channel = Channel::new(move |body| {
            clone.lock().unwrap().push(body.deserialize().unwrap());
            Ok(())
        });
        (channel, received)
    }

    #[test]
    fn synthetic_view_has_frozen_protocol_epoch_tick_and_ack() {
        let view = synthetic_view(7, 3, 99).unwrap();
        assert_eq!(view.protocol(), WORLD_VIEW_PROTOCOL);
        assert_eq!(view.version(), WORLD_VIEW_VERSION);
        assert_eq!(view.world_epoch(), 7);
        assert_eq!(view.server_tick(), 3);
        assert_eq!(view.ack_seq(), 0);
        assert_eq!(view.world_id(), TP06A_WORLD_ID);
        validate_view(&view, 7).unwrap();
        assert!(validate_view(&view, 8).is_err());
    }

    #[test]
    fn start_stop_and_replacement_reject_stale_generation() {
        let runtime = Tp06aRuntime::default();
        let (channel_a, received_a) = capture_channel();
        let session_a = runtime
            .start_with_cadence(channel_a, 7, Duration::from_millis(2))
            .unwrap();
        let (channel_b, received_b) = capture_channel();
        let session_b = runtime
            .start_with_cadence(channel_b, 8, Duration::from_millis(2))
            .unwrap();
        assert_ne!(session_a, session_b);
        assert!(!runtime.is_active(session_a));
        assert!(runtime.is_active(session_b));
        assert!(runtime.stop(session_a).is_err());
        assert!(received_a
            .lock()
            .unwrap()
            .iter()
            .all(|view| view.world_epoch() == 7));
        assert!(received_b
            .lock()
            .unwrap()
            .iter()
            .all(|view| view.world_epoch() == 8));
        runtime.stop(session_b).unwrap();
        assert!(!runtime.is_active(session_b));
    }

    #[test]
    fn channel_error_is_mapped_and_close_is_idempotent() {
        let runtime = Tp06aRuntime::default();
        let channel = Channel::<WorldView>::new(|_| Err(tauri::Error::FailedToReceiveMessage));
        assert_eq!(
            runtime.start_with_cadence(channel, 7, Duration::from_millis(1)),
            Err(Tp06Error::ChannelSendFailed)
        );
        runtime.close();
        runtime.close();
        let (channel, _) = capture_channel();
        assert_eq!(runtime.start(channel, 7), Err(Tp06Error::Closing));
    }

    #[test]
    fn validation_rejects_invalid_epoch_and_zero_cadence() {
        let runtime = Tp06aRuntime::default();
        let (channel, _) = capture_channel();
        assert_eq!(runtime.start(channel, 0), Err(Tp06Error::InvalidEpoch));
        let (channel, _) = capture_channel();
        assert_eq!(
            runtime.start_with_cadence(channel, 7, Duration::ZERO),
            Err(Tp06Error::InvalidView)
        );
    }

    #[test]
    fn callback_send_is_not_inside_runtime_state_lock() {
        let runtime = Arc::new(Tp06aRuntime::default());
        let callback_count = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&callback_count);
        let runtime_for_callback = Arc::clone(&runtime);
        let channel = Channel::new(move |body| {
            let _: WorldView = body.deserialize().unwrap();
            count.fetch_add(1, Ordering::Relaxed);
            let _ = runtime_for_callback.is_active(Tp06SessionId(999));
            Ok(())
        });
        let session = runtime
            .start_with_cadence(channel, 7, Duration::from_millis(1))
            .unwrap();
        runtime.stop(session).unwrap();
        assert!(callback_count.load(Ordering::Relaxed) >= 1);
    }
}
