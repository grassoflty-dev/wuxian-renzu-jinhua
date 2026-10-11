/* 余界：归航协议 · release UI
 * 仅负责呈现 Rust 权威状态、串行发送动作和界面输入；不在浏览器本地推进游戏状态。
 */
(() => {
  "use strict";

  if (window.__CampaignReleaseUiLoaded) return;
  window.__CampaignReleaseUiLoaded = true;

  const $ = (id) => document.getElementById(id);
  const $$ = (selector, root = document) => Array.from(root.querySelectorAll(selector));
  const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
  const numberOr = (value, fallback = 0) => Number.isFinite(Number(value)) ? Number(value) : fallback;
  const textOr = (value, fallback = "") => value === null || value === undefined ? fallback : String(value);
  const hasTauri = () => Boolean(window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function");

  const WORLD_FALLBACKS = {
    grey_hive: { name: "灰巢设施", kicker: "设施回收 / 冷区", description: "冷混凝土包住断续的暖黄灯。封锁门后仍有救援信标在呼吸。", mechanic: "断电 / 封锁 / 救援", accent: "#e0b35e" },
    mist_harbor: { name: "雾港余烬", kicker: "潮汐回收 / 外港", description: "青绿海水吞掉码头边界，远处的灯塔用三短一长呼叫救援。", mechanic: "海水 / 信号 / 救援", accent: "#70b9ae" },
    clockworks: { name: "钟骨工厂", kicker: "回路校正 / 铸造层", description: "铜色齿轮停在半拍之间，停机的回路里藏着最后一枚核心。", mechanic: "机械 / 铜锈 / 停机", accent: "#cf8b4e" },
  };
  const BUILD_META = {
    vanguard: { label: "先锋", detail: "高生命近战" },
    echo: { label: "回声", detail: "高能量技能" },
  };
  const NEARBY_KIND_LABELS = {
    objective: "目标", point: "调查点", npc: "联络员", exit: "出口", portal: "出口",
    gate: "封锁门", prop: "设施", switch: "开关", terminal: "终端", enemy: "威胁", hazard: "危险区",
  };
  const WORLD_IDS = Object.keys(WORLD_FALLBACKS);
  const DIRECTIONS = {
    w: { dx: 0, dy: -1 }, arrowup: { dx: 0, dy: -1 },
    s: { dx: 0, dy: 1 }, arrowdown: { dx: 0, dy: 1 },
    a: { dx: -1, dy: 0 }, arrowleft: { dx: -1, dy: 0 },
    d: { dx: 1, dy: 0 }, arrowright: { dx: 1, dy: 0 },
  };
  const SKILL_KEYS = ["Q", "E", "R"];
  const PREF_KEY = "campaign-release-ui-prefs-v2";
  const REQUEST_TIMEOUT_MS = 8000;

  const ui = {
    app: $("app"),
    boot: $("bootView"), hub: $("hubView"), world: $("worldView"),
    toastRegion: $("toastRegion"), connectionBadge: $("connectionBadge"), saveBadge: $("saveBadge"),
    resolutionSelect: $("resolutionSelect"), audioToggle: $("audioToggle"), saveButton: $("saveButton"),
    bootError: $("bootError"), hubTitle: $("hubTitle"), hubSubtitle: $("hubSubtitle"), hubSaveNote: $("hubSaveNote"),
    continueButton: $("continueButton"), newButton: $("newButton"), clearedCount: $("clearedCount"),
    hubEventFeed: $("hubEventFeed"), loadoutToggle: $("loadoutToggle"), loadoutPanel: $("loadoutPanel"), loadoutClose: $("loadoutClose"),
    taskPanel: $("taskPanel"), taskPanelWorld: $("taskPanelWorld"), taskPanelHint: $("taskPanelHint"), taskObjectiveList: $("taskObjectiveList"), taskEventFeed: $("taskEventFeed"),
    archivePanel: $("archivePanel"), exchangePanel: $("exchangePanel"), characterPanel: $("characterPanel"), exchangeCatalog: $("exchangeCatalog"), rewardPointsValue: $("rewardPointsValue"), certificateCount: $("certificateCount"),
    loadoutPlayerName: $("loadoutPlayerName"), loadoutPlayerBuild: $("loadoutPlayerBuild"), loadoutPlayerStats: $("loadoutPlayerStats"),
    upgradeList: $("upgradeList"), hubSkillList: $("hubSkillList"),
    worldKicker: $("worldKicker"), worldTitle: $("worldTitle"), hpMeter: $("hpMeter"), hpValue: $("hpValue"),
    energyMeter: $("energyMeter"), energyValue: $("energyValue"), medkitValue: $("medkitValue"), rewardPointsWorldValue: $("rewardPointsWorldValue"),
    objectiveList: $("objectiveList"), nearbyHint: $("nearbyHint"), aimHint: $("aimHint"), worldEventFeed: $("worldEventFeed"),
    canvasShell: $("canvasShell"), gameCanvas: $("gameCanvas"), stageZoneLabel: $("stageZoneLabel"), stageCoords: $("stageCoords"), canvasNotice: $("canvasNotice"), apiRetryButton: $("apiRetryButton"),
    mapPanel: $("mapPanel"), mapToggle: $("mapToggle"), mapBody: $("mapBody"), mapTitle: $("mapTitle"), mapCanvas: $("mapCanvas"), extractButton: $("extractButton"), skillRow: $("skillRow"),
    dialogModal: $("dialogModal"), dialogCounter: $("dialogCounter"), dialogSpeaker: $("dialogSpeaker"), dialogTitle: $("dialogTitle"), dialogBody: $("dialogBody"), dialogChoices: $("dialogChoices"),
    pauseRetryButton: $("pauseRetryButton"),
    outcomeModal: $("outcomeModal"), outcomeKicker: $("outcomeKicker"), outcomeTitle: $("outcomeTitle"), outcomeWorld: $("outcomeWorld"), outcomeSummary: $("outcomeSummary"), outcomeStats: $("outcomeStats"), outcomeRetry: $("outcomeRetry"),
    pauseModal: $("pauseModal"),
  };

  const runtime = {
    state: null,
    lastWorldId: null,
    facing: { dx: 0, dy: -1 },
    mapOpen: true,
    paused: false,
    actionBusy: false,
    normalQueue: [],
    pendingMove: null,
    pendingTick: false,
    held: new Map(),
    heldOrder: [],
    moveTimer: null,
    tickTimer: null,
    resizeObserver: null,
    fallbackResizeHandler: null,
    renderer: null,
    rendererReady: false,
    eventLog: [],
    lastEvent: "",
    lastSaveWarning: "",
    lastFeedbackKey: "",
    lastFeedbackEvent: "",
    lastEffectKey: "",
    lastEffectEvent: "",
    lastEffectShape: "",
    lastRendererError: "",
    rendererFailed: false,
    tickEnabled: true,
    lastQueueToastAt: 0,
    modalFocusKey: "",
    exchangeFilter: "all",
    openPanel: null,
    booting: false,
    skillSignature: "",
    preferences: { muted: false, resolution: "1920x1080", mapOpen: true },
    tp06a: { channel: null, sessionId: null, epoch: 0, generation: 0, lastTick: -1, records: [], status: "idle", stopPromise: null, active: false, starting: false },
  };

  function readPreferences() {
    try {
      const raw = JSON.parse(localStorage.getItem(PREF_KEY) || "{}");
      runtime.preferences.muted = Boolean(raw.muted);
      runtime.preferences.mapOpen = raw.mapOpen !== false;
      if (["1280x720", "1920x1080", "2560x1440"].includes(raw.resolution)) runtime.preferences.resolution = raw.resolution;
    } catch (_) { /* 偏好读取失败不影响游戏 */ }
    runtime.mapOpen = runtime.preferences.mapOpen;
  }

  function writePreferences() {
    try { localStorage.setItem(PREF_KEY, JSON.stringify(runtime.preferences)); } catch (_) { /* 偏好不可用时保持内存设置 */ }
  }

  function setConnection(kind, label) {
    if (!ui.connectionBadge) return;
    ui.connectionBadge.classList.toggle("status-chip--pending", kind === "pending");
    ui.connectionBadge.classList.toggle("status-chip--warn", kind === "warn");
    const labelNode = ui.connectionBadge.lastChild;
    if (labelNode && labelNode.nodeType === Node.TEXT_NODE) labelNode.nodeValue = label;
    else ui.connectionBadge.appendChild(document.createTextNode(label));
  }

  function setSaveBadge(state) {
    if (!ui.saveBadge) return;
    const warning = state && state.save_warning;
    ui.saveBadge.classList.toggle("status-chip--warn", Boolean(warning));
    ui.saveBadge.classList.toggle("status-chip--pending", false);
    const label = warning ? "存档需检查" : state && state.has_save ? "存档已接入" : "暂无存档";
    const labelNode = ui.saveBadge.lastChild;
    if (labelNode && labelNode.nodeType === Node.TEXT_NODE) labelNode.nodeValue = label;
    else ui.saveBadge.appendChild(document.createTextNode(label));
  }

  function showToast(message, kind = "info", duration = 3600) {
    if (!message || !ui.toastRegion) return;
    const toast = document.createElement("div");
    toast.className = `toast ${kind === "error" ? "toast--error" : kind === "success" ? "toast--success" : ""}`.trim();
    toast.textContent = textOr(message);
    ui.toastRegion.appendChild(toast);
    window.setTimeout(() => toast.remove(), duration);
  }

  function setText(node, value, fallback = "—") { if (node) node.textContent = textOr(value, fallback); }

  function escapeColor(value, fallback) {
    const candidate = textOr(value, "").trim();
    return /^#[0-9a-f]{3,8}$/i.test(candidate) || /^(?:rgb|hsl)a?\([\d\s,.%+-]+\)$/i.test(candidate) ? candidate : fallback;
  }

  function playerState() { return runtime.state && runtime.state.player && typeof runtime.state.player === "object" ? runtime.state.player : {}; }
  function currentMap() { return runtime.state && runtime.state.map && typeof runtime.state.map === "object" ? runtime.state.map : null; }
  function currentMode() { return runtime.state && textOr(runtime.state.mode, "boot"); }
  function isTypingTarget(target) { return Boolean(target && target.closest && target.closest("input, textarea, select, [contenteditable='true']")); }
  function worldInputAllowed() { return currentMode() === "world" && !runtime.paused && !document.hidden && !runtime.booting; }

  function normalizedPayload(payload) {
    if (!payload || typeof payload !== "object") throw new Error("服务器没有返回有效状态");
    if (payload.error && !payload.mode && !payload.version) throw new Error(textOr(payload.error, "服务器返回错误"));
    const candidate = payload.state && typeof payload.state === "object" && (payload.state.mode || payload.state.version) ? payload.state : payload;
    if (!candidate || typeof candidate !== "object") throw new Error("状态格式无法识别");
    return candidate;
  }

  async function responseJson(response) {
    const contentType = response.headers.get("content-type") || "";
    const payload = contentType.includes("json") ? await response.json() : JSON.parse(await response.text());
    if (!response.ok) throw new Error(textOr(payload && (payload.error || payload.message), `请求失败（${response.status}）`));
    return payload;
  }

  function withTimeout(promise, label) {
    let timer;
    const timeout = new Promise((_, reject) => { timer = window.setTimeout(() => reject(new Error(`${label}超时，请重试`)), REQUEST_TIMEOUT_MS); });
    return Promise.race([Promise.resolve(promise), timeout]).finally(() => window.clearTimeout(timer));
  }

  async function fetchWithTimeout(url, options, label) {
    const controller = typeof AbortController === "function" ? new AbortController() : null;
    const request = controller ? fetch(url, { ...options, signal: controller.signal }) : fetch(url, options);
    const timer = window.setTimeout(() => controller?.abort(), REQUEST_TIMEOUT_MS);
    try { return await request; }
    catch (error) { if (error && error.name === "AbortError") throw new Error(`${label}超时，请重试`); throw error; }
    finally { window.clearTimeout(timer); }
  }

  async function campaignView() {
    if (hasTauri()) return normalizedPayload(await withTimeout(window.__TAURI__.core.invoke("campaign_view"), "读取现场"));
    return normalizedPayload(await responseJson(await fetchWithTimeout("/api/view", { method: "GET", cache: "no-store", headers: { Accept: "application/json" } }, "读取现场")));
  }

  async function campaignAction(action, args = {}) {
    if (hasTauri()) return normalizedPayload(await withTimeout(window.__TAURI__.core.invoke("campaign_action", { action, args }), "发送动作"));
    return normalizedPayload(await responseJson(await fetchWithTimeout("/api/action", { method: "POST", cache: "no-store", headers: { "Content-Type": "application/json", Accept: "application/json" }, body: JSON.stringify({ action, args }) }, "发送动作")));
  }

  async function campaignSave() {
    if (hasTauri()) return withTimeout(window.__TAURI__.core.invoke("campaign_save"), "保存进度");
    const response = await fetchWithTimeout("/api/save", { method: "POST", cache: "no-store", headers: { Accept: "application/json" } }, "保存进度");
    if (!response.ok) throw new Error(`保存服务不可用（${response.status}）`);
    const payload = response.headers.get("content-type")?.includes("json") ? await response.json() : null;
    if (payload && (payload.memory_only === true || payload.saved === false)) return { persisted: false, memoryOnly: true, message: "内存试玩，关闭预览不保存" };
    if (!payload || payload.saved !== true) throw new Error("保存接口未确认写入");
    return { persisted: true, payload };
  }

  function setMode(mode) {
    const value = textOr(mode, "boot");
    ui.app.dataset.gameMode = value;
    document.body.dataset.gameMode = value;
    document.documentElement.dataset.gameMode = value;
    const hasWorldSnapshot = Boolean(runtime.state && runtime.state.map && runtime.state.player);
    const inWorldView = hasWorldSnapshot && ["world", "dialog", "result", "dead"].includes(value);
    // 变更：灰巢 grey_hive 已改由 Babylon 第三人称渲染（js/hive-side2d.js 停用），
    // 全流程 TPS —— 任何世界都不再套用横版 2D 版式（class="side2d"），避免 HUD/键位
    // 图例与第三人称画面互相矛盾。dataset.worldLayout 保留为可观测字段。
    ui.app.classList.remove("side2d");
    document.body.classList.remove("side2d");
    document.documentElement.classList.remove("side2d");
    ui.app.dataset.worldLayout = inWorldView ? "tps" : "hub";
    const baseView = ["dialog", "result", "dead"].includes(value) ? (hasWorldSnapshot ? "world" : "hub") : value;
    $$("[data-view]").forEach((view) => { view.hidden = view.dataset.view !== baseView; });
  }

  function closePanels() {
    [ui.taskPanel, ui.archivePanel, ui.exchangePanel, ui.characterPanel].forEach((panel) => { if (panel) panel.hidden = true; });
    runtime.openPanel = null;
  }

  function openPanel(id) {
    const panel = $(id); if (!panel) return;
    [ui.taskPanel, ui.archivePanel, ui.exchangePanel, ui.characterPanel].forEach((other) => { if (other && other !== panel) other.hidden = true; });
    panel.hidden = false; runtime.openPanel = id; panel.querySelector("button:not([disabled])")?.focus();
    if (id === "taskPanel") renderTaskPanel(runtime.state || {});
    if (id === "exchangePanel") renderExchange(runtime.state || {});
  }

  function updateEventLog(state) {
    const event = textOr(state && state.last_event, "").trim();
    if (!event || event === runtime.lastEvent) return;
    runtime.lastEvent = event;
    runtime.eventLog.unshift(event);
    runtime.eventLog = runtime.eventLog.slice(0, 4);
  }

  function renderEventFeed(node) {
    if (!node) return;
    node.replaceChildren();
    if (!runtime.eventLog.length) {
      const empty = document.createElement("p"); empty.className = "empty-state"; empty.textContent = "尚无回传。"; node.appendChild(empty); return;
    }
    runtime.eventLog.slice(0, 4).forEach((event) => { const line = document.createElement("p"); line.className = "event-line"; line.textContent = event; node.appendChild(line); });
  }

  function renderHub(state) {
    setText(ui.hubTitle, state.title, "无限人族进化");
    setText(ui.hubSubtitle, state.subtitle, "在三个失联世界中找到回家的证据，带回余界核心。");
    setSaveBadge(state);
    if (ui.continueButton) ui.continueButton.disabled = !state.has_save || Boolean(state.save_warning);
    if (ui.newButton) ui.newButton.disabled = false;
    if (ui.hubSaveNote) {
      ui.hubSaveNote.textContent = state.save_warning ? textOr(state.save_warning) : state.has_save ? "上次回传已接入。继续旅程或选择一个世界再次出发。" : "选择一处世界即可开始，不需要先完成设置。";
    }
    const worlds = Array.isArray(state.worlds) ? state.worlds : [];
    let cleared = 0;
    $$("[data-world-card]").forEach((card) => {
      const id = card.dataset.worldCard;
      const backendWorld = worlds.find((world) => textOr(world && world.id) === id);
      const fallback = WORLD_FALLBACKS[id] || {};
      const world = backendWorld || fallback;
      const isCleared = Boolean(world.cleared);
      const available = world.available !== false;
      if (isCleared) cleared += 1;
      card.classList.toggle("is-cleared", isCleared);
      card.classList.toggle("is-unavailable", !available);
      card.style.setProperty("--card-accent", escapeColor(world.accent, fallback.accent || "#d8a64c"));
      setText(card.querySelector("[data-world-kicker]"), world.kicker, fallback.kicker);
      setText(card.querySelector("[data-world-name]"), world.name, fallback.name || id);
      setText(card.querySelector("[data-world-description]"), world.description, fallback.description);
      setText(card.querySelector("[data-world-mechanic]"), world.mechanic, fallback.mechanic);
      const status = card.querySelector("[data-world-status]");
      if (status) { status.classList.toggle("is-cleared", isCleared); status.textContent = !available ? "未开放" : isCleared ? "已回收" : "待进入"; }
      const button = card.querySelector("[data-enter-world]");
      if (button) { button.disabled = !available; button.querySelector("span")?.replaceChildren(document.createTextNode(isCleared ? "再次进入" : "进入现场")); }
    });
    setText(ui.clearedCount, cleared, "0");
    renderEventFeed(ui.hubEventFeed);
    renderLoadout(state);
    renderExchange(state);
    renderTaskPanel(state);
  }

  function renderLoadout(state) {
    const player = playerState();
    setText(ui.loadoutPlayerName, player.name, "岑遥");
    const rawBuild = player.build && typeof player.build === "object" ? (player.build.id ?? player.build.name) : player.build;
    const buildId = textOr(rawBuild, "").trim().toLowerCase();
    const buildMeta = BUILD_META[buildId];
    setText(ui.loadoutPlayerBuild, buildMeta ? `${buildMeta.label} · ${buildMeta.detail}` : rawBuild, "待读取战术构型");
    $$("[data-build-id]").forEach((button) => { const active = textOr(button.dataset.buildId).toLowerCase() === buildId; button.classList.toggle("is-active", active); button.setAttribute("aria-pressed", String(active)); });
    if (ui.loadoutPlayerStats) {
      ui.loadoutPlayerStats.replaceChildren();
      [["生命", `${textOr(player.hp, "—")} / ${textOr(player.max_hp, "—")}`], ["能量", `${textOr(player.energy, "—")} / ${textOr(player.max_energy, "—")}`], ["奖励点数", textOr(player.reward_points ?? player.rewards, "—")], ["支线凭证", textOr(player.certificates ?? player.side_story_tokens, "—")]].forEach(([label, value]) => {
        const item = document.createElement("span"); item.textContent = `${label} ${value}`; ui.loadoutPlayerStats.appendChild(item);
      });
    }
    renderHubSkills(Array.isArray(state.skills) ? state.skills : []);
  }

  function renderHubSkills(skills) {
    if (!ui.hubSkillList) return;
    ui.hubSkillList.replaceChildren();
    if (!skills.length) { const empty = document.createElement("p"); empty.className = "empty-state"; empty.textContent = "暂无已同步技能。"; ui.hubSkillList.appendChild(empty); return; }
    skills.forEach((skill, index) => {
      const item = document.createElement("div"); item.className = "hub-skill-item";
      const copy = document.createElement("div"); const title = document.createElement("b"); title.textContent = `${textOr(skill.key, SKILL_KEYS[index % 3])} · ${textOr(skill.name, textOr(skill.id, "技能"))}`;
      const desc = document.createElement("p"); desc.textContent = textOr(skill.description, "进入现场后可使用。"); copy.append(title, desc); item.append(copy); ui.hubSkillList.appendChild(item);
    });
  }

  function firstArray(source, keys) {
    for (const key of keys) if (Array.isArray(source?.[key])) return source[key];
    return [];
  }

  function renderTaskPanel(state) {
    const map = state?.map && typeof state.map === "object" ? state.map : null;
    setText(ui.taskPanelWorld, map?.world_name || "归航站", "归航站");
    // 变更：灰巢已改第三人称，腕表提示不再区分横版。
    setText(ui.taskPanelHint, map ? "腕表持续接收现场目标。按 T 或 Esc 关闭。" : "进入世界后，目标会在这里展开。", "进入世界后，目标会在这里展开。");
    renderObjectivesInto(ui.taskObjectiveList, Array.isArray(state?.objectives) ? state.objectives : []);
    renderEventFeed(ui.taskEventFeed);
  }

  function renderExchange(state) {
    const player = state?.player && typeof state.player === "object" ? state.player : {};
    const inventory = state?.inventory && typeof state.inventory === "object" ? state.inventory : {};
    const rewards = state?.rewards && typeof state.rewards === "object" ? state.rewards : {};
    const certificates = state?.certificates ?? rewards.certificates ?? player.certificates ?? player.side_story_tokens ?? inventory.certificates;
    const points = state?.reward_points ?? state?.rewardPoints ?? rewards.points ?? rewards.reward_points ?? player.reward_points ?? player.rewards;
    setText(ui.rewardPointsValue, points, "0"); setText(ui.certificateCount, certificates, "0");
    if (!ui.exchangeCatalog) return;
    const catalog = firstArray(state, ["exchange_catalog", "exchangeCatalog", "exchanges", "catalog"]);
    const items = catalog.length ? catalog : [
      { id: "field_kit_d", name: "现场应急套件", type: "equipment", tier: "D", description: "提升现场探索的容错。", cost: 80 },
      { id: "echo_lineage_d", name: "回声感知", type: "bloodline", tier: "D", description: "让失真的信号留下短暂轮廓。", cost: 120 },
      { id: "phase_step_dd", name: "相位步", type: "skill", tier: "DD", description: "以短距位移穿过危险间隙。", cost: 160 },
      { id: "signal_spear_c", name: "信号矛", type: "equipment", tier: "C", description: "将回收能量凝成近战锋面。", cost: 240 },
    ];
    const owned = new Set(firstArray(inventory, ["items", "equipment", "bloodlines", "skills"]).map(item => textOr(item?.id ?? item)));
    const filter = runtime.exchangeFilter || "all";
    ui.exchangeCatalog.replaceChildren();
    const visible = items.filter(item => {
      const rawCategory = textOr(item?.type, item?.category).toLowerCase();
      const category = rawCategory === "lineage" ? "bloodline" : rawCategory;
      return filter === "all" || category === filter;
    });
    if (!visible.length) { const empty = document.createElement("p"); empty.className = "empty-state"; empty.textContent = "此类别暂无可兑换条目。"; ui.exchangeCatalog.appendChild(empty); return; }
    visible.forEach(item => {
      const card = document.createElement("article"); card.className = "exchange-item";
      const title = document.createElement("b"); title.textContent = textOr(item?.name, item?.id || "未命名条目");
      const desc = document.createElement("p"); desc.textContent = textOr(item?.description, "等待兑换说明。");
      const rewardCost = item?.reward_cost ?? item?.cost ?? item?.price ?? "—"; const tokenCost = item?.token_cost ?? 0;
      const meta = document.createElement("small"); meta.textContent = `${textOr(item?.tier, item?.grade || "D")} · ${rewardCost} 奖励点数${tokenCost ? ` · ${tokenCost} 支线凭证` : ""}`;
      const button = document.createElement("button"); button.type = "button"; button.textContent = item?.owned || owned.has(textOr(item?.id)) ? "已拥有" : "兑换"; button.disabled = Boolean(item?.owned) || owned.has(textOr(item?.id)); button.dataset.exchangeId = textOr(item?.id);
      card.append(title, desc, meta, button); ui.exchangeCatalog.appendChild(card);
    });
  }

  function renderPlayer(player) {
    const hp = numberOr(player.hp, 0), maxHp = Math.max(1, numberOr(player.max_hp, 1));
    const energy = numberOr(player.energy, 0), maxEnergy = Math.max(1, numberOr(player.max_energy, 1));
    if (ui.hpMeter) ui.hpMeter.style.width = `${clamp(hp / maxHp * 100, 0, 100)}%`;
    if (ui.energyMeter) ui.energyMeter.style.width = `${clamp(energy / maxEnergy * 100, 0, 100)}%`;
    setText(ui.hpValue, `${textOr(player.hp, "—")} / ${textOr(player.max_hp, "—")}`);
    setText(ui.energyValue, `${textOr(player.energy, "—")} / ${textOr(player.max_energy, "—")}`);
    setText(ui.medkitValue, player.medkits);
    setText(ui.rewardPointsWorldValue, player.reward_points ?? player.scrap);
  }

  function renderObjectivesInto(node, objectives) {
    if (!node) return;
    node.replaceChildren();
    if (!Array.isArray(objectives) || !objectives.length) { const empty = document.createElement("li"); empty.className = "empty-state"; empty.textContent = "等待现场目标…"; node.appendChild(empty); return; }
    objectives.forEach((objective) => {
      const item = document.createElement("li"); item.className = `objective-item${objective.done ? " is-done" : ""}${objective.optional ? " is-optional" : ""}`;
      const title = document.createElement("span"); title.textContent = textOr(objective.title, "未命名目标"); item.appendChild(title);
      if (objective.description) { const desc = document.createElement("small"); desc.textContent = objective.description; item.appendChild(desc); }
      node.appendChild(item);
    });
  }

  function renderObjectives(objectives) { renderObjectivesInto(ui.objectiveList, objectives); renderObjectivesInto(ui.taskObjectiveList, objectives); }

  function renderSkills(skills) {
    const row = $("skillRow"); if (!row) return;
    const entries = Array.isArray(skills) ? skills.slice(0, 3) : [];
    const structureSignature = JSON.stringify(entries.map((skill) => ({ id: skill && skill.id, key: skill && skill.key })));
    if (structureSignature !== runtime.skillSignature) {
      runtime.skillSignature = structureSignature;
      row.replaceChildren();
      if (!entries.length) { const empty = document.createElement("p"); empty.className = "empty-state"; empty.textContent = "暂无技能槽。"; row.appendChild(empty); return; }
      entries.forEach((_, index) => {
        const card = document.createElement("div"); card.className = "skill-card";
        const key = document.createElement("span"); key.className = "skill-key";
        const copy = document.createElement("div"); const name = document.createElement("b"); const description = document.createElement("small"); copy.append(name, description);
        const cost = document.createElement("span"); cost.className = "skill-cost";
        const button = document.createElement("button"); button.type = "button"; button.dataset.skillSlot = String(index);
        card.append(key, copy, cost, button); row.appendChild(card);
      });
    }
    entries.forEach((skill, index) => {
      const card = row.children[index]; if (!card || card.classList.contains("empty-state")) return;
      const ready = skill.ready !== false; const cooldownTicks = Math.max(0, Math.ceil(numberOr(skill.cooldown, 0))); const seconds = cooldownTicks * 0.25;
      const key = card.querySelector(".skill-key"); const name = card.querySelector("b"); const description = card.querySelector("small"); const cost = card.querySelector(".skill-cost"); const button = card.querySelector("button");
      card.classList.toggle("is-ready", ready); card.classList.toggle("is-unready", !ready);
      setText(key, textOr(skill.key, SKILL_KEYS[index])); setText(name, textOr(skill.name, textOr(skill.id, "技能"))); setText(description, textOr(skill.description, "稳定现场信号。"));
      const cooldown = cooldownTicks > 0 ? `CD ${seconds.toFixed(2).replace(/0+$/, "").replace(/\.$/, "")}s` : ready ? "就绪" : "能量不足";
      setText(cost, `${textOr(skill.cost, 0)}能 · ${cooldown}`); if (button) { button.disabled = !ready; button.setAttribute("aria-label", `${textOr(skill.name, "技能")}，${textOr(skill.key, SKILL_KEYS[index])}`); }
    });
  }

  function nearestNearby(map) {
    const nearby = map && Array.isArray(map.nearby) ? map.nearby : [];
    if (!nearby.length) return null;
    return nearby.slice().sort((a, b) => (Math.abs(numberOr(a.dx)) + Math.abs(numberOr(a.dy))) - (Math.abs(numberOr(b.dx)) + Math.abs(numberOr(b.dy))))[0];
  }

  function nearbyKindLabel(kind) {
    const value = textOr(kind, "").trim().toLowerCase();
    if (NEARBY_KIND_LABELS[value]) return NEARBY_KIND_LABELS[value];
    if (value.includes("objective")) return "目标";
    if (value.includes("npc")) return "联络员";
    if (value.includes("exit") || value.includes("portal")) return "出口";
    if (value.includes("gate")) return "封锁门";
    if (value.includes("enemy") || value.includes("hazard")) return "威胁";
    return value || "交互点";
  }

  function renderWorld(state, sourceAction = null) {
    const map = currentMap(); const player = playerState();
    const worldId = textOr(map && map.world_id, runtime.lastWorldId || "");
    runtime.lastWorldId = worldId || runtime.lastWorldId;
    const fallback = WORLD_FALLBACKS[worldId] || {};
    setText(ui.worldKicker, fallback.kicker || "现场 / 信号稳定");
    setText(ui.worldTitle, map && map.world_name, fallback.name || "现场");
    setText(ui.mapTitle, map && map.world_name, fallback.name || "现场图");
    setText(ui.stageZoneLabel, `区域 / ${textOr(map && map.world_name, fallback.name || "现场")}`);
    const mapX = map && Number(map.px), mapY = map && Number(map.py);
    setText(ui.stageCoords, worldId === "grey_hive" ? "横版现场 / 地层信号" : (Number.isFinite(mapX) && Number.isFinite(mapY) ? `坐标 ${mapX}, ${mapY}` : "坐标 —"));
    renderPlayer(player); renderObjectives(state.objectives); renderSkills(state.skills);
    const nearby = nearestNearby(map);
    if (ui.nearbyHint) ui.nearbyHint.textContent = nearby ? `${textOr(nearby.name, "目标")} · ${nearbyKindLabel(nearby.kind)}` : "保持移动，扫描附近信号。";
    if (ui.aimHint) ui.aimHint.textContent = nearby ? `F · 交互「${textOr(nearby.name, "目标")}」` : (worldId === "grey_hive" ? "左键 / J 攻击 · F 交互" : "点击画面朝向并按 J 近战");
    if (ui.extractButton) {
      const activePortal = Array.isArray(map && map.portals) && map.portals.some((portal) => portal && portal.active !== false);
      ui.extractButton.disabled = !activePortal;
      ui.extractButton.title = activePortal ? "在出口且所有主目标完成时执行" : "当前现场没有可用出口";
    }
    renderEventFeed(ui.worldEventFeed); drawMinimap(map); updateRenderer(state, sourceAction);
  }

  function outcomeStats(result) {
    if (!result || typeof result.stats !== "object" || result.stats === null) return [];
    if (Array.isArray(result.stats)) return result.stats.map((stat, index) => [textOr(stat && stat.label, `记录 ${index + 1}`), textOr(stat && (stat.value ?? stat.amount), "—")]);
    return Object.entries(result.stats).slice(0, 4).map(([key, value]) => [key, value]);
  }

  function renderOutcome(state) {
    const result = state.result && typeof state.result === "object" ? state.result : {};
    const dead = currentMode() === "dead";
    setText(ui.outcomeKicker, dead ? "生命信号中断" : result.all_complete ? "归航记录 / 三世界完成" : "现场结算");
    setText(ui.outcomeTitle, dead ? "这一次，先回站。" : result.title, dead ? "这一次，先回站。" : "记录已保存");
    setText(ui.outcomeWorld, result.world_name || (currentMap() && currentMap().world_name), dead ? "现场信号已中断" : "世界现场");
    setText(ui.outcomeSummary, result.summary || state.last_event, dead ? textOr(state.last_event, "现场信号中断，未带回本次记录。") : "—");
    if (ui.outcomeStats) {
      ui.outcomeStats.replaceChildren();
      const stats = outcomeStats(result);
      if (result.rank !== undefined) stats.unshift(["评级", result.rank]);
      if (result.reward !== undefined) stats.push(["回收", result.reward]);
      (stats.length ? stats : [["状态", dead ? "需要重试" : "已回传"]]).slice(0, 4).forEach(([label, value]) => { const item = document.createElement("div"); item.className = "outcome-stat"; const valueNode = document.createElement("b"); valueNode.textContent = textOr(value, "—"); const labelNode = document.createElement("span"); labelNode.textContent = textOr(label, "记录"); item.append(valueNode, labelNode); ui.outcomeStats.appendChild(item); });
    }
    if (ui.outcomeRetry) { ui.outcomeRetry.textContent = dead ? "重试本次现场" : "再次进入现场"; ui.outcomeRetry.disabled = !textOr((currentMap() && currentMap().world_id) || runtime.lastWorldId, ""); }
  }

  function renderDialog(state) {
    const dialog = state.dialog && typeof state.dialog === "object" ? state.dialog : null;
    if (!dialog) return;
    setText(ui.dialogSpeaker, dialog.speaker, "未知信号"); setText(ui.dialogTitle, dialog.title, "通讯接入"); setText(ui.dialogBody, dialog.body, "正在解码…");
    const choices = Array.isArray(dialog.choices) ? dialog.choices : [];
    setText(ui.dialogCounter, choices.length ? `${String(choices.length).padStart(2, "0")} 项回应` : "等待回应");
    ui.dialogChoices.replaceChildren();
    if (!choices.length) { const empty = document.createElement("p"); empty.className = "empty-state"; empty.textContent = "没有可用回应。"; ui.dialogChoices.appendChild(empty); return; }
    choices.forEach((choice, index) => {
      const button = document.createElement("button"); button.type = "button"; button.className = "choice-button"; button.dataset.choiceIndex = String(index);
      const number = document.createElement("span"); number.className = "choice-index"; number.textContent = String(index + 1).padStart(2, "0");
      const copy = document.createElement("span"); const label = document.createElement("b"); label.textContent = textOr(choice && choice.label, "回应"); copy.appendChild(label); if (choice && choice.description) { const description = document.createElement("small"); description.textContent = choice.description; copy.appendChild(description); }
      const arrow = document.createElement("span"); arrow.className = "choice-arrow"; arrow.textContent = "→"; button.append(number, copy, arrow); ui.dialogChoices.appendChild(button);
    });
  }

  function renderPause() {
    if (ui.pauseModal) ui.pauseModal.hidden = !runtime.paused;
  }

  function renderState(nextState, sourceAction = null) {
    const previousMode = currentMode();
    runtime.state = nextState;
    runtime.booting = false;
    updateEventLog(nextState);
    const mode = currentMode();
    if (mode !== previousMode) clearQueuedActions();
    if (mode !== "world" || nextState.dialog) { runtime.paused = false; clearHeld(); }
    setMode(mode);
    ui.dialogModal.hidden = !(mode === "dialog" && nextState.dialog);
    ui.outcomeModal.hidden = !(mode === "result" || mode === "dead");
    if (ui.apiRetryButton && !runtime.rendererFailed) { ui.apiRetryButton.hidden = true; ui.apiRetryButton.textContent = "重试连接"; }
    if (ui.pauseRetryButton && !runtime.rendererFailed) ui.pauseRetryButton.hidden = true;
    setConnection("ok", "现场在线");
    setSaveBadge(nextState);
    if (nextState.save_warning && textOr(nextState.save_warning) !== runtime.lastSaveWarning) { runtime.lastSaveWarning = textOr(nextState.save_warning); showToast(nextState.save_warning, "error", 5600); }
    if (!nextState.save_warning) runtime.lastSaveWarning = "";
    const hasWorldSnapshot = Boolean(nextState && nextState.map && nextState.player);
    if (mode === "hub" || (["dialog", "result", "dead"].includes(mode) && !hasWorldSnapshot)) renderHub(nextState);
    if (mode === "world" || (["dialog", "result", "dead"].includes(mode) && hasWorldSnapshot)) renderWorld(nextState, sourceAction);
    if (mode === "dialog") renderDialog(nextState);
    if (mode === "result" || mode === "dead") renderOutcome(nextState);
    renderPause();
    if (mode !== previousMode) resizeRenderer();
    if (mode !== previousMode || mode === "dialog" || mode === "result" || mode === "dead") focusModalOnce(mode);
    AudioFx.event(nextState, sourceAction);
  }

  function focusModalOnce(mode) {
    const key = `${mode}:${runtime.lastEvent}`; if (key === runtime.modalFocusKey) return; runtime.modalFocusKey = key;
    window.requestAnimationFrame(() => {
      const target = mode === "dialog" ? ui.dialogChoices?.querySelector("button") : mode === "result" || mode === "dead" ? ui.outcomeRetry : runtime.paused ? ui.pauseModal?.querySelector("button") : null;
      target?.focus();
    });
  }

  function feedbackKey(state, sourceAction = null) {
    const map = state && state.map;
    const revision = map && map.revision !== undefined ? textOr(map.revision) : textOr(state && (state.revision ?? state.action_revision), "");
    const action = textOr(sourceAction || (state && (state.action || state.last_action || state.action_id)), "view");
    const effect = state && state.effect && typeof state.effect === "object" ? JSON.stringify(state.effect) : "none";
    return `${action}|${revision}|${effect}|${textOr(state && state.last_event, "")}`;
  }

  function updateRenderer(state, sourceAction = null) {
    if (runtime.rendererFailed) return;
    if (!runtime.renderer) { handleRendererFailure(new Error("现场画面模块未就绪")); return; }
    const map = state && state.map;
    try {
      if (typeof runtime.renderer.setData === "function") runtime.renderer.setData(map || null);
      const effectKey = feedbackKey(state, sourceAction);
      const eventText = textOr(state && state.last_event, "");
      const effectShape = state && state.effect && typeof state.effect === "object" ? JSON.stringify(state.effect) : "none";
      const repeatedTickFeedback = sourceAction === "tick" && eventText && eventText === runtime.lastEffectEvent && effectShape === runtime.lastEffectShape;
      if (typeof runtime.renderer.setEffect === "function" && !repeatedTickFeedback && effectKey !== runtime.lastEffectKey) { runtime.lastEffectKey = effectKey; runtime.renderer.setEffect(state && state.effect ? state.effect : null); }
      runtime.lastEffectEvent = eventText; runtime.lastEffectShape = effectShape;
      if (typeof runtime.renderer.setPlayerState === "function") runtime.renderer.setPlayerState(playerState());
    } catch (error) { handleRendererFailure(error); }
  }

  function resizeRenderer() {
    if (runtime.rendererFailed || typeof runtime.renderer?.resize !== "function") return;
    try { runtime.renderer.resize(); } catch (error) { handleRendererFailure(error); }
  }

  function handleRendererFailure(error) {
    const message = textOr(error && error.message, textOr(error, "未知画面错误"));
    runtime.rendererFailed = true;
    runtime.tickEnabled = false;
    clearHeld();
    clearQueuedActions();
    if (currentMode() === "world") { runtime.paused = true; renderPause(); }
    if (ui.apiRetryButton) { ui.apiRetryButton.hidden = false; ui.apiRetryButton.textContent = "重试画面"; }
    if (ui.pauseRetryButton) ui.pauseRetryButton.hidden = false;
    if (ui.canvasNotice) {
      ui.canvasNotice.hidden = false;
      const noticeText = ui.canvasNotice.querySelector("span:last-child");
      if (noticeText) noticeText.textContent = "现场画面已暂停 · 点击“重试画面”";
    }
    if (message !== runtime.lastRendererError) { runtime.lastRendererError = message; showToast(`现场画面已暂停：${message}。请重试画面。`, "error", 5600); }
  }

  function clearRendererResizeWatch() {
    runtime.resizeObserver?.disconnect?.(); runtime.resizeObserver = null;
    if (runtime.fallbackResizeHandler) { window.removeEventListener("resize", runtime.fallbackResizeHandler); runtime.fallbackResizeHandler = null; }
  }

  function initRenderer() {
    if (!window.CampaignRenderer || !ui.gameCanvas) { ui.canvasNotice.hidden = false; return; }
    try {
      const options = {
        onTarget: (target) => handleCanvasTarget(target),
        onHover: (target) => handleCanvasHover(target),
      };
      const initialized = typeof window.CampaignRenderer.init === "function" ? window.CampaignRenderer.init(ui.gameCanvas, options) : null;
      runtime.renderer = initialized && typeof initialized === "object" ? initialized : window.CampaignRenderer;
      runtime.rendererReady = true; ui.canvasNotice.hidden = true;
      if (runtime.state) updateRenderer(runtime.state);
      if (typeof runtime.renderer.resize === "function") runtime.renderer.resize();
      if (typeof ResizeObserver === "function" && ui.canvasShell) { runtime.resizeObserver = new ResizeObserver(() => resizeRenderer()); runtime.resizeObserver.observe(ui.canvasShell); }
      else { runtime.fallbackResizeHandler = () => resizeRenderer(); window.addEventListener("resize", runtime.fallbackResizeHandler, { passive: true }); }
    } catch (error) { handleRendererFailure(error); }
  }

  function retryRenderer() {
    if (runtime.actionBusy) return;
    runtime.rendererFailed = false; runtime.lastRendererError = ""; runtime.tickEnabled = true; runtime.lastEffectKey = ""; runtime.lastEffectEvent = ""; runtime.lastEffectShape = "";
    if (ui.pauseRetryButton) ui.pauseRetryButton.hidden = true;
    clearHeld(); clearQueuedActions(); runtime.paused = false; clearRendererResizeWatch();
    try { runtime.renderer?.dispose?.(); } catch (_) { /* 旧画面释放失败不阻止重试 */ }
    runtime.renderer = null; runtime.rendererReady = false; initRenderer();
    if (runtime.state) renderState(runtime.state, "view");
  }

  function handleCanvasHover(target) {
    const direction = directionFromTarget(target);
    if (!direction) return;
    runtime.facing = direction;
    if (ui.aimHint && worldInputAllowed()) ui.aimHint.textContent = document.body.classList.contains("side2d") ? `朝向 ${directionLabel(direction)} · 左键/J 攻击` : `朝向 ${directionLabel(direction)} · 左键/J 近战`;
  }

  function handleCanvasTarget(target) {
    if (!worldInputAllowed()) return;
    const direction = directionFromTarget(target);
    if (direction) runtime.facing = direction;
    requestAction("attack", { dx: runtime.facing.dx, dy: runtime.facing.dy }, { kind: "normal" });
  }

  function directionFromTarget(target) {
    if (!target || typeof target !== "object") return null;
    const directDx = Number(target.dx), directDy = Number(target.dy);
    if (Number.isFinite(directDx) && Number.isFinite(directDy) && (directDx !== 0 || directDy !== 0)) return { dx: Math.sign(directDx), dy: Math.sign(directDy) };
    const map = currentMap(); if (!map) return null;
    const x = Number(target.x), y = Number(target.y), px = Number(map.px), py = Number(map.py);
    if (![x, y, px, py].every(Number.isFinite)) return null;
    const dx = Math.sign(x - px), dy = Math.sign(y - py);
    return dx || dy ? { dx, dy } : null;
  }

  function directionLabel(direction) {
    if (direction.dx === 0 && direction.dy < 0) return "北"; if (direction.dx === 0 && direction.dy > 0) return "南"; if (direction.dx < 0 && direction.dy === 0) return "西"; if (direction.dx > 0 && direction.dy === 0) return "东"; return "斜向";
  }

  function clearHeld() { runtime.held.clear(); runtime.heldOrder = []; }
  function clearQueuedActions() { runtime.normalQueue = []; runtime.pendingMove = null; runtime.pendingTick = false; }
  function currentHeldDirection() { for (let i = runtime.heldOrder.length - 1; i >= 0; i -= 1) { const direction = runtime.held.get(runtime.heldOrder[i]); if (direction) return direction; } return null; }
  function setFacing(direction) { if (!direction) return; runtime.facing = { dx: direction.dx, dy: direction.dy }; }

  function requestMove(direction) { if (!worldInputAllowed()) return; setFacing(direction); requestAction("move", { dx: direction.dx, dy: direction.dy }, { kind: "move" }); }
  function requestAttack() { if (!worldInputAllowed()) return; requestAction("attack", { dx: runtime.facing.dx, dy: runtime.facing.dy }, { kind: "normal" }); }
  function requestDash() { if (!worldInputAllowed()) return; requestAction("dash", { dx: runtime.facing.dx, dy: runtime.facing.dy }, { kind: "normal" }); }
  function requestSkill(slot) { if (!worldInputAllowed()) return; requestAction("skill", { slot, dx: runtime.facing.dx, dy: runtime.facing.dy }, { kind: "normal" }); }
  function requestHeal() { if (!worldInputAllowed()) return; requestAction("heal", {}, { kind: "normal" }); }
  function requestInteract() {
    if (!worldInputAllowed()) return;
    const nearby = nearestNearby(currentMap());
    if (!nearby || nearby.id === undefined || nearby.id === null) { showToast("没有可交互目标。靠近信号点后再按 F。", "info", 2400); return; }
    requestAction("interact", { id: nearby.id }, { kind: "normal" });
  }

  function requestAction(action, args = {}, options = {}) {
    if (!runtime.state || runtime.booting) return;
    if (runtime.actionBusy) {
      if (options.kind === "normal") {
        if (runtime.normalQueue.length < 3) runtime.normalQueue.push({ action, args });
        else if (Date.now() - runtime.lastQueueToastAt > 900) { runtime.lastQueueToastAt = Date.now(); showToast("操作过快，已保留前三个指令。", "info", 2200); }
      } else if (options.kind === "move") runtime.pendingMove = { action, args };
      else if (options.kind === "tick") runtime.pendingTick = true;
      return;
    }
    startAction(action, args);
  }

  async function startAction(action, args) {
    if (runtime.actionBusy) return;
    runtime.actionBusy = true;
    try {
      const nextState = await campaignAction(action, args);
      renderState(nextState, action);
      if (["enter", "continue", "retreat", "restart", "new"].includes(action)) closePanels();
      if (action === "skill") {
        const slot = Number(args?.slot);
        // A miss is still a cast and needs readable audiovisual feedback.
        // Combat outcome stays authoritative in Rust; this only supplies the
        // presentation event omitted by invalid/miss responses.
        if (!nextState?.effect && typeof runtime.renderer?.setEffect === "function") {
          runtime.renderer.setEffect({
            kind: `skill_${slot}`,
            x: nextState?.map?.px,
            y: nextState?.map?.py,
            dx: Number(args?.dx) || runtime.facing.dx || 1,
            dy: Number(args?.dy) || runtime.facing.dy || 0,
          });
        }
        flashSkill(slot);
      }
      if (action === "enter" && args.world_id) runtime.lastWorldId = textOr(args.world_id);
      if (!runtime.rendererFailed) runtime.tickEnabled = true;
    } catch (error) {
      runtime.tickEnabled = false;
      clearQueuedActions();
      if (ui.apiRetryButton && currentMode() === "world") { ui.apiRetryButton.hidden = false; ui.apiRetryButton.textContent = "重试连接"; }
      setConnection("warn", "回传失败"); showToast(`动作未发送：${error.message || error}。请重试。`, "error", 5000);
    } finally {
      runtime.actionBusy = false;
      if (worldInputAllowed() && runtime.normalQueue.length) {
        const nextNormal = runtime.normalQueue.shift(); startAction(nextNormal.action, nextNormal.args);
      } else if (worldInputAllowed() && runtime.pendingMove) {
        const pendingMove = runtime.pendingMove; runtime.pendingMove = null; startAction(pendingMove.action, pendingMove.args);
      } else if (worldInputAllowed() && runtime.pendingTick) {
        runtime.pendingTick = false; startAction("tick", {});
      } else if (!worldInputAllowed()) clearQueuedActions();
    }
  }

  function flashSkill(slot) {
    const card = ui.skillRow?.children?.[slot]; if (!card) return;
    card.classList.remove("is-triggered"); void card.offsetWidth; card.classList.add("is-triggered");
    ui.canvasShell?.classList.remove("skill-cast"); void ui.canvasShell?.offsetWidth; ui.canvasShell?.classList.add("skill-cast");
    window.setTimeout(() => card.classList.remove("is-triggered"), 650);
    window.setTimeout(() => ui.canvasShell?.classList.remove("skill-cast"), 520);
  }

  function togglePause() {
    if (currentMode() !== "world") return;
    runtime.paused = !runtime.paused;
    clearHeld();
    if (runtime.paused) clearQueuedActions();
    renderPause();
    if (runtime.paused) { focusModalOnce("pause"); flushSaveOnPause(); }
    else ui.gameCanvas?.focus();
  }

  async function flushSaveOnPause() {
    try {
      const result = await campaignSave();
      if (result && result.persisted === false) showToast(result.message || "内存试玩，关闭预览不保存。", "info", 4600);
      else showToast("暂停时已保存当前进度。", "success", 2400);
    } catch (error) { showToast(`暂停已生效；当前预览未能持久化：${error.message || error}`, "info", 4600); }
  }

  function setMapOpen(open, persist = true) {
    runtime.mapOpen = Boolean(open); if (persist) { runtime.preferences.mapOpen = runtime.mapOpen; writePreferences(); }
    ui.mapBody.hidden = !runtime.mapOpen; ui.mapToggle.setAttribute("aria-expanded", String(runtime.mapOpen)); ui.mapToggle.innerHTML = runtime.mapOpen ? "收起 <span aria-hidden=\"true\">⌃</span>" : "展开 <span aria-hidden=\"true\">⌄</span>";
  }

  function toggleMap() { setMapOpen(!runtime.mapOpen, true); }

  function drawMinimap(map) {
    const canvas = ui.mapCanvas; if (!canvas) return;
    const rect = canvas.getBoundingClientRect(); const width = Math.max(1, Math.floor(rect.width || 220)); const height = Math.max(1, Math.floor(rect.height || 132)); const dpr = Math.min(2, window.devicePixelRatio || 1);
    if (canvas.width !== Math.floor(width * dpr) || canvas.height !== Math.floor(height * dpr)) { canvas.width = Math.floor(width * dpr); canvas.height = Math.floor(height * dpr); }
    const ctx = canvas.getContext("2d"); if (!ctx) return; ctx.setTransform(dpr, 0, 0, dpr, 0, 0); ctx.clearRect(0, 0, width, height); ctx.fillStyle = "#0a1720"; ctx.fillRect(0, 0, width, height);
    if (!map || !Array.isArray(map.tiles) || !map.tiles.length) { ctx.fillStyle = "#50626b"; ctx.font = "10px monospace"; ctx.fillText("MAP / WAITING", 12, 22); return; }
    const rows = map.tiles; const h = Math.max(1, numberOr(map.h, rows.length)); const w = Math.max(1, numberOr(map.w, Math.max(...rows.map((row) => textOr(row).length)))); const pad = 8; const cell = Math.max(2, Math.min((width - pad * 2) / w, (height - pad * 2) / h)); const ox = (width - cell * w) / 2; const oy = (height - cell * h) / 2;
    const explored = new Set(Array.isArray(map.explored) ? map.explored.map(textOr) : []); const hasExplored = explored.size > 0;
    for (let y = 0; y < h; y += 1) for (let x = 0; x < w; x += 1) {
      const tile = textOr(rows[y] && rows[y][x], "#"); const known = !hasExplored || explored.has(`${x}:${y}`) || (Math.abs(x - numberOr(map.px)) <= 2 && Math.abs(y - numberOr(map.py)) <= 2); if (!known) continue;
      ctx.fillStyle = tile === "#" ? "#1a2c37" : tile === "~" ? "#12343a" : "#24424a"; ctx.fillRect(ox + x * cell, oy + y * cell, Math.max(1, cell - .5), Math.max(1, cell - .5));
    }
    const point = (x, y, color, size = 2.5) => { if (!Number.isFinite(Number(x)) || !Number.isFinite(Number(y))) return; ctx.fillStyle = color; ctx.beginPath(); ctx.arc(ox + (Number(x) + .5) * cell, oy + (Number(y) + .5) * cell, Math.max(1.5, Math.min(size, cell * .45)), 0, Math.PI * 2); ctx.fill(); };
    (Array.isArray(map.points) ? map.points : []).filter((item) => item && !item.done).forEach((item) => point(item.x, item.y, "#69b4ac", 2.5));
    (Array.isArray(map.portals) ? map.portals : []).filter((item) => item && item.active !== false).forEach((item) => point(item.x, item.y, "#d8a64c", 3));
    (Array.isArray(map.enemies) ? map.enemies : []).filter((item) => item && item.alive !== false).forEach((item) => point(item.x, item.y, "#e0725d", 2.2));
    point(map.px, map.py, "#f1c976", 3.5);
  }

  function handleClick(event) {
    const target = event.target.closest("button, a"); if (!target || !ui.app.contains(target)) return;
    if (target.tagName === "BUTTON") event.stopPropagation();
    if (target.dataset.panel) { openPanel(target.dataset.panel); return; }
    if (target.dataset.closePanel !== undefined) { closePanels(); return; }
    if (target.dataset.exchangeFilter) { runtime.exchangeFilter = target.dataset.exchangeFilter; $$("[data-exchange-filter]").forEach((tab) => { const active = tab.dataset.exchangeFilter === runtime.exchangeFilter; tab.classList.toggle("is-active", active); tab.setAttribute("aria-selected", String(active)); }); renderExchange(runtime.state || {}); return; }
    const enterId = target.dataset.enterWorld;
    if (enterId) { if (!target.disabled) { runtime.lastWorldId = enterId; requestAction("enter", { world_id: enterId }, { kind: "normal" }); } return; }
    if (target.dataset.skillSlot !== undefined) { requestSkill(Number(target.dataset.skillSlot)); return; }
    if (target.dataset.choiceIndex !== undefined) {
      const choices = runtime.state?.dialog?.choices; const choice = Array.isArray(choices) ? choices[Number(target.dataset.choiceIndex)] : null; if (choice) requestAction("choice", { option_id: choice.id }, { kind: "normal" }); return;
    }
    if (target.dataset.upgradeId !== undefined) { requestAction("upgrade", { id: target.dataset.upgradeId }, { kind: "normal" }); return; }
    if (target.dataset.equipId !== undefined) { requestAction("equip", { id: target.dataset.equipId }, { kind: "normal" }); return; }
    if (target.dataset.exchangeId !== undefined) { requestAction("exchange", { id: target.dataset.exchangeId }, { kind: "normal" }); return; }
    const action = target.dataset.action;
    if (!action) return;
    switch (action) {
      case "retry-view": loadInitial(true); break;
      case "retry-renderer": if (runtime.rendererFailed) retryRenderer(); else loadInitial(true); break;
      case "go-hub": event.preventDefault(); if (currentMode() === "world") togglePause(); else if (currentMode() === "result" || currentMode() === "dead") requestAction("continue", {}, { kind: "normal" }); else if (currentMode() === "dialog") showToast("请先完成当前通讯。", "info", 2400); break;
      case "resume": requestAction("resume", {}, { kind: "normal" }); break;
      case "new": if (!runtime.state?.has_save || window.confirm("已有存档。新建旅程会先备份当前存档，确定开始新的旅程吗？")) requestAction("new", {}, { kind: "normal" }); break;
      case "pause": togglePause(); break;
      case "resume-pause": runtime.paused = false; clearHeld(); renderPause(); ui.gameCanvas?.focus(); break;
      case "retreat": if (window.confirm("安全撤离会放弃本次现场进度，确定返回归航站吗？")) { runtime.paused = false; requestAction("retreat", {}, { kind: "normal" }); } break;
      case "attack": requestAttack(); break;
      case "interact": requestInteract(); break;
      case "heal": requestHeal(); break;
      case "dash": requestDash(); break;
      case "extract": requestAction("extract", {}, { kind: "normal" }); break;
      case "retry": requestAction("restart", {}, { kind: "normal" }); break;
      case "continue": requestAction("continue", {}, { kind: "normal" }); break;
      default: break;
    }
  }

  function handleKeyDown(event) {
    if (isTypingTarget(event.target)) return;
    const key = textOr(event.key).toLowerCase();
    if (key === "escape") { event.preventDefault(); if (runtime.openPanel) { closePanels(); return; } if (currentMode() === "world") togglePause(); return; }
    if (key === "t") { event.preventDefault(); if (runtime.openPanel === "taskPanel") closePanels(); else openPanel("taskPanel"); return; }
    if (key === "m") { if (currentMode() === "world" && !runtime.paused) { event.preventDefault(); toggleMap(); } return; }
    if (!worldInputAllowed()) return;
    if (DIRECTIONS[key]) { event.preventDefault(); if (!runtime.held.has(key)) { runtime.held.set(key, DIRECTIONS[key]); runtime.heldOrder.push(key); setFacing(DIRECTIONS[key]); requestMove(DIRECTIONS[key]); } return; }
    if (key === "shift") { event.preventDefault(); if (!event.repeat) requestDash(); return; }
    if (key === "j") { event.preventDefault(); if (!event.repeat) requestAttack(); return; }
    if (key === "f") { event.preventDefault(); if (!event.repeat) requestInteract(); return; }
    if (key === "h") { event.preventDefault(); if (!event.repeat) requestHeal(); return; }
    if (key === "q" || key === "e" || key === "r") { event.preventDefault(); if (!event.repeat) requestSkill(SKILL_KEYS.indexOf(key.toUpperCase())); }
  }

  function handleKeyUp(event) { const key = textOr(event.key).toLowerCase(); if (DIRECTIONS[key]) { runtime.held.delete(key); runtime.heldOrder = runtime.heldOrder.filter((entry) => entry !== key); } }

  async function loadInitial(isRetry = false) {
    if (runtime.actionBusy || (runtime.booting && !isRetry)) return;
    runtime.booting = true; runtime.tickEnabled = true; runtime.paused = false; clearHeld(); clearQueuedActions(); ui.dialogModal.hidden = true; ui.outcomeModal.hidden = true; if (ui.apiRetryButton) ui.apiRetryButton.hidden = true; setMode("boot"); setConnection("pending", "连接中"); if (ui.bootError) ui.bootError.hidden = true;
    try { const state = await campaignView(); renderState(state); await startTp06aHeartbeat(); }
    catch (error) { runtime.booting = false; setConnection("warn", "连接失败"); ui.bootError.hidden = false; ui.bootError.textContent = `${error.message || error}。可以按“重试连接”再次读取。`; showToast("无法读取现场状态，请重试。", "error", 5200); }
  }

  function recordTp06a(status, detail = {}) {
    const record = { status, atMs: Math.round(performance.now()), ...detail };
    runtime.tp06a.records.push(record);
    if (runtime.tp06a.records.length > 16) runtime.tp06a.records.shift();
    runtime.tp06a.status = status;
    window.__TP06A_DIAGNOSTIC__ = { status, records: runtime.tp06a.records.slice() };
    return record;
  }

  function validateTp06aSnapshot(snapshot) {
    if (!snapshot || typeof snapshot !== "object") return { ok: false, reason: "invalid_view" };
    if (snapshot.protocol !== "continuous-ipc" || snapshot.version !== 1) return { ok: false, reason: "invalid_view" };
    if (snapshot.worldId !== "tp06a-synthetic" || snapshot.worldEpoch !== runtime.tp06a.epoch || snapshot.ackSeq !== 0) return { ok: false, reason: "invalid_view" };
    if (!Number.isSafeInteger(Number(snapshot.serverTick)) || Number(snapshot.serverTick) <= runtime.tp06a.lastTick) return { ok: false, reason: "stale_tick" };
    if (!Number.isFinite(Number(snapshot.serverTimeMs))) return { ok: false, reason: "invalid_view" };
    const transform = snapshot.player?.transform;
    const position = transform?.positionM;
    const velocity = snapshot.player?.velocityMps;
    const numbers = [position?.xM, position?.yM, position?.zM, transform?.yawRad, velocity?.xM, velocity?.yM, velocity?.zM];
    if (!numbers.every((value) => Number.isFinite(Number(value)))) return { ok: false, reason: "invalid_view" };
    return { ok: true, tick: Number(snapshot.serverTick), position, yawRad: Number(transform.yawRad) };
  }

  async function stopTp06aHeartbeat(reason = "stop") {
    if (runtime.tp06a.stopPromise) return runtime.tp06a.stopPromise;
    const sessionId = runtime.tp06a.sessionId;
    const generation = runtime.tp06a.generation;
    const worldEpoch = runtime.tp06a.epoch;
    runtime.tp06a.sessionId = null;
    runtime.tp06a.channel = null;
    runtime.tp06a.lastTick = -1;
    runtime.tp06a.active = false;
    runtime.tp06a.starting = false;
    if (sessionId === null || !hasTauri()) { recordTp06a(reason, { generation, worldEpoch }); return; }
    runtime.tp06a.stopPromise = (async () => {
      try { await window.__TAURI__.core.invoke("tp06a_stop", { sessionId }); recordTp06a(reason, { sessionId, generation, worldEpoch }); }
      catch (error) { recordTp06a("stop_failed", { sessionId, generation, worldEpoch, error: String(error?.message || error) }); }
      finally { runtime.tp06a.stopPromise = null; }
    })();
    return runtime.tp06a.stopPromise;
  }

  function allocateTp06aStart() {
    const previousEpoch = Number(runtime.tp06a.epoch) || 0;
    const epoch = previousEpoch + 1;
    if (!Number.isSafeInteger(epoch) || epoch <= previousEpoch) throw new Error("TP-06A epoch exhausted");
    runtime.tp06a.epoch = epoch;
    runtime.tp06a.generation += 1;
    return { epoch, generation: runtime.tp06a.generation };
  }

  async function startTp06aHeartbeat() {
    if (!hasTauri() || !runtime.rendererReady) return;
    const core = window.__TAURI__.core;
    await stopTp06aHeartbeat("replace");
    let allocation;
    try { allocation = allocateTp06aStart(); }
    catch (error) { recordTp06a("start_failed", { error: String(error?.message || error) }); return; }
    const { epoch, generation } = allocation;
    runtime.tp06a.starting = true;
    if (typeof core.Channel !== "function") {
      runtime.tp06a.starting = false;
      recordTp06a("channel_unavailable", { worldEpoch: epoch, generation });
      return;
    }
    const channel = new core.Channel();
    runtime.tp06a.channel = channel;
    runtime.tp06a.lastTick = -1;
    channel.onmessage = (snapshot) => {
      if (!runtime.tp06a.starting && !runtime.tp06a.active) { recordTp06a("stale_session", { worldEpoch: snapshot?.worldEpoch, serverTick: snapshot?.serverTick, generation }); return; }
      const checked = validateTp06aSnapshot(snapshot);
      if (!checked.ok) { recordTp06a(checked.reason, { worldEpoch: snapshot?.worldEpoch, serverTick: snapshot?.serverTick }); return; }
      runtime.tp06a.lastTick = checked.tick;
      recordTp06a("accepted", { worldEpoch: runtime.tp06a.epoch, generation, serverTick: checked.tick, ackSeq: 0, worldId: "tp06a-synthetic", position: checked.position, yawRad: checked.yawRad });
    };
    try {
      runtime.tp06a.sessionId = await core.invoke("tp06a_start", { channel, worldEpoch: epoch });
      runtime.tp06a.starting = false;
      runtime.tp06a.active = true;
      recordTp06a("started", { sessionId: runtime.tp06a.sessionId, worldEpoch: epoch, generation });
    } catch (error) {
      runtime.tp06a.starting = false;
      runtime.tp06a.active = false;
      runtime.tp06a.sessionId = null;
      runtime.tp06a.channel = null;
      recordTp06a("start_failed", { error: String(error?.message || error), worldEpoch: epoch, generation });
    }
  }

  function handleResolution() {
    if (!ui.resolutionSelect) return;
    const value = textOr(ui.resolutionSelect.value, runtime.preferences.resolution); const [width, height] = value.split("x").map(Number); runtime.preferences.resolution = value; writePreferences();
    if (!hasTauri()) { ui.resolutionSelect.value = runtime.preferences.resolution; showToast("浏览器预览固定当前窗口；桌面版可切换分辨率。", "info", 3300); return; }
    window.__TAURI__.core.invoke("campaign_window", { width, height }).then(() => showToast(`窗口已切换至 ${width} × ${height}。`, "success", 2200)).catch((error) => showToast(`窗口切换失败：${error.message || error}`, "error"));
  }

  function initAudio() {
    if (!ui.audioToggle) return;
    ui.audioToggle.setAttribute("aria-pressed", String(!runtime.preferences.muted)); ui.audioToggle.title = runtime.preferences.muted ? "声音已关闭" : "声音已开启";
  }

  const AudioFx = {
    ctx: null,
    event(state, sourceAction = null) {
      if (runtime.preferences.muted || !state || !state.last_event) return;
      const event = textOr(state.last_event); const key = feedbackKey(state, sourceAction);
      if ((sourceAction === "tick" && event === runtime.lastFeedbackEvent) || key === runtime.lastFeedbackKey) return;
      runtime.lastFeedbackKey = key; runtime.lastFeedbackEvent = event;
      if (/攻击|命中|击倒|受伤/.test(event)) this.beep(130, .07, "sawtooth"); else if (/回收|完成|撤离|解锁/.test(event)) this.beep(620, .11, "triangle");
    },
    ensure() { if (!this.ctx) { try { this.ctx = new (window.AudioContext || window.webkitAudioContext)(); } catch (_) { this.ctx = null; } } if (this.ctx?.state === "suspended") this.ctx.resume().catch(() => {}); return this.ctx; },
    beep(frequency, duration, type = "sine") { const ctx = this.ensure(); if (!ctx) return; try { const oscillator = ctx.createOscillator(); const gain = ctx.createGain(); oscillator.type = type; oscillator.frequency.setValueAtTime(frequency, ctx.currentTime); gain.gain.setValueAtTime(.045, ctx.currentTime); gain.gain.exponentialRampToValueAtTime(.001, ctx.currentTime + duration); oscillator.connect(gain); gain.connect(ctx.destination); oscillator.start(); oscillator.stop(ctx.currentTime + duration + .015); } catch (_) { /* 声音失败不阻断 */ } },
  };

  function handleAudioToggle() { runtime.preferences.muted = !runtime.preferences.muted; writePreferences(); initAudio(); if (!runtime.preferences.muted) AudioFx.beep(520, .08); }

  function startTimers() {
    runtime.moveTimer = window.setInterval(() => { if (!worldInputAllowed()) return; const direction = currentHeldDirection(); if (direction) requestMove(direction); }, 160);
    runtime.tickTimer = window.setInterval(() => { if (!runtime.tickEnabled || !worldInputAllowed()) return; requestAction("tick", {}, { kind: "tick" }); }, 250);
  }

  function init() {
    readPreferences();
    if (ui.resolutionSelect) { ui.resolutionSelect.value = runtime.preferences.resolution; if (!hasTauri()) { ui.resolutionSelect.disabled = true; ui.resolutionSelect.title = "桌面版可切换窗口分辨率"; } }
    if (!hasTauri() && ui.saveButton) { ui.saveButton.disabled = true; ui.saveButton.title = "内存试玩，关闭预览不保存"; ui.saveButton.setAttribute("aria-label", "内存试玩，关闭预览不保存"); }
    initAudio(); setMapOpen(runtime.mapOpen, false);
    ui.app.addEventListener("click", handleClick); document.addEventListener("keydown", handleKeyDown); document.addEventListener("keyup", handleKeyUp); window.addEventListener("blur", clearHeld); window.addEventListener("focus", clearHeld); document.addEventListener("visibilitychange", () => { if (document.hidden) clearHeld(); });
    ui.audioToggle?.addEventListener("click", (event) => { event.stopPropagation(); handleAudioToggle(); }); ui.saveButton?.addEventListener("click", async (event) => { event.stopPropagation(); if (!hasTauri()) { showToast("内存试玩，关闭预览不保存。", "info", 4600); return; } try { const result = await campaignSave(); if (result && result.persisted === false) showToast(result.message || "内存试玩，关闭预览不保存。", "info", 4600); else showToast("进度已保存。", "success", 2200); } catch (error) { showToast(`保存失败：${error.message || error}`, "error"); } }); ui.resolutionSelect?.addEventListener("change", handleResolution);
    if (ui.loadoutToggle && ui.loadoutPanel) ui.loadoutToggle.addEventListener("click", () => { const open = ui.loadoutPanel.hidden; ui.loadoutPanel.hidden = !open; ui.loadoutToggle.setAttribute("aria-expanded", String(open)); });
    if (ui.loadoutClose && ui.loadoutPanel) ui.loadoutClose.addEventListener("click", () => { ui.loadoutPanel.hidden = true; ui.loadoutToggle?.setAttribute("aria-expanded", "false"); });
    ui.mapToggle?.addEventListener("click", (event) => { event.stopPropagation(); toggleMap(); });
    initRenderer(); startTimers(); loadInitial();
    window.addEventListener("beforeunload", () => { void stopTp06aHeartbeat("beforeunload"); }, { once: true });
    window.addEventListener("pagehide", () => { void stopTp06aHeartbeat("pagehide"); }, { once: true });
  }

  init();
})();
