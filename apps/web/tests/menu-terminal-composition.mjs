import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";

const main = fs.readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
const presenter = fs.readFileSync(new URL("../src/ui/CoreUiPresenter.ts", import.meta.url), "utf8");

test("main menu exposes the four explicit return-station actions", () => {
  for (const id of ["new-journey", "continue-journey", "main-settings", "exit-game"]) {
    assert.match(main, new RegExp(`id=\\\"${id}\\\"`));
  }
  assert.match(main, /mainSettingsButton\.addEventListener\("click"/);
  assert.match(main, /getCurrentWindow\(\)\.close\(\)/);
  assert.match(main, /预览模式不会关闭窗口/);
});

test("terminal navigation keeps authority-safe legacy entries while prioritizing top-level routes", () => {
  assert.match(main, /role=\"tablist\" aria-label=\"终端主导航\"/);
  assert.match(main, /core-panel-tabs-primary.*data-core-panel=\"world_network\".*data-core-panel=\"capability\".*data-core-panel=\"mission\".*data-core-panel=\"archive\".*data-core-panel=\"settings\"/s);
  assert.match(main, /core-panel-tabs-secondary.*data-core-panel=\"inventory\".*data-core-panel=\"character\".*data-core-panel=\"save\"/s);
  for (const id of ["inventory", "character"]) assert.match(main, new RegExp(`data-core-panel=\\\"${id}\\\"`));
  assert.match(presenter, /ArrowLeft/);
  assert.match(presenter, /aria-selected/);
  assert.match(presenter, /button:not\(\[disabled\]\):not\(\[tabindex=\\"-1\\"\]\)/);
  assert.match(presenter, /select:not\(\[disabled\]\)/);
});
