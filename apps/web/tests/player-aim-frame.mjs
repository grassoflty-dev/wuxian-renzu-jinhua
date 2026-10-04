import test from 'node:test';
import assert from 'node:assert/strict';
import { pointerOffsetFromPresentedFrame } from '../dist/renderer/PlayerAimFrame.js';
import { InputController } from '../dist/game/InputController.js';
import { boundActorPreviewIds } from '../dist/renderer/ActorPresentationBindings.js';
import { readFile } from 'node:fs/promises';

const frame = { width: 1280, height: 720, footX: 570, footY: 325 };
const rect = { left: 20, top: 30, width: 1280, height: 720 };
test('pointer is measured from actual presented feet, including camera lead and CSS resize', () => {
  assert.deepEqual(pointerOffsetFromPresentedFrame(frame, rect, 590, 355), { x: 0, y: 0 });
  assert.deepEqual(pointerOffsetFromPresentedFrame(frame, rect, 690, 355), { x: 100, y: 0 });
  assert.deepEqual(pointerOffsetFromPresentedFrame(frame, {left:20,top:30,width:640,height:180}, 355, 111.25), { x: 100, y: 0 });
  assert.equal(pointerOffsetFromPresentedFrame(frame, {...rect,width:0}, 1, 1), null);
  assert.equal(pointerOffsetFromPresentedFrame(frame, rect, Number.NaN, 1), null);
});
test('no mouse event invents facing, and movement does not overwrite a fixed aim', () => {
  const input = new InputController();
  assert.equal(input.sample(1, 1).aimX, 0);
  assert.equal(input.sample(1, 2).aimZ, 0);
  input.pointer(100,0);
  input.keyDown('a'); const left = input.sample(1,3); input.keyUp('a');
  input.keyDown('d'); const right = input.sample(1,4);
  assert.equal(left.aimX,right.aimX); assert.equal(left.aimZ,right.aimZ);
  assert.equal(left.moveX,-right.moveX); assert.equal(left.moveZ,-right.moveZ);
});
test('only exact authored actor previews bind to their runtime spawn', async () => {
  const scene = JSON.parse(await readFile(new URL('../../../content/scenes/compiled/gh_entry_maintenance.json',import.meta.url),'utf8'));
  assert.deepEqual([...boundActorPreviewIds(scene)], [
    ['entry_worker_01_visual','gh_entry_worker_01'],['entry_worker_02_visual','gh_entry_worker_02']]);
  scene.presentation.sprites.push({ id:'decorative_worker_visual',assetId:'runtime2d.enemy.infected_maintenance_worker.v1',layer:'visual.props_dynamic',position:[8,0,4] });
  assert.equal(boundActorPreviewIds(scene).has('decorative_worker_visual'),false);
  scene.spawns.push({...scene.spawns[1]});
  assert.equal(boundActorPreviewIds(scene).has('entry_worker_01_visual'),false);
});
