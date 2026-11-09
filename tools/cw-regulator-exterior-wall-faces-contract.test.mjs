import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const helper = readFileSync(resolve(root, 'apps/godot_playable/scripts/world/cw_regulator_environment_visual.gd'), 'utf8');
const scene = JSON.parse(readFileSync(resolve(root, 'apps/godot_playable/data/scenes/cw_regulator_core.json'), 'utf8'));
const expected = [
	{ id: 'cw_regulator_core_wall_south', indices: [3, 2], a: [0, 16], b: [24, 16], height: 0.18 },
	{ id: 'cw_regulator_core_wall_east_north', indices: [1, 2], a: [24, 0.5], b: [24, 6], height: 0.18 },
	{ id: 'cw_regulator_core_wall_east_south', indices: [1, 2], a: [24, 9.5], b: [24, 15.5], height: 0.18 },
];

function project(point, ppm = 1) {
	const [x, y, z] = point;
	return [(x - z) * ppm, ((x + z) / 2 - y) * ppm];
}

function polygonArea(points) {
	return points.reduce((sum, point, index) => {
		const next = points[(index + 1) % points.length];
		return sum + point[0] * next[1] - point[1] * next[0];
	}, 0);
}

function containsConvex(point, polygon) {
	let sign = 0;
	for (let i = 0; i < polygon.length; i += 1) {
		const a = polygon[i];
		const b = polygon[(i + 1) % polygon.length];
		const cross = (b[0] - a[0]) * (point[1] - a[1]) - (b[1] - a[1]) * (point[0] - a[0]);
		if (cross === 0) continue;
		if (sign !== 0 && Math.sign(cross) !== sign) return false;
		sign = Math.sign(cross);
	}
	return sign !== 0;
}

test('source-static exterior edge table matches canonical wall geometry and heights', () => {
	const tableStart = helper.indexOf('const OUTER_WALL_EDGES: Dictionary = {');
	assert.ok(tableStart >= 0, 'exterior edge table exists');
	const tableEnd = helper.indexOf('\n}', tableStart);
	assert.ok(tableEnd > tableStart, 'exterior edge table closes');
	const table = helper.slice(tableStart, tableEnd);
	const matches = [...table.matchAll(/'([^']+)':\s*\[Vector2\(([^)]+)\),\s*Vector2\(([^)]+)\)\]/g)];
	assert.equal(matches.length, expected.length, 'exactly three exterior edges are declared');
	const wallStart = helper.indexOf('const WALLS: Dictionary = {');
	assert.ok(wallStart >= 0, 'canonical helper wall table exists');
	const wallEnd = helper.indexOf('\n}', wallStart);
	assert.ok(wallEnd > wallStart, 'canonical helper wall table closes');
	const wallLines = helper.slice(wallStart, wallEnd).split(/\r?\n/);
	for (const edge of expected) {
		const found = matches.find((match) => match[1] === edge.id);
		assert.ok(found, 'edge table includes ' + edge.id);
		const pair = (value) => value.split(',').map(Number);
		assert.deepEqual([pair(found[2]), pair(found[3])], [edge.a, edge.b]);
		const row = scene.collision.filter((item) => item.id === edge.id);
		assert.equal(row.length, 1, 'canonical collision row is unique for ' + edge.id);
		assert.deepEqual(row[0].polygon[edge.indices[0]], edge.a);
		assert.deepEqual(row[0].polygon[edge.indices[1]], edge.b);
		const wallLine = wallLines.find((line) => line.includes("'" + edge.id + "':"));
		assert.ok(wallLine, 'existing wall presentation exists for ' + edge.id);
		assert.match(wallLine, /,\s*0\.18\],?\s*$/, 'existing display height is preserved');
	}
});

test('source-static projected faces are finite, noncollapsed, and cover the south counterexample', () => {
	for (const edge of expected) {
		const world = [[edge.a[0], 0, edge.a[1]], [edge.b[0], 0, edge.b[1]], [edge.b[0], edge.height, edge.b[1]], [edge.a[0], edge.height, edge.a[1]]];
		for (const ppm of [24, 48, 96, 256]) {
			const projected = world.map((point) => project(point, ppm));
			const area = polygonArea(projected);
			assert.ok(Number.isFinite(area), edge.id + ' projection is finite at ' + ppm + ' ppm');
			assert.notEqual(area, 0, edge.id + ' projection is noncollapsed at ' + ppm + ' ppm');
		}
	}
	const south = expected[0];
	const quad = [[south.a[0], 0, south.a[1]], [south.b[0], 0, south.b[1]], [south.b[0], south.height, south.b[1]], [south.a[0], south.height, south.a[1]]].map((point) => project(point));
	assert.equal(containsConvex(project([12, 0.09, 16]), quad), true, 'south exterior counterexample lies within the added face');
	assert.equal(expected[2].b[1], 15.5, 'east-south exterior ends at the authored outer edge, not the clipped inner curb');
});

test('source-static bound staging validates before emission and retains the legacy cap path', () => {
	const start = helper.indexOf('func draw_wall(');
	assert.ok(start >= 0, 'existing draw_wall entry exists');
	const draw = helper.slice(start);
	const contact = draw.indexOf('if not _polygon(plan,camera,contact');
	assert.ok(contact >= 0, 'legacy contact and inner side staging exists');
	const extraStart = draw.indexOf('var extra_count: int = 0');
	assert.ok(extraStart > contact, 'extra-face staging follows legacy contact and side');
	const joint = draw.indexOf('for joint: float in joint_positions:', extraStart);
	assert.ok(joint > extraStart, 'legacy joint loop remains after extra-face staging');
	const cap = draw.indexOf("if not _polygon(plan,camera,cap,Color('#4c5658'))", joint);
	assert.ok(cap > joint, 'legacy cap remains after joints');
	const extra = draw.slice(extraStart, joint);
	for (const binding of [
		"canvas.get('snapshot') == snapshot",
		"canvas.get('scene') == scene",
		"canvas.get('current_key') == current_key",
		"canvas.get('camera') == camera",
		"canvas.get('viewport_rect') == viewport",
		"camera.get('viewport') == bound_viewport.size",
	]) assert.ok(extra.includes(binding), 'addition is bound to ' + binding);
	const finite = extra.indexOf('if not is_finite(doubled_area):');
	assert.ok(finite >= 0, 'projected area has a finite guard');
	const zeroArea = extra.indexOf('if doubled_area != 0.0:');
	assert.ok(zeroArea > finite, 'exactly collapsed addition is skipped');
	const tempValidation = extra.indexOf('if not _plan_valid(outer_plan,1):');
	assert.ok(tempValidation > zeroArea, 'noncollapsed temporary command is validated');
	const append = extra.indexOf('plan.append(outer_plan[0])');
	assert.ok(append > tempValidation, 'validated command is appended only afterward');
	const combined = draw.indexOf('not _plan_valid(plan,13+extra_count)', cap);
	assert.ok(combined > cap, 'legacy ceiling admits only the staged additional command');
	const emit = draw.indexOf('_emit(canvas,plan)', combined);
	assert.ok(emit > combined, 'combined plan validates before the single emission');
});
