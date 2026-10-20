import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const rendererUrl = new URL("../apps/godot_playable/scripts/world/renderer.gd", import.meta.url);
const helperUrl = new URL("../apps/godot_playable/scripts/world/grey_hive_state_props_visual.gd", import.meta.url);
const sceneUrl = new URL("../apps/godot_playable/data/scenes/gh_power_room.json", import.meta.url);

test("console culling rescue is limited to the resolved canonical Power Room body", async () => {
  const [renderer, helper, sceneText] = await Promise.all([
    readFile(rendererUrl, "utf8"),
    readFile(helperUrl, "utf8"),
    readFile(sceneUrl, "utf8"),
  ]);
  const start = renderer.indexOf("func _object_culling_state(");
  const end = renderer.indexOf("\nfunc _gh_bio_log_art_caption_offset(", start);
  assert.ok(start >= 0 && end > start);
  const culling = renderer.slice(start, end);
  const rescueAt = culling.indexOf("if o.get('kind')=='gh_console_body'");
  const doorAt = culling.indexOf("if o.get('kind')=='door'");
  const genericAt = culling.indexOf("viewport_rect.grow(100).has_point(at)");
  assert.ok(rescueAt >= 0 && rescueAt < doorAt && doorAt < genericAt);
  const rescue = culling.slice(rescueAt, doorAt);
  assert.match(rescue, /P\.snapshot\(snapshot\)/);
  assert.match(rescue, /snapshot\.has\('entryToken'\)/);
  assert.match(rescue, /snapshot\.get\('entryToken'\)==null/);
  assert.match(rescue, /P\.counter\(snapshot\.get\('worldEpoch'\),1\)/);
  assert.match(rescue, /current_key=='grey_hive:gh_power_room:'\+str\(snapshot\.get\('worldEpoch'\)\)/);
  assert.match(rescue, /_gh_visual_row\('power_console_off_on','runtime2d\.prop\.power_console\.off_on\.v1',\[4,0,7\]\)/);
  assert.match(rescue, /scene\.get\('worldId'\)=='grey_hive'.*scene\.get\('sceneId'\)=='gh_power_room'/);
  assert.match(rescue, /o\.p==Vector3\(4,0,7\)/);
  assert.match(rescue, /_gh_console_body\(gh_power_console_row\)/);
  assert.match(rescue, /gh_power_console_resolved==o/);
  assert.match(rescue, /canvas_rect\('power_console',gh_power_console_state,at,camera\.ppm\)/);
  assert.match(rescue, /_rect_intersects\(viewport_rect,gh_power_console_rect\)/);
  assert.match(culling, /return \{'furnaceKey':'','culled':not viewport_rect\.grow\(100\)\.has_point\(at\)/);

  const scene = JSON.parse(sceneText);
  assert.equal(scene.worldId, "grey_hive");
  assert.equal(scene.sceneId, "gh_power_room");
  const consoleRows = scene.presentation.sprites.filter((row) => row.id === "power_console_off_on");
  assert.equal(consoleRows.length, 1);
  assert.equal(consoleRows[0].assetId, "runtime2d.prop.power_console.off_on.v1");
  assert.equal(consoleRows[0].layer, "visual.props_dynamic");
  assert.deepEqual(consoleRows[0].position, [4, 0, 7]);

  const configText = helper.match(/^const CONFIG = (.+)$/m)?.[1];
  assert.ok(configText);
  const config = JSON.parse(configText);
  assert.equal(config.power_console.heightM, 1.45);
  assert.deepEqual(config.power_console.states.off, [24, 2, 718, 918]);
  assert.deepEqual(config.power_console.states.on, [752, 2, 696, 918]);
  assert.match(helper, /func canvas_rect\(key: String, state: String, foot: Vector2, ppm: float\)/);
  assert.match(helper, /float\(c\.heightM\)\*ppm\/float\(region\[3\]\)/);
  assert.match(rescue, /gh_power_console_state_value in \['off','on'\]/);
});

test("both registered console crops overlap the admitted edge while its fallback does not", async () => {
  const helper = await readFile(helperUrl, "utf8");
  const configText = helper.match(/^const CONFIG = (.+)$/m)?.[1];
  assert.ok(configText);
  const config = JSON.parse(configText);
  const viewport = { width: 6832, height: 3840 };
  const foot = { x: -120, y: 1944 };
  const ppm = 256;

  for (const state of ["off", "on"]) {
    const [, , width, height] = config.power_console.states[state];
    const factor = config.power_console.heightM * ppm / height;
    const left = foot.x - width * factor / 2;
    const top = foot.y - height * factor;
    assert.ok(left < 0 && left + width * factor > 0, `${state} artwork should cross the left edge`);
    assert.ok(top < viewport.height && top + height * factor > 0, `${state} artwork should remain vertically visible`);
  }

  const placeholderScale = ppm / 48;
  const placeholderLeft = foot.x - 16 * placeholderScale;
  const placeholderRight = placeholderLeft + 32 * placeholderScale;
  assert.ok(foot.x < -100, "the old 100px foot-point margin culls this anchor");
  assert.ok(foot.y >= -100 && foot.y <= viewport.height + 100);
  assert.ok(placeholderRight < 0, "the generic placeholder is wholly outside the viewport");
});
