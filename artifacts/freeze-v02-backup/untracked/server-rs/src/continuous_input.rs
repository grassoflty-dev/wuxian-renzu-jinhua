//! Versioned, bounded input-frame receiver for the future continuous runtime.
//!
//! This module is an in-memory protocol component only. It does not open a
//! Tauri channel, advance WorldStateV3, call KCC, or implement a snapshot
//! receiver. A caller supplies receipt time and consumes `InputSample` values
//! alongside a fixed-step tick plan.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::fmt;

pub const INPUT_PROTOCOL: &str = "continuous-ipc";
pub const INPUT_VERSION: u32 = 1;
pub const MAX_INPUT_FRAME_BYTES: usize = 1_024;
pub const MAX_EDGE_EVENTS_PER_FRAME: usize = 8;
pub const MAX_PENDING_FRAMES: usize = 32;
pub const MAX_QUEUED_EDGE_EVENTS: usize = MAX_PENDING_FRAMES * MAX_EDGE_EVENTS_PER_FRAME;
pub const HELD_EXPIRY_NS: u64 = 100_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeldAxes {
    x: f32,
    z: f32,
}

impl HeldAxes {
    pub fn new(x: f32, z: f32) -> Result<Self, InputFrameError> {
        if !x.is_finite() || !z.is_finite() {
            return Err(InputFrameError::NonFiniteAxis);
        }
        if !(-1.0..=1.0).contains(&x) || !(-1.0..=1.0).contains(&z) {
            return Err(InputFrameError::AxisOutOfRange);
        }
        Ok(Self { x, z })
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, z: 0.0 }
    }

    pub const fn x(&self) -> f32 {
        self.x
    }

    pub const fn z(&self) -> f32 {
        self.z
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputButtons {
    jump: bool,
    dash: bool,
    interact: bool,
}

impl InputButtons {
    pub const fn new(jump: bool, dash: bool, interact: bool) -> Self {
        Self {
            jump,
            dash,
            interact,
        }
    }

    pub const fn jump(&self) -> bool {
        self.jump
    }

    pub const fn dash(&self) -> bool {
        self.dash
    }

    pub const fn interact(&self) -> bool {
        self.interact
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Jump,
    Dash,
    Interact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeEvent {
    id: u64,
    kind: EdgeKind,
}

impl EdgeEvent {
    pub fn new(id: u64, kind: EdgeKind) -> Result<Self, InputFrameError> {
        if id == 0 {
            return Err(InputFrameError::InvalidEdgeId);
        }
        Ok(Self { id, kind })
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn kind(&self) -> EdgeKind {
        self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFrame {
    protocol: String,
    version: u32,
    world_epoch: u64,
    seq: u64,
    client_time_ms: u64,
    held_axes: HeldAxes,
    buttons: InputButtons,
    edge_events: Vec<EdgeEvent>,
}

impl InputFrame {
    pub fn new(
        world_epoch: u64,
        seq: u64,
        client_time_ms: u64,
        held_axes: HeldAxes,
        buttons: InputButtons,
        edge_events: Vec<EdgeEvent>,
    ) -> Result<Self, InputFrameError> {
        let frame = Self {
            protocol: INPUT_PROTOCOL.to_owned(),
            version: INPUT_VERSION,
            world_epoch,
            seq,
            client_time_ms,
            held_axes,
            buttons,
            edge_events,
        };
        frame.validate_shape()?;
        Ok(frame)
    }

    pub fn from_json(payload: &[u8]) -> Result<Self, InputDecodeError> {
        if payload.len() > MAX_INPUT_FRAME_BYTES {
            return Err(InputDecodeError::PayloadTooLarge);
        }
        let frame: Self =
            serde_json::from_slice(payload).map_err(|_| InputDecodeError::InvalidJson)?;
        frame
            .validate_shape()
            .map_err(InputDecodeError::InvalidFrame)?;
        Ok(frame)
    }

    fn validate_shape(&self) -> Result<(), InputFrameError> {
        if self.seq == 0 {
            return Err(InputFrameError::InvalidSequence);
        }
        HeldAxes::new(self.held_axes.x, self.held_axes.z)?;
        if self.edge_events.len() > MAX_EDGE_EVENTS_PER_FRAME {
            return Err(InputFrameError::TooManyEdges);
        }
        let mut previous = 0;
        for event in &self.edge_events {
            if event.id == 0 {
                return Err(InputFrameError::InvalidEdgeId);
            }
            if event.id <= previous {
                return Err(InputFrameError::NonMonotonicEdgeIds);
            }
            previous = event.id;
        }
        Ok(())
    }

    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn world_epoch(&self) -> u64 {
        self.world_epoch
    }

    pub const fn seq(&self) -> u64 {
        self.seq
    }

    pub const fn client_time_ms(&self) -> u64 {
        self.client_time_ms
    }

    pub const fn held_axes(&self) -> HeldAxes {
        self.held_axes
    }

    pub const fn buttons(&self) -> InputButtons {
        self.buttons
    }

    pub fn edge_events(&self) -> &[EdgeEvent] {
        &self.edge_events
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputFrameError {
    NonFiniteAxis,
    AxisOutOfRange,
    InvalidSequence,
    TooManyEdges,
    InvalidEdgeId,
    NonMonotonicEdgeIds,
}

impl fmt::Display for InputFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NonFiniteAxis => "held axis must be finite",
            Self::AxisOutOfRange => "held axis must be within [-1, 1]",
            Self::InvalidSequence => "sequence must be positive",
            Self::TooManyEdges => "too many edge events",
            Self::InvalidEdgeId => "edge id must be positive",
            Self::NonMonotonicEdgeIds => "edge ids must be strictly increasing",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InputFrameError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputDecodeError {
    PayloadTooLarge,
    InvalidJson,
    InvalidFrame(InputFrameError),
}

impl fmt::Display for InputDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge => formatter.write_str("input frame exceeds 1024 UTF-8 bytes"),
            Self::InvalidJson => formatter.write_str("input frame is not valid JSON"),
            Self::InvalidFrame(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for InputDecodeError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropReason {
    PayloadTooLarge,
    InvalidPayload,
    WrongProtocol,
    WrongVersion,
    WrongEpoch,
    Paused,
    ClockRegression,
    DuplicateOrOldSequence,
    GapOverflow,
    SequenceOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiveOutcome {
    Applied { applied_frames: usize, ack_seq: u64 },
    Buffered { pending_frames: usize, ack_seq: u64 },
    Dropped { reason: DropReason, ack_seq: u64 },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputCounters {
    accepted_frames: u64,
    applied_frames: u64,
    buffered_frames: u64,
    dropped_frames: u64,
    dropped_edges: u64,
    duplicate_edges: u64,
    protocol_rejections: u64,
    version_rejections: u64,
    epoch_rejections: u64,
    paused_rejections: u64,
    clock_rejections: u64,
    sequence_rejections: u64,
    gap_overflow_rejections: u64,
}

impl InputCounters {
    pub const fn accepted_frames(&self) -> u64 {
        self.accepted_frames
    }

    pub const fn applied_frames(&self) -> u64 {
        self.applied_frames
    }

    pub const fn buffered_frames(&self) -> u64 {
        self.buffered_frames
    }

    pub const fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }

    pub const fn dropped_edges(&self) -> u64 {
        self.dropped_edges
    }

    pub const fn duplicate_edges(&self) -> u64 {
        self.duplicate_edges
    }

    pub const fn protocol_rejections(&self) -> u64 {
        self.protocol_rejections
    }

    pub const fn version_rejections(&self) -> u64 {
        self.version_rejections
    }

    pub const fn epoch_rejections(&self) -> u64 {
        self.epoch_rejections
    }

    pub const fn paused_rejections(&self) -> u64 {
        self.paused_rejections
    }

    pub const fn clock_rejections(&self) -> u64 {
        self.clock_rejections
    }

    pub const fn sequence_rejections(&self) -> u64 {
        self.sequence_rejections
    }

    pub const fn gap_overflow_rejections(&self) -> u64 {
        self.gap_overflow_rejections
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputSample {
    tick_index: u64,
    held_axes: HeldAxes,
    buttons: InputButtons,
    edge_events: Vec<EdgeEvent>,
}

impl InputSample {
    pub const fn tick_index(&self) -> u64 {
        self.tick_index
    }

    pub const fn held_axes(&self) -> HeldAxes {
        self.held_axes
    }

    pub const fn buttons(&self) -> InputButtons {
        self.buttons
    }

    pub fn edge_events(&self) -> &[EdgeEvent] {
        &self.edge_events
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsumeError {
    ReceiptClockRegression,
    TickRegression,
}

impl fmt::Display for ConsumeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ReceiptClockRegression => "receipt clock regressed",
            Self::TickRegression => "tick index regressed or repeated",
        })
    }
}

impl std::error::Error for ConsumeError {}

struct PendingFrame {
    frame: InputFrame,
}

pub struct InputReceiver {
    world_epoch: u64,
    next_seq: Option<u64>,
    ack_seq: u64,
    pending: BTreeMap<u64, PendingFrame>,
    held_axes: HeldAxes,
    buttons: InputButtons,
    held_receipt_ns: Option<u64>,
    edge_queue: VecDeque<EdgeEvent>,
    edge_watermark: Option<u64>,
    last_receipt_ns: Option<u64>,
    last_tick_index: Option<u64>,
    paused: bool,
    resume_barrier: bool,
    counters: InputCounters,
}

impl InputReceiver {
    pub fn new(world_epoch: u64) -> Self {
        Self {
            world_epoch,
            next_seq: Some(1),
            ack_seq: 0,
            pending: BTreeMap::new(),
            held_axes: HeldAxes::zero(),
            buttons: InputButtons::default(),
            held_receipt_ns: None,
            edge_queue: VecDeque::new(),
            edge_watermark: None,
            last_receipt_ns: None,
            last_tick_index: None,
            paused: false,
            resume_barrier: false,
            counters: InputCounters::default(),
        }
    }

    pub const fn world_epoch(&self) -> u64 {
        self.world_epoch
    }

    pub const fn ack_seq(&self) -> u64 {
        self.ack_seq
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub const fn held_axes(&self) -> HeldAxes {
        self.held_axes
    }

    pub const fn buttons(&self) -> InputButtons {
        self.buttons
    }

    pub fn queued_edge_len(&self) -> usize {
        self.edge_queue.len()
    }

    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    pub const fn counters(&self) -> InputCounters {
        self.counters
    }

    fn increment(value: &mut u64) {
        *value = value.saturating_add(1);
    }

    fn dropped(&mut self, reason: DropReason) -> ReceiveOutcome {
        Self::increment(&mut self.counters.dropped_frames);
        match reason {
            DropReason::WrongProtocol => Self::increment(&mut self.counters.protocol_rejections),
            DropReason::WrongVersion => Self::increment(&mut self.counters.version_rejections),
            DropReason::WrongEpoch => Self::increment(&mut self.counters.epoch_rejections),
            DropReason::Paused => Self::increment(&mut self.counters.paused_rejections),
            DropReason::ClockRegression => Self::increment(&mut self.counters.clock_rejections),
            DropReason::DuplicateOrOldSequence | DropReason::SequenceOverflow => {
                Self::increment(&mut self.counters.sequence_rejections)
            }
            DropReason::GapOverflow => Self::increment(&mut self.counters.gap_overflow_rejections),
            DropReason::PayloadTooLarge | DropReason::InvalidPayload => {}
        }
        ReceiveOutcome::Dropped {
            reason,
            ack_seq: self.ack_seq,
        }
    }

    pub fn receive_json(&mut self, payload: &[u8], receipt_ns: u64) -> ReceiveOutcome {
        if payload.len() > MAX_INPUT_FRAME_BYTES {
            return self.dropped(DropReason::PayloadTooLarge);
        }
        let frame = match InputFrame::from_json(payload) {
            Ok(frame) => frame,
            Err(InputDecodeError::PayloadTooLarge) => {
                return self.dropped(DropReason::PayloadTooLarge)
            }
            Err(InputDecodeError::InvalidJson | InputDecodeError::InvalidFrame(_)) => {
                return self.dropped(DropReason::InvalidPayload)
            }
        };
        self.receive(frame, receipt_ns)
    }

    pub fn receive(&mut self, frame: InputFrame, receipt_ns: u64) -> ReceiveOutcome {
        if self
            .last_receipt_ns
            .is_some_and(|previous| receipt_ns < previous)
        {
            return self.dropped(DropReason::ClockRegression);
        }
        if self.paused {
            return self.dropped(DropReason::Paused);
        }
        if frame.protocol != INPUT_PROTOCOL {
            return self.dropped(DropReason::WrongProtocol);
        }
        if frame.version != INPUT_VERSION {
            return self.dropped(DropReason::WrongVersion);
        }
        if frame.world_epoch != self.world_epoch {
            return self.dropped(DropReason::WrongEpoch);
        }
        self.last_receipt_ns = Some(receipt_ns);

        if self.resume_barrier {
            self.next_seq = Some(frame.seq);
            self.ack_seq = frame.seq.saturating_sub(1);
            self.resume_barrier = false;
        }
        let Some(next_seq) = self.next_seq else {
            return self.dropped(DropReason::SequenceOverflow);
        };
        if frame.seq < next_seq || self.pending.contains_key(&frame.seq) {
            return self.dropped(DropReason::DuplicateOrOldSequence);
        }
        if frame.seq > next_seq {
            if self.pending.len() >= MAX_PENDING_FRAMES {
                return self.dropped(DropReason::GapOverflow);
            }
            Self::increment(&mut self.counters.accepted_frames);
            self.pending.insert(frame.seq, PendingFrame { frame });
            Self::increment(&mut self.counters.buffered_frames);
            return ReceiveOutcome::Buffered {
                pending_frames: self.pending.len(),
                ack_seq: self.ack_seq,
            };
        }

        Self::increment(&mut self.counters.accepted_frames);
        let mut applied_frames = 0;
        self.apply_frame(frame);
        applied_frames += 1;
        while let Some(expected) = self.next_seq {
            let Some(pending) = self.pending.remove(&expected) else {
                break;
            };
            self.apply_frame(pending.frame);
            applied_frames += 1;
        }
        self.counters.applied_frames = self
            .counters
            .applied_frames
            .saturating_add(applied_frames as u64);
        ReceiveOutcome::Applied {
            applied_frames,
            ack_seq: self.ack_seq,
        }
    }

    fn apply_frame(&mut self, frame: InputFrame) {
        self.held_axes = frame.held_axes;
        self.buttons = frame.buttons;
        self.held_receipt_ns = self.last_receipt_ns;
        for edge in frame.edge_events {
            if self
                .edge_watermark
                .is_some_and(|watermark| edge.id <= watermark)
            {
                Self::increment(&mut self.counters.duplicate_edges);
                continue;
            }
            if self.edge_queue.len() >= MAX_QUEUED_EDGE_EVENTS {
                Self::increment(&mut self.counters.dropped_edges);
                self.edge_watermark = Some(edge.id);
                continue;
            }
            self.edge_watermark = Some(edge.id);
            self.edge_queue.push_back(edge);
        }
        self.ack_seq = frame.seq;
        self.next_seq = frame.seq.checked_add(1);
    }

    /// Clears input and creates a pause barrier. Input delivered while paused
    /// is dropped and cannot be replayed after resume.
    pub fn pause(&mut self) {
        self.paused = true;
        self.resume_barrier = true;
        self.pending.clear();
        self.edge_queue.clear();
        self.held_axes = HeldAxes::zero();
        self.buttons = InputButtons::default();
        self.held_receipt_ns = None;
    }

    /// Hidden/blurred windows use the same barrier policy as an explicit pause.
    pub fn hidden(&mut self) {
        self.pause();
    }

    /// Reopens input. The first accepted post-barrier sequence becomes the
    /// new contiguous baseline, so stale pre-pause gaps are not replayed.
    pub fn resume(&mut self) {
        self.paused = false;
        self.resume_barrier = true;
    }

    /// Switches epoch and resets sequence, held, edge, and receipt baselines.
    pub fn set_world_epoch(&mut self, world_epoch: u64) {
        self.world_epoch = world_epoch;
        self.next_seq = Some(1);
        self.ack_seq = 0;
        self.pending.clear();
        self.held_axes = HeldAxes::zero();
        self.buttons = InputButtons::default();
        self.held_receipt_ns = None;
        self.edge_queue.clear();
        self.edge_watermark = None;
        self.last_receipt_ns = None;
        self.last_tick_index = None;
        self.paused = false;
        self.resume_barrier = false;
    }

    pub fn consume_for_tick(
        &mut self,
        tick_index: u64,
        receipt_ns: u64,
    ) -> Result<InputSample, ConsumeError> {
        if self
            .last_receipt_ns
            .is_some_and(|previous| receipt_ns < previous)
        {
            return Err(ConsumeError::ReceiptClockRegression);
        }
        if self
            .last_tick_index
            .is_some_and(|previous| tick_index <= previous)
        {
            return Err(ConsumeError::TickRegression);
        }
        self.last_receipt_ns = Some(receipt_ns);
        self.last_tick_index = Some(tick_index);
        let expired = self
            .held_receipt_ns
            .is_none_or(|held_receipt| receipt_ns.saturating_sub(held_receipt) > HELD_EXPIRY_NS);
        let (held_axes, buttons) = if expired {
            (HeldAxes::zero(), InputButtons::default())
        } else {
            (self.held_axes, self.buttons)
        };
        let edge_events = self.edge_queue.drain(..).collect();
        Ok(InputSample {
            tick_index,
            held_axes,
            buttons,
            edge_events,
        })
    }

    pub fn consume_tick_plan(
        &mut self,
        plan: &crate::fixed_step::TickPlan,
        receipt_ns: u64,
    ) -> Result<Vec<InputSample>, ConsumeError> {
        let mut samples = Vec::with_capacity(plan.tick_count());
        for &tick_index in plan.tick_indices() {
            samples.push(self.consume_for_tick(tick_index, receipt_ns)?);
        }
        Ok(samples)
    }
}
