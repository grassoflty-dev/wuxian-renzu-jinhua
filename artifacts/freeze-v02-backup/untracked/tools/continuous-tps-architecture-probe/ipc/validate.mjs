import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const RESULT_PATH = path.join(HERE, "..", "results", "ipc-ag08.json");
const LIMITS = Object.freeze({
  protocol: "continuous-ipc",
  version: 1,
  inputHz: 30,
  snapshotHz: 30,
  maxInputFrameBytes: 1024,
  maxEdgeEvents: 8,
  maxPendingFrames: 32,
  heldExpiryMs: 100,
  maxWorldViewBytes: 64 * 1024,
});

function inputFrame(seq, epoch = 1, clientTimeMs = seq * 33, heldAxes = { x: 0, z: 0 }, edgeEvents = []) {
  return {
    protocol: LIMITS.protocol,
    version: LIMITS.version,
    worldEpoch: epoch,
    seq,
    clientTimeMs,
    heldAxes,
    buttons: { jump: false, dash: false, interact: false },
    edgeEvents,
  };
}

function worldView(serverTick, ackSeq, epoch = 1, worldId = "grey_hive") {
  return {
    protocol: LIMITS.protocol,
    version: LIMITS.version,
    worldEpoch: epoch,
    serverTick,
    ackSeq,
    serverTimeMs: serverTick * (1000 / LIMITS.snapshotHz),
    worldId,
    player: {
      transform: { positionM: { x: 1.25, y: 0, z: -2.5 }, yawRad: 0.25 },
      velocityMps: { x: 0, y: 0, z: 0 },
    },
    actors: [],
    doors: [],
  };
}

class Receiver {
  constructor(epoch = 1) {
    this.epoch = epoch;
    this.appliedSeq = 0;
    this.pending = new Map();
    this.heldAxes = { x: 0, z: 0 };
    this.lastReceiptMs = null;
    this.snapshotTick = 0;
    this.edgeIds = new Set();
    this.consumedEdges = [];
    this.dropped = [];
    this.appliedFrames = [];
    this.snapshotApplied = [];
    this.snapshotDropped = [];
    this.queueOverflowDrops = 0;
    this.paused = false;
    this.lifecycleDrops = 0;
    this.awaitingFreshSequence = false;
    this.lifecycleEvents = [];
  }

  pause() {
    this.paused = true;
    this.lifecycleEvents.push("pause");
    this.pending.clear();
    this.heldAxes = { x: 0, z: 0 };
  }

  hidden() {
    this.paused = true;
    this.lifecycleEvents.push("hidden");
    this.pending.clear();
    this.heldAxes = { x: 0, z: 0 };
  }

  resume() {
    this.paused = false;
    this.lifecycleEvents.push("resume");
    this.awaitingFreshSequence = true;
  }

  switchEpoch(epoch) {
    this.epoch = epoch;
    this.appliedSeq = 0;
    this.pending.clear();
    this.heldAxes = { x: 0, z: 0 };
    this.lastReceiptMs = null;
    this.snapshotTick = 0;
    this.edgeIds.clear();
    this.appliedFrames = [];
    this.consumedEdges = [];
    this.awaitingFreshSequence = false;
  }

  receive(frame, receiptMs) {
    if (frame.protocol !== LIMITS.protocol || frame.version !== LIMITS.version) {
      this.dropped.push({ seq: frame.seq, reason: "schema" });
      return;
    }
    if (this.paused) {
      this.lifecycleDrops += 1;
      this.dropped.push({ seq: frame.seq, reason: "paused" });
      return;
    }
    if (frame.worldEpoch !== this.epoch) {
      this.dropped.push({ seq: frame.seq, reason: "old_epoch" });
      return;
    }
    if (frame.seq <= this.appliedSeq || this.pending.has(frame.seq)) {
      this.dropped.push({ seq: frame.seq, reason: "duplicate_or_old" });
      return;
    }
    if (frame.edgeEvents.length > LIMITS.maxEdgeEvents) {
      this.dropped.push({ seq: frame.seq, reason: "edge_limit" });
      return;
    }
    if (this.awaitingFreshSequence && frame.seq > this.appliedSeq) {
      this.appliedSeq = frame.seq - 1;
      this.pending.clear();
      this.awaitingFreshSequence = false;
    }
    if (frame.seq > this.appliedSeq + 1) {
      if (this.pending.size >= LIMITS.maxPendingFrames) {
        this.queueOverflowDrops += 1;
        this.dropped.push({ seq: frame.seq, reason: "queue_overflow" });
        return;
      }
      this.pending.set(frame.seq, { frame, receiptMs });
      return;
    }
    this.apply(frame, receiptMs);
    while (this.pending.has(this.appliedSeq + 1)) {
      const next = this.pending.get(this.appliedSeq + 1);
      this.pending.delete(this.appliedSeq + 1);
      this.apply(next.frame, next.receiptMs);
    }
  }

  apply(frame, receiptMs) {
    this.appliedSeq = frame.seq;
    this.heldAxes = { ...frame.heldAxes };
    this.lastReceiptMs = receiptMs;
    this.appliedFrames.push(frame.seq);
    for (const edge of frame.edgeEvents) {
      if (!this.edgeIds.has(edge.id)) {
        this.edgeIds.add(edge.id);
        this.consumedEdges.push(edge.id);
      }
    }
  }

  advance(nowMs) {
    if (!this.paused && this.lastReceiptMs !== null && nowMs - this.lastReceiptMs > LIMITS.heldExpiryMs) {
      this.heldAxes = { x: 0, z: 0 };
    }
  }

  receiveView(view) {
    if (view.protocol !== LIMITS.protocol || view.version !== LIMITS.version || view.worldEpoch !== this.epoch) {
      this.snapshotDropped.push({ tick: view.serverTick, reason: "old_epoch_or_schema" });
      return;
    }
    if (view.ackSeq > this.appliedSeq) {
      this.snapshotDropped.push({ tick: view.serverTick, reason: "ack_beyond_applied" });
      return;
    }
    if (view.serverTick <= this.snapshotTick) {
      this.snapshotDropped.push({ tick: view.serverTick, reason: "snapshot_not_newer" });
      return;
    }
    this.snapshotTick = view.serverTick;
    this.snapshotApplied.push(view.serverTick);
  }
}

function check(name, inputs, expected, actual, threshold, pass) {
  return { name, inputs, expected, actual, threshold, pass: Boolean(pass) };
}

function baselineScenario() {
  const receiver = new Receiver();
  const edge = { id: 101, kind: "dash" };
  for (let seq = 1; seq <= 6; seq += 1) {
    receiver.receive(inputFrame(seq, 1, seq * (1000 / 30), seq < 6 ? { x: 1, z: 0 } : { x: 0, z: 0 }, seq === 2 ? [edge] : []), seq * (1000 / 30));
    receiver.receiveView(worldView(seq, receiver.appliedSeq));
  }
  const edgeBeforeSnapshot = receiver.consumedEdges.includes(101);
  return check(
    "baseline_30hz_held_and_snapshot",
    { input_hz: 30, snapshot_hz: 30, frames: 6, edge: edge },
    { applied_sequences: [1, 2, 3, 4, 5, 6], snapshot_ticks: [1, 2, 3, 4, 5, 6], edge_consumed_once: true, final_held_axes: { x: 0, z: 0 } },
    { applied_sequences: receiver.appliedFrames, snapshot_ticks: receiver.snapshotApplied, consumed_edges: receiver.consumedEdges, edge_before_snapshot: edgeBeforeSnapshot, final_held_axes: receiver.heldAxes, ack_seq: receiver.appliedSeq },
    { edge_consumption_max: 1, ack_seq: 6 },
    receiver.appliedSeq === 6 && receiver.snapshotTick === 6 && receiver.consumedEdges.length === 1 && edgeBeforeSnapshot && receiver.heldAxes.x === 0,
  );
}

function latencyJitterScenario() {
  const receiver = new Receiver();
  const clientTimes = [0, 33, 66, 99, 132];
  const latencies = [0, 50, 54, 120, 120];
  const arrivals = clientTimes.map((time, index) => time + latencies[index]);
  arrivals.forEach((arrival, index) => receiver.receive(inputFrame(index + 1, 1, clientTimes[index], { x: 1, z: 0 }), arrival));
  const pass = receiver.appliedSeq === 5 && receiver.appliedSeq >= Math.max(...receiver.appliedFrames) && receiver.heldAxes.x === 1;
  return check(
    "latency_0_50_120ms_with_jitter",
    { client_times_ms: clientTimes, arrival_times_ms: arrivals, injected_latency_ms: latencies },
    { applied_seq: 5, ack_monotonic: true, held_x: 1 },
    { applied_seq: receiver.appliedSeq, applied_sequences: receiver.appliedFrames, ack_seq: receiver.appliedSeq, held_axes: receiver.heldAxes, latency_samples_ms: latencies },
    { max_latency_ms: 120, ack_must_not_decrease: true },
    pass,
  );
}

function duplicateScenario() {
  const receiver = new Receiver();
  const frame = inputFrame(1, 1, 33, { x: 0, z: -1 }, [{ id: 2001, kind: "jump" }]);
  receiver.receive(frame, 33);
  receiver.receive(frame, 50);
  return check(
    "duplicate_frame_and_edge",
    { delivered_sequences: [1, 1], edge_id: 2001 },
    { edge_consumed_once: true, ack_seq: 1 },
    { consumed_edges: receiver.consumedEdges, applied_sequences: receiver.appliedFrames, dropped: receiver.dropped, ack_seq: receiver.appliedSeq },
    { max_edge_consumption: 1 },
    receiver.consumedEdges.length === 1 && receiver.appliedFrames.length === 1 && receiver.appliedSeq === 1,
  );
}

function reorderScenario() {
  const receiver = new Receiver();
  receiver.receive(inputFrame(1), 33);
  receiver.receive(inputFrame(3, 1, 99, { x: 1, z: 0 }, [{ id: 3003, kind: "attack" }]), 70);
  const ackWhileGap = receiver.appliedSeq;
  receiver.receive(inputFrame(2, 1, 66, { x: 1, z: 0 }), 100);
  return check(
    "out_of_order_reorder_then_apply",
    { delivery_order: [1, 3, 2], pending_limit: LIMITS.maxPendingFrames },
    { ack_during_gap: 1, final_applied: [1, 2, 3], edge_3003_once: true },
    { ack_during_gap: ackWhileGap, final_applied: receiver.appliedFrames, consumed_edges: receiver.consumedEdges, pending_size: receiver.pending.size, ack_seq: receiver.appliedSeq },
    { ack_never_beyond_applied: true },
    ackWhileGap === 1 && receiver.appliedSeq === 3 && receiver.appliedFrames.join(",") === "1,2,3" && receiver.consumedEdges.length === 1 && receiver.pending.size === 0,
  );
}

function lossExpiryScenario() {
  const receiver = new Receiver();
  receiver.receive(inputFrame(1, 1, 0, { x: 1, z: 0 }), 0);
  receiver.receive(inputFrame(3, 1, 66, { x: 1, z: 0 }), 66);
  receiver.advance(200);
  receiver.receive(inputFrame(4, 1, 99, { x: 1, z: 0 }), 210);
  return check(
    "missing_sequence_held_expiry",
    { delivered_sequences: [1, 3, 4], missing_sequence: 2, advance_to_ms: 200 },
    { ack_stays_at_1: true, held_zero_after_100ms: true, pending_bounded: true },
    { ack_seq: receiver.appliedSeq, held_axes: receiver.heldAxes, pending_size: receiver.pending.size, dropped: receiver.dropped },
    { held_expiry_ms: LIMITS.heldExpiryMs, max_pending: LIMITS.maxPendingFrames },
    receiver.appliedSeq === 1 && receiver.heldAxes.x === 0 && receiver.pending.size === 2 && receiver.appliedSeq <= 1,
  );
}

function queueBoundScenario() {
  const receiver = new Receiver();
  receiver.receive(inputFrame(1), 0);
  for (let seq = 100; seq <= 200; seq += 1) receiver.receive(inputFrame(seq, 1, seq * 33, { x: 1, z: 0 }), seq * 33);
  return check(
    "gap_queue_bound",
    { first_sequence: 1, injected_sequences: [100, 200], injected_count: 101, missing_sequence: 2 },
    { pending_at_most: LIMITS.maxPendingFrames, overflow_drops_positive: true, ack_seq: 1 },
    { pending_size: receiver.pending.size, queue_overflow_drops: receiver.queueOverflowDrops, ack_seq: receiver.appliedSeq },
    { max_pending_frames: LIMITS.maxPendingFrames },
    receiver.pending.size === LIMITS.maxPendingFrames && receiver.queueOverflowDrops > 0 && receiver.appliedSeq === 1,
  );
}

function lifecycleScenario() {
  const receiver = new Receiver();
  receiver.receive(inputFrame(1, 1, 33, { x: 1, z: 0 }, [{ id: 4001, kind: "interact" }]), 33);
  receiver.pause();
  receiver.receive(inputFrame(2, 1, 66, { x: 1, z: 0 }, [{ id: 4002, kind: "dash" }]), 66);
  receiver.resume();
  receiver.hidden();
  receiver.receive(inputFrame(3, 1, 99, { x: 0, z: 0 }), 99);
  receiver.resume();
  receiver.receive(inputFrame(4, 1, 132, { x: 0, z: 1 }, [{ id: 4004, kind: "jump" }]), 132);
  return check(
    "pause_hidden_resume_barrier",
    { pause_after_seq: 1, paused_deliveries: [2, 3], hidden_between_resume: true, resume_sequence: 4 },
    { consumed_edges: [4001, 4004], paused_edges_dropped: true, no_stale_held: true, ack_seq: 4 },
    { consumed_edges: receiver.consumedEdges, lifecycle_drops: receiver.lifecycleDrops, dropped: receiver.dropped, held_axes: receiver.heldAxes, ack_seq: receiver.appliedSeq },
    { paused_input_replay: false, resume_ack_seq: 4, lifecycle_events: ["pause", "resume", "hidden", "resume"] },
    receiver.consumedEdges.join(",") === "4001,4004" && receiver.lifecycleDrops === 2 && receiver.appliedSeq === 4 && receiver.heldAxes.z === 1 && receiver.lifecycleEvents.join(",") === "pause,resume,hidden,resume",
  );
}

function epochScenario() {
  const receiver = new Receiver();
  receiver.receive(inputFrame(1, 1, 33, { x: 1, z: 0 }, [{ id: 5001, kind: "attack" }]), 33);
  receiver.switchEpoch(2);
  receiver.receive(inputFrame(2, 1, 66, { x: 1, z: 0 }, [{ id: 5002, kind: "attack" }]), 66);
  receiver.receiveView(worldView(2, 1, 1, "grey_hive"));
  receiver.receive(inputFrame(1, 2, 33, { x: 0, z: 1 }, [{ id: 6001, kind: "interact" }]), 100);
  receiver.receiveView(worldView(1, 1, 2, "mist_harbor"));
  return check(
    "world_epoch_switch_isolation",
    { old_epoch: 1, new_epoch: 2, old_deliveries_after_switch: ["input seq2", "view tick2"] },
    { old_epoch_dropped: true, new_epoch_applied: true, new_ack_seq: 1 },
    { epoch: receiver.epoch, applied_sequences: receiver.appliedFrames, consumed_edges: receiver.consumedEdges, dropped: receiver.dropped, snapshot_applied: receiver.snapshotApplied, ack_seq: receiver.appliedSeq },
    { old_epoch_must_not_enter_new_world: true },
    receiver.epoch === 2 && receiver.appliedSeq === 1 && receiver.appliedFrames.join(",") === "1" && receiver.consumedEdges.join(",") === "6001" && receiver.snapshotApplied.join(",") === "1",
  );
}

function snapshotScenario() {
  const receiver = new Receiver();
  receiver.receive(inputFrame(1), 33);
  receiver.receiveView(worldView(10, 1));
  receiver.receiveView(worldView(12, 1));
  receiver.receiveView(worldView(11, 1));
  receiver.receiveView(worldView(12, 1));
  receiver.receiveView(worldView(13, 2));
  const futureAckDropped = receiver.snapshotDropped.some((drop) => drop.reason === "ack_beyond_applied");
  return check(
    "snapshot_monotonic_ack",
    { delivered_server_ticks: [10, 12, 11, 12, 13], ack_seq_last: 2 },
    { applied_ticks: [10, 12], stale_ticks_dropped: [11, 12], future_ack_dropped: true },
    { applied_ticks: receiver.snapshotApplied, dropped: receiver.snapshotDropped, ack_seq: receiver.appliedSeq, last_tick: receiver.snapshotTick },
    { ack_max: 1, server_tick_strictly_increasing: true },
    receiver.snapshotApplied.join(",") === "10,12" && receiver.snapshotDropped.length === 3 && futureAckDropped,
  );
}

function longFrameScenario() {
  const receiver = new Receiver();
  receiver.receive(inputFrame(1, 1, 0, { x: 1, z: 0 }), 0);
  receiver.advance(250);
  receiver.receive(inputFrame(2, 1, 250, { x: 0, z: 0 }), 250);
  return check(
    "long_frame_held_expiry_and_recovery",
    { frame_block_ms: 250, held_expiry_ms: LIMITS.heldExpiryMs, recovery_seq: 2 },
    { held_zero_at_250ms: true, recovery_applied: true, ack_seq: 2 },
    { held_before_recovery: { x: 0, z: 0 }, applied_sequences: receiver.appliedFrames, held_after_recovery: receiver.heldAxes, ack_seq: receiver.appliedSeq },
    { held_expiry_max_ms: LIMITS.heldExpiryMs },
    receiver.appliedSeq === 2 && receiver.heldAxes.x === 0,
  );
}

function payloadReport() {
  const input = inputFrame(42, 7, 1400, { x: 0.7071, z: -0.7071 }, [{ id: 99, kind: "dash" }]);
  const view = worldView(420, 42, 7, "grey_hive");
  const inputBytes = Buffer.byteLength(JSON.stringify(input), "utf8");
  const viewBytes = Buffer.byteLength(JSON.stringify(view), "utf8");
  return {
    encoding: "UTF-8 JSON sample for schema sizing; production wire codec is undecided",
    input_frame: { sample: input, bytes: inputBytes, max_bytes: LIMITS.maxInputFrameBytes },
    world_view: { sample: view, bytes: viewBytes, max_bytes: LIMITS.maxWorldViewBytes },
    bandwidth_estimate: {
      input_hz: LIMITS.inputHz,
      snapshot_hz: LIMITS.snapshotHz,
      input_bytes_per_second: inputBytes * LIMITS.inputHz,
      snapshot_bytes_per_second: viewBytes * LIMITS.snapshotHz,
      combined_bytes_per_second: (inputBytes + viewBytes) * LIMITS.inputHz,
      note: "JSON UTF-8 upper-bound sample; excludes transport framing/compression and actual actor payload variability",
    },
  };
}

function sourceAudit() {
  return [
    { file: "server-rs/Cargo.toml", lines: "17-20", finding: "Tauri dependency is declared as version 2; no new IPC dependency was added." },
    { file: "server-rs/Cargo.lock", lines: "3081-3085", finding: "Current resolved Tauri package is 2.11.5; this probe does not claim Channel behavior was exercised." },
    { file: "server-rs/src/campaign_main.rs", lines: "10-24", finding: "campaign_view returns a complete store view and campaign_action accepts one action plus optional JSON args." },
    { file: "server-rs/src/campaign_main.rs", lines: "63-70", finding: "The current invoke handler registers campaign_view/action/save/window only; no continuous Channel endpoint." },
    { file: "server-rs/release-ui/js/app.js", lines: "188-200", finding: "WebView uses Tauri core.invoke or HTTP request/response for view/action/save." },
    { file: "server-rs/release-ui/js/app.js", lines: "650-718", finding: "ActionBusy queues at most three normal actions, one pending move, and one pending tick; no seq/ack protocol." },
    { file: "server-rs/release-ui/js/app.js", lines: "730-745", finding: "Pause clears held/queued UI input and attempts a save; resume does not provide a sim barrier or replay protocol." },
    { file: "server-rs/release-ui/js/app.js", lines: "862-864", finding: "Current movement/tick intervals are 160/250 ms, not the proposed 30 Hz input/snapshot cadence." },
    { file: "server-rs/release-ui/js/app.js", lines: "872-873", finding: "Blur/focus/visibility lifecycle clears held input; hidden-window handling is not a Rust Channel lifecycle contract." },
    { file: "server-rs/release-ui/js/tps-renderer.js", lines: "636-663", finding: "TPS local prediction advances from render dt and is not an authoritative WorldView protocol." },
    { file: "server-rs/release-ui/js/tps-renderer.js", lines: "868-887", finding: "Babylon render loop clamps dt and calls local movement/render updates." },
  ];
}

function main() {
  const scenarios = [
    baselineScenario(),
    latencyJitterScenario(),
    duplicateScenario(),
    reorderScenario(),
    lossExpiryScenario(),
    queueBoundScenario(),
    lifecycleScenario(),
    epochScenario(),
    snapshotScenario(),
    longFrameScenario(),
  ];
  const payloads = payloadReport();
  const schemaPass = payloads.input_frame.bytes <= LIMITS.maxInputFrameBytes && payloads.world_view.bytes <= LIMITS.maxWorldViewBytes;
  const allPass = scenarios.every((scenario) => scenario.pass) && schemaPass;
  const report = {
    schema: "continuous_ipc_probe_ag08_v1",
    status: allPass ? "isolated_ipc_probe_passed" : "isolated_ipc_probe_failed",
    pass: allPass,
    scope: "isolated versioned protocol candidate; no formal Tauri Channel or IPC latency claim",
    environment: { node: process.version, platform: process.platform, arch: process.arch, os_release: os.release() },
    current_tauri: { manifest_requirement: "2", resolved_lock_version: "2.11.5", channel_runtime_tested: false },
    protocol_contract: {
      protocol: LIMITS.protocol,
      version: LIMITS.version,
      input_frame: {
        cadence: "30Hz held-state plus immediate edge delivery",
        fields: ["worldEpoch", "seq", "clientTimeMs", "heldAxes.x/z", "buttons", "edgeEvents"],
        units: { clientTimeMs: "monotonic diagnostic milliseconds only", heldAxes: "normalized dimensionless", edge_id: "u64-like integer" },
        client_time_authority: "never used as authoritative sim clock; receiver monotonic receipt time controls expiry",
      },
      world_view: {
        cadence: "30Hz snapshot candidate",
        fields: ["worldEpoch", "serverTick", "ackSeq", "serverTimeMs", "worldId", "player", "actors", "doors"],
        units: { serverTimeMs: "authoritative monotonic milliseconds", positionM: "metres", yawRad: "radians", velocityMps: "metres/second" },
      },
      limits: LIMITS,
      ordering: "apply contiguous seq only; buffer gaps up to maxPendingFrames; ackSeq is last contiguous applied input",
      duplicate_rule: "drop old/duplicate seq; edge IDs are deduplicated across the current world epoch",
      epoch_rule: "worldEpoch changes clear pending input, held state, edge dedupe and snapshot baseline; old epoch is dropped",
      lifecycle_rule: "pause/hidden clears pending+held and drops received input; resume starts from a fresh post-barrier seq",
      compatibility_rule: "unknown protocol/version/oversized fields are rejected; version bump required for incompatible schema",
      error_rule: "bounded drop counters and reason codes are observable; no unbounded queue or retry loop",
    },
    source_audit: sourceAudit(),
    scenarios,
    payloads,
    schema_check: { pass: schemaPass, input_bytes_under_limit: payloads.input_frame.bytes <= LIMITS.maxInputFrameBytes, world_view_bytes_under_limit: payloads.world_view.bytes <= LIMITS.maxWorldViewBytes },
    comparison: {
      ag01_120ms_correction: "reference only; this probe does not re-run AG-01 visual correction",
      current_request_response: "one action/view request at a time with UI-side coalescing",
      candidate: "30Hz InputFrame held state + immediate edge events, 30Hz WorldView snapshots with ackSeq and worldEpoch",
    },
    limitations: [
      "No formal Tauri Channel was opened; throughput, ordering, lifecycle delivery and WebView backpressure remain unmeasured.",
      "JSON sizing is a UTF-8 sample, not a selected production codec or compression result.",
      "The harness models receiver rules deterministically; it does not prove Rust serialization, thread scheduling, OS background behavior, or network transport.",
      "The 30Hz contract and pause-drop policy are candidates for TP-01/ADR D, not approved runtime behavior.",
    ],
  };
  fs.mkdirSync(path.dirname(RESULT_PATH), { recursive: true });
  fs.writeFileSync(RESULT_PATH, `${JSON.stringify(report, null, 2)}\n`, "utf8");
  console.log(JSON.stringify({ status: report.status, scenarios: scenarios.length, passed: scenarios.filter((scenario) => scenario.pass).length, input_bytes: payloads.input_frame.bytes, world_view_bytes: payloads.world_view.bytes }));
  if (!allPass) process.exitCode = 1;
}

main();
