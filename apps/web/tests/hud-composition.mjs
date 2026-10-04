import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const testsDir = dirname(fileURLToPath(import.meta.url));
const source = await readFile(resolve(testsDir, "../src/main.ts"), "utf8");
const hud = await readFile(resolve(testsDir, "../src/ui/Hud.ts"), "utf8");
const portrait = await readFile(resolve(testsDir, "../src/ui/components/HudPortrait.ts"), "utf8");
const css = await readFile(resolve(testsDir, "../src/visual-authority.css"), "utf8");

test("HUD DOM has stable component regions and keeps journey controls in main", () => {
  assert.match(source, /id="hud-portrait"/);
  assert.match(source, /id="hud-skills"/);
  assert.match(source, /<section class="hud-objectives" id="hud-objectives"[\s\S]*id="hud-objective-list"/);
  assert.match(source, /id="hud-interaction"[^>]*hidden/);
  assert.match(source, /new HudPresenter\(\{/);
  assert.match(source, /runtimeAssetLoader\.load\(\{ signal \}\)/);
  assert.match(source, /hud\.setPortraitRegistry\(registry\)/);
  assert.doesNotMatch(source, /document\.createElement\("img"\)/);
});

test("HUD composition uses approved portrait resolution, three-objective cap, and responsive skill sizing", () => {
  assert.match(hud, /slice\(0, 3\)/);
  assert.match(portrait, /portrait\.cenyao\.v1/);
  assert.match(css, /clamp\(48px, 3\.7vw, 56px\)/);
  assert.match(css, /\.hud-actions \{[\s\S]*right: 50%;[\s\S]*justify-content: center;[\s\S]*transform: translateX\(50%\)/);
  assert.match(css, /1280|1920|2560|clamp\(/);
  assert.match(css, /prefers-reduced-motion/);
});
