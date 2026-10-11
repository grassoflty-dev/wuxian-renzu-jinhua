const world = (id, region, suffix = "a") => ({
  id, region, position_m: [10, 0, 10], rotation_rad: [0, 0, 0], velocity_mps: [0, 0, 0],
  extents_m: [4, 2, 4], collision: { id: `collision-${suffix}`, extents_m: [4, 2, 4] },
  interaction: { id: `interaction-${suffix}`, center_m: [10, 0, 10], radius_m: 2 },
  entities: [{ id: `player-${suffix}`, kind: "player", position_m: [10, 0, 10], radius_m: 0.5 }, { id: `npc-${suffix}`, kind: "npc", state: "idle", position_m: [12, 0, 10], radius_m: 0.5 }],
  triggers: [{ id: `entry-${suffix}`, kind: "entry", center_m: [8, 0, 10], extents_m: [1, 2, 2], target_region: region }, { id: `exit-${suffix}`, kind: "exit", center_m: [14, 0, 10], radius_m: 1, target_region: region }],
  navigation_targets: [{ id: `nav-${suffix}`, position_m: [12, 0, 10], region }],
  events: [{ id: `event-${suffix}`, kind: "hive_lockdown", trigger_id: `entry-${suffix}`, radius_m: 3, reads: ["hive_lockdown"] }],
  progression: { hive_lockdown: false, hive_choice: "unresolved", npc_state: "idle", gate_b_reads: "hive_lockdown" },
});

export const positiveFixtures = [
  world("grey_hive", "GH-B05", "gh"), world("grey_hive_aux", "GH-B01", "gh2"),
  world("mist_harbor", "MH-A01", "mh"), world("mist_harbor_aux", "MH-A02", "mh2"),
  world("clockwork_city", "CC-A01", "cc"), world("clockwork_city_aux", "CC-A02", "cc2"),
];

const clone = (value) => structuredClone(value);
const fixture = (name, mutate, expected) => { const value = world("grey_hive", "GH-B05", name.slice(0, 4)); mutate(value); return { name, value, expected }; };

export const negativeFixtures = [
  fixture("grid_row_field", (v) => { v.row = 2; }, "E_GRID_SEMANTIC"),
  fixture("grid_tile_field", (v) => { v.collision.tile = 4; }, "E_GRID_SEMANTIC"),
  fixture("grid_xy_field", (v) => { v.gridX = 2; v.gridY = 3; }, "E_GRID_SEMANTIC"),
  fixture("missing_position_unit", (v) => { v.entities[0].position = [1, 0, 1]; delete v.entities[0].position_m; }, "E_UNIT_REQUIRED"),
  fixture("wrong_unit", (v) => { v.velocity_kph = [1, 0, 0]; delete v.velocity_mps; }, "E_UNIT_REQUIRED"),
  fixture("nan_coordinate", (v) => { v.position_m[0] = NaN; }, "E_NON_FINITE"),
  fixture("infinite_coordinate", (v) => { v.position_m[0] = Infinity; }, "E_NON_FINITE"),
  fixture("zero_radius", (v) => { v.interaction.radius_m = 0; }, "E_POSITIVE_RANGE"),
  fixture("negative_extent", (v) => { v.extents_m[0] = -1; }, "E_POSITIVE_RANGE"),
  fixture("dangling_region", (v) => { v.triggers[0].target_region = "MISSING"; }, "E_DANGLING_REF"),
  fixture("dangling_trigger", (v) => { v.events[0].trigger_id = "missing-trigger"; }, "E_DANGLING_REF"),
  fixture("dangling_navigation_region", (v) => { v.navigation_targets[0].region = "MISSING"; }, "E_DANGLING_REF"),
  fixture("duplicate_id", (v) => { v.entities[1].id = v.entities[0].id; }, "E_DUPLICATE_ID"),
  fixture("gate_reads_choice", (v) => { v.progression.gate_b_reads = "hive_choice"; }, "E_GATE_B_SEMANTICS"),
  fixture("illegal_choice", (v) => { v.progression.hive_choice = "maybe"; }, "E_CHOICE_VALUE"),
  fixture("npc_coupled_to_choice", (v) => { v.progression.npc_state = "taken"; }, "E_NPC_CHOICE_COUPLING"),
  fixture("return_station_formal", (v) => { v.id = "return_station"; v.formal = true; }, "E_RETURN_STATION"),
  fixture("unknown_world", (v) => { v.id = "unknown_world"; }, "E_UNKNOWN_WORLD"),
  fixture("missing_collision", (v) => { delete v.collision; }, "E_REQUIRED_STRUCTURE"),
  fixture("missing_interaction", (v) => { delete v.interaction; }, "E_REQUIRED_STRUCTURE"),
  fixture("missing_navigation", (v) => { delete v.navigation_targets; }, "E_REQUIRED_STRUCTURE"),
  fixture("missing_event_range", (v) => { delete v.events[0].radius_m; }, "E_REQUIRED_STRUCTURE"),
  fixture("negative_event_range", (v) => { v.events[0].radius_m = -2; }, "E_POSITIVE_RANGE"),
  fixture("invalid_rotation", (v) => { v.rotation_rad = [0, 0]; }, "E_VECTOR_SHAPE"),
  fixture("invalid_velocity", (v) => { v.velocity_mps = [0, 0, "fast"]; }, "E_NON_NUMERIC"),
];

export function allFixtures() { return [...positiveFixtures, ...negativeFixtures.map((entry) => ({ ...entry, value: clone(entry.value) }))]; }
