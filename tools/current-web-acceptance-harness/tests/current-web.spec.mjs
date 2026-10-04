import { test, expect, boot, calls, callCount, inputCheckpoint, observedInputCount, control, enterNew, paused, noDocumentScroll, inViewport } from "./helpers.mjs";

test("built Core UI is shared, keyboard accessible, and honest about absent projections", async ({ page, evidence }) => {
  await boot(page, evidence, { capabilityItems: [
    { capabilityId: "mobility.air_step_i", granted: true, selected: true, cooldownRemainingMs: 0 },
    { capabilityId: "information.local_map_i", granted: false, selected: false, cooldownRemainingMs: 0 },
    { capabilityId: "body.regeneration_i", granted: "true", selected: true },
    { capabilityId: "space.invented_flight", granted: true, selected: true },
  ] });
  await expect(page.locator("#hub .actions button")).toHaveCount(4);
  await page.locator("#core-ui-open").click();
  await expect(page.locator("#core-panel")).toBeVisible();
  await expect(page.locator("#core-panel-close")).toBeFocused();
  const panels = { world_network: "世界网络", capability: "能力", mission: "任务", archive: "档案",
    settings: "系统", inventory: "背包", character: "角色", save: "存档" };
  for (const [id, label] of Object.entries(panels)) {
    await page.locator(`[data-core-panel="${id}"]`).click();
    await expect(page.locator("#core-panel-title")).toHaveText(label);
    await expect(page.locator(`#core-panel [data-core-panel="${id}"]`)).toHaveAttribute("aria-selected", "true");
  }
  await page.locator('[data-core-panel="inventory"]').click();
  await expect(page.locator("#core-panel-content")).toContainText("尚未收到权威背包投影");
  await expect(page.locator("#core-panel-content")).toContainText("尚未收到权威装备投影");
  await expect(page.locator("#core-panel-content button")).toHaveCount(0);
  await page.locator('[data-core-panel="capability"]').click();
  await expect(page.locator(".capability-group")).toHaveCount(10);
  await expect(page.locator(".capability-row")).toHaveCount(1);
  await expect(page.locator(".capability-row")).toContainText("空中踏步");
  await expect(page.locator("#core-panel-content")).toContainText("不提供解锁操作");
  await expect(page.locator("#core-panel-content button")).toHaveCount(0);
  await page.locator('[data-core-panel="archive"]').click();
  await expect(page.locator("#core-panel-content")).toContainText("不会根据未提供的数据生成条目");
  await page.locator('[data-core-panel="world_network"]').click();
  await expect(page.locator("#core-panel-content")).toContainText("后续阶段暂未开放");
  await page.keyboard.press("ArrowRight");
  await expect(page.locator('[data-core-panel="capability"]')).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(page.locator("#core-panel")).toBeHidden();
  await expect(page.locator("#core-ui-open")).toBeFocused();
  await page.locator("#main-settings").click();
  const contrast = page.getByRole("button", { name: "高对比度", exact: true });
  await contrast.click(); await expect(contrast).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("button", { name: "减少动态效果", exact: true }).click();
  await page.locator("#core-panel-close").click();
  await page.reload();
  await expect(page.locator("#new-journey")).toBeEnabled();
  await page.locator("#main-settings").click();
  await expect(contrast).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("button", { name: "减少动态效果", exact: true })).toHaveAttribute("aria-pressed", "true");
});

test("New Journey rejects failure and suppresses duplicate entry before the actual renderer starts", async ({ page, evidence }) => {
  await boot(page, evidence, { withSlots: false });
  await expect(page.locator("#continue-journey")).toBeDisabled();
  await control(page, "failNext", "formal_new", "E_MOCK_NEW_REJECTED");
  await page.locator("#new-journey").click();
  await expect(page.locator("#feedback")).toContainText("E_MOCK_NEW_REJECTED");
  await expect(page.locator("#new-journey")).toBeEnabled();
  await control(page, "block", "formal_new");
  await page.locator("#new-journey").click();
  await expect.poll(async () => await callCount(page, "formal_new")).toBe(2);
  await expect(page.locator("#new-journey")).toBeDisabled();
  await page.locator("#new-journey").dispatchEvent("click");
  expect(await callCount(page, "formal_new")).toBe(2);
  const firstInput = await inputCheckpoint(page);
  await control(page, "release", "formal_new");
  await expect.poll(async () => await observedInputCount(page, firstInput)).toBeGreaterThan(0);
  await expect(page.locator("#hud-scene")).toHaveText("rs_core_room");
  await expect(page.locator("#world-canvas")).toHaveCount(1);
  expect(evidence.requests.some(url => url.includes("/scene-definitions/compiled/return_station/rs_core_room.json"))).toBeTruthy();
  expect(evidence.requests.some(url => url.includes("/governance/assets/RUNTIME_ASSET_MANIFEST.json"))).toBeTruthy();
});

test("New Journey returns to the hub and re-enters exactly one actual renderer", async ({ page, evidence }) => {
  await boot(page, evidence, { withSlots: false });
  await enterNew(page);
  await page.locator("#back-to-hub").click();
  await expect(page.locator("#new-journey")).toBeEnabled();
  await enterNew(page);
  await expect(page.locator("#world-canvas")).toHaveCount(1);
  expect(await callCount(page, "formal_new")).toBe(2);
  expect(await callCount(page, "formal_return")).toBe(1);
});

test("Continue cancels legacy migration, rejects corrupt slots, and recovers an interrupted request", async ({ page, evidence }) => {
  await boot(page, evidence);
  await page.locator("#continue-journey").click();
  await expect(page.locator("#continue-slot option")).toHaveCount(3);
  await expect(page.locator('#continue-slot option[value="corrupt-slot"]')).toBeDisabled();
  await page.locator("#continue-slot").selectOption("legacy-slot");
  page.once("dialog", dialog => dialog.dismiss());
  await page.locator("#continue-selected").click();
  expect(await callCount(page, "formal_continue_slot")).toBe(0);
  await expect(page.locator(".shell")).toHaveAttribute("data-view", "hub");
  await page.locator("#continue-slot").selectOption("valid-slot");
  await control(page, "block", "formal_continue_slot");
  await page.locator("#continue-selected").click();
  await expect.poll(async () => await callCount(page, "formal_continue_slot")).toBe(1);
  await expect(page.locator("#continue-selected")).toBeDisabled();
  await page.locator("#continue-selected").dispatchEvent("click");
  expect(await callCount(page, "formal_continue_slot")).toBe(1);
  await control(page, "failNext", "formal_continue_slot", "E_MOCK_CONTINUE_INTERRUPTED");
  await control(page, "release", "formal_continue_slot");
  await expect(page.locator("#feedback")).toContainText("E_MOCK_CONTINUE_INTERRUPTED");
  await expect(page.locator("#continue-selected")).toBeEnabled();
  const continuedInput = await inputCheckpoint(page);
  await page.locator("#continue-selected").click();
  await expect.poll(async () => await observedInputCount(page, continuedInput)).toBeGreaterThan(0);
  await expect(page.locator("#hud-scene")).toHaveText("gh_entry_maintenance");
  await expect(page.locator(".shell")).toHaveAttribute("data-world", "grey_hive");
  expect(await callCount(page, "formal_continue_slot")).toBe(2);
});

test("pause acknowledgement and reversal respect authority", async ({ page, evidence }) => {
  await boot(page, evidence); await enterNew(page);
  await control(page, "block", "formal_pause");
  await page.locator("#hud-pause").click();
  await expect(page.locator("#pause-title")).toHaveText("正在请求暂停…");
  await expect(page.locator("#save-new-slot")).toBeDisabled();
  await page.locator("#hud-resume").click();
  await control(page, "release", "formal_pause");
  await expect(page.locator("#pause-overlay")).toBeHidden();
  await expect.poll(async () => await callCount(page, "formal_resume")).toBe(1);
});

test("Core UI Escape preserves acknowledged pause without submitting or resuming", async ({ page, evidence }) => {
  await boot(page, evidence); await enterNew(page); await paused(page);
  const inputs = await callCount(page, "formal_submit_input");
  await page.locator("#core-ui-open").click();
  await expect(page.locator("#core-panel")).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.locator("#core-panel")).toBeHidden();
  await expect(page.locator("#pause-title")).toHaveText("旅程已暂停");
  expect(await callCount(page, "formal_submit_input")).toBe(inputs);
  expect(await callCount(page, "formal_resume")).toBe(0);
});

test("legacy overwrite and cancelled save never issue a save command", async ({ page, evidence }) => {
  await boot(page, evidence); await enterNew(page); await paused(page);
  await page.locator("#pause-slot").selectOption("legacy-slot");
  await expect(page.locator("#overwrite-slot")).toBeDisabled();
  await page.locator("#pause-slot").selectOption("valid-slot");
  page.once("dialog", dialog => dialog.dismiss());
  await page.locator("#overwrite-slot").click();
  expect(await callCount(page, "formal_save_slot")).toBe(0);
});

test("a newly saved mock slot survives hub return and Continue with fresh input", async ({ page, evidence }) => {
  await boot(page, evidence); await enterNew(page); await paused(page);
  await page.locator("#new-slot-name").fill("当前 UI 测试存档");
  await page.locator("#save-new-slot").click();
  await expect(page.locator("#save-feedback")).toContainText("已保存「当前 UI 测试存档」");
  const saved = (await calls(page, "formal_save_slot"))[0].args.slotId;
  await page.locator("#hud-resume").click();
  await expect(page.locator("#pause-overlay")).toBeHidden();
  await page.locator("#back-to-hub").click();
  await expect(page.locator("#continue-journey")).toBeEnabled();
  await page.locator("#continue-journey").click();
  await page.locator("#continue-slot").selectOption(saved);
  const continueCheckpoint = await inputCheckpoint(page);
  const inputsBeforeContinue = continueCheckpoint.inputCount;
  await page.locator("#continue-selected").click();
  await expect(page.locator(".shell")).toHaveAttribute("data-view", "journey");
  await expect.poll(async () => await observedInputCount(page, continueCheckpoint)).toBeGreaterThan(inputsBeforeContinue);
  await expect(page.locator("#hud-scene")).toHaveText("rs_core_room");
  // This is in-memory mock save/continue UI coverage, never cross-process Save V6 evidence.
  expect((await calls(page, "formal_continue_slot"))[0].args.slotId).toBe(saved);
});

test("a timed-out Continue stays fail-closed even when its late mock receipt arrives", async ({ page, evidence }) => {
  await boot(page, evidence);
  await page.locator("#continue-journey").click();
  await page.locator("#continue-slot").selectOption("valid-slot");
  await control(page, "block", "formal_continue_slot");
  await page.locator("#continue-selected").click();
  await expect(page.locator("#feedback")).toHaveText("世界操作超时，需退出并重启此程序。", { timeout: 20_000 });
  await expect(page.locator("#new-journey")).toBeDisabled();
  await expect(page.locator("#continue-journey")).toBeDisabled();
  await expect(page.locator("#continue-selected")).toBeDisabled();
  await control(page, "release", "formal_continue_slot");
  await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
  await expect(page.locator(".shell")).toHaveAttribute("data-view", "hub");
  await expect(page.locator("#journey")).toBeHidden();
  expect(await callCount(page, "formal_continue_slot")).toBe(1);
  expect(await callCount(page, "formal_submit_input")).toBe(0);
});

test("720p, 1080p and 1440p hub and Core UI fit without document scroll", async ({ page, evidence }) => {
  await boot(page, evidence);
  await page.locator("#continue-journey").click();
  for (const viewport of [{ width: 1280, height: 720 }, { width: 1920, height: 1080 }, { width: 2560, height: 1440 }]) {
    await page.setViewportSize(viewport); await noDocumentScroll(page);
    await inViewport(page, "#new-journey"); await inViewport(page, "#continue-selected");
    const hubFits = await page.locator("#hub").evaluate(element => element.scrollHeight <= element.clientHeight + 1);
    expect(hubFits).toBeTruthy();
    await page.locator("#core-ui-open").click();
    await page.locator('[data-core-panel="capability"]').click();
    await noDocumentScroll(page); await inViewport(page, "#core-panel-close");
    await page.locator("#core-panel-close").click();
  }
});

test("cold 1440p renderer entry and 720p, 1080p, 1440p resize keep one correctly sized live canvas", async ({ page, evidence }) => {
  await page.setViewportSize({ width: 2560, height: 1440 });
  await boot(page, evidence);
  await enterNew(page); await paused(page);
  for (const viewport of [{ width: 1280, height: 720 }, { width: 1920, height: 1080 }, { width: 2560, height: 1440 }, { width: 1280, height: 720 }]) {
    await page.setViewportSize(viewport); await noDocumentScroll(page);
    await inViewport(page, "#world-canvas"); await inViewport(page, "#hud-resume");
    await expect.poll(() => page.locator("#world-canvas").evaluate(canvas => {
      const box = canvas.getBoundingClientRect();
      return canvas.width > 0 && canvas.height > 0 && Math.abs(canvas.width - box.width) <= 2 && Math.abs(canvas.height - box.height) <= 2;
    })).toBeTruthy();
    await expect(page.locator("#world-canvas")).toHaveCount(1);
  }
});
