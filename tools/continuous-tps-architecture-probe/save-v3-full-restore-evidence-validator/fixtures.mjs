import crypto from "node:crypto";

const clone = (value) => structuredClone(value);
const digest = (value) => crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex");

export function checksumFor(save) {
  const copy = clone(save); delete copy.checksum;
  if (copy.migration) delete copy.migration.newChecksum;
  return `sha256:${digest(copy)}`;
}

function save(worldId, suffix, options = {}) {
  const choice = options.choice ?? "unresolved";
  const data = {
    saveVersion: 3, schemaVersion: "save-v3", worldId, region: `${worldId}-region`,
    position_m: [10, 0, 10], rotation_rad: [0, 0, 0], tick: 120, revision: 4, epoch: 2,
    player: { id: `player-${suffix}`, hp: 100, position_m: [10, 0, 10], inventoryIds: [`item-${suffix}`], capabilityIds: [`cap-${suffix}`] },
    npcs: [{ id: `npc-${suffix}`, state: "idle", position_m: [12, 0, 10] }],
    inventoryIds: [`item-${suffix}`], capabilityIds: [`cap-${suffix}`],
    progression: { hive_lockdown: Boolean(options.lockdown), hive_choice: choice, gate_b_reads: "hive_lockdown" },
    worldTransition: { from: `${worldId}-start`, to: `${worldId}-region`, status: "active" },
    eventFlags: [`event-${suffix}`], questFlags: [`quest-${suffix}`],
    rng: { algorithm: "pcg32", seed: 42, state: 991, drawCount: 12 }, sourceIdentity: `synthetic-${suffix}`,
    ...options.extra,
  };
  data.checksum = checksumFor(data);
  if (data.migration) { data.migration.newChecksum = data.checksum; data.checksum = checksumFor(data); }
  return data;
}

export const positiveFixtures = [
  { name: "v3-grey-hive-lockdown", save: save("grey_hive", "gh", { lockdown: true, choice: "taken" }), evidence: { equality: "exact", roundTrip: "canonical_equal" } },
  { name: "v3-mist-harbor-unresolved", save: save("mist_harbor", "mh", { choice: "unresolved" }), evidence: { equality: "exact", roundTrip: "canonical_equal" } },
  { name: "v3-clockwork-city-left", save: save("clockwork_city", "cc", { choice: "left" }), evidence: { equality: "normalized", diffs: [{ field: "checksum", rule: "recomputed" }], roundTrip: "canonical_equal" } },
  { name: "v2-to-v3-migrated", save: save("grey_hive", "mig", { extra: { migration: { sourceVersion: 2, steps: ["add_epoch", "split_progression_flags"], defaults: { epoch: 0 }, lossReport: [], sourcePreserved: true, sourceChecksum: "sha256:v2-source", newChecksum: "pending" } } }), evidence: { equality: "intentionally_migrated", diffs: [{ field: "schemaVersion", rule: "v2_to_v3" }], roundTrip: "canonical_equal" } },
  { name: "v3-npc-independent-choice", save: save("mist_harbor", "ind", { choice: "unresolved", extra: { npcs: [{ id: "npc-ind", state: "rescued", position_m: [12, 0, 10] }] } }), evidence: { equality: "exact", roundTrip: "canonical_equal" } },
  { name: "v3-return-station-deferred", save: save("clockwork_city", "defer", { extra: { deferredMetadata: { returnStation: "deferred" } } }), evidence: { equality: "exact", roundTrip: "canonical_equal" } },
];

const bad = (name, mutate, expected) => { const value = save("grey_hive", name.slice(0, 4)); mutate(value); return { name, save: value, evidence: { equality: "exact", roundTrip: "canonical_equal" }, expected }; };
export const negativeFixtures = [
  bad("unknown_world", (v) => { v.worldId = "unknown"; }, "E_UNKNOWN_WORLD"),
  bad("unknown_capability", (v) => { v.capabilityIds.push("alien-capability"); }, "E_UNKNOWN_ID"),
  bad("unknown_item", (v) => { v.inventoryIds.push("alien-item"); }, "E_UNKNOWN_ID"),
  bad("unknown_event", (v) => { v.eventFlags.push("alien-event"); }, "E_UNKNOWN_ID"),
  bad("unknown_schema", (v) => { v.schemaVersion = "save-v9"; }, "E_SCHEMA_VERSION"),
  bad("bad_checksum", (v) => { v.checksum = "sha256:corrupt"; }, "E_CHECKSUM"),
  bad("partial_write", (v) => { delete v.player; }, "E_REQUIRED_FIELD"),
  bad("duplicate_id", (v) => { v.npcs.push({ id: v.player.id, state: "idle", position_m: [1, 0, 1] }); }, "E_DUPLICATE_ID"),
  bad("dangling_reference", (v) => { v.player.inventoryIds = ["item-missing"]; }, "E_DANGLING_REF"),
  bad("illegal_choice", (v) => { v.progression.hive_choice = "maybe"; }, "E_CHOICE_VALUE"),
  bad("gate_reads_choice", (v) => { v.progression.gate_b_reads = "hive_choice"; }, "E_GATE_B_SEMANTICS"),
  bad("npc_choice_coupling", (v) => { v.npcs[0].state = "taken"; }, "E_NPC_CHOICE_COUPLING"),
  bad("grid_reinjection", (v) => { v.player.gridX = 2; }, "E_GRID_SEMANTIC"),
  bad("return_station_current", (v) => { v.worldId = "return_station"; v.deferredMetadata = { returnStation: "current" }; }, "E_RETURN_STATION"),
  bad("nonfinite_position", (v) => { v.position_m[0] = NaN; }, "E_NON_FINITE"),
  bad("negative_tick", (v) => { v.tick = -1; }, "E_NON_NEGATIVE"),
  bad("bad_position_shape", (v) => { v.position_m = [1, 2]; }, "E_VECTOR_SHAPE"),
  bad("missing_rng", (v) => { delete v.rng; }, "E_REQUIRED_FIELD"),
  bad("missing_source_identity", (v) => { delete v.sourceIdentity; }, "E_REQUIRED_FIELD"),
  bad("silent_v2_loss", (v) => { v.migration = { sourceVersion: 2, steps: [], defaults: {}, lossReport: [], sourcePreserved: false }; }, "E_MIGRATION_PROOF"),
  bad("missing_migration_steps", (v) => { v.migration = { sourceVersion: 2, defaults: {}, lossReport: [], sourcePreserved: true }; }, "E_MIGRATION_PROOF"),
  bad("unknown_result", (v) => { v.worldTransition.status = "mystery"; }, "E_UNKNOWN_RESULT"),
  bad("round_trip_mismatch", (v) => { v.roundTripCanonicalHash = "sha256:not-equal"; }, "E_ROUND_TRIP"),
  bad("missing_equality_rule", (v) => { v.evidenceRequired = true; }, "E_EQUALITY_EVIDENCE"),
];
