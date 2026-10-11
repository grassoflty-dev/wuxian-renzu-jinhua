(function (root) {
  "use strict";

  const byId = id => document.getElementById(id);
  const invoke = root.__TAURI__?.core?.invoke?.bind(root.__TAURI__?.core);
  const runtime = { view: null, rendererStarted: false, timer: 0, seq: 0, inFlight: false, clientTimeMs: 0, combat: [], requestId: 0, keys: new Set(), panel: null };
  const unavailable = "此入口尚未接入 Freeze v0.2 权威运行时；不会回退到旧版地图、存档或 HTTP 接口。";

  function notify(message, error = false) {
    const region = byId("toastRegion");
    if (!region) return;
    region.textContent = message;
    region.classList.toggle("is-error", error);
    region.classList.add("is-visible");
    clearTimeout(notify.timer);
    notify.timer = setTimeout(() => region.classList.remove("is-visible"), 4200);
  }

  function requireInvoke() {
    if (typeof invoke !== "function") throw new Error("E_TAURI_INVOKE_UNAVAILABLE");
    return invoke;
  }

  function fullEnvelope(view) {
    if (!view || typeof view !== "object") throw new Error("E_FORMAL_VIEW_MISSING");
    return {
      kind: "full", schemaVersion: view.schemaVersion, worldId: view.worldId,
      worldEpoch: view.worldEpoch, serverTick: view.serverTick,
      authorityRevision: view.authorityRevision, ackSeq: view.ackSeq, view
    };
  }

  function consume(view) {
    const result = root.FormalView.consume(fullEnvelope(view));
    if (!result.accepted) throw new Error(`E_FORMAL_VIEW_REJECTED:${result.reason}`);
    runtime.view = view;
    updateHud(view);
    return view;
  }

  async function requestSnapshot() {
    const view = await requireInvoke()("formal_snapshot");
    try { consume(view); } catch (_) { /* A full snapshot may be equal to the last accepted revision. */ }
    return view;
  }

  function updateHud(view) {
    const player = view.player;
    const hp = player.currentHp;
    const maxHp = player.maxHp;
    byId("hpValue").textContent = `${hp} / ${maxHp}`;
    byId("hpMeter").style.width = `${maxHp ? Math.max(0, Math.min(100, hp / maxHp * 100)) : 0}%`;
    byId("energyValue").textContent = "— / —";
    byId("medkitValue").textContent = "未接入";
    byId("rewardPointsWorldValue").textContent = "未接入";
    const p = player.transform.positionM;
    const powered = view.doors?.some(door => door.doorId === "gh_gate_a" && door.open) || false;
    const inShaft = p.zM >= 10;
    const nearConsole = !powered && view.actors?.some(actor => actor.entityId === "gh_power_console" && actor.active
      && Math.hypot(p.xM - actor.transform.positionM.xM, p.yM - actor.transform.positionM.yM, p.zM - actor.transform.positionM.zM) <= 2.5);
    byId("worldTitle").textContent = "灰巢设施 · 动力间首段";
    byId("worldKicker").textContent = "Rust 权威 · 灰巢可玩切片";
    byId("objectiveTitle").textContent = inShaft ? "主轴入口已抵达" : powered ? "通过 Gate A" : "恢复主电";
    byId("objectiveList").innerHTML = inShaft
      ? "<li>已进入中央主轴入口</li><li>阶段进度已纳入 Save v3</li>"
      : powered
        ? "<li>主电已恢复</li><li>Gate A 已开放；沿通道 +Z 前进</li>"
        : "<li>前往动力台并按 F 启动主电</li><li>Gate A 关闭时无法通行</li>";
    byId("nearbyHint").textContent = nearConsole
      ? "[F] 启动主配电台"
      : powered
        ? "Gate A 已开放 · 前往中央主轴入口"
        : "动力间 · 靠近主配电台以操作";
    byId("aimHint").textContent = "WASD / 方向键移动 · F 交互 · J 近战 · Space 跳跃 · Shift 闪避";
    byId("stageZoneLabel").textContent = inShaft ? "灰巢 / 中央主轴入口" : powered && p.zM >= 6 ? "灰巢 / Gate A 通道" : "灰巢 / 动力间";
    byId("stageCoords").textContent = `坐标 ${p.xM.toFixed(1)}, ${p.yM.toFixed(1)}, ${p.zM.toFixed(1)}`;
    const map = view.capabilities?.exploredMap;
    byId("worldView").dataset.serverTick = String(view.serverTick);
    byId("worldView").dataset.authorityRevision = String(view.authorityRevision);
    byId("worldView").dataset.ackSeq = String(view.ackSeq);
    byId("mapTitle").textContent = map?.worldId || "灰巢设施";
    byId("connectionBadge").textContent = "Rust 权威运行时";
    byId("connectionBadge").classList.remove("status-chip--pending");
  }

  function showScreen(name) {
    for (const section of document.querySelectorAll("[data-view]")) section.hidden = section.dataset.view !== name;
    const app = byId("app");
    app.dataset.gameMode = name;
  }

  function closePanels() {
    for (const panel of document.querySelectorAll(".side-panel")) panel.hidden = true;
    runtime.panel = null;
  }

  function openPanel(id) {
    closePanels();
    const panel = byId(id);
    if (!panel) return notify(unavailable, true);
    panel.hidden = false;
    runtime.panel = panel;
  }

  function wrapView(view) {
    const result = root.FormalView.consume(fullEnvelope(view));
    if (!result.accepted) throw new Error(`E_FORMAL_VIEW_REJECTED:${result.reason}`);
    runtime.view = view;
    updateHud(view);
  }

  function stopRuntime() {
    clearInterval(runtime.timer);
    runtime.timer = 0;
    runtime.inFlight = false;
    runtime.keys.clear();
    if (byId("saveButton")) byId("saveButton").disabled = true;
    if (runtime.rendererStarted) {
      root.FreezeV02RendererBoot.stop();
      runtime.rendererStarted = false;
    }
  }

  async function submitTick() {
    if (!runtime.view || runtime.inFlight || byId("worldView").hidden) return;
    runtime.inFlight = true;
    const axis = key => runtime.keys.has(key) ? 1 : 0;
    const moveX = axis("d") + axis("ArrowRight") - axis("a") - axis("ArrowLeft");
    const moveZ = axis("w") + axis("ArrowUp") - axis("s") - axis("ArrowDown");
    const now = Math.max(runtime.clientTimeMs + 1, Math.floor(performance.now()));
    runtime.clientTimeMs = now;
    const sample = {
      protocol: "continuous-input", worldEpoch: runtime.view.worldEpoch,
      seq: ++runtime.seq, clientTimeMs: now,
      moveX: Math.max(-1, Math.min(1, moveX)), moveZ: Math.max(-1, Math.min(1, moveZ))
    };
    const combat = runtime.combat.splice(0);
    try {
      const view = await requireInvoke()("formal_submit_input", { sample, combat });
      wrapView(view);
    } catch (error) {
      runtime.keys.clear();
      notify(`正式输入已停止：${String(error.message || error)}`, true);
    } finally {
      runtime.inFlight = false;
    }
  }

  async function startGreyHive(continueSaved = false) {
    stopRuntime();
    closePanels();
    try {
      const view = await requireInvoke()(continueSaved ? "formal_continue" : "formal_new");
      runtime.view = view;
      runtime.seq = view.ackSeq || 0;
      runtime.clientTimeMs = view.serverTimeMs || 0;
      runtime.combat = [];
      showScreen("world");
      await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      root.FreezeV02RendererBoot.start({
        canvas: byId("gameCanvas"), mapCanvas: byId("mapCanvas"),
        quality: { preset: "low", shadows: false, effects: false, renderScale: 1 },
        submitInput: submitTick, requestFullSnapshot: requestSnapshot,
        worldId: view.worldId, worldEpoch: view.worldEpoch
      });
      runtime.rendererStarted = true;
      consume(view);
      runtime.timer = setInterval(submitTick, 50);
      byId("saveButton").disabled = false;
      notify(continueSaved ? "已继续上次保存的灰巢 blockout 行动。" : "已创建新的灰巢 blockout 行动。仅包含开放房间与 Rust 连续移动。", false);
    } catch (error) {
      stopRuntime();
      showScreen("hub");
      const message = `正式入口失败关闭：${String(error.message || error)}`;
      byId("hubSubtitle").textContent = message;
      notify(message, true);
    }
  }

  function unavailableAction(label) { notify(`${label}暂不可用。${unavailable}`, true); }

  function queueCombat(kind) {
    runtime.combat.push({ kind, requestId: ++runtime.requestId });
    if (!runtime.timer) void submitTick();
  }

  async function interactNearby() {
    if (!runtime.view || byId("worldView").hidden) return;
    const actor = runtime.view.actors?.find(item => item.entityId === "gh_power_console" && item.active);
    if (!actor) return notify("当前没有可操作的目标。", true);
    try {
      const response = await requireInvoke()("formal_interact", {
        actorId: actor.entityId,
        requestId: `power-${runtime.view.worldEpoch}-${++runtime.requestId}-${Math.floor(performance.now())}`
      });
      wrapView(response.view);
      if (response.errorCode) notify(response.errorCode === "E_INTERACTION_OUT_OF_RANGE" ? "靠近主配电台后再操作。" : `交互失败：${response.errorCode}`, true);
      else if (response.applied) notify("主电已恢复，Gate A 开始开放。", false);
      else if (response.alreadyApplied) notify("主电已经恢复，Gate A 保持开放。", false);
    } catch (error) {
      notify(`交互失败：${String(error.message || error)}`, true);
    }
  }

  function handleAction(action) {
    switch (action) {
      case "pause": byId("pauseModal").hidden = false; runtime.keys.clear(); break;
      case "resume-pause": byId("pauseModal").hidden = true; break;
      case "retreat": byId("pauseModal").hidden = true; unavailableAction("安全撤离 / 归航站"); break;
      case "go-hub": stopRuntime(); closePanels(); showScreen("hub"); break;
      case "attack": queueCombat("attack"); break;
      case "dash": queueCombat("dash"); break;
      case "jump": queueCombat("jump"); break;
      case "interact": void interactNearby(); break;
      case "heal": unavailableAction("医疗"); break;
      case "extract": unavailableAction("撤离结算"); break;
      case "continue": void startGreyHive(true); break;
      case "retry": unavailableAction("死亡重试"); break;
      case "retry-view": void startupProbe(); break;
      case "retry-renderer": unavailableAction("渲染器重试；请先重新启动行动"); break;
      default: unavailableAction(action || "此操作");
    }
  }

  async function startupProbe() {
    const badge = byId("connectionBadge");
    badge.textContent = "连接 FormalRuntime…";
    try {
      const view = await requestSnapshot();
      const hasSave = await requireInvoke()("formal_has_save");
      byId("hubSubtitle").textContent = `Rust FormalRuntime 已连接。当前世界：${view.worldId}；正式 Save v3 可用。`;
      byId("hubSaveNote").textContent = hasSave ? "检测到可验证的正式连续世界存档。" : "尚无正式连续世界存档；旧版 Grid 存档不会被读取。";
      byId("continueButton").disabled = !hasSave;
      byId("continueButton").textContent = hasSave ? "继续任务" : "继续任务（暂无存档）";
      byId("saveBadge").textContent = hasSave ? "正式存档可继续" : "暂无正式存档";
      byId("newGameButton").disabled = false;
    } catch (error) {
      badge.textContent = "运行时未连接";
      byId("hubSubtitle").textContent = `入口失败关闭：${String(error.message || error)}`;
      byId("newGameButton").disabled = true;
      byId("continueButton").disabled = true;
      byId("saveButton").disabled = true;
    }
  }

  function wire() {
    showScreen("hub");
    byId("continueButton").disabled = true;
    byId("saveButton").disabled = true;
    byId("saveButton").addEventListener("click", async () => {
      try {
        await requireInvoke()("formal_save");
        byId("saveBadge").textContent = "正式存档已保存";
        notify("连续世界存档已安全写入并验证。", false);
      } catch (error) {
        notify(`保存失败：${String(error.message || error)}`, true);
      }
    });
    byId("newGameButton").addEventListener("click", () => void startGreyHive());
    byId("continueButton").addEventListener("click", () => void startGreyHive(true));
    byId("audioToggle").addEventListener("click", event => {
      const pressed = event.currentTarget.getAttribute("aria-pressed") !== "true";
      event.currentTarget.setAttribute("aria-pressed", String(pressed));
    });
    for (const button of document.querySelectorAll("[data-action]")) {
      button.addEventListener("click", event => {
        event.preventDefault();
        handleAction(button.dataset.action);
      });
    }
    for (const button of document.querySelectorAll("[data-panel]")) {
      button.addEventListener("click", () => {
        const panel = button.dataset.panel;
        if (panel === "archivePanel") openPanel(panel);
        else unavailableAction(panel.replace("Panel", ""));
      });
    }
    for (const button of document.querySelectorAll("[data-close-panel]")) button.addEventListener("click", closePanels);
    for (const button of document.querySelectorAll("[data-enter-world]")) {
      button.addEventListener("click", () => {
        if (button.dataset.enterWorld === "grey_hive") void startGreyHive();
        else unavailableAction(`进入 ${button.dataset.enterWorld}`);
      });
    }
    byId("mapToggle").addEventListener("click", () => {
      const collapsed = byId("mapBody").hidden = !byId("mapBody").hidden;
      byId("mapToggle").setAttribute("aria-expanded", String(!collapsed));
    });
    byId("gameCanvas").addEventListener("click", () => queueCombat("attack"));
    byId("outcomeRetry").disabled = true;
    document.addEventListener("keydown", event => {
      if (event.repeat) return;
      const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
      if (["w", "a", "s", "d", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(key)) {
        if (!byId("worldView").hidden && byId("pauseModal").hidden) { runtime.keys.add(key); event.preventDefault(); }
      } else if (key === "Escape" && !byId("worldView").hidden) {
        byId("pauseModal").hidden = !byId("pauseModal").hidden;
        if (!byId("pauseModal").hidden) runtime.keys.clear();
      } else if (key === "j" && !byId("worldView").hidden) queueCombat("attack");
      else if (key === "f" && !byId("worldView").hidden) void interactNearby();
      else if (key === " " && !byId("worldView").hidden) { queueCombat("jump"); event.preventDefault(); }
      else if (key === "Shift" && !byId("worldView").hidden) queueCombat("dash");
    });
    document.addEventListener("keyup", event => runtime.keys.delete(event.key.length === 1 ? event.key.toLowerCase() : event.key));
    void startupProbe();
  }

  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", wire, { once: true });
  else wire();
})(typeof window !== "undefined" ? window : globalThis);
