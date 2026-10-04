import test from "node:test";
import assert from "node:assert/strict";
import { OCCLUDER_FADE_MS, OccluderFader, pointInPolygon, worldSkin } from "../dist/renderer/WorldSkin.js";

test("four skin profiles preserve distinct world colour treatments", () => {
  const skins = ["return_station", "grey_hive", "mist_harbor", "clockworks"].map(worldSkin);
  assert.equal(new Set(skins.map(skin => skin.background)).size, 4);
  assert.ok(skins[3].floorTint !== skins[1].floorTint);
  assert.throws(() => worldSkin("unreleased_world"), /E_WORLD_SKIN_UNKNOWN/);
});

test("occluder coverage includes edges and interpolates in 200 ms", () => {
  const polygon = [[0, 0], [2, 0], [2, 2], [0, 2]];
  assert.equal(pointInPolygon(1, 1, polygon), true);
  assert.equal(pointInPolygon(0, 1, polygon), true);
  assert.equal(pointInPolygon(3, 1, polygon), false);
  const fader = new OccluderFader();
  assert.equal(fader.alpha("wall", 0.2, false, 1000), 1);
  assert.equal(fader.alpha("wall", 0.2, true, 1000), 1);
  assert.equal(fader.alpha("wall", 0.2, true, 1000 + OCCLUDER_FADE_MS / 2), 0.6);
  assert.equal(fader.alpha("wall", 0.2, true, 1000 + OCCLUDER_FADE_MS), 0.2);
  assert.equal(fader.alpha("wall", 0.2, false, 1200), 0.2);
  assert.equal(fader.alpha("wall", 0.2, false, 1400), 1);
  fader.forget("wall");
  assert.equal(fader.alpha("wall", 0.2, true, 1500), 0.2);
  fader.clear();
  assert.throws(() => fader.alpha("x", 2, false, 1500), /E_OCCLUDER_FADE_INPUT/);
});
