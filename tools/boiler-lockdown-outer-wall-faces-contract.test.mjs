import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => readFileSync(path.join(root, relative), "utf8");
const boiler = read("apps/godot_playable/scripts/world/cw_boiler_environment_visual.gd");
const lockdown = read("apps/godot_playable/scripts/world/gh_lockdown_environment_visual.gd");
const projection = read("apps/godot_playable/scripts/world/projection.gd");
const scenes = [
  { id: "boiler", source: boiler, scene: JSON.parse(read("apps/godot_playable/data/scenes/cw_boiler_chamber.json")), prefix: "cw_boiler_chamber_", height: 0.22, openingNorth: 6, indices: { south: [3, 2], east_north: [1, 2], east_south: [1, 2] } },
  { id: "lockdown", source: lockdown, scene: JSON.parse(read("apps/godot_playable/data/scenes/gh_lockdown.json")), prefix: "lockdown_", height: 0.18, openingNorth: 6.5, indices: { south: [3, 2], east_north: [1, 2], east_south: [1, 2] } },
];
function section(source, start, end) {
  const a = source.indexOf(start);
  assert.ok(a >= 0, `missing source anchor: ${start}`);
  const b = source.indexOf(end, a + start.length);
  assert.ok(b >= 0, `missing end anchor: ${end}`);
  return source.slice(a, b);
}
function edgeTable(source) {
  const start = source.indexOf("const OUTER_WALL_EDGES: Dictionary = {");
  assert.ok(start >= 0, "missing outer-edge table");
  const end = source.indexOf("\n}", start);
  assert.ok(end >= 0, "unterminated outer-edge table");
  const table = new Map();
  for (const line of source.slice(start, end).split(/\r?\n/).slice(1)) {
    const entry = line.match(/^\s*["']([^"']+)["']\s*:\s*(.+?),?\s*$/);
    if (!entry) continue;
    const vectorPairs = [...entry[2].matchAll(/Vector2\(\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\)/g)];
    const pairs = vectorPairs.length ? vectorPairs : [...entry[2].matchAll(/\[\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\]/g)];
    assert.equal(pairs.length, 2, `edge ${entry[1]} must have two endpoints`);
    assert.ok(!table.has(entry[1]), `duplicate edge ${entry[1]}`);
    table.set(entry[1], pairs.map((pair) => [Number(pair[1]), Number(pair[2])]));
  }
  return table;
}
function closePoint(actual, expected) {
  assert.equal(actual.length, 2);
  assert.ok(actual.every(Number.isFinite));
  assert.ok(actual.every((value, index) => Math.abs(value - expected[index]) < 1e-9));
}
function project([x, y, z], ppm) {
  return [(x - z) * ppm, ((x + z) * 0.5 - y) * ppm];
}
function twiceArea(points) {
  return points.reduce((sum, point, index) => {
    const next = points[(index + 1) % points.length];
    return sum + point[0] * next[1] - next[0] * point[1];
  }, 0);
}
function containsConvex(point, polygon) {
  const crosses = polygon.map((a, index) => {
    const b = polygon[(index + 1) % polygon.length];
    return (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);
  });
  return crosses.every((value) => value >= -1e-8) || crosses.every((value) => value <= 1e-8);
}
function before(source, first, second, label) {
  const a = source.indexOf(first);
  const b = source.indexOf(second);
  assert.ok(a >= 0, `${label}: missing ${first}`);
  assert.ok(b >= 0, `${label}: missing ${second}`);
  assert.ok(a < b, `${label}: ${first} must precede ${second}`);
}
test("SOURCE_STATIC_GEOMETRY_NOT_GDSCRIPT_OR_VISUAL: Boiler and Lockdown exterior faces", () => {
  const projectionCode = projection.replace(/\s/g, "");
  assert.ok(projectionCode.includes("Vector2((d.x-d.z)*ppm,(d.x+d.z)*ppm*0.5-d.y*ppm)"), "projection basis changed; recompute the static geometry check");
  const suffixes = ["south", "east_north", "east_south"];

  for (const room of scenes) {
    const table = edgeTable(room.source);
    const expectedIds = suffixes.map((suffix) => `${room.prefix}wall_${suffix}`).sort();
    assert.deepEqual([...table.keys()].sort(), expectedIds, `${room.id}: exact three-edge scope`);
    const selected = new Map();
    for (const suffix of suffixes) {
      const id = `${room.prefix}wall_${suffix}`;
      const row = room.scene.collision.find((candidate) => candidate.id === id);
      assert.ok(row && Array.isArray(row.polygon), `${room.id}: missing canonical row ${id}`);
      const endpoints = room.indices[suffix].map((index) => row.polygon[index]);
      assert.deepEqual(table.get(id), endpoints, `${room.id}: ${id} must follow its canonical exterior edge`);
      selected.set(id, { endpoints, height: room.height });
    }
    const eastNorth = table.get(`${room.prefix}wall_east_north`);
    const eastSouth = table.get(`${room.prefix}wall_east_south`);
    assert.equal(eastNorth[1][1], room.openingNorth, `${room.id}: north opening endpoint`);
    assert.equal(eastSouth[0][1], 9.5, `${room.id}: south opening endpoint`);
    assert.ok(eastNorth[1][1] < eastSouth[0][1], `${room.id}: east opening remains open`);

    for (const [id, edge] of selected) {
      const [a, b] = edge.endpoints;
      for (const ppm of [24, 48, 96, 256]) {
        const quad = [project([a[0], 0, a[1]], ppm), project([b[0], 0, b[1]], ppm), project([b[0], edge.height, b[1]], ppm), project([a[0], edge.height, a[1]], ppm)];
        assert.ok(quad.flat().every(Number.isFinite), `${id} at ${ppm} ppm must be finite`);
        assert.ok(Math.abs(twiceArea(quad)) > 0, `${id} at ${ppm} ppm must be nondegenerate`);
      }
    }
    const south = selected.get(`${room.prefix}wall_south`);
    const [southA, southB] = south.endpoints;
    const southQuad = [project([southA[0], 0, southA[1]], 48), project([southB[0], 0, southB[1]], 48), project([southB[0], room.height, southB[1]], 48), project([southA[0], room.height, southA[1]], 48)];
    const counterexample = project([12, room.height / 2, 16], 48);
    closePoint(counterexample, southQuad.reduce((sum, p) => [sum[0] + p[0] / 4, sum[1] + p[1] / 4], [0, 0]));
    assert.ok(containsConvex(counterexample, southQuad), `${room.id}: south counterexample is covered by the new face`);
    const southRow = room.scene.collision.find((candidate) => candidate.id === `${room.prefix}wall_south`);
    const innerA = southRow.polygon[0];
    const innerB = southRow.polygon[1];
    const oldFace = [project([innerA[0], 0, innerA[1]], 48), project([innerB[0], 0, innerB[1]], 48), project([innerB[0], room.height, innerB[1]], 48), project([innerA[0], room.height, innerA[1]], 48)];
    const oldCap = southRow.polygon.map(([x, z]) => project([x, room.height, z], 48));
    assert.ok(!containsConvex(counterexample, oldFace) && !containsConvex(counterexample, oldCap), `${room.id}: source counterexample was outside the legacy inner face and cap`);
    assert.ok(Math.abs(twiceArea(southQuad)) > 0, `${room.id}: south counterexample lies on the added face`);
    assert.equal(south.height, room.height);
  }
  const boilerDraw = section(boiler, "func draw_wall(", "func _outer_face_context_matches(");
  assert.match(boilerDraw, /var wall_height: float = 0\.75 if high_wall else 0\.22/);
  assert.match(boilerDraw, /var high_wall: bool = id_value == 'cw_boiler_chamber_wall_north' or id_value == 'cw_boiler_chamber_wall_west_north' or id_value == 'cw_boiler_chamber_wall_west_south'/);
  assert.match(boilerDraw, /_valid_context\(snapshot_value,scene_value,current_key_value,projection,viewport_value\)/);
  before(boilerDraw, "_outer_face_context_matches(", "canvas.draw_colored_polygon(side_points", "Boiler binding before paint");
  for (const points of ["side_points", "outer_points", "cap_points"]) before(boilerDraw, `_points_have_area(${points})`, "canvas.draw_colored_polygon(side_points", `Boiler ${points} validation`);
  before(boilerDraw, "canvas.draw_colored_polygon(side_points,WALL_SIDE)", "canvas.draw_colored_polygon(outer_points,WALL_SIDE)", "Boiler inner then outer");
  before(boilerDraw, "canvas.draw_colored_polygon(outer_points,WALL_SIDE)", "canvas.draw_colored_polygon(cap_points,WALL_CAP)", "Boiler outer before cap");
  const boilerBinding = section(boiler, "func _outer_face_context_matches(", "func _outer_face_points(");
  for (const property of ["snapshot", "scene", "current_key", "camera", "viewport_rect"]) assert.ok(boilerBinding.includes(`canvas.get('${property}')`), `Boiler actual canvas ${property} binding`);
  assert.match(boilerBinding, /canvas_snapshot == snapshot/);
  assert.match(boilerBinding, /canvas_scene == scene/);
  assert.match(boilerBinding, /canvas_key == current_key_value/);
  assert.match(boilerBinding, /canvas_camera == projection/);
  assert.match(boilerBinding, /canvas_viewport == viewport/);
  assert.match(boilerBinding, /P\.snapshot\(snapshot_value\)/);
  assert.match(boilerBinding, /snapshot\.get\('entryToken'\) != null/);
  assert.match(boilerBinding, /projection_viewport == viewport\.size/);
  assert.match(boilerBinding, /initialized == true/);
  assert.match(section(boiler, "func _valid_projection(", "func _number_matches("), /get_script\(\) != ProjectionScript/);
  const boilerAreaAt = boiler.indexOf("func _points_have_area(");
  assert.ok(boilerAreaAt >= 0, "Boiler nonzero-area validator exists");
  assert.match(boiler.slice(boilerAreaAt), /is_finite\(twice_area\) and twice_area != 0\.0/);
  const boilerPoints = section(boiler, "func _outer_face_points(", "func _points_have_area(");
  assert.match(boilerPoints, /PackedVector2Array\(\[a,b,top_b,top_a\]\)/);
  for (const anchor of ["_project_point(projection,float(first_value[0]),0.0,float(first_value[1]))", "_project_point(projection,float(second_value[0]),0.0,float(second_value[1]))", "wall_height,float(second_value[1])", "wall_height,float(first_value[1])"]) assert.ok(boilerPoints.includes(anchor), `Boiler staged vertex ${anchor}`);
  const lockdownDraw = section(lockdown, "func draw_wall(", "func _outer_wall_bindings_match(");
  assert.match(lockdownDraw, /_valid_context\(snapshot, scene_data, current_key\)/);
  before(lockdownDraw, "if edge_variant != null and _outer_wall_bindings_match(", "draw_outer_face = true", "Lockdown selected binding gates addition");
  before(lockdownDraw, "_outer_wall_bindings_match(", "canvas.draw_colored_polygon(contact_points", "Lockdown binding before paint");
  before(lockdownDraw, "if draw_outer_face and (", "canvas.draw_colored_polygon(contact_points", "Lockdown combined validation before paint");
  for (const points of ["contact_points", "face_points", "outer_face_points", "cap_points"]) before(lockdownDraw, `_nonzero_polygon_area(${points})`, "canvas.draw_colored_polygon(contact_points", `Lockdown ${points} area validation`);
  before(lockdownDraw, "canvas.draw_colored_polygon(face_points,", "canvas.draw_colored_polygon(outer_face_points,", "Lockdown inner then outer");
  before(lockdownDraw, "canvas.draw_colored_polygon(outer_face_points,", "canvas.draw_colored_polygon(cap_points,", "Lockdown outer before cap");
  const lockdownFace = lockdownDraw.match(/canvas\.draw_colored_polygon\(face_points,\s*(Color\([^\n]+\))\)/);
  const lockdownOuter = lockdownDraw.match(/canvas\.draw_colored_polygon\(outer_face_points,\s*(Color\([^\n]+\))\)/);
  assert.ok(lockdownFace && lockdownOuter, "Lockdown face colors must be present");
  assert.equal(lockdownOuter[1], lockdownFace[1], "Lockdown outer face reuses its existing side color");
  const lockdownSpecs = section(lockdown, "func _wall_spec(", "func _matches_row(");
  for (const suffix of suffixes) {
    const id = `lockdown_wall_${suffix}`;
    const specAt = lockdownSpecs.indexOf(`"${id}"`);
    assert.ok(specAt >= 0, `Lockdown wall spec ${id}`);
    const branch = lockdownSpecs.slice(specAt, lockdownSpecs.indexOf("\n", specAt));
    const height = branch.match(/"height":\s*(\d+(?:\.\d+)?)/);
    assert.ok(height, `Lockdown height for ${id}`);
    assert.equal(Number(height[1]), 0.18, `Lockdown existing height for ${id}`);
  }
  assert.match(lockdownDraw, /PackedVector2Array\(\[outer_ground_a, outer_ground_b, outer_raised_b, outer_raised_a\]\)/);
  for (const anchor of ["Vector3(outer_a.x, 0.0, outer_a.y)", "Vector3(outer_b.x, 0.0, outer_b.y)", "Vector3(outer_a.x, height, outer_a.y)", "Vector3(outer_b.x, height, outer_b.y)"]) assert.ok(lockdownDraw.includes(anchor), `Lockdown staged vertex ${anchor}`);
  const lockdownBinding = section(lockdown, "func _outer_wall_bindings_match(", "func _nonzero_polygon_area(");
  for (const property of ["snapshot", "scene", "current_key", "camera", "viewport_rect"]) assert.ok(lockdownBinding.includes(`canvas.get('${property}')`) || lockdownBinding.includes(`canvas.get(\"${property}\")`), `Lockdown actual canvas ${property} binding`);
  assert.match(lockdownBinding, /_valid_identity\(snapshot, scene_data, current_key\)/);
  assert.match(lockdownBinding, /_valid_projection\(projection, viewport\)/);
  assert.match(lockdownBinding, /actual_snapshot.*_deep_equal\(actual_snapshot, snapshot\)/);
  assert.match(lockdownBinding, /actual_scene.*_deep_equal\(actual_scene, scene_data\)/);
  assert.match(lockdownBinding, /actual_camera == projection/);
  assert.match(lockdownBinding, /actual_viewport == expected_viewport/);
  assert.match(lockdownBinding, /camera_viewport == expected_viewport\.size/);
  assert.match(section(lockdown, "func _valid_identity(", "func _valid_context("), /get\("entryToken"\) != null/);
  assert.match(section(lockdown, "func _valid_projection(", "func _valid_canvas("), /get_script\(\) != PROJECTION_SCRIPT/);
  assert.match(section(lockdown, "func _nonzero_polygon_area(", "func build_dressing("), /absf\(twice_area\) > 0\.0/);
});
