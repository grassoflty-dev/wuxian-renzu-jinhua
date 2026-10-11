/* 无限轮回 · 引擎：状态机 / 打字机 / 战斗 / WebAudio */
"use strict";

/* ---------------- 全局状态 ---------------- */
const S = {
  name: "郑吒", hp: 100, san: 100, points: 0, weapon: null,
  geneLock: false, geneLockUsed: false,
  flags: {},
  team: { one: "一号·马修", rain: "蕾恩", kaplan: "卡普兰", jd: "J.D." },
  deadTeam: [],
  sceneId: "s_office",
  fight: null,
  laserFails: 0,
};

const WEAPONS = {
  axe:   { name: "消防斧", dmg: [22, 34], heavy: true, desc: "高伤 · 出手沉重" },
  gun:   { name: "9mm手枪", dmg: [14, 20], ammo: 6,    desc: "中庸 · 远程6发" },
  sword: { name: "军刀",   dmg: [10, 16], fast: true, desc: "迅捷 · 低伤连击" },
};

/* ---------------- 工具 ---------------- */
const $ = id => document.getElementById(id);
const rnd = (a, b) => a + Math.floor(Math.random() * (b - a + 1));
const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

function killTeam(key, why) {
  if (!S.deadTeam.includes(key)) {
    S.deadTeam.push(key);
    if (why) flog(`<span class="crit">【队伍】${S.team[key]} —— 阵亡</span>`);
    refreshHUD();
  }
}

function hurtPlayer(n, deathScene) {
  S.hp = clamp(S.hp - n, 0, 100);
  AudioSys.sfx("hit");
  document.body.classList.add("hurt");
  refreshHUD();
  setTimeout(() => document.body.classList.remove("hurt"), 900);
  if (S.hp <= 0) { setTimeout(() => go(deathScene || "e_generic"), 700); return true; }
  return false;
}
function applySan(d, why) {
  S.san = clamp(S.san + d, 0, 100);
  refreshHUD();
  if (why && S.fight) flog(`<span class="miss">[理智] ${why} ${d > 0 ? "+" : ""}${d}</span>`);
  if (S.san <= 0) { setTimeout(() => go("e_sancollapse"), 600); }
}
function gainPts(n, why) {
  S.points += n; refreshHUD();
  if (why && S.fight) flog(`<span class="hit">[主神] ${why} +${n}奖励点数</span>`);
}

/* ---------------- 音频（WebAudio 合成环境音）---------------- */
const AudioSys = {
  ctx: null, droneNodes: [], droneMaster: null, heartTimer: null,
  ensure() {
    if (!this.ctx) {
      try { this.ctx = new (window.AudioContext || window.webkitAudioContext)(); } catch (e) { return; }
    }
    if (this.ctx.state === "suspended") this.ctx.resume();
  },
  drone(mood) {
    this.ensure(); if (!this.ctx) return;
    this.droneStop();
    const c = this.ctx;
    const g = c.createGain(); g.gain.value = 0; g.connect(c.destination);
    const filt = c.createBiquadFilter(); filt.type = "lowpass";
    filt.frequency.value = mood === "danger" ? 320 : 180;
    filt.connect(g);
    const freqs = mood === "danger" ? [46, 61] : [38, 57];
    freqs.forEach((f, i) => {
      const o = c.createOscillator(); o.type = i ? "triangle" : "sawtooth";
      o.frequency.value = f; o.detune.value = rnd(-8, 8);
      o.connect(filt); o.start(); this.droneNodes.push(o);
    });
    const buf = c.createBuffer(1, c.sampleRate * 2, c.sampleRate);
    const d = buf.getChannelData(0);
    for (let i = 0; i < d.length; i++) d[i] = (Math.random() * 2 - 1) * .25;
    const ns = c.createBufferSource(); ns.buffer = buf; ns.loop = true;
    const ng = c.createGain(); ng.gain.value = mood === "danger" ? .05 : .02;
    ns.connect(ng); ng.connect(filt); ns.start(); this.droneNodes.push(ns);
    g.gain.linearRampToValueAtTime(.16, c.currentTime + 2);
    this.droneMaster = g;
  },
  droneStop() {
    if (this.droneMaster) { try { this.droneMaster.gain.linearRampToValueAtTime(0, this.ctx.currentTime + .6); } catch (e) {} }
    const nodes = this.droneNodes; this.droneNodes = [];
    setTimeout(() => nodes.forEach(n => { try { n.stop(); } catch (e) {} }), 700);
  },
  heartbeat(on) {
    if (this.heartTimer) { clearInterval(this.heartTimer); this.heartTimer = null; }
    if (!on) return;
    const thump = () => {
      this.ensure(); if (!this.ctx) return;
      const c = this.ctx, o = c.createOscillator(), g = c.createGain();
      o.type = "sine"; o.frequency.setValueAtTime(58, c.currentTime);
      o.frequency.exponentialRampToValueAtTime(30, c.currentTime + .18);
      g.gain.setValueAtTime(.5, c.currentTime);
      g.gain.exponentialRampToValueAtTime(.001, c.currentTime + .22);
      o.connect(g); g.connect(c.destination); o.start(); o.stop(c.currentTime + .24);
    };
    thump(); setTimeout(thump, 260);
    this.heartTimer = setInterval(() => { thump(); setTimeout(thump, 260); }, 1150);
  },
  sfx(kind) {
    this.ensure(); if (!this.ctx) return;
    const c = this.ctx, t = c.currentTime;
    let o, g;
    if (kind === "laser") {
      o = c.createOscillator(); g = c.createGain();
      o.type = "sawtooth"; o.frequency.setValueAtTime(1400, t);
      o.frequency.exponentialRampToValueAtTime(90, t + .5);
      g.gain.setValueAtTime(.28, t); g.gain.exponentialRampToValueAtTime(.001, t + .55);
      o.connect(g); g.connect(c.destination); o.start(t); o.stop(t + .6);
    } else if (kind === "hit") {
      o = c.createOscillator(); g = c.createGain();
      o.type = "square"; o.frequency.setValueAtTime(160, t);
      o.frequency.exponentialRampToValueAtTime(40, t + .12);
      g.gain.setValueAtTime(.3, t); g.gain.exponentialRampToValueAtTime(.001, t + .14);
      o.connect(g); g.connect(c.destination); o.start(t); o.stop(t + .15);
    } else {
      o = c.createOscillator(); g = c.createGain();
      o.type = "sine"; o.frequency.value = 720;
      g.gain.setValueAtTime(.12, t); g.gain.exponentialRampToValueAtTime(.001, t + .07);
      o.connect(g); g.connect(c.destination); o.start(t); o.stop(t + .08);
    }
  },
  voice(file) {
    try {
      if (this._v) this._v.pause();
      this._v = new Audio("assets/audio/" + file);
      this._v.volume = .95;
      this._v.play().catch(() => {});
    } catch (e) {}
  },
};

/* ---------------- 打字机 ---------------- */
const TW = { timer: null, full: "", done: false, cb: null };
function typewrite(el, html, cb) {
  clearTimeout(TW.timer);
  TW.full = html; TW.done = false; TW.cb = cb || null;
  const tokens = []; let tag = "", text = "";
  for (let i = 0; i < html.length; i++) {
    const ch = html[i];
    if (ch === "<") { if (text) { tokens.push({ t: "x", v: text }); text = ""; } tag += ch; }
    else if (ch === ">") { tag += ch; tokens.push({ t: "g", v: tag }); tag = ""; }
    else if (tag) tag += ch;
    else text += ch;
  }
  if (text) tokens.push({ t: "x", v: text });
  el.innerHTML = ""; el.classList.add("cursorBlink");
  let out = "";
  function step() {
    if (TW.done) return;
    let budget = 2;
    while (budget > 0 && TW.pos < tokens.length) {
      const tk = tokens[TW.pos];
      if (tk.t === "g") { out += tk.v; TW.pos++; continue; }
      if (tk.v.length <= budget) { out += tk.v; budget -= tk.v.length; TW.pos++; }
      else { out += tk.v.slice(0, budget); tk.v = tk.v.slice(budget); budget = 0; }
    }
    el.innerHTML = out;
    $("narrBox").scrollTop = $("narrBox").scrollHeight;
    if (TW.pos >= tokens.length || TW.pos === undefined) {
      el.classList.remove("cursorBlink"); TW.done = true;
      if (TW.cb) TW.cb();
    } else TW.timer = setTimeout(step, 26);
  }
  TW.pos = 0;
  step();
}
function skipType() {
  if (!TW.done) {
    TW.done = true; clearTimeout(TW.timer);
    $("narrText").innerHTML = TW.full;
    $("narrText").classList.remove("cursorBlink");
    if (TW.cb) { const cb = TW.cb; TW.cb = null; cb(); }
    return true;
  }
  return false;
}

/* ---------------- 背景切换 ---------------- */
let bgFlip = false;
function showBg(imgFile, locName) {
  if (!imgFile) return;
  const a = $("bgA"), b = $("bgB");
  const next = bgFlip ? a : b, cur = bgFlip ? b : a;
  next.style.backgroundImage = `url('assets/img/${imgFile}')`;
  next.classList.add("show"); cur.classList.remove("show");
  bgFlip = !bgFlip;
  if (locName !== undefined) $("locName").textContent = locName || "";
}

/* ---------------- HUD ---------------- */
function refreshHUD() {
  $("hpFill").style.width = clamp(S.hp, 0, 100) + "%";
  $("sanFill").style.width = clamp(S.san, 0, 100) + "%";
  $("hpVal").textContent = Math.max(0, S.hp);
  $("sanVal").textContent = Math.max(0, S.san);
  $("ptsVal").textContent = S.points;
  $("wpnVal").textContent = S.weapon ? WEAPONS[S.weapon].name : "—";
  const t = $("teamHud"); t.innerHTML = "";
  Object.entries(S.team).forEach(([k, nm]) => {
    const dead = S.deadTeam.includes(k);
    const sp = document.createElement("span");
    sp.className = dead ? "dead" : "alive";
    sp.textContent = nm.split("·")[0];
    t.appendChild(sp);
    t.appendChild(document.createTextNode(" "));
  });
  document.body.classList.toggle("hurt", S.hp <= 30);
  AudioSys.heartbeat(S.hp <= 35 && S.hp > 0);
}

/* ---------------- 节点渲染 ---------------- */
function renderNode(node) {
  $("speaker").textContent = "";
  $("choices").innerHTML = "";
  $("fightBar").style.display = "none";
  if (node.mood) AudioSys.drone(node.mood);
  showBg(node.bg, node.loc);

  if (node.fightCfg) {           // 战斗场景
    $("speaker").textContent = node.speaker || "⚔ 遭遇战";
    typewrite($("narrText"), typeof node.text === "function" ? node.text(S) : (node.text || ""), () => {
      startFight(node.fightCfg);
    });
    return;
  }
  $("speaker").textContent = node.speaker || "";
  if (node.voice) AudioSys.voice(node.voice);
  typewrite($("narrText"), typeof node.text === "function" ? node.text(S) : (Array.isArray(node.text) ? node.text.join("\n\n") : (node.text || "")), () => {
    const list = typeof node.choices === "function" ? node.choices(S) : (node.choices || []);
    const choicesEl = $("choices"); choicesEl.innerHTML = "";
    list.forEach(c => {
      if (c.if && !c.if(S)) return;
      const btn = document.createElement("button");
      btn.className = "choice";
      btn.innerHTML = c.t + (c.sub ? `<span class="sub">${c.sub}</span>` : "");
      btn.onclick = () => {
        AudioSys.sfx("click");
        if (c.do) c.do(S);
        saveGame();
        go(typeof c.go === "function" ? c.go(S) : c.go);
      };
      choicesEl.appendChild(btn);
    });
  });
}

function go(sceneOrFn, effectFn) {
  if (effectFn) effectFn(S);
  const sid = typeof sceneOrFn === "function" ? sceneOrFn(S) : sceneOrFn;
  if (!sid || !SCENES[sid]) return;
  S.sceneId = sid;
  saveGame();
  refreshHUD();
  const node = SCENES[sid];
  if (node.video) {
    playCine(node.video, node.cineLabel || "", () => renderNode(node));
  } else if (node.overlay) {
    showOverlay(node);
  } else {
    renderNode(node);
  }
}

/* ---------------- 视频过场 ---------------- */
function playCine(src, label, onEnd) {
  const w = $("cineWrap"), v = $("cineVid");
  $("cineTag").textContent = label || "MINIMAX H3 本地生成过场影像";
  w.classList.add("on");
  let ended = false;
  const done = () => {
    if (ended) return; ended = true;
    w.classList.remove("on");
    try { v.pause(); } catch (e) {}
    onEnd && onEnd();
  };
  v.onended = done; v.onerror = done;
  v.src = "assets/video/" + src;
  v.play().catch(done);
  $("cineSkip").onclick = done;
}

/* ---------------- 通用卡片覆盖层 ---------------- */
function showCard(html, buttons) {
  const card = $("ovCard");
  card.innerHTML = html;
  const wrap = document.createElement("div");
  wrap.style.cssText = "display:flex;flex-direction:column;gap:10px;margin-top:22px;";
  buttons.forEach(b => {
    const btn = document.createElement("button");
    btn.className = "mbtn"; btn.textContent = b.t;
    btn.onclick = () => { AudioSys.sfx("click"); $("endOverlay").classList.remove("on"); b.do && b.do(); };
    wrap.appendChild(btn);
  });
  card.appendChild(wrap);
  $("endOverlay").classList.add("on");
}

function showOverlay(node) {
  const ov = node.overlay;
  if (ov.death) {
    recordDeath(ov.death.title, ov.death.cause);
    AudioSys.drone("calm");
  }
  showCard(ov.html(S), ov.buttons(S));
  if (ov.voice) AudioSys.voice(ov.voice);
}

/* ---------------- 战斗系统 ---------------- */
function startFight(cfg) {
  S.fight = Object.assign({}, cfg, { hp: cfg.hp, maxHp: cfg.hp });
  const f = S.fight;
  $("fightBar").style.display = "block";
  $("enemyName").textContent = f.name;
  $("enemyFill").style.width = "100%";
  $("enemyHpTxt").textContent = `${f.hp}/${f.maxHp}`;
  $("fightLog").innerHTML = "";
  flog(f.intro || "");
  renderFightChoices();
}
function updateEnemyBar() {
  const f = S.fight;
  $("enemyFill").style.width = clamp(f.hp / f.maxHp * 100, 0, 100) + "%";
  $("enemyHpTxt").textContent = `${Math.max(0, f.hp)}/${f.maxHp}`;
}
function flog(html) {
  const el = $("fightLog");
  if (!el) return;
  const div = document.createElement("div"); div.innerHTML = html;
  el.appendChild(div); el.scrollTop = el.scrollHeight;
}
function weaponLabel() { return WEAPONS[S.weapon] ? WEAPONS[S.weapon].name : "武器"; }

function renderFightChoices() {
  const f = S.fight, w = WEAPONS[S.weapon];
  const choicesEl = $("choices"); choicesEl.innerHTML = "";
  const opts = [];
  const fname = typeof f.finisherName === "function" ? f.finisherName(S, f) : (f.finisherName || "终结技");
  if (f.finisherIf && f.finisherIf(S, f)) {
    opts.push({ t: "☠ 终结技 · " + fname, sub: "就是现在！", act: "finisher" });
  }
  if (S.weapon === "gun") {
    if (w.ammo > 0) opts.push({ t: `▸ 射击（剩余 ${w.ammo} 发）`, sub: "远程 · 稳定", act: "shoot" });
    opts.push({ t: "▸ 枪托肉搏", sub: "子弹告急时的选择", act: "melee_gun" });
  } else {
    opts.push({ t: `▸ ${w.name}攻击`, sub: w.desc, act: "attack" });
  }
  opts.push({ t: "▸ 全力一搏", sub: "高风险 · 高伤害 · 易露破绽", act: "allout" });
  opts.push({ t: "▸ 后撤观察", sub: "恢复架势 · 提升下一轮闪避", act: "guard" });
  opts.forEach(o => {
    const btn = document.createElement("button");
    btn.className = "choice";
    btn.innerHTML = o.t + `<span class="sub">${o.sub}</span>`;
    btn.onclick = () => { AudioSys.sfx("click"); doFightAction(o.act); };
    choicesEl.appendChild(btn);
  });
}

function doFightAction(act) {
  const f = S.fight, w = WEAPONS[S.weapon];
  $("choices").innerHTML = "";
  let hitChance = .82, dmgMul = 1;

  if (act === "guard") {
    f.guardTurn = true;
    flog(`<span class="miss">你后撤半步压低重心，视线锁死${f.name}的一举一动。</span>`);
    setTimeout(enemyTurn, 420);
    return;
  }
  if (act === "finisher") {
    AudioSys.sfx("laser");
    const fdesc = typeof f.finisherDesc === "function" ? f.finisherDesc(S) : (f.finisherDesc || "你抓住了那个唯一的瞬间。");
    flog(`<span class="crit">☠ ${fdesc}</span>`);
    finishFightWin();
    return;
  }
  if (act === "allout") { hitChance = .55; dmgMul = 1.9; }
  if (S.san < 30) hitChance -= .12;
  if (S.flags.B1) hitChance += .06;   // 支线B1：掌握敌人行动规律

  if (Math.random() < hitChance) {
    let base = 0;
    if (act === "shoot") {
      w.ammo--; base = rnd(w.dmg[0], w.dmg[1]);
      flog(`双手举枪连开数枪——<span class="hit">命中！伤害 ${base}</span>（剩余${w.ammo}发）`);
    } else if (act === "melee_gun") {
      base = rnd(8, 13);
      flog(`枪托狠狠砸下——<span class="hit">伤害 ${base}</span>`);
    } else if (w.heavy) {
      base = Math.round(rnd(w.dmg[0], w.dmg[1]) * dmgMul);
      const crit = Math.random() < .18;
      if (crit) base = Math.round(base * 1.6);
      flog(`${crit ? '<span class="crit">会心一击！</span>' : ""}${weaponLabel()}劈落——<span class="hit">伤害 ${base}</span>${act === "allout" ? '（全力）' : ""}`);
    } else if (w.fast) {
      const hits = rnd(2, 3), parts = [];
      for (let i = 0; i < hits; i++) { const d = Math.round(rnd(w.dmg[0], w.dmg[1]) * dmgMul); base += d; parts.push(d); }
      flog(`军刀连续刺出 ${hits} 刀——<span class="hit">${parts.join(" + ")} = 伤害 ${base}</span>`);
    }
    if (S.geneLock) { const bonus = rnd(6, 12); base += bonus; flog(`<span class="crit">【基因锁状态】动作快到不可思议 · 追加伤害 +${bonus}</span>`); }
    f.hp -= base;
    AudioSys.sfx("hit");
    updateEnemyBar();
    if (f.hp <= 0) { finishFightWin(); return; }
    if (f.rageAt && f.hp <= f.rageAt && !f.raged) {
      f.raged = true;
      f.dmg = [f.dmg[0] + 4, f.dmg[1] + 6];
      flog(`<span class="crit">${f.rageText || (f.name + "狂性大发！")}</span>`);
      if (f.onRage) f.onRage(S, f);
    }
  } else {
    flog(`<span class="miss">你的攻势落空了！${f.name}抓住破绽逼近——</span>`);
  }
  setTimeout(enemyTurn, 420);
}

function enemyTurn() {
  const f = S.fight;
  if (!f || f.hp <= 0) return;
  const dodge = (f.guardTurn ? .55 : .16) + (S.geneLock ? .15 : 0);
  if (Math.random() >= dodge) {
    const dmg = Math.max(2, rnd(f.dmg[0], f.dmg[1]) - (S.geneLock ? 4 : 0));
    S.hp = clamp(S.hp - dmg, 0, 100);
    AudioSys.sfx("hit");
    document.body.classList.add("hurt");
    setTimeout(() => document.body.classList.remove("hurt"), 800);
    flog(`<span class="crit">${f.name}击中了你！体力 -${dmg}</span>`);
    refreshHUD();
  } else {
    flog(`<span class="miss">${f.name}扑击而来——你在千钧一发之际避开了！</span>`);
  }
  f.guardTurn = false;
  // 基因锁觉醒判定
  if (S.hp > 0 && S.hp <= 30 && !S.geneLockUsed && S.san >= 20 && f.id === "licker") {
    awakenGeneLock();
    return;
  }
  if (S.hp <= 0) {
    const sc = f.deathScene || "e_generic";
    S.fight = null;
    setTimeout(() => go(sc), 700);
    return;
  }
  renderFightChoices();
}

function awakenGeneLock() {
  S.geneLockUsed = true;
  S.geneLock = true;
  AudioSys.voice("vo_awaken.wav");
  AudioSys.drone("danger");
  refreshHUD();
  showCard(
    `<h2>基 因 锁 · 开</h2>
     <p style="text-align:center;color:#ff8a8a;font-size:15px;letter-spacing:2px">
     就在此刻——你听见了自己的心跳。</p>
     <p>世界慢了下来。舔食者的利爪、飞溅的涎水、管道上滚落的灰尘……一切都清晰得可怕。
     恐惧没有消失，而是被某种更原始的东西<em>点燃</em>了——
     大脑深处某扇从未开启的门被踹开，肾上腺素如同岩浆般涌遍四肢百骸。</p>
     <p style="color:#ffd76a">【基因锁第一阶段 · 觉醒】攻击附带额外伤害，闪避率提升。这是人类潜能的枷锁碎裂之声。</p>
     <p style="color:#666;font-size:12px">「原来……这才是真正的，活着。」</p>`,
    [{ t: "睁 开 眼", do: () => renderFightChoices() }]
  );
}

function finishFightWin() {
  const f = S.fight;
  updateEnemyBar();
  $("choices").innerHTML = "";
  const winScene = typeof f.winScene === "function" ? f.winScene(S) : f.winScene;
  const reward = f.reward || 0;
  S.fight = null;
  setTimeout(() => {
    if (reward) gainPts(reward, f.rewardWhy || "战斗胜利");
    $("fightBar").style.display = "none";
    go(winScene);
  }, 950);
}

/* ---------------- 存档 ---------------- */
function saveGame() {
  try {
    localStorage.setItem("whc1_save", JSON.stringify({
      hp: S.hp, san: S.san, points: S.points, weapon: S.weapon,
      flags: S.flags, deadTeam: S.deadTeam, sceneId: S.sceneId,
      geneLock: S.geneLock, geneLockUsed: S.geneLockUsed,
      laserFails: S.laserFails, name: S.name,
    }));
  } catch (e) {}
}
function loadGame() {
  try {
    const d = JSON.parse(localStorage.getItem("whc1_save"));
    if (!d || !SCENES[d.sceneId]) return false;
    Object.assign(S, d); S.fight = null;
    return true;
  } catch (e) { return false; }
}
function recordDeath(title, cause) {
  try {
    const arr = JSON.parse(localStorage.getItem("whc1_deaths") || "[]");
    arr.unshift({ t: title, c: cause, time: new Date().toLocaleString() });
    localStorage.setItem("whc1_deaths", JSON.stringify(arr.slice(0, 30)));
  } catch (e) {}
}

/* ---------------- 噪点层 ---------------- */
(function grainLoop() {
  const cv = $("grain"), ctx2 = cv.getContext("2d");
  function rs() { cv.width = innerWidth / 3; cv.height = innerHeight / 3; }
  addEventListener("resize", rs); rs();
  setInterval(() => {
    try {
      const img = ctx2.createImageData(cv.width, cv.height);
      for (let i = 0; i < img.data.length; i += 4) {
        const v = Math.random() * 255 | 0;
        img.data[i] = img.data[i + 1] = img.data[i + 2] = v; img.data[i + 3] = 40;
      }
      ctx2.putImageData(img, 0, 0);
    } catch (e) {}
  }, 120);
})();
$("narrBox").addEventListener("click", () => skipType());

/* ---------------- 标题菜单 ---------------- */
$("btnNew").onclick = () => {
  AudioSys.ensure();
  S.hp = 100; S.san = 100; S.points = 0; S.weapon = null;
  S.flags = {}; S.deadTeam = []; S.geneLock = false; S.geneLockUsed = false;
  S.laserFails = 0; S.fight = null;
  WEAPONS.gun.ammo = 6;
  $("titleScreen").style.display = "none";
  $("hud").style.display = "flex"; $("story").style.display = "block";
  refreshHUD();
  go("s_office");
};
$("btnContinue").onclick = () => {
  AudioSys.ensure();
  if (!loadGame()) return;
  WEAPONS.gun.ammo = WEAPONS.gun.ammo ?? 6;
  $("titleScreen").style.display = "none";
  $("hud").style.display = "flex"; $("story").style.display = "block";
  refreshHUD();
  go(S.sceneId);
};
$("btnDeaths").onclick = () => {
  const arr = (() => { try { return JSON.parse(localStorage.getItem("whc1_deaths") || "[]"); } catch (e) { return []; } })();
  $("ovCard").innerHTML = `<h2>死 亡 档 案</h2>
    <p style="text-align:center;color:#666">「在这里死了，就是真的死了。」<br>但主神空间记得每一次轮回。</p>
    <div class="deathArchive">${
      arr.length ? arr.map(d => `<div><b>${d.t}</b> — ${d.c}<br><span style="color:#444;font-size:11px">${d.time}</span></div>`).join("")
        : "<div style='text-align:center;color:#555;padding:20px 0'>暂无记录。你还活着……暂时。</div>"
    }</div>`;
  const back = document.createElement("button");
  back.className = "mbtn"; back.textContent = "返 回"; back.style.marginTop = "18px";
  back.onclick = () => $("endOverlay").classList.remove("on");
  $("ovCard").appendChild(back);
  $("endOverlay").classList.add("on");
};
if (localStorage.getItem("whc1_save")) $("btnContinue").disabled = false;
