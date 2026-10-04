import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const testsDir = dirname(fileURLToPath(import.meta.url));
const source = await readFile(resolve(testsDir, "../src/main.ts"), "utf8");
const rendererSource = await readFile(resolve(testsDir, "../src/renderer/WorldRenderer.ts"), "utf8");
const clientSource = await readFile(resolve(testsDir, "../src/bridge/tauri-client.ts"), "utf8");
const enhancementHudSource = await readFile(resolve(testsDir, "../src/ui/EnhancementStatusHud.ts"), "utf8");
const coreUiSource = await readFile(resolve(testsDir, "../src/ui/CoreUiPresenter.ts"), "utf8");

test("journey entry verifies the runtime manifest before initializing and rendering its first snapshot", () => {
  assert.match(source, /import \{ RuntimeAssetLoader \} from "\.\/assets\/RuntimeAssetLoader\.js"/);
  assert.match(source, /const registry = await runtimeAssetLoader\.load\(\{ signal \}\)/);
  assert.match(source, /sceneSession = new SceneDefinitionSession\(renderer, sceneDefinitionLoader\)/);
  assert.match(source, /await sceneSession\.prepare\(snapshot, signal\)/);
  assert.match(source, /sessionLoop = new SessionLoop\(client, sceneSession/);
  assert.match(source, /await sessionLoop\.start\(snapshot\)/);
  assert.match(source, /sceneSession\?\.acceptSnapshot\(latest\)/);
  assert.doesNotMatch(source, /new AssetRegistry\(\)/);
});

test("only Rust-accepted presentation events feed temporary audio and lifecycle hooks silence playback", () => {
  assert.match(source, /import \{ AudioCuePlayer \} from "\.\/audio\/AudioCuePlayer\.js"/);
  assert.match(source, /new CoreUiPresenter\(root\.querySelector<HTMLElement>\("#core-panel"\)!, accessibilityPreferences, audioCuePlayer,\s*async \(action, source\) =>/);
  assert.match(source, /onEvents:\s*events\s*=>\s*\{\s*audioCuePlayer\.handlePresentationEvents\(events\);\s*\}/);
  assert.match(source, /audioCuePlayer\.setEpoch\(latest\.worldEpoch\)/);
  assert.match(source, /if \(status === "running"\) audioCuePlayer\.resume\(\);\s*else audioCuePlayer\.suspend\(\)/);
  assert.match(source, /audioCuePlayer\.setScene\(snapshot\.worldId, snapshot\.sceneId\)/);
  assert.match(source, /void audioCuePlayer\.dispose\(\)/);
  assert.match(source, /audioCuePlayer\.playUiConfirm\(\)/);
  assert.match(source, /audioCuePlayer\.playUiError\(\)/);
  assert.match(coreUiSource, /volume\.type = "range"/);
  assert.match(coreUiSource, /volume\.min = "0"/);
  assert.match(coreUiSource, /volume\.max = "1"/);
  assert.match(coreUiSource, /mute\.textContent = "静音"/);
  assert.match(coreUiSource, /this\.audioCuePlayer\.setMuted\(!this\.audioCuePlayer\.settings\.muted\)/);
  assert.match(coreUiSource, /{ label: "高对比度", key: "highContrast" }/);
  assert.match(coreUiSource, /{ label: "减少动态效果", key: "reducedMotion" }/);
});

test("a fresh Pixi canvas is connected before each journey renderer is initialized", () => {
  assert.match(source, /let canvas = root\.querySelector<HTMLCanvasElement>\("#world-canvas"\)!/);
  assert.match(source, /const canvasHost = canvas\.parentElement!/);
  assert.match(source, /if \(!canvas\.isConnected\) \{\s*const replacement = canvas\.cloneNode\(false\) as HTMLCanvasElement;\s*canvasHost\.append\(replacement\);\s*canvas = replacement;\s*\}\s*await renderer\.init\(canvas\)/);
});

test("uncertain native continue operations time out and disable repeated journey commands", () => {
  assert.match(clientSource, /CONTINUE_SLOT_TIMEOUT_MS = 15_000/);
  assert.match(clientSource, /E_CONTINUE_SLOT_TIMEOUT/);
  assert.match(source, /error\.message === "E_CONTINUE_SLOT_TIMEOUT" \|\|\s*error\.message === "E_CONTINUE_TIMEOUT" \|\| error\.message === "E_SESSION_STOP_UNCERTAIN_MUTATION"\)\) nativeJourneyUncertain = true/);
  assert.match(source, /error\.message === "E_SESSION_STOP_TIMEOUT"\) nativeJourneyUncertain = true/);
  assert.match(source, /nativeJourneyUncertain \|\| !isTauri\(\)/);
});

test("production renderer binds selected frames, swaps texture on facing changes, and clears epoch resources", () => {
  assert.match(rendererSource, /this\.ensureSprite\(item\.key, item\.asset, layerId, item\.frameSelection, item\.displayScale\)/);
  assert.match(rendererSource, /viewportPixelsPerMeter\(this\.app\.screen\.width, this\.app\.screen\.height\)/);
  assert.match(rendererSource, /resolveSpriteDisplayScale\(asset, frame\[2\], frame\[3\]/);
  assert.match(rendererSource, /previous\.sprite\.texture = nextTexture/);
  assert.match(rendererSource, /previous\.sprite\.anchor\.set\(frame\.anchor\[0\], frame\.anchor\[1\]\)/);
  assert.match(rendererSource, /E_RENDERER_ATLAS_NOT_PRELOADED/);
  assert.match(rendererSource, /if \(this\.sceneEpoch\.enter\(scene\)\) await this\.clearSceneResources\(\)/);
  assert.match(rendererSource, /for \(const texture of this\.frameTextures\.values\(\)\) texture\.destroy\(false\)/);
  assert.match(rendererSource, /const unloading = retireUnownedAtlas\(url\)/);
  assert.match(rendererSource, /await Promise\.all\(retirement\)/);
});

test("authoritative scene changes blank the old renderer and invalidate frames during atlas loading", () => {
  assert.match(rendererSource, /expectSceneIdentity\(identity: \{ worldId: string; sceneId: string; worldEpoch: number \}\)/);
  assert.match(rendererSource, /this\.sceneInvalidationCleanup = this\.clearSceneResources\(\)/);
  assert.match(rendererSource, /await this\.sceneInvalidationCleanup/);
  assert.match(rendererSource, /renderRevision !== this\.sceneRequestRevision/);
  assert.match(rendererSource, /E_SCENE_PRESENTATION_IDENTITY_MISMATCH/);
});

test("returning to the hub stops the session loop before the authoritative hub transition", () => {
  assert.match(source, /await sessionLoop\?\.stop\("hub"\);\s*if \(requestId !== journeyRequestId\) return;\s*const snapshot = await client\.returnToHub\(\)/);
});

test("journey entry pins the scene manifest and cancels pending loads on pagehide", () => {
  assert.match(source, /expectedManifestSha256: __SCENE_DEFINITION_MANIFEST_SHA256__/);
  assert.match(source, /window\.addEventListener\("pagehide"/);
  assert.match(source, /activeJourneyLoad\?\.abort\(\)/);
});

test("Arena pressure wave events reach only the dedicated renderer projection", () => {
  assert.match(source, /acceptPressureWavePresentationEvents\(events, latest\)/);
  assert.match(source, /ForgedGuardPressureWindup/);
  assert.match(source, /ForgedGuardPressureImpact/);
  assert.match(source, /PrimeRegulatorPressureWindup/);
  assert.match(source, /PrimeRegulatorPressureImpact/);
  assert.match(rendererSource, /acceptPressureWavePresentationEvents/);
  assert.match(rendererSource, /frame\.kind\.endsWith\("Windup"\)/);
  assert.match(rendererSource, /temporary_visual:clockworks-pressure-wave/);
});

test("Prime Regulator identity mark is temporary, snapshot-authorized, static, and cleared on identity changes", () => {
  assert.match(rendererSource, /projectPrimeRegulatorBossMark\(snapshot, reduceFogMotion\)/);
  assert.match(rendererSource, /mark\.label = "temporary_visual=true:prime-regulator-boss-mark"/);
  assert.match(rendererSource, /mark\.alpha = frame\.reducedMotion \? 0\.78 : 0\.9/);
  assert.match(rendererSource, /expectSceneIdentity\(identity:[\s\S]*?this\.clearTransientPresentation\(\)/);
  assert.match(rendererSource, /clearTransientPresentation\(\): void \{[\s\S]*?this\.clearRegulatorBossMark\(\)/);
  assert.match(rendererSource, /if \(!frame\) \{\s*this\.clearRegulatorBossMark\(\);/);
});

test("journey HUD is driven by initial and accepted v3 snapshots and cleared on every exit path", () => {
  assert.match(source, /hud\.apply\(snapshot\)/);
  assert.match(source, /if \(latest\.protocolVersion === 3\) \{[\s\S]*?if \(hudIdentity !== nextHudIdentity\) \{[\s\S]*?hud\.reset\(\);[\s\S]*?hud\.apply\(latest\)/);
  assert.match(source, /hud\.reset\(\);/);
  assert.match(source, /onPauseState: \(status, message\) => \{\s*if \(sessionLoop !== ownedLoop\) return;\s*hud\.setPauseStatus\(status, message\);/);
  assert.match(source, /onInteract: latest => interactFromSnapshot\(latest\)/);
  assert.match(clientSource, /"formal_interact"/);
  assert.match(source, /snapshot\.interactables\.find\(item => item\.entityId === state\.interactionId\)/);
  assert.match(source, /dispatchInteractable\(client, interactable, snapshot\.worldEpoch,\s*\{ worldId: snapshot\.worldId, sceneId: snapshot\.sceneId \}\)/);
  assert.match(source, /interactionErrorText\(result\.errorCode \|\| "E_INTERACTION_NOT_APPLIED"\)/);
  assert.match(source, /const result = await dispatchInteractable\(client, interactable, snapshot\.worldEpoch,\s*\{ worldId: snapshot\.worldId, sceneId: snapshot\.sceneId \}\)/);
  assert.match(source, /await activeLoop\.acceptAuthoritativeSnapshot\(result\.snapshot\)/);
  assert.match(source, /isCurrentSceneInteractionResult\(sessionLoop === activeLoop && activeLoop\.acceptsExternalResults/);
});

test("pause controls queue authoritative transitions and leave renderer session shutdown ordered", () => {
  assert.match(source, /sessionLoop\.pausePresentationState === "error"\) void sessionLoop\.retryPauseState\(\)/);
  assert.match(source, /else if \(sessionLoop\.isPaused\) void sessionLoop\.resume\(\)/);
  assert.match(source, /await sessionLoop\?\.stop\("hub"\);\s*if \(requestId !== journeyRequestId\) return;\s*const snapshot = await client\.returnToHub\(\)/);
  assert.match(source, /if \(sessionLoop\) void sessionLoop\.stop\("unload"\)/);
});

test("first-clear choices live in a dedicated terminal modal with contextual pause and stale receipt guards", async () => {
  const evolution = await readFile(new URL("../src/game/FirstEvolution.ts", import.meta.url), "utf8");
  assert.match(source, /id="first-enhancement"[^>]*role="dialog"/);
  assert.match(source, /evolution.open/);
  assert.match(evolution, /worldId !== "return_station"/);
  assert.match(evolution, /sceneId !== "rs_core_room"/);
  assert.match(evolution, /firstEnhancementChoice === null/);
  assert.match(evolution, /await owner.loop.pause/);
  assert.match(evolution, /owner.loop.runWhilePaused/);
  assert.match(evolution, /pauseCommandSequence/);
  assert.match(evolution, /E_ENHANCEMENT_RECEIPT_STALE_SESSION/);
  assert.match(clientSource, /"formal_choose_first_enhancement", \{ capabilityId, context:/);
});

test("selected enhancement HUD follows initial, Continue, and accepted snapshots, then clears on identity or session exit", () => {
  assert.match(source, /enhancementStatusHud\.apply\(snapshot\)/);
  assert.match(source, /enhancementStatusHud\.apply\(latest\)/);
  assert.match(source, /if \(hudIdentity !== nextHudIdentity\) \{\s*hud\.reset\(\);\s*enhancementStatusHud\.reset\(\);/);
  assert.match(source, /await enterJourney\(receipt\.snapshot, controller\.signal, requestId\)/);
  assert.match(source, /enhancementStatusHud\.reset\(\);\s*audioCuePlayer\.suspend\(\);\s*await sessionLoop\?\.stop\("hub"\);\s*if \(requestId !== journeyRequestId\) return;\s*const snapshot = await client\.returnToHub\(\);\s*if \(requestId !== journeyRequestId\) return;\s*audioCuePlayer\.setEpoch\(snapshot\.worldEpoch\);\s*audioCuePlayer\.setScene\(snapshot\.worldId, snapshot\.sceneId\);\s*audioCuePlayer\.resume\(\);\s*hud\.reset\(\);\s*enhancementStatusHud\.reset\(\)/);
  assert.match(source, /window\.addEventListener\("pagehide"[\s\S]*?enhancementStatusHud\.reset\(\)/);
  assert.match(enhancementHudSource, /firstEnhancementChoice/);
  assert.match(enhancementHudSource, /world\.completed === true/);
});


test("Build UI transport shares session ownership and guards hub entry and closing", () => {
  assert.match(source,/loop\.runWithSession\(\(\) => client\.buildCommand\(action, source\)\)/);
  assert.match(source,/pendingHubBuild = operation/);
  assert.match(source,/await waitForHubBuild\(\);\s*assertJourneyRequest\(controller\.signal, requestId\);\s*const snapshot = await client\.newJourney\(\)/);
  assert.match(source,/await waitForHubBuild\(\);\s*assertJourneyRequest\(controller\.signal, requestId\);\s*const receipt = latestSave \? \{ snapshot: await client\.continueJourney\(\) \}\s*: await client\.continueSlot/);
  assert.match(source,/buildCloseBarrier\.close\(async \(\) =>/);
  assert.match(source,/if \(busy \|\| buildCloseBarrier\.closing \|\| nativeJourneyUncertain\) throw/);
  assert.match(source,/if \(busy \|\| buildCloseBarrier\.closing \|\| nativeJourneyUncertain \|\| !isTauri\(\)\) return/);
  assert.match(source,/coreUi\.currentSnapshot, source, receipt/);
});


test("detached loop callbacks cannot publish pause or errors over the current journey", () => {
  assert.match(source, /onError: error => \{\s*if \(sessionLoop !== ownedLoop\) return;\s*if \(ownedLoop.hasUnsettledStopMutation\) nativeJourneyUncertain = true;\s*showError\(error\);/);
});
