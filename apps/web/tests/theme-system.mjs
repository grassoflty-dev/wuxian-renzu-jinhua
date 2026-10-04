import assert from "node:assert/strict";
import test from "node:test";
import { ThemeManager, resolveTheme } from "../dist/ui/theme/ThemeManager.js";
import { isThemeSchema } from "../dist/ui/theme/ThemeSchema.js";
import { THEMES, WORLD_THEME_IDS } from "../dist/ui/themes/index.js";

function makeTarget() {
  const properties = new Map();
  return {
    dataset: { contrast: "high", motion: "reduced" },
    style: { setProperty(name, value) { properties.set(name, value); } },
    properties,
  };
}

test("every registered theme has a valid shared schema", () => {
  assert.deepEqual(Object.keys(THEMES).sort(), ["base", "clockworks", "grey_hive", "mist_harbor"]);
  for (const [id, theme] of Object.entries(THEMES)) {
    assert.equal(isThemeSchema(theme), true, `${id} schema`);
    assert.equal(theme.id, id);
  }
  assert.equal(isThemeSchema({ ...THEMES.base, colors: { ...THEMES.base.colors, accent: "amber" } }), false);
});

test("world snapshots hot-switch tokens on the same shared UI target", () => {
  const target = makeTarget();
  const manager = new ThemeManager();
  const first = manager.applyWorld("return_station", target);
  assert.equal(first.id, "base");
  assert.equal(target.properties.get("--theme-background"), THEMES.base.colors.background);

  for (const worldId of ["grey_hive", "mist_harbor", "clockworks"]) {
    const theme = manager.applyWorld(worldId, target);
    assert.equal(target.dataset.theme, theme.id);
    assert.equal(target.properties.get("--theme-system"), theme.colors.system);
    assert.equal(target.dataset.iconStyle, theme.iconStyle);
    assert.equal(target.dataset.mapStyle, theme.mapStyle);
    assert.equal(target.dataset.interactionStyle, theme.interactionStyle);
  }
  assert.equal(manager.activeThemeId, "clockworks");
  manager.applyWorld("return_station", target);
  assert.equal(manager.activeThemeId, "base");
  assert.equal(target.properties.get("--theme-background"), THEMES.base.colors.background);
  assert.equal(target.dataset.contrast, "high");
  assert.equal(target.dataset.motion, "reduced");
});

test("unknown world identifiers safely resolve to base without claiming a world theme", () => {
  const target = makeTarget();
  const theme = resolveTheme("future_world_not_registered");
  assert.equal(theme.id, "base");
  new ThemeManager().applyWorld("future_world_not_registered", target);
  assert.equal(target.dataset.theme, "base");
  assert.equal(target.properties.get("--theme-background"), THEMES.base.colors.background);
  assert.equal(WORLD_THEME_IDS.future_world_not_registered, undefined);
});
