//! 引擎：视图渲染 / 选择流转 / 回合制战斗 / 结算

use crate::defs::*;
use crate::state::{Fight, GameState, Mode};
use rand::Rng;
use serde_json::{json, Value};

fn rnd(a: i32, b: i32) -> i32 {
    rand::thread_rng().gen_range(a..=b)
}

/* ---------------- 玩家加成查表总和（数据驱动，替代原内联 str_bonus/vampire if） ----------------
   数值必须与旧公式严格等价（金标准：playthrough + 6 副本 flow）。
   - 基因锁：旧 `if st.gene_lock { rnd(6,12) 攻击 / 闪避+0.15 / 受击-4 }` == GENE_STAGES 一阶表值
   - 血统：旧 `is_vampire → 命中吸血 leech4 / 受击减伤 reduce3` == BLOODLINES vampire 被动
   - 战力：str_bonus×5（攻击）、agi_bonus×0.05（闪避），保持不变（非表化项）
   - 装备：GEAR/WEAPONS/TRESURE_DEFS 常驻减伤/攻击/闪避/倍率（新系统，无旧行为）
*/

/// 装备（护甲/饰品/法宝）常驻闪避总和
fn gear_dodge(st: &GameState) -> f64 {
    let mut d = 0.0;
    let eq = &st.equipment;
    if let Some(armor) = &eq.armor {
        if let Some(g) = crate::items_data::gear_def(armor) { d += g.dodge; }
    }
    if let Some(acc) = &eq.accessory {
        if let Some(g) = crate::items_data::gear_def(acc) { d += g.dodge; }
    }
    for t in eq.treasure.iter().flatten() {
        if let Some(td) = crate::items_data::treasure_def(t) { d += td.dodge; }
    }
    d
}

/// 装备常驻受击减伤总和
fn gear_reduce(st: &GameState) -> i32 {
    let mut r = 0;
    let eq = &st.equipment;
    if let Some(armor) = &eq.armor {
        if let Some(g) = crate::items_data::gear_def(armor) { r += g.dmg_reduce; }
    }
    if let Some(acc) = &eq.accessory {
        if let Some(g) = crate::items_data::gear_def(acc) { r += g.dmg_reduce; }
    }
    for t in eq.treasure.iter().flatten() {
        if let Some(td) = crate::items_data::treasure_def(t) { r += td.dmg_reduce; }
    }
    r
}

/// 装备常驻攻击追加总和
fn gear_atk_flat(st: &GameState) -> i32 {
    let mut a = 0;
    let eq = &st.equipment;
    if let Some(armor) = &eq.armor {
        if let Some(g) = crate::items_data::gear_def(armor) { a += g.atk_flat; }
    }
    if let Some(acc) = &eq.accessory {
        if let Some(g) = crate::items_data::gear_def(acc) { a += g.atk_flat; }
    }
    for t in eq.treasure.iter().flatten() {
        if let Some(td) = crate::items_data::treasure_def(t) { a += td.atk_flat; }
    }
    a
}

/// 装备伤害倍率之积（默认 1.0）
fn gear_dmg_mult(st: &GameState) -> f64 {
    let mut m = 1.0;
    let eq = &st.equipment;
    if let Some(armor) = &eq.armor {
        if let Some(g) = crate::items_data::gear_def(armor) { m *= g.dmg_mult; }
    }
    if let Some(acc) = &eq.accessory {
        if let Some(g) = crate::items_data::gear_def(acc) { m *= g.dmg_mult; }
    }
    for t in eq.treasure.iter().flatten() {
        if let Some(td) = crate::items_data::treasure_def(t) { m *= td.dmg_mult; }
    }
    m
}

/// 基因锁本次攻击追加（查 GENE_STAGES；一阶 == 旧 rnd(6,12)）
fn gene_atk(st: &GameState) -> i32 {
    crate::combat_data::gene_stage_cfg(st).map(|c| rnd(c.atk.0, c.atk.1)).unwrap_or(0)
}

/// 基因锁闪避（一阶 == 旧 +0.15）
fn gene_dodge(st: &GameState) -> f64 {
    crate::combat_data::gene_stage_cfg(st).map(|c| c.dodge).unwrap_or(0.0)
}

/// 基因锁受击减伤（一阶 == 旧 -4）
fn gene_reduce(st: &GameState) -> i32 {
    crate::combat_data::gene_stage_cfg(st).map(|c| c.dmg_reduce).unwrap_or(0)
}

/* ---------------- 血统熟练度强度（bloodline_tiers 进阶路线） ----------------
   强度公式：bloodline_strength(st) = 当前阶 base + contamination（贡献值）。
   - 每次使用当前血统相关技能/攻击：contrib += k（当前阶系数），proficiency += 1。
   - 进阶：贡献砍半（向下取整）、换新 base、保底不下降（见 bloodline_tiers::try_advance_bloodline）。

   战斗数值接入映射（写清楚）：
   - 攻击追加：血统被动 atk_flat 之外，再叠加 `strength / 100`（向下取整，>=1 才有实际收益）。
     → 阶1 base=100 初始即 +1 攻击；贡献练到 1000 时 +10；高级 10000 起步 +100。
   - 受击减伤：血统被动 dmg_reduce 之外，再叠加 `strength / 200`（减半斜率，避免越堆越肉）。
   - 吸血：被动 leech_on_hit 之外，再叠加 `strength / 1000`（高级血统才明显）。
   全部为「基础被动 + strength 缩放」的加法映射，血统为空 → strength=0 → 零影响（零回归）。
*/

/// 血统熟练度贡献：每次使用当前血统相关技能/攻击调用一次。
/// contrib += k（当前阶系数），proficiency += 1（使用次数展示口径）。
fn bloodline_use(st: &mut GameState) {
    let id = match crate::combat_data::current_bloodline_id(st) {
        Some(id) => id.to_string(),
        None => return,
    };
    let tier = crate::combat_data::current_bloodline_tier(st);
    let k = crate::combat_data::bloodline_tier_info(&id, tier).map(|c| c.k).unwrap_or(0);
    if k <= 0 { return; }
    *st.bloodline_contribution.entry(id.clone()).or_insert(0) += k;
    *st.bloodline_proficiency.entry(id).or_insert(0) += 1;
}

/// 血统强度缩放后的攻击追加：strength / 100（向下取整）。
fn bloodline_strength_atk(st: &GameState) -> i32 {
    crate::combat_data::bloodline_strength(st) / 100
}

/// 血统强度缩放后的受击减伤：strength / 200（向下取整）。
fn bloodline_strength_reduce(st: &GameState) -> i32 {
    crate::combat_data::bloodline_strength(st) / 200
}

/// 血统强度缩放后的吸血：strength / 1000（向下取整）。
fn bloodline_strength_leech(st: &GameState) -> i32 {
    crate::combat_data::bloodline_strength(st) / 1000
}

/// 当前技能是否属于「当前血统相关」：school=Blood，或 need_bloodline == 当前血统。
fn skill_belongs_to_bloodline(st: &GameState, sk: &crate::defs::SkillDef) -> bool {
    if sk.school == crate::defs::SkillSchool::Blood {
        return true;
    }
    match (&sk.need_bloodline, &st.bloodline) {
        (Some(b), Some(cur)) => b == cur,
        _ => false,
    }
}

/* ---------------- 技能熟练度 → 技能强度（两层熟练度加成） ----------------
   两层熟练度（已有字段，接入 do_skill 效果计算）：
   1. skill_proficiency（单技能熟练度，专属）：每 10 点 +1 强度，只对该技能有效。
   2. category_proficiency（大类熟练度，主角自身积累通用）：每 20 点 +1 强度，
      练同大类（14 类）任何技能都涨，换同大类新技能（skill_proficiency=0）也吃该加成。

   技能强度公式（对技能「效果数值」的加法增量，口径统一）：
      bonus = skill_proficiency / 10 + category_proficiency / 20     （均向下取整）
      → 实际效果 = 基础效果 + bonus

   映射（写清楚）：
   - Striking 伤害：bonus 加到每段伤害总和（hits 段加完再 +bonus，不逐段累乘，
     避免多段技能把 bonus 反复放大；1 段伤害则等价 +bonus）。
   - SelfBuff：hp/san 恢复量各 +bonus；atk_flat 追加 +bonus（guard/dodge 为布尔/概率
     阈值，不改）。
   - DebuffEnemy：直伤 dmg +bonus；dot amt +bonus（每回合）。
   技能熟练度每 10 点 +1、大类熟练度每 20 点 +1 均向下取整；bonus=0 时零回归。
*/

/// 核心公式：skill_proficiency/10 + category_proficiency/20（向下取整）。
/// 换同大类新技能时 skill_proficiency=0，仍吃 category_proficiency/20（通用积累）。
pub fn skill_strength_bonus(sk: &crate::defs::SkillDef, st: &GameState) -> i32 {
    let sp = st.skill_proficiency.get(sk.id).copied().unwrap_or(0) as i32;
    let cat = crate::skills_data::skill_category(sk).to_string();
    let cp = st.category_proficiency.get(&cat).copied().unwrap_or(0) as i32;
    sp / 10 + cp / 20
}
/* ---------------- 视图 ---------------- */
pub fn hud_json(st: &GameState) -> Value {
    let team: Vec<Value> = ["one", "rain", "kaplan", "jd"].iter().map(|k| json!({
        "key": k,
        "name": match *k { "one" => "一号", "rain" => "蕾恩", "kaplan" => "卡普兰", _ => "J.D." },
        "alive": st.team_alive(k),
    })).collect();
    json!({
        "hp": st.hp, "san": st.san, "points": st.points,
        "weapon": st.weapon.map(|w| w.name().to_string()).unwrap_or_else(|| "—".into()),
        "ammo": st.ammo,
        "geneLock": st.gene_lock,
        "strBonus": st.str_bonus, "agiBonus": st.agi_bonus,
        "bloodline": st.bloodline.clone(),
        "bloodline_tier": st.bloodline.clone().map(|id| st.bloodline_tier.get(&id).copied().unwrap_or(1)).unwrap_or(0),
        "bloodline_proficiency": st.bloodline.clone().map(|id| st.bloodline_proficiency.get(&id).copied().unwrap_or(0)).unwrap_or(0),
        "bloodline_contribution": st.bloodline.clone().map(|id| st.bloodline_contribution.get(&id).copied().unwrap_or(0)).unwrap_or(0),
        "bloodline_strength": crate::combat_data::bloodline_strength(st),
        "qi": st.qi, "qiMax": st.qi_max,
        "techShield": st.tech_shield, "techShieldMax": st.tech_shield_max,
        "geneStage": crate::combat_data::gene_stage_of(st),
        "cultivationStage": st.cultivation_stage,
        "bloodlineName": crate::combat_data::bloodline_of(st).map(|b| b.name).unwrap_or(""),
        "skills": st.skills.clone(),
        "equipped_skills": st.equipped_skills.clone(),
        "skill_proficiency": st.skill_proficiency.clone(),
        "category_proficiency": st.category_proficiency.clone(),
        "team": team,
    })
}

/// 战斗可用动作列表（渲染与执行共用，保证一致）。
/// 绝对顺序语义保持向后兼容：新动作只在玩家持有对应资源（内功/技能/道具）时追加在标准动作之后，
/// 未购技能的玩家动作列表与现状完全一致（零回归）。
pub fn fight_actions(st: &GameState) -> Vec<String> {
    let mut v: Vec<&'static str> = vec![];
    if let Some(f) = &st.fight {
        // 防御：fight_id 未登记到 FIGHTS 表时不 panic，仅跳过 finisher
        if let Some(cfg) = crate::scenes::fight_cfg(&f.id) {
            if (cfg.finisher_if)(st, f.hp) {
                v.push("finisher");
            }
        }
    }
    match st.weapon {
        Some(crate::state::Weapon::Gun) => {
            if st.ammo > 0 { v.push("shoot"); }
            v.push("melee_gun");
        }
        _ => v.push("attack"),
    }
    v.push("allout");
    v.push("guard");

    // —— 真气绝学（art）：已学内功 && qi 足够时出现 ——
    if let Some(art) = art_cost_and_name(st) {
        if st.qi >= art.0 {
            v.push("art");
        }
    }

    // —— 技能动作（sk_<id>）：拥有且前置满足的主动技能 ——
    for sk in crate::skills_data::skills_owned(st) {
        if crate::skills_data::skill_usable_in_fight(st, sk) {
            v.push(Box::leak(format!("sk_{}", sk.id).into_boxed_str()));
        }
    }

    // —— 战斗内道具（item_<id>）：FIGHT_ITEMS 白名单内且当前携带 ——
    for (fid, _fx) in crate::combat_data::FIGHT_ITEMS {
        if crate::items_data::has_item(st, fid) {
            v.push(Box::leak(fid.to_string().into_boxed_str()));
        }
    }

    v.into_iter().map(|s| s.to_string()).collect()
}

/// 真气绝学的消耗与名称（None=未学内功）。返回 (cost, 名称)。
fn art_cost_and_name(st: &GameState) -> Option<(i32, &'static str)> {
    match st.inner_art.as_deref() {
        Some("wuming") => Some((20, "问心一剑")),
        Some("jingxin") => Some((10, "静心凝神")),
        _ => None,
    }
}

fn action_label(act: &str, st: &GameState) -> (String, String) {
    match act {
        "finisher" => {
            // 防御：fight 为空或 fight_id 未登记时不 panic
            let nm = st.fight.as_ref()
                .and_then(|f| crate::scenes::fight_cfg(&f.id))
                .map(|cfg| (cfg.finisher_name)(st))
                .unwrap_or_else(|| "终结技".to_string());
            ("☠ 终结技 · ".to_string() + &nm, "就是现在！".into())
        }
        "shoot" => (format!("▸ 射击（剩余 {} 发）", st.ammo), "远程 · 稳定".into()),
        "melee_gun" => ("▸ 枪托肉搏".into(), "子弹告急时的选择".into()),
        "attack" => {
            let w = st.weapon.unwrap();
            let desc = match w.name() { "消防斧" => "高伤 · 出手沉重", "军刀" => "迅捷 · 低伤连击", _ => "" };
            (format!("▸ {}攻击", w.name()), desc.into())
        }
        "allout" => ("▸ 全力一搏".into(), "高风险 · 高伤害 · 易露破绽".into()),
        "art" => {
            let name = art_cost_and_name(st).map(|(_, n)| n).unwrap_or("绝学");
            let tag = if st.inner_art.is_some() { "真气" } else { "" };
            (format!("✹ {tag}绝学 · {name}"), "消耗真气 · 高伤贯穿".into())
        }
        // 兜底识别技能动作（sk_/skx_/cu_/sk_gene_ 前缀）与战斗内道具
        _ => {
            if act.starts_with("item_") {
                let base = act.strip_prefix("item_").unwrap_or(act);
                let nm = crate::items_data::item_def(act)
                    .map(|d| d.name)
                    .or_else(|| crate::items_data::item_def(base).map(|d| d.name))
                    .unwrap_or(base);
                let n = crate::items_data::count_item(st, act);
                (format!("▸ 使用 {nm}"), if n > 1 { format!("道具 ×{n} · 不耗命中") } else { "道具 · 不耗命中".into() })
            } else if act.starts_with("sk_") || act.starts_with("skx_") || act.starts_with("cu_") || act.starts_with("sk_gene_") {
                let sid = act.strip_prefix("sk_").unwrap_or(act);
                let sk = crate::skills_data::skill(sid).or_else(|| crate::skills_data::skill(act));
                match sk {
                    Some(s) => (format!("✺ {}{}", school_tag(s.school), s.name), cost_desc(s)),
                    None => (format!("▸ {act}"), "".into()),
                }
            } else {
                ("▸ 后撤观察".into(), "恢复架势 · 提升下一轮闪避".into())
            }
        }
    }
}

/// 技能派系短标
fn school_tag(sc: crate::defs::SkillSchool) -> &'static str {
    use crate::defs::SkillSchool::*;
    match sc { Wushu => "【武道】", Gene => "【基因】", Blood => "【血统】", Holy => "【圣光】", Tech => "【科技】", Nt => "【灵能】", Meme => "【模因】", Util => "【辅助】", Xiu => "【修真】" }
}

/// 技能消耗/前置文案
fn cost_desc(s: &crate::defs::SkillDef) -> String {
    let cost = match &s.cost {
        crate::defs::SkillCost::Qi(n) => format!("真气 {n} · "),
        crate::defs::SkillCost::Item(it) => format!("消耗 {it} · "),
        crate::defs::SkillCost::None => String::new(),
    };
    let limit = if s.per_fight_uses.is_some() { "每场限用" } else { "主动技" };
    cost + limit
}

pub fn render(st: &GameState) -> Value {
    let mut base = json!({ "hud": hud_json(st) });
    match &st.mode {
        Mode::AwaitCard(card) => {
            base["kind"] = json!("card");
            base["card"] = json!({
                "title": card.title, "good": card.good,
                "bodyHtml": card.body_html,
                "voice": card.voice,
                "buttons": card.buttons.iter().map(|(l, r)| json!({"label": l, "route": r})).collect::<Vec<_>>(),
            });
        }
        Mode::Fight => {
            let scene = crate::scenes::scene(&st.scene_id).expect("scene");
            let acts = fight_actions(st);
            base["kind"] = json!("scene");
            fill_scene_common(&mut base, st, scene);
            base["choices"] = json!(acts.iter().enumerate().map(|(i, a)| {
                let (l, s) = action_label(a.as_str(), st);
                json!({"index": i, "label": l, "sub": s})
            }).collect::<Vec<_>>());
            // 防御：fight 为空时不 panic（显示空敌人占位）
            if let Some(f) = st.fight.as_ref() {
                base["fight"] = json!({"name": f.name, "hp": f.hp.max(0), "maxHp": f.max_hp, "log": f.pending_log});
            } else {
                base["fight"] = json!({"name": "未知敌人", "hp": 0, "maxHp": 0, "log": []});
            }
        }
        Mode::Normal => {
            let scene = crate::scenes::scene(&st.scene_id).expect("scene");
            base["kind"] = json!(if scene.overlay.is_some() { "card" } else { "scene" });
            if let Some(ov) = &scene.overlay {
                let card = (ov.card)(st);
                base["card"] = json!({
                    "title": card.title, "good": card.good,
                    "bodyHtml": card.body_html,
                    "voice": ov.voice,
                    "buttons": card.buttons.iter().map(|(l, r)| json!({"label": l, "route": r})).collect::<Vec<_>>(),
                });
            } else {
                fill_scene_common(&mut base, st, scene);
                let mut idx = 0i32;
                base["choices"] = json!(scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).map(|c| {
                    let i = idx; idx += 1;
                    json!({"index": i, "label": c.label, "sub": c.sub})
                }).collect::<Vec<_>>());
            }
        }
    }
    base
}

fn fill_scene_common(base: &mut Value, st: &GameState, scene: &'static SceneDef) {
    base["bg"] = json!(scene.bg);
    base["loc"] = json!(scene.loc.unwrap_or(""));
    base["speaker"] = json!(scene.speaker.unwrap_or(""));
    base["mood"] = json!(scene.mood);
    base["voice"] = json!(scene.voice);
    base["paragraphs"] = json!(scene.text.render(st));
    if let (Some(v), Some(l)) = (scene.video, scene.cine_label) {
        base["video"] = json!({"src": v, "label": l});
    } else {
        base["video"] = Value::Null;
    }
}

/* ---------------- 进入场景 ---------------- */
/// 场景切换；返回需要记录的死亡档案（若有）
pub fn goto(st: &mut GameState, target: &str, deaths: &mut Vec<(&'static str, &'static str)>) {
    // 理智归零：普通场景转为崩溃结局
    let mut target = target.to_string();
    if target != "e_sancollapse" && st.san <= 0
        && crate::scenes::scene(&target).map_or(false, |s| s.overlay.is_none()) {
        target = "e_sancollapse".to_string();
    }
    let scene = match crate::scenes::scene(&target) {
        Some(s) => s,
        None => { return; }
    };
    st.scene_id = scene.id.to_string();
    st.pending_death = None;
    // 结算场景：进入时先算总分（其本身是覆盖层，勿在覆盖层分支后遗漏）
    if scene.id == "s_settle" {
        let (total, rank, _, _, _) = crate::scenes::compute_settlement(st);
        st.settle_total = total;
        st.settle_rank = rank;
    }
    if let Some(ov) = &scene.overlay {
        if let Some((t, c)) = ov.death {
            deaths.push((t, c));
        }
        st.mode = Mode::AwaitCard((ov.card)(st));
        return;
    }
    if let Some(fid) = scene.fight_id {
        if let Some(cfg) = crate::scenes::fight_cfg(fid) {
            st.fight = Some(crate::power::scaled_fight(fid, cfg, st, vec![format!("<span class='miss'>{}</span>", cfg.intro)]));
            st.mode = Mode::Fight;
            return;
        }
        // cfg 缺失（配置遗漏）：不进战斗，回普通模式，避免 panic
        st.mode = Mode::Normal;
        return;
    }
    st.mode = Mode::Normal;
}

/* ---------------- 存档恢复 ---------------- */
pub fn rebuild_mode(st: &mut GameState) {
    let scene = match crate::scenes::scene(&st.scene_id) {
        Some(s) => s,
        None => return,
    };
    if let Some(fid) = scene.fight_id {
        if st.fight.is_none() {
            // 重建为满血敌人（轮回的仁慈）；cfg 缺失时跳过，避免 panic
            if let Some(cfg) = crate::scenes::fight_cfg(fid) {
                st.fight = Some(crate::power::scaled_fight(fid, cfg, st, vec![]));
            }
        }
    }
    st.mode = match (&scene.overlay, &scene.fight_id) {
        (Some(ov), _) => Mode::AwaitCard((ov.card)(st)),
        (None, Some(_)) => Mode::Fight,
        _ => Mode::Normal,
    };
}

/* ---------------- 玩家选择 ---------------- */
pub fn choose(st: &mut GameState, index: i32, deaths: &mut Vec<(&'static str, &'static str)>) {
    match &st.mode {
        Mode::AwaitCard(_) => {
            let buttons: Vec<(String, String)> = match &st.mode {
                Mode::AwaitCard(c) => c.buttons.clone(),
                _ => vec![],
            };
            let route = buttons.get(index as usize).map(|(_, r)| r.clone()).unwrap_or_default();
            match route.as_str() {
                "__resume_fight__" => st.mode = Mode::Fight,
                "__title__" => { /* 客户端负责重置会话 */ }
                "__back_to_world__" => { /* 客户端负责返回地图 */ }
                _ => {}
            }
        }
        Mode::Fight => fight_turn(st, index, deaths),
        Mode::Normal => {
            let scene = crate::scenes::scene(&st.scene_id).expect("scene");
            let visible: Vec<&ChoiceDef> = scene.choices.iter().filter(|c| c.cond.map_or(true, |f| f(st))).collect();
            if let Some(c) = visible.get(index as usize) {
                for e in c.effects {
                    e.apply(st);
                }
                if let Some(d) = &st.pending_death {
                    let t = d.clone();
                    goto(st, &t, deaths);
                    return;
                }
                let target = match &c.route {
                    Route::To(s) => s.to_string(),
                    Route::Dyn(f) => f(st),
                };
                goto(st, &target, deaths);
            }
        }
    }
}

/* ---------------- 战斗 ---------------- */
fn push_log(st: &mut GameState, line: String) {
    if let Some(f) = st.fight.as_mut() {
        f.pending_log.push(line);
    }
}

/* ---------------- 新战斗动作解析/应用（art / sk_* / item_*） ---------------- */

/// 真气绝学动作：消耗 qi，无视敌 armor 高伤。返回是否执行（消耗是否足够）。
fn do_art(st: &mut GameState) -> bool {
    let (cost, name) = match art_cost_and_name(st) {
        Some(x) => x,
        None => return false,
    };
    if st.qi < cost {
        return false;
    }
    st.qi -= cost;
    let d = rnd(30, 40);
    // 无视护甲直接命中敌方 HP
    if let Some(f) = st.fight.as_mut() {
        f.hp -= d;
    }
    push_log(st, format!("<span class='crit'>✹ {name}！剑气贯穿，无视护甲 —— 伤害 {d}</span>"));
    true
}

/// 战斗内道具动作：消耗一份道具，确定性效果（不耗命中 roll）。返回是否执行成功。
fn do_fight_item(st: &mut GameState, fid: &str) -> bool {
    let fx = crate::combat_data::FIGHT_ITEMS.iter()
        .find(|(id, _)| *id == fid)
        .map(|(_, fx)| fx);
    let fx = match fx {
        Some(fx) => fx,
        None => return false,
    };
    if !crate::items_data::consume_item(st, fid) {
        return false;
    }
    let name = crate::items_data::item_def(fid).map(|d| d.name).unwrap_or(fid);
    match &fx.effect {
        crate::defs::ItemEffect::Heal(h) => {
            let real = { let before = st.hp; st.hp = (st.hp + h).min(100); st.hp - before };
            push_log(st, format!("<span class='hit'>使用 「{name}」 · 复原 +{real} 体力</span>"));
        }
        crate::defs::ItemEffect::San(s) => {
            let real = { let before = st.san; st.san = (st.san + s).clamp(0, 100); st.san - before };
            push_log(st, format!("<span class='hit'>使用 「{name}」 · 心神 +{real}</span>"));
        }
        crate::defs::ItemEffect::Ammo => {
            st.ammo = 6;
            push_log(st, format!("<span class='hit'>使用 「{name}」 · 弹药补满</span>"));
        }
        crate::defs::ItemEffect::Throw { dmg_over_time, weak, flat_dmg } => {
            let mut dmg = *flat_dmg;
            if dmg == 0 {
                dmg = rnd(40, 60);
            }
            let mut base = dmg;
            // 弱词倍率（对弱火/弱电敌 ×1.3 示意；weak 含 weak_fire/weak_electric 则走克制）
            if let Some(wk) = weak {
                let mods = st.fight.as_ref().and_then(|f| crate::combat_data::battle_mods(&f.id));
                let weaksum = mods.map(|m| {
                    let mut s = 0;
                    if wk.contains(&"fire") { s += m.weak_fire; }
                    if wk.contains(&"electric") { s += m.weak_electric; }
                    if wk.contains(&"holy") { s += 30; }
                    if wk.contains(&"silver") { s += 30; }
                    s
                }).unwrap_or(0);
                if weaksum > 0 { base = (base as f64 * (100 + weaksum) as f64 / 100.0) as i32; }
            }
            if let Some((amt, turns)) = dmg_over_time {
                if *amt > 0 {
                    push_log(st, format!("<span class='hit'>使用 「{name}」 · 灼烧 {amt}×{turns} 回合</span>"));
                }
            }
            if let Some(f) = st.fight.as_mut() { f.hp -= base; }
            push_log(st, format!("<span class='crit'>「{name}」掷出 —— 命中！伤害 {base}</span>"));
        }
        crate::defs::ItemEffect::Charm { immune_death, cure_debuff } => {
            let _ = (immune_death, cure_debuff);
            push_log(st, format!("使用 「{name}」 · 护符加持"));
        }
        crate::defs::ItemEffect::None => {
            push_log(st, format!("使用 「{name}」"));
        }
    }
    true
}

/// 主动技能动作（sk_<id>）：按 SkillEffectKind 分发 Striking/SelfBuff/DebuffEnemy。
/// 返回是否需要终结这一玩家回合（命中判定是否已被技能消耗跳过 → false 表示已结算，不额外攻击）。
fn do_skill(st: &mut GameState, sk: &crate::defs::SkillDef) {
    // 消耗资源（qi / item）
    match &sk.cost {
        crate::defs::SkillCost::Qi(n) => { st.qi = (st.qi - (*n as i32)).max(0); }
        crate::defs::SkillCost::Item(it) => { crate::items_data::consume_item(st, it); }
        crate::defs::SkillCost::None => {}
    }
    push_log(st, format!("<span class='hit'>✺ 施展「{}」</span>", sk.name));

    // 熟练度提升：技能成功施展（资源已扣）即 +1 技能熟练度与该大类熟练度
    {
        let sid = sk.id.to_string();
        *st.skill_proficiency.entry(sid).or_insert(0) += 1;
        let cat = crate::skills_data::skill_category(sk).to_string();
        *st.category_proficiency.entry(cat).or_insert(0) += 1;
    }

    // 血统熟练度提升：技能属于当前血统（school=Blood 或 need_bloodline 匹配）→ contrib +k / use +1
    if skill_belongs_to_bloodline(st, sk) {
        bloodline_use(st);
    }

    // 两层熟练度强度加成：skill/10 + category/20（向下取整）。在熟练度 +1 之后读取，
    // 使本次施展即吃到刚积累的熟练度（使用即养成）。
    let bonus = skill_strength_bonus(sk, st);

    // 命中判定（Skills 为独立打击，参考普攻 0.82 命中）
    let mut hit = 0.82;
    if st.san < 30 { hit -= 0.12; }
    let roll: f64 = rand::thread_rng().gen();
    if roll < hit {
        match &sk.effect {
            crate::defs::SkillEffect::Striking { dmg, ignore_armor, hits, weak } => {
                let mut total = 0;
                for _ in 0..*hits {
                    total += rnd(dmg.0, dmg.1);
                }
                // 两层熟练度强度：bonus 加到整段伤害总和（多段不逐段放大）
                total += bonus;
                if bonus > 0 {
                    push_log(st, format!("<span class='hit'>【技能熟练】伤害 +{bonus}</span>"));
                }
                // 弱词/护甲
                let mods = st.fight.as_ref().and_then(|f| crate::combat_data::battle_mods(&f.id));
                if let Some(wk) = weak {
                    if !wk.is_empty() {
                        let weaksum = mods.map(|m| {
                            let mut s = 0;
                            if wk.contains(&"fire") { s += m.weak_fire; }
                            if wk.contains(&"electric") { s += m.weak_electric; }
                            if wk.contains(&"holy") { s += 30; }
                            if wk.contains(&"silver") { s += 30; }
                            s
                        }).unwrap_or(0);
                        if weaksum > 0 { total = (total as f64 * (100 + weaksum) as f64 / 100.0) as i32; }
                    }
                } else if let Some(m) = mods {
                    if *ignore_armor {
                        // 无视护甲不做扣减
                    } else if m.armor > 0 {
                        total = (total - m.armor).max(1);
                    }
                }
                if let Some(f) = st.fight.as_mut() { f.hp -= total; }
                push_log(st, format!("<span class='crit'>技能命中！伤害 {total}</span>"));
            }
            crate::defs::SkillEffect::SelfBuff { hp, san, guard, dodge_bonus, atk_flat } => {
                if *guard > 0 || *dodge_bonus > 0.0 {
                    // guard/dodge_bonus 自buff → 落当下守护态（护盾/减伤兜底，见敌回合）
                    if let Some(f) = st.fight.as_mut() {
                        f.guard_turn = true;
                    }
                }
                if *hp > 0 {
                    let heal = *hp + bonus;
                    let before = st.hp;
                    st.hp = (st.hp + heal).min(100);
                    push_log(st, format!("<span class='hit'>治愈 +{} 体力</span>", st.hp - before));
                }
                if *san > 0 {
                    let rec = *san + bonus;
                    let before = st.san;
                    st.san = (st.san + rec).clamp(0, 100);
                    push_log(st, format!("<span class='hit'>心神 +{} 恢复</span>", st.san - before));
                }
                if *atk_flat > 0 {
                    push_log(st, format!("<span class='hit'>攻击力沉稳汇聚 +{}</span>", *atk_flat + bonus));
                }
            }
            crate::defs::SkillEffect::DebuffEnemy { no_dodge, stun, dmg_over_time, dmg } => {
                let _ = (no_dodge, stun);
                if dmg.0 > 0 || dmg.1 > 0 {
                    let d = rnd(dmg.0, dmg.1) + bonus;
                    if let Some(f) = st.fight.as_mut() { f.hp -= d; }
                    push_log(st, format!("<span class='crit'>技能命中！伤害 {d}</span>"));
                }
                if let Some((amt, _turns)) = dmg_over_time {
                    if *amt > 0 {
                        let dot = *amt + bonus;
                        push_log(st, format!("<span class='hit'>dot 附加 {dot}/回合</span>"));
                    }
                }
            }
            crate::defs::SkillEffect::Passive { .. } => {
                // 被动不入动作
            }
        }
    } else {
        push_log(st, format!("<span class='miss'>「{}」落空了！</span>", sk.name));
    }
}

/// 依动作字符串解析技能 id：动作键 `sk_<skill_id>`。
fn skill_by_action(act: &str) -> Option<&'static crate::defs::SkillDef> {
    let sid = act.strip_prefix("sk_")?;
    crate::skills_data::skill(sid)
}

/* ---------------- 3D 副本（zone）技能格结算 ---------------- */

/// 技能当前不可用的中文原因（None = 可用）。与 `skill_usable_in_fight` 同一套判定规则，
/// 额外补齐 Item 消耗可行性（`skill_usable_in_fight` 只查 Qi 上限与当前真气）。
fn skill_unusable_reason(st: &GameState, sk: &crate::defs::SkillDef) -> Option<String> {
    if sk.kind != crate::defs::SkillKind::Active {
        return Some(format!("技能「{}」为被动技能，无法在战斗中施展。", sk.name));
    }
    if let Some(b) = sk.need_bloodline {
        if st.bloodline.as_deref() != Some(b) {
            return Some(format!("技能「{}」需要血统「{b}」，当前不满足。", sk.name));
        }
    }
    if let Some(g) = sk.need_stage {
        if crate::combat_data::gene_stage_of(st) < g {
            return Some(format!("技能「{}」需要基因锁 {g} 阶，当前不满足。", sk.name));
        }
    }
    if let Some(cs) = sk.need_cultivation {
        if st.cultivation_stage < cs {
            return Some(format!("技能「{}」需要修真境界 {cs} 重，当前不满足。", sk.name));
        }
    }
    if let Some(m) = sk.need_qi {
        if st.qi_max < m {
            return Some(format!("技能「{}」需要真气上限 {m}，当前不满足。", sk.name));
        }
    }
    match &sk.cost {
        crate::defs::SkillCost::Qi(n) if (st.qi as u32) < *n => {
            return Some(format!("真气不足：技能「{}」需要真气 {n}，当前 {}.", sk.name, st.qi));
        }
        crate::defs::SkillCost::Item(it) if !crate::items_data::has_item(st, it) => {
            return Some(format!("道具不足：技能「{}」需要道具「{it}」。", sk.name));
        }
        _ => {}
    }
    None
}

/// 副本战斗初始化：`st.fight` 为空时按 `fight_id` 的 fight_cfg 建场。
/// 与 main 的 `run_zone_combat_round` 初始化段保持一致（不缩放、写 intro 日志）；觉醒卡片等待则恢复战斗态。
fn ensure_zone_fight(st: &mut GameState, fight_id: &str) {
    if st.fight.is_none() {
        let cfg = crate::scenes::fight_cfg(fight_id);
        let (name, hp, dmg, reward, why, intro) = match cfg {
            Some(c) => (c.name, c.hp, c.dmg, c.reward, c.reward_why, c.intro),
            None => ("未知敌人", 30, (5, 10), 0, "", ""),
        };
        st.fight = Some(Fight {
            id: fight_id.to_string(),
            name: name.to_string(),
            hp, max_hp: hp,
            dmg, reward, reward_why: why.to_string(),
            raged: false, rage_at: None, guard_turn: false,
            pending_log: vec![format!("<span class='hit'>{intro}</span>")],
        });
        st.mode = Mode::Fight;
    }
    if matches!(st.mode, Mode::AwaitCard(_)) {
        st.mode = Mode::Fight;
    }
}

/// 3D 副本内主动技能格结算（供 main 的 `api_zone_action` "skill" 分支调用）。
///
/// - `slot`：`st.equipped_skills` 的下标（0/1/2）。
/// - 非法槽 / 技能不存在 / 前置或资源不足 → `Err(中文原因)`（不扣血、不推进回合、不崩）。
/// - 战斗未初始化时，按 `st.zone.ref_id` 的 fight_cfg 建场（复用 `ensure_zone_fight`）。
/// - 调 `do_skill` 执行技能（Striking 伤害 / SelfBuff / DebuffEnemy 各自生效 + 熟练度 +1），
///   随后结算敌方回合；击杀则结算胜利。
/// - 返回 `render(st)` 视图；胜负由调用方按 `fight.is_none() && !scene_id.starts_with("e_")` 判定。
pub fn do_zone_skill(st: &mut GameState, slot: usize) -> Result<Value, String> {
    // 1. slot 合法 & 已装备
    let sid = st.equipped_skills.get(slot)
        .ok_or_else(|| format!("技能格 {slot} 未装备（当前装备 {} 个）。", st.equipped_skills.len()))?
        .clone();
    // 2. 技能定义存在
    let sk = crate::skills_data::skill(&sid)
        .ok_or_else(|| format!("技能「{sid}」不存在。"))?;
    // 3. 前置 / 资源充足（中文原因）
    if let Some(reason) = skill_unusable_reason(st, sk) {
        return Err(reason);
    }
    // 4. 战斗未初始化则按 zone.ref_id 建场
    let fight_id = st.zone.as_ref()
        .map(|z| z.ref_id.clone())
        .unwrap_or_else(|| "horde".to_string());
    ensure_zone_fight(st, &fight_id);

    // 5. 执行技能（伤害 / buff / debuff + 熟练度 +1）
    do_skill(st, sk);

    // 6. 结算：击杀则胜利结算，否则敌方回合
    let mut deaths: Vec<(&'static str, &'static str)> = vec![];
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        fight_win(st);
    } else {
        enemy_turn(st, &mut deaths);
    }

    Ok(render(st))
}

fn fight_turn(st: &mut GameState, index: i32, _deaths: &mut Vec<(&'static str, &'static str)>) {
    let act = fight_actions(st).get(index as usize).cloned().unwrap_or_default();
    let actc: &str = act.as_str();

    if actc == "finisher" {
        // 防御：fight 为空或 fight_id 未登记时不 panic
        let desc = st.fight.as_ref()
            .and_then(|f| crate::scenes::fight_cfg(&f.id))
            .map(|cfg| (cfg.finisher_desc)(st))
            .unwrap_or_else(|| "致命一击".to_string());
        push_log(st, format!("<span class='crit'>☠ {desc}</span>"));
        fight_win(st);
        return;
    }

    // —— 战斗内道具动作（确定性，不耗命中）——
    if actc.starts_with("item_") {
        if do_fight_item(st, actc) {
            // 道具已结算本回合；敌方照常行动
        }
        enemy_turn(st, _deaths);
        return;
    }

    // —— 真气绝学 ——
    if actc == "art" {
        if do_art(st) {
            // 绝学命中自消耗 qi、直接造成无视甲伤害
            if let Some(f) = st.fight.as_ref() { if f.hp <= 0 { fight_win(st); return; } }
        }
        enemy_turn(st, _deaths);
        return;
    }

    // —— 技能动作（sk_/skx_/cu_/sk_gene_ 前缀）——
    if actc.starts_with("sk_") && !st.skills.is_empty() {
        if let Some(sk) = skill_by_action(actc) {
            do_skill(st, sk);
            if let Some(f) = st.fight.as_ref() { if f.hp <= 0 { fight_win(st); return; } }
            enemy_turn(st, _deaths);
            return;
        }
    }

    // 玩家行动（普攻 / 全力一搏；加成走查表，数值与旧内联严格等价）
    if actc == "guard" {
        if let Some(f) = st.fight.as_mut() { f.guard_turn = true; }
        let en = st.fight.as_ref().map(|f| f.name.clone()).unwrap_or_else(|| "敌人".to_string());
        push_log(st, format!("<span class='miss'>你后撤半步压低重心，视线锁死{en}的一举一动。</span>"));
    } else {
        let mut hit = 0.82;
        if actc == "allout" { hit = 0.55; }
        if st.san < 30 { hit -= 0.12; }
        if st.flag("B1") { hit += 0.06; }
        let roll: f64 = rand::thread_rng().gen();
        if roll < hit {
            let mut base = 0;
            match (actc, st.weapon) {
                ("shoot", _) => {
                    st.ammo -= 1;
                    base = rnd(14, 20);
                    let am = st.ammo;
                    push_log(st, format!("双手举枪连开数枪——<span class='hit'>命中！伤害 {base}</span>（剩余{am}发）"));
                }
                ("melee_gun", _) => {
                    base = rnd(8, 13);
                    push_log(st, format!("枪托狠狠砸下——<span class='hit'>伤害 {base}</span>"));
                }
                (_, Some(crate::state::Weapon::Axe)) => {
                    let d = rnd(22, 34);
                    base = if actc == "allout" { (d as f64 * 1.9) as i32 } else { d };
                    let crit = rand::thread_rng().gen_bool(0.18);
                    if crit { base = (base as f64 * 1.6) as i32; }
                    let tail = if actc == "allout" { "（全力）" } else { "" };
                    if crit {
                        push_log(st, format!("<span class='crit'>会心一击！</span>消防斧劈落——<span class='hit'>伤害 {base}</span>{tail}"));
                    } else {
                        push_log(st, format!("消防斧劈落——<span class='hit'>伤害 {base}</span>{tail}"));
                    }
                }
                (_, Some(crate::state::Weapon::Sword)) => {
                    let hits = rnd(2, 3);
                    let mut parts = vec![];
                    for _ in 0..hits {
                        let d = rnd(10, 16);
                        base += d;
                        parts.push(d.to_string());
                    }
                    if actc == "allout" { base = (base as f64 * 1.5) as i32; }
                    push_log(st, format!("军刀连续刺出 {hits} 刀——<span class='hit'>{} = 伤害 {base}</span>", parts.join(" + ")));
                }
                _ => {}
            }
            // 基因锁攻击追加（查 GENE_STAGES；一阶 == 旧 rnd(6,12)）
            let geneb = gene_atk(st);
            if geneb > 0 {
                base += geneb;
                push_log(st, format!("<span class='crit'>【基因锁状态】动作快到不可思议 · 追加伤害 +{geneb}</span>"));
            }
            // 组合联动（synergy_check）：血统 × 武器 × 法宝 × 技能 的隐藏协同。
            // 幂等确定性：命中规则便固定追加伤害/吸血，并写一行 `<span class='synergy'>` 提示文案。
            if !crate::synergies::synergy_check(st).is_empty() {
                let syn_defs = crate::synergies::synergy_check(st);
                let syn_flat = crate::synergies::synergy_flat(st);
                let syn_leech = crate::synergies::synergy_leech(st);
                for d in &syn_defs {
                    let (f, l, line) = crate::synergies::synergy_line(d);
                    let _ = f; let _ = l; // 数值已由 synergy_flat/leech 汇总；此处仅用文案
                    push_log(st, line);
                }
                base += syn_flat;
                if syn_leech > 0 {
                    st.hp = (st.hp + syn_leech).min(100);
                }
            }
            // P1 兑换 · 细胞活力强化：额外攻击（str_bonus × 5）
            if st.str_bonus > 0 {
                let add = st.str_bonus * 5;
                base += add;
                push_log(st, format!("<span class='hit'>【细胞活力强化】体魄迸发出超越常人的力量 · 追加 +{add}</span>"));
            }
            // 血统被动攻击（查 BLOODLINES；吸血鬼 atk_flat=0，等价原无此加成）
            // 血统熟练度强度接入：在被动 atk_flat 之外叠加 strength/100 缩放（血族进阶路线）。
            if let Some(b) = crate::combat_data::bloodline_of(st) {
                let s_atk = bloodline_strength_atk(st);
                let total_atk = b.passive.atk_flat + s_atk;
                if total_atk > 0 {
                    base += total_atk;
                    push_log(st, format!("<span class='hit'>【{}】血脉之力 · 追加 +{}</span>", b.name, total_atk));
                }
            }
            // 装备常驻攻击（护甲/饰品/法宝）
            let ge = gear_atk_flat(st);
            if ge > 0 {
                base += ge;
                push_log(st, format!("<span class='hit'>【装备】神兵助力 · 追加 +{ge}</span>"));
            }
            // 装备伤害倍率 + 敌弱点（weak_fire/weak_electric ×1.3）倍率乘算
            let gmult = gear_dmg_mult(st);
            let mut weaksum = 0;
            if let Some(m) = st.fight.as_ref().and_then(|f| crate::combat_data::battle_mods(&f.id)) {
                weaksum = m.weak_fire + m.weak_electric;
            }
            let mult = gmult * (100 + weaksum) as f64 / 100.0;
            if mult != 1.0 {
                base = (base as f64 * mult) as i32;
            }
            // 敌护甲（BATTLE_MODS.armor）命中后固定扣减
            if let Some(m) = st.fight.as_ref().and_then(|f| crate::combat_data::battle_mods(&f.id)) {
                if m.armor > 0 { base = (base - m.armor).max(1); }
            }
            // 血统命中吸血（BLOODLINES vampire leech_on_hit=4 == 原 is_vampire leech4）
            // 血统熟练度强度接入：strength/1000 缩放追加吸血（高级血统才明显）。
            if let Some(b) = crate::combat_data::bloodline_of(st) {
                let leech = b.passive.leech_on_hit + bloodline_strength_leech(st);
                if leech > 0 {
                    st.hp = (st.hp + leech).min(100);
                    push_log(st, format!("<span class='hit'>【{}】鲜红的渴望被满足 · 吸取体力 +{leech}</span>", b.name));
                }
            }
            if let Some(f) = st.fight.as_mut() {
                f.hp -= base;
            }
            // 血统熟练度提升：本次成功命中普攻即 contrib +k / use +1（血统进阶路线养成）
            bloodline_use(st);
        } else {
            let en = st.fight.as_ref().map(|f| f.name.clone()).unwrap_or_else(|| "敌人".to_string());
            push_log(st, format!("<span class='miss'>你的攻势落空了！{en}抓住破绽逼近——</span>"));
        }
    }

    // 敌人败北？
    if st.fight.as_ref().map(|f| f.hp <= 0).unwrap_or(false) {
        fight_win(st);
        return;
    }

    // 狂暴触发
    let should_rage = st.fight.as_ref()
        .and_then(|f| f.rage_at.map(|at| f.hp <= at && !f.raged))
        .unwrap_or(false);
    if should_rage {
        if let Some(f) = st.fight.as_mut() {
            f.raged = true;
            f.dmg = (f.dmg.0 + 4, f.dmg.1 + 6);
        }
        let (rt, fid) = {
            let f = st.fight.as_ref();
            let cfg = f.and_then(|ff| crate::scenes::fight_cfg(&ff.id));
            let fid = f.map(|ff| ff.id.clone()).unwrap_or_default();
            (cfg.map(|c| c.rage_text.to_string()).unwrap_or_else(|| "它彻底狂暴了！".to_string()), fid)
        };
        push_log(st, format!("<span class='crit'>{rt}</span>"));
        if let Some(cfg) = crate::scenes::fight_cfg(&fid) {
            let mut logs = vec![];
            (cfg.on_rage)(st, &mut logs);
            for l in logs { push_log(st, l); }
        }
    }

    enemy_turn(st, _deaths);
    gene_awaken_check(st);
}

/// 敌人回合：BATTLE_MODS（aura/armor/regen/no_dodge）+ 玩家科技护盾先吞伤。
/// 数值等价验证：未挂 mods / tech_shield=0 时，闪避与受击公式与旧内联完全一致。
fn enemy_turn(st: &mut GameState, _deaths: &mut Vec<(&'static str, &'static str)>) {
    let mods = st.fight.as_ref().and_then(|f| crate::combat_data::battle_mods(&f.id));

    // aura：敌回合顶部全队 San- 侵蚀
    if let Some(m) = mods {
        if m.aura > 0 {
            let drop = m.aura;
            let old = st.san;
            st.san = (st.san - drop).clamp(0, 100);
            if st.san < old {
                push_log(st, format!("<span class='crit'>蚀心气场笼罩 —— 理智 -{}</span>", old - st.san));
            }
        }
    }

    let guard = st.fight.as_ref().map(|f| f.guard_turn).unwrap_or(false);
    let agi_bonus = (st.agi_bonus as f64) * 0.05;
    let bl_dodge = crate::combat_data::bloodline_of(st).map(|b| b.passive.dodge_bonus).unwrap_or(0.0);
    let base_dodge = if guard { 0.55 } else { 0.16 };
    let dodge = base_dodge + gene_dodge(st) + bl_dodge + gear_dodge(st) + agi_bonus;

    let no_dodge = mods.map(|m| m.no_dodge).unwrap_or(false);
    let en_name = st.fight.as_ref().map(|f| f.name.clone()).unwrap_or_else(|| "敌人".to_string());
    let mut died = false;
    let hit_landed = if no_dodge { true } else {
        let roll: f64 = rand::thread_rng().gen();
        roll >= dodge
    };
    if hit_landed {
        let raw = {
            st.fight.as_ref().map(|f| rnd(f.dmg.0, f.dmg.1)).unwrap_or(5)
        };
        // 受击减伤：基因锁 + 血统 + 装备（等价原 gene(-4)+vamp(-3)）
        // 血统熟练度强度接入：被动 dmg_reduce 之外叠加 strength/200 缩放减伤。
        let bl_reduce = crate::combat_data::bloodline_of(st)
            .map(|b| b.passive.dmg_reduce + bloodline_strength_reduce(st))
            .unwrap_or(0);
        let reduce = gene_reduce(st) + bl_reduce + gear_reduce(st);
        let mut dmg = (raw - reduce).max(2);
        // 科技护盾先吃伤（穿透才扣 hp）
        if st.tech_shield > 0 {
            let absorb = st.tech_shield.min(dmg);
            st.tech_shield -= absorb;
            dmg -= absorb;
            if absorb > 0 {
                push_log(st, format!("<span class='hit'>纳米护盾抵挡 {absorb}</span>"));
            }
        }
        st.hp = (st.hp - dmg).max(0);
        push_log(st, format!("<span class='crit'>{en_name}击中了你！体力 -{dmg}</span>"));
        if st.hp <= 0 { died = true; }
    } else {
        push_log(st, format!("<span class='miss'>{en_name}扑击而来——你在千钧一发之际避开了！</span>"));
    }
    // 敌每回合再生（regen）
    if let Some(m) = mods {
        if let Some((amt, _turns)) = m.regen {
            if amt > 0 {
                if let Some(f) = st.fight.as_mut() { f.hp += amt; }
                push_log(st, format!("<span class='hit'>敌方气息翻涌 —— 再生 +{amt}</span>"));
            }
        }
    }
    if let Some(f) = st.fight.as_mut() { f.guard_turn = false; }

    if died {
        let death = st.fight.as_ref()
            .and_then(|f| crate::scenes::fight_cfg(f.id.as_str()))
            .map(|c| c.death)
            .unwrap_or("你倒在了敌人面前……");
        st.fight = None;
        st.mode = Mode::Normal;
        goto(st, death, _deaths);
    }
}

/// 基因锁濒死觉醒（通用化，承接 licker）。一次战斗一升；逐阶 hp% 阈值 30/25/20/15。
fn gene_awaken_check(st: &mut GameState) {
    // 在场战斗与是否已在本场觉醒（压用 gene_lock_used 作临场「已升一次」标记）
    if st.fight.is_none() || st.gene_lock_used { return; }
    let cur = crate::combat_data::gene_stage_of(st);
    // 下一阶（当前阶+1，未开则为 1）
    let next = cur + 1;
    let cfg = match crate::combat_data::GENE_STAGES.iter().find(|c| c.stage == next) {
        Some(c) => c,
        None => return, // 已满阶
    };
    // 濒死阈值：HP% ≤ 该阶 hp_low_threshold（HP 上限恒 100，故直接比 hp）
    if st.hp > cfg.hp_low_threshold { return; }
    if st.san < 20 { return; }
    // 决定性觉醒（消耗 SAN + 演出卡片）；一次战斗一升
    st.gene_lock_used = true;
    st.san = (st.san - cfg.awakening_cost_san).max(0);
    crate::combat_data::set_gene_stage(st, next);
    st.mode = Mode::AwaitCard(crate::scenes::gene_lock_card());
}

fn fight_win(st: &mut GameState) {
    // BATTLE_MODS·post_kill：击杀后对玩家副作用（Hurt）
    if let Some(fid) = st.fight.as_ref().map(|f| f.id.clone()) {
        if let Some(m) = crate::combat_data::battle_mods(&fid) {
            if let Some((amt, txt)) = m.post_kill {
                if amt > 0 {
                    let before = st.hp;
                    st.hp = (st.hp - amt).max(0);
                    push_log(st, format!("<span class='crit'>击杀后遗祸 —— {txt} 体力 -{}</span>", before - st.hp));
                }
            }
        }
    }
    let (reward, why, win_target) = {
        // 防御：fight 为空或 fight_id 未登记时不 panic，走默认胜利
        let cfg = st.fight.as_ref().and_then(|f| crate::scenes::fight_cfg(&f.id));
        match cfg {
            Some(c) => (c.reward, c.reward_why.to_string(), (c.win)(st)),
            None => (0, "战斗胜利".to_string(), "back_world".to_string()),
        }
    };
    st.points += reward;
    push_log(st, format!("<span class='hit'>[主神] {why} +{reward}奖励点数</span>"));
    st.fight = None;
    st.mode = Mode::Normal;
    let mut no_deaths: Vec<(&'static str, &'static str)> = vec![];
    goto(st, &win_target, &mut no_deaths);
}
