//! 场景定义类型：声明式剧本 DSL（纯函数指针，线程安全）
//! 战斗体系数据层（包 A）：结构体定义集中于此，数据表置于新模块
//!   combat_data / skills_data / items_data，本文件仅定义类型并 re-export。

use crate::state::{Card, GameState, Weapon};

/// 动态/静态文本
pub enum TextSpec {
    Static(&'static [&'static str]),
    Dyn(fn(&GameState) -> String),
}

impl TextSpec {
    /// 返回段落数组
    pub fn render(&self, st: &GameState) -> Vec<String> {
        match self {
            TextSpec::Static(arr) => arr.iter().map(|s| s.to_string()).collect(),
            TextSpec::Dyn(f) => vec![f(st)],
        }
    }
}

pub type CondFn = fn(&GameState) -> bool;
pub type RouteFn = fn(&mut GameState) -> String;

pub enum Route {
    To(&'static str),
    Dyn(RouteFn),
}

#[derive(Clone, Copy)]
pub enum Eff {
    SetFlag(&'static str),
    San(i32),
    Points(i32),
    /// 旗标成立时才给点数（支线奖励）
    PointsIfFlag(&'static str, i32),
    KillTeam(&'static str),
    /// 扣血；若归零跳转到指定死亡场景
    Hurt(i32, &'static str),
    Weapon(Weapon),
    /// 道具加入物品栏（去重）
    AddItem(&'static str),
    /// 完成某个世界调查点（地图变灰 + 文本门控）
    MarkPoint(&'static str),
}

impl Eff {
    pub fn apply(&self, st: &mut GameState) {
        match self {
            Eff::SetFlag(k) => st.set_flag(k),
            Eff::San(d) => st.san = (st.san + d).clamp(0, 100),
            Eff::Points(p) => st.points += p,
            Eff::PointsIfFlag(k, p) => {
                if st.flag(k) { st.points += p; }
            }
            Eff::KillTeam(k) => st.kill_team(k),
            Eff::Hurt(d, death_route) => {
                st.hp = (st.hp - d).max(0);
                if st.hp <= 0 {
                    st.pending_death = Some(death_route.to_string());
                }
            }
            Eff::Weapon(w) => {
                st.weapon = Some(*w);
                if *w == Weapon::Gun {
                    st.ammo = 6;
                }
            }
            Eff::AddItem(item) => crate::world::add_item(st, item),
            Eff::MarkPoint(id) => crate::world::mark_point(st, id),
        }
    }
}

pub struct ChoiceDef {
    pub label: &'static str,
    pub sub: &'static str,
    pub cond: Option<CondFn>,
    pub effects: &'static [Eff],
    pub route: Route,
}

/// 战斗配置表条目（fns 按 id 查找）
pub struct FightCfg {
    pub name: &'static str,
    pub hp: i32,
    pub dmg: (i32, i32),
    pub reward: i32,
    pub reward_why: &'static str,
    pub intro: &'static str,
    pub rage_at: Option<i32>,
    pub rage_text: &'static str,
    pub on_rage: fn(&mut GameState, &mut Vec<String>),
    pub finisher_if: fn(&GameState, i32) -> bool, // (state, enemy_hp)
    pub finisher_name: fn(&GameState) -> String,
    pub finisher_desc: fn(&GameState) -> String,
    pub win: fn(&GameState) -> String,
    pub death: &'static str,
}

/// 覆盖层卡片
pub struct OverlayDef {
    pub voice: Option<&'static str>,
    /// (死亡档案标题, 死因) —— 进入时记录
    pub death: Option<(&'static str, &'static str)>,
    pub card: fn(&GameState) -> Card,
}

pub struct SceneDef {
    pub id: &'static str,
    pub bg: Option<&'static str>,
    pub loc: Option<&'static str>,
    pub mood: &'static str,
    pub speaker: Option<&'static str>,
    pub voice: Option<&'static str>,
    pub text: TextSpec,
    pub choices: &'static [ChoiceDef],
    /// 战斗场景引用的 FightCfg id
    pub fight_id: Option<&'static str>,
    pub video: Option<&'static str>,
    pub cine_label: Option<&'static str>,
    pub overlay: Option<OverlayDef>,
}

// ============================================================================
// 战斗体系数据层 · 类型定义（包 A；数据表在 skills_data/items_data/combat_data）
// 全部数值均为「建议值·可调」；新持久字段全 #[serde(default)]。
// ============================================================================

// ---------- 基因锁（gene_stage） ----------
/// 基因锁各阶配置（静态表 GENE_STAGES 条目）。数值「建议值·可调」。
#[derive(Clone, Copy, Debug)]
pub struct GeneStageCfg {
    pub stage: u8,                  // 1..=4
    pub name: &'static str,         // 阶名
    pub sp_grade: Option<char>,     // 主神兑换所需支线评级（濒死觉醒路径无需）
    pub atk: (i32, i32),            // 攻击追加 (min,max)
    pub dodge: f64,                 // 闪避
    pub dmg_reduce: i32,            // 受击固定减伤
    pub finisher_bonus: i32,        // 终结技额伤
    pub hp_low_threshold: i32,      // 濒死觉醒判定用 hp 百分比阈值
    pub awakening_cost_san: i32,    // 濒死觉醒消耗的 SAN
}

// ---------- 血统（BLOODLINES） ----------
/// 血统被动数值（静态表）。数值「建议值·可调」。
#[derive(Clone, Copy, Debug)]
pub struct BloodlinePassive {
    pub atk_flat: i32,        // 攻击追加（正向），0 无
    pub leech_on_hit: i32,    // 命中吸血量
    pub dodge_bonus: f64,     // 闪避
    pub dmg_reduce: i32,      // 受击固定减伤
    pub san_resist: i32,      // 每回合 SAN 侵蚀豁免
    pub rage_bonus_atk: i32,  // 狂暴时额外攻击
    pub label: &'static str,  // HUD/兑换文案片段
}
#[derive(Clone, Copy, Debug)]
pub struct BloodlineDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub passive: BloodlinePassive,
}

// ---------- 敌人战斗修饰（BATTLE_MODS，旁附 key by fight_id） ----------
#[derive(Clone, Debug)]
pub struct BattleMods {
    pub weak_fire: i32,             // 受火系 ×1.3 → 记 30（百分比增量整数）
    pub weak_electric: i32,         // 受电/量子克制
    pub armor: i32,                 // 每击减伤（dmg_ - armor），>=0
    pub regen: Option<(i32, u32)>,  // (每回合量, 持续回合)
    pub aura: i32,                  // 每回合全队 San- 侵蚀
    pub post_kill: Option<(i32, &'static str)>, // 击杀后副作用 (量,文案)
    pub waves: &'static [&'static str],          // 连锁的下一 fight_id 序列
    pub no_dodge: bool,             // 此敌不可闪避
}

// ---------- 技能（SKILLS） ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillSchool { Wushu, Gene, Blood, Holy, Tech, Nt, Meme, Util, Xiu }

impl SkillSchool {
    /// 派系稳定字符串名（熟练度 map 的 key，避免枚举序列化麻烦；前端直接可用）
    pub fn school_name(&self) -> &'static str {
        match self {
            SkillSchool::Wushu => "Wushu",
            SkillSchool::Gene => "Gene",
            SkillSchool::Blood => "Blood",
            SkillSchool::Holy => "Holy",
            SkillSchool::Tech => "Tech",
            SkillSchool::Nt => "Nt",
            SkillSchool::Meme => "Meme",
            SkillSchool::Util => "Util",
            SkillSchool::Xiu => "Xiu",
        }
    }
}

/// 熟练度分级：0-9=0级、10-29=1级、30-59=2级、60+=3级（无上限）
pub fn proficiency_tier(proficiency: u32) -> u32 {
    match proficiency {
        0..=9 => 0,
        10..=29 => 1,
        30..=59 => 2,
        _ => 3,
    }
}

#[derive(Clone, Debug)]
pub enum SkillCost { None, Qi(u32), Item(&'static str) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillKind { Active, Passive }

/// 技能战斗效果（SkillEffectKind 的携带数据；统计式静态表用，engine 包 B 按此分发）
#[derive(Clone, Debug)]
pub enum SkillEffect {
    Striking { dmg: (i32, i32), ignore_armor: bool, hits: i32, weak: Option<&'static [&'static str]> },
    SelfBuff { hp: i32, san: i32, guard: i32, dodge_bonus: f64, atk_flat: i32 },
    DebuffEnemy { no_dodge: i32, stun: i32, dmg_over_time: Option<(i32, i32)>, dmg: (i32, i32) },
    Passive {
        atk_flat: i32, leech: i32, dodge_bonus: f64, dmg_reduce: i32,
        san_resist: i32, qi_max: i32, hp_max: i32, per_turn_qi: i32,
    },
}

#[derive(Clone, Debug)]
pub struct SkillDef {
    pub id: &'static str,
    pub name: &'static str,
    pub school: SkillSchool,
    pub desc: &'static str,
    pub price: i32,
    pub need_grade: Option<char>,
    pub need_bloodline: Option<&'static str>,
    pub need_qi: Option<i32>,
    pub need_stage: Option<u8>,        // 基因阶门槛
    pub need_cultivation: Option<u8>,  // 修真境界门槛
    pub kind: SkillKind,
    pub cost: SkillCost,
    pub effect: SkillEffect,
    pub per_fight_uses: Option<u32>,
    pub cooldown: u32,                 // 预留；全表置 0
    /// 可飞行/浮空（御剑飞行、浮空术、腾云、轻功等）；前端用于跨越地形/浮空动画
    pub flying: bool,
    /// 远程探测视野加成（单位=格，0=无）；前端用于小地图视野放大（鹰眼/扫描/侦查等）
    pub detect_bonus: u32,
}

// ---------- 修真境界（CULTIVATION_STAGES） ----------
#[derive(Clone, Copy, Debug, Default)]
pub struct StagePassive {
    pub dmg_reduce: i32,
    pub san_resist: i32,
    pub per_turn_qi: i32,
    pub atk_flat: i32,
    pub hp_max: i32,
    pub ignore_death: bool,   // 每场一次挡必死
    pub dodge_bonus: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct CultivationStageCfg {
    pub stage: u8,
    pub name: &'static str,
    pub need_points: i64,
    pub need_grade: Option<char>,
    pub prev: Option<u8>,
    pub qi_max_cap: i32,
    pub hp_regen: i32,
    pub san_regen: i32,
    pub passive: StagePassive,
}

// ---------- 装备 / 道具（Equipment · WEAPONS · GEAR · ITEMS · TRESURES） ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageType { Kinetic, Energy, Holy, Silver }

#[derive(Clone, Copy, Debug)]
pub enum WeaponSpecial { Leech(i32), Pierce(i32), Burn((i32, i32)), Stun(f64) }

/// 装备格主手武器（带增强/伤害类型/特殊属性）。与现 `Weapon` 枚举旁附共存。
/// id 用 String（持久字段，反序列化安全）。
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct WeaponSlot {
    pub id: String,
    pub enhance: u8,   // +0..+max_enhance
}

#[derive(Clone, Debug)]
pub struct WeaponDef {
    pub id: &'static str,
    pub name: &'static str,
    /// 黑魂式碎片叙事描述（无限流世界观；兑换面板小字展示，空串表示未撰写）
    pub desc: &'static str,
    pub tier: u8,
    pub dmg: (i32, i32),
    pub ammo: Option<u32>,
    pub dmg_type: DamageType,
    pub special: &'static [WeaponSpecial],
    pub base_price: i32,
    pub need_grade: Option<char>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearSlot { Armor, Accessory, Treasure }

#[derive(Clone, Copy, Debug)]
pub struct GearDef {
    pub id: &'static str,
    pub name: &'static str,
    pub slot: GearSlot,
    pub dmg_reduce: i32,
    pub atk_flat: i32,
    pub dodge: f64,
    pub san_resist: i32,
    pub qi_max: i32,
    pub hp_max: i32,
    pub per_turn_qi: i32,
    pub dmg_mult: f64,
    pub price: i32,
    pub need_grade: Option<char>,
}

/// 装备格结构（武器旁附/护甲/饰品/法宝三格）。全 serde default。
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Equipment {
    pub weapon: Option<WeaponSlot>,
    pub armor: Option<String>,
    pub accessory: Option<String>,
    pub treasure: [Option<String>; 3],   // 本命/护身/辅助
}

// 法宝（treasure）格子分装备类（TRESURE 装配）；与 GEAR 同结构复用 GearDef(slot=Treasure)
#[derive(Clone, Debug)]
pub struct TreasureDef {
    pub id: &'static str,
    pub name: &'static str,
    /// 黑魂式碎片叙事描述（无限流世界观；兑换面板小字展示，空串表示未撰写）
    pub desc: &'static str,
    pub slot: u8,                 // 槽位 0=本命 1=护身 2=辅助
    pub dmg_reduce: i32,
    pub atk_flat: i32,
    pub dodge: f64,
    pub san_resist: i32,
    pub qi_max: i32,
    pub hp_max: i32,
    pub per_turn_qi: i32,
    pub dmg_mult: f64,
    pub ignore_death: bool,
    pub price: i32,
    pub need_grade: Option<char>,
}

// ---------- 道具（ITEMS · FIGHT_ITEMS · RECIPES） ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemSub { Heal, San, Ammo, Throw, CharmProt, CharmAnti, Scroll, Quest, Reliquary }

#[derive(Clone, Debug)]
pub enum ItemEffect {
    Heal(i32),
    San(i32),
    Ammo,                 // 回满 ammo
    Throw { dmg_over_time: Option<(i32, i32)>, weak: Option<&'static str>, flat_dmg: i32 },
    Charm { immune_death: bool, cure_debuff: bool },
    None,
}

#[derive(Clone, Debug)]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ItemSub,
    pub stack: bool,                // 可堆叠（拆 id item_id_k）
    pub usable_in_fight: bool,
    pub effect: ItemEffect,
    pub price: i32,
    pub need_grade: Option<char>,
}

/// 战斗内道具效果（combat §2.3 方案 A 白名单；FIGHT_ITEMS: &[(id, effect)]）
#[derive(Clone, Debug)]
pub struct FightItemEffect {
    pub kind: ItemSub,
    pub effect: ItemEffect,
}

/// 合成表：result ← 原料列表
#[derive(Clone, Debug)]
pub struct Recipe {
    pub result: &'static str,
    pub ingredients: &'static [&'static str],
}

// re-export 数据表常量（实现于 skills_data/items_data/combat_data）
pub use crate::combat_data::{BATTLE_MODS, BLOODLINES, CULTIVATION_STAGES, FIGHT_ITEMS, GENE_STAGES};
pub use crate::items_data::{GEAR, ITEMS, RECIPES, TRESURE_DEFS, WEAPONS};
pub use crate::skills_data::SKILLS;

// re-export 血统进阶路线（实现于 bloodline_tiers）
pub use crate::bloodline_tiers::{BloodlineTierCfg, BLOODLINE_TIERS};

