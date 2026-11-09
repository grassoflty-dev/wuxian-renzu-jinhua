import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const source = readFileSync(resolve(dirname(fileURLToPath(import.meta.url)), '../apps/godot_playable/scripts/world/gh_power_environment_visual.gd'), 'utf8');

function functionBody(name) {
  const start = source.indexOf(`func ${name}(`);
  assert.ok(start >= 0, `missing ${name}`);
  const next = source.indexOf('\nfunc ', start + 1);
  return source.slice(start, next < 0 ? source.length : next);
}

test('off state adds only the registered static emergency pool', () => {
  const build = functionBody('build_packets');
  const states = [...build.matchAll(/if light_state=='([^']+)'/g)].map((match) => match[1]);
  assert.deepEqual(states, ['on', 'off']);
  const onAt = build.indexOf("if light_state=='on' and not _add_task_light(packets,projection):");
  const offAt = build.indexOf("if light_state=='off' and not _add_off_emergency_light(packets,projection):");
  const resultAt = build.indexOf('return packets if packets.size()<=MAX_PACKETS else []');
  assert.ok(onAt >= 0 && offAt >= 0 && resultAt >= 0 && onAt < offAt && offAt < resultAt);

  const method = functionBody('_add_off_emergency_light');
  const call = method.match(/return _add_ground_ellipse\(packets,'([^']+)','([^']+)',Vector3\(([^)]+)\),([^,]+),([^,]+),Color\(([^)]+)\),projection,'([^']+)'\)/);
  assert.ok(call, 'off helper delegates its actual source identity and values');
  assert.equal(call[1], 'gh_power_console:off-emergency-light');
  assert.equal(call[2], 'off_emergency_light');
  assert.deepEqual(call[3].split(',').map(Number), [4.75, 0, 6.45]);
  assert.deepEqual([Number(call[4]), Number(call[5])], [0.72, 0.48]);
  assert.deepEqual(call[6].split(',').map(Number), [0.55, 0.18, 0.16, 0.06]);
  assert.equal(call[7], 'power_console_off_on');

  const ellipse = functionBody('_add_ground_ellipse');
  assert.ok(ellipse.includes('for index in range(20)'));
  assert.ok(ellipse.includes('TAU*float(index)/20.0'));
  assert.ok(ellipse.includes('Vector3(center.x+cos(angle)*radius_x,0.0,center.z+sin(angle)*radius_z)'));
  const center = call[3].split(',').map(Number);
  const rx = Number(call[4]);
  const rz = Number(call[5]);
  const points = Array.from({ length: 20 }, (_, index) => {
    const angle = 2 * Math.PI * index / 20;
    return [center[0] + Math.cos(angle) * rx, 0, center[2] + Math.sin(angle) * rz];
  });
  assert.equal(points.length, 20);
  assert.ok(points.every((point) => point.every(Number.isFinite) && point[1] === 0));
  assert.ok(points.every((point) => point[0] >= center[0] - rx && point[0] <= center[0] + rx && point[2] >= center[2] - rz && point[2] <= center[2] + rz));
});
