import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const rendererPath = path.join(root, 'apps/godot_playable/scripts/world/renderer.gd');
const helperPath = path.join(root, 'apps/godot_playable/scripts/world/gh_exit_environment_visual.gd');
const renderer = fs.readFileSync(rendererPath, 'utf8');
const helper = fs.readFileSync(helperPath, 'utf8');

function functionSource(source, name) {
  const start = source.indexOf(`func ${name}(`);
  assert.notEqual(start, -1, `missing ${name}`);
  const end = source.indexOf('\nfunc ', start + 1);
  return source.slice(start, end < 0 ? source.length : end);
}

test('grounding is emitted after existing prop shadows and before object sorting', () => {
  const propShadows = renderer.indexOf('gh_exit_environment_visual.draw_contact_shadow');
  const grounding = renderer.indexOf('gh_exit_environment_visual.draw_body_grounding');
  const sorting = renderer.indexOf('objects.sort_custom', grounding);
  assert.ok(propShadows >= 0 && propShadows < grounding && grounding < sorting);
  assert.match(renderer, /var gh_exit_grounding_ids: Array\[String\] = _gh_exit_body_grounding_ids\(\)/);
});

test('selector reuses current body selectors, cache readiness, and existing culling without loading', () => {
  const selector = functionSource(renderer, '_gh_exit_body_grounding_ids');
  for (const token of ['_gh_story_prop(', '_gh_crate_selected(', '_object_culling_state(', 'get(\'attempted\')', 'Texture2D', '1270', '1086', '1238', '1233']) {
    assert.ok(selector.includes(token), `selector missing ${token}`);
  }
  assert.doesNotMatch(selector, /prepare_resources|_load_once|\.draw_body\(/);
  assert.match(selector, /image_allowed/);
});

test('helper validates and stages exact ground ellipses before drawing', () => {
  const draw = functionSource(helper, 'draw_body_grounding');
  for (const token of ['_valid_canvas(', '_valid_context(', '_valid_projection(', 'exit_extraction_console_approved', 'exit_console_crates', 'Vector3(8,0,8)', 'Vector2(0.40,0.22)', 'Vector3(18,0,11.5)', 'Vector2(0.30,0.18)', 'SHADOW_STEPS', '_finite_points(', 'Color(0.025,0.035,0.038,0.16)']) {
    assert.ok(draw.includes(token), `helper missing ${token}`);
  }
  assert.doesNotMatch(draw, /FileAccess|_load_once|prepare_resources|serverTick|reduced_motion/);
  assert.ok(draw.indexOf('commands.append(points)') < draw.indexOf('canvas.draw_colored_polygon'));
});

test('specified shadow envelopes stay clear of the authored transition and sealed barrier', () => {
  const targets = [
    { x: 8, z: 8, rx: 0.40, rz: 0.22 },
    { x: 18, z: 11.5, rx: 0.30, rz: 0.18 },
  ];
  const transition = { x0: 0.5, x1: 2.5, z0: 7, z1: 9 };
  const barrier = { x0: 21.5, x1: 22.5, z0: 0.5, z1: 15.5 };
  const overlaps = (a, b) => a.x - a.rx < b.x1 && a.x + a.rx > b.x0 && a.z - a.rz < b.z1 && a.z + a.rz > b.z0;
  for (const target of targets) {
    assert.equal(overlaps(target, transition), false);
    assert.equal(overlaps(target, barrier), false);
    const projected = Array.from({ length: 24 }, (_, i) => {
      const angle = 2 * Math.PI * i / 24;
      const x = target.x + Math.cos(angle) * target.rx;
      const z = target.z + Math.sin(angle) * target.rz;
      return [x - z, (x + z) / 2];
    });
    assert.equal(projected.length, 24);
    assert.ok(projected.flat().every(Number.isFinite));
  }
});
