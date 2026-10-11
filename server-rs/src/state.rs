//! 全局游戏状态：数值、旗标、队伍、战斗、结算
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weapon {
    Axe,
    Gun,
    Sword,
}

impl Weapon {
    pub fn name(&self) -> &'static str {
        match self {
            Weapon::Axe => "消防斧",
            Weapon::Gun => "9mm手枪",
            Weapon::Sword => "军刀",
        }
    }
    pub fn dmg(&self) -> (i32, i32) {
        match self {
            Weapon::Axe => (22, 34),
            Weapon::Gun => (14, 20),
            Weapon::Sword => (10, 16),
        }
    }
}

/// 战斗中的敌人实例
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fight {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub dmg: (i32, i32),
    pub reward: i32,
    pub reward_why: String,
    pub raged: bool,
    pub rage_at: Option<i32>,
    pub guard_turn: bool,
    /// 玩家选择的动作记录（供日志）
    #[serde(skip)]
    pub pending_log: Vec<String>,
}

/// 覆盖层卡片（死亡/结算/基因锁等）
#[derive(Clone, Debug, Serialize)]
pub struct Card {
    pub title: String,
    pub good: bool,
    pub body_html: String,
    pub buttons: Vec<(String, String)>, // (label, route)
    #[serde(skip)]
    pub voice: Option<&'static str>,
}

/// 会话模式
#[derive(Clone, Debug)]
pub enum Mode {
    Normal,
    Fight,
    AwaitCard(Card),
}

impl Default for Mode {
    fn default() -> Self { Mode::Normal }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct GameState {
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub san: i32,
    #[serde(default)]
    pub points: i32,
    #[serde(default)]
    pub weapon: Option<Weapon>,
    #[serde(default)]
    pub ammo: i32,
    #[serde(default)]
    pub gene_lock: bool,
    #[serde(default)]
    pub gene_lock_used: bool,
    #[serde(default)]
    pub flags: std::collections::BTreeMap<String, bool>,
    #[serde(default)]
    pub dead_team: Vec<String>,
    /// P1 复活闭环：最近一次成功复活的队友名（用于复活成功场景演出版文本）
    #[serde(default)]
    pub resurrected_name: Option<String>,
    #[serde(default)]
    pub scene_id: String,
    #[serde(default)]
    pub laser_fails: i32,
    #[serde(default)]
    pub fight: Option<Fight>,
    /// 开放世界：玩家坐标（地图格子）
    #[serde(default)]
    pub px: usize,
    #[serde(default)]
    pub py: usize,
    /// 当前楼层 (0=F1入口 1=F2实验 2=F3核心 3=F4底层)
    #[serde(default)]
    pub floor: usize,
    /// 物品栏
    #[serde(default)]
    pub inventory: Vec<String>,
    /// 地图对象状态（钥匙卡/门/调查点等）
    #[serde(default)]
    pub map_objs: std::collections::BTreeMap<String, bool>,
    /// 地图敌人存活状态
    #[serde(default)]
    pub enemies_alive: std::collections::BTreeMap<String, bool>,
    /// 当前 3D 副本会话（None = 在 2D 地图）
    #[serde(default)]
    pub zone: Option<ZoneSession>,
    /// 已探索格子集合（"world:floor:x:y"）—— 轮回记忆：死亡重开后保留地图记忆
    #[serde(default)]
    pub explored: std::collections::BTreeSet<String>,
    /// 当前世界 id（多世界框架 v2，全新增字段默认保护）
    #[serde(default = "default_world_id")]
    pub world_id: String,
    /// 非活跃世界的运行时快照（惰性；活跃世界镜像在顶层 map_objs/enemies_alive）
    #[serde(default)]
    pub world_states: std::collections::BTreeMap<String, WorldRuntime>,
    /// 存档版本（v2=2；迁移幂等屏障）
    #[serde(default)]
    pub save_version: u32,
    /// 最近一次支线评级 D/C/B/A/S（P3 咒怨启用，未获 None）
    #[serde(default)]
    pub sp_grade: Option<char>,
    /// 点数消费 · 细胞活力强化（体质）：每级 +5 点攻击（P1 兑换闭环新增）
    #[serde(default)]
    pub str_bonus: i32,
    /// 点数消费 · 敏捷强化：每点 +0.05 闪避（P1 兑换闭环新增；当前由吸血鬼血统附带）
    #[serde(default)]
    pub agi_bonus: i32,
    /// 点数消费 · 血统（P1 兑换闭环新增）：Some("vampire") = 初级吸血鬼血统
    #[serde(default)]
    pub bloodline: Option<String>,
    /// 战斗体系数据层（包 A/A′/A″/A‴）：全部 #[serde(default)]，旧档天然可读
    /// 基因锁多阶（0=未开，1~4=阶）；权威字段，gene_lock 为布尔视图（= gene_stage>=1）
    #[serde(default)]
    pub gene_stage: u8,
    /// 真气/内力当前量（回合内消耗，跨回合持久）
    #[serde(default)]
    pub qi: i32,
    /// 真气上限（内功心法/修真境界提供）
    #[serde(default)]
    pub qi_max: i32,
    /// 内功心法 id（None=未学；如 "wuming" 无名剑诀 / "jingxin" 静心诀）
    #[serde(default)]
    pub inner_art: Option<String>,
    /// 科技侧最小实现：当前纳米护盾值（0=无），受击先吃盾再扣 hp
    #[serde(default)]
    pub tech_shield: i32,
    /// 纳米护盾上限
    #[serde(default)]
    pub tech_shield_max: i32,
    /// 修真境界（0=未修真，1..=7：练气~合道）；权威境界
    #[serde(default)]
    pub cultivation_stage: u8,
    /// 修真境界档位（当前 qi_max 天棚）
    #[serde(default)]
    pub cultivation_qi_max: i32,
    /// 已装法宝格 id 列表（法宝装配；equipment.treasure 为装配权威时此处保留拥有标记）
    #[serde(default)]
    pub treasures: Vec<String>,
    /// 自选修真流派标签（丹/剑/符/阵，供技能分页与 cond_show）
    #[serde(default)]
    pub sect: Option<String>,
    /// 已购技能 id 列表（拥有标记；定义/数值全在静态表 SKILLS）
    #[serde(default)]
    pub skills: Vec<String>,
    /// 技能熟练度（技能 id → 熟练度；使用技能成功后 +1）。旧档缺省读 Default。
    #[serde(default)]
    pub skill_proficiency: std::collections::HashMap<String, u32>,
    /// 大类熟练度（技能中文大类名如 "剑类"/"光明魔法类"/"体修类" → 熟练度；共 14 类，见 skills_data::skill_category）。
    #[serde(default)]
    pub category_proficiency: std::collections::HashMap<String, u32>,
    /// 装备的 3 个主动技能 id（最多 3 个；切换面板维护）。旧档缺省读 Default（空）。
    #[serde(default)]
    pub equipped_skills: Vec<String>,
    /// 装备格（武器旁附/护甲/饰品/法宝三格）；全 serde default → 旧档读 Default
    #[serde(default)]
    pub equipment: crate::defs::Equipment,
    /// 血统阶位（血统 id → 当前阶位，默认 1）。HashMap 更通用：切换血统后回来仍记得阶位。
    #[serde(default)]
    pub bloodline_tier: std::collections::HashMap<String, u8>,
    /// 血统熟练度（血统 id → 使用次数）。每次使用血统相关技能/攻击 +1（展示口径）。
    #[serde(default)]
    pub bloodline_proficiency: std::collections::HashMap<String, u32>,
    /// 血统熟练度贡献值（血统 id → 当前阶内贡献值 i32）。每次使用相关技能/攻击 +k；
    /// 进阶时砍半（向下取整）、换新 base。直接存贡献值避免「反推熟练度整除丢失」精度问题。
    #[serde(default)]
    pub bloodline_contribution: std::collections::HashMap<String, i32>,
    #[serde(skip)]
    pub mode: Mode,
    #[serde(skip)]
    pub pending_death: Option<String>,
    #[serde(skip)]
    pub settle_total: i32,
    #[serde(skip)]
    pub settle_rank: char,
    /// 动态难度缩放开关（§动态缩放）：true=按主角强度×副本难度系数缩放敌人；
    /// false=缩放恒为 1.0（测试安全阀，保证既有 flow 数值回归全绿）。序列化默认 true。
    #[serde(default = "default_scaling_enabled")]
    pub scaling_enabled: bool,
}

/// scaling_enabled 序列化缺省回填：默认关闭（测试安全阀）；游戏运行时在新局/进副本时显式开启
fn default_scaling_enabled() -> bool {
    false
}

/// 每世界运行时快照（非活跃世界的 map_objs/enemies_alive 备份）
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorldRuntime {
    pub map_objs: std::collections::BTreeMap<String, bool>,
    pub enemies_alive: std::collections::BTreeMap<String, bool>,
    pub entered: bool,
}

/// 世界 id 缺省回填：生化（唯一已注册世界）
fn default_world_id() -> String {
    crate::worlds::WORLD_BIOHAZARD.to_string()
}

/// 3D 副本会话（战斗/解密小副本）
#[derive(Clone, Serialize, Deserialize)]
pub struct ZoneSession {
    pub zone_id: String,
    /// fight / puzzle
    pub kind: String,
    pub ref_id: String,
    /// 玩家实时坐标（副本内，单位米）
    pub zx: f32,
    pub zz: f32,
    /// 玩家朝向角
    pub zyaw: f32,
    /// 玩家实时血量（副本内扣减）
    pub zhp: i32,
    /// 副本进度/状态
    pub progress: i32,
    /// 玩家上一步动作（用于日志/判定）
    pub last_action: String,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            hp: 100,
            san: 100,
            points: 0,
            weapon: None,
            ammo: 6,
            gene_lock: false,
            gene_lock_used: false,
            flags: Default::default(),
            dead_team: vec![],
            resurrected_name: None,
            scene_id: "s_office".into(),
            laser_fails: 0,
            fight: None,
            px: 1,
            py: 1,
            floor: 0,
            inventory: vec![],
            map_objs: Default::default(),
            enemies_alive: Default::default(),
            zone: None,
            explored: Default::default(),
            world_id: crate::worlds::WORLD_BIOHAZARD.to_string(),
            world_states: Default::default(),
            save_version: 2,
            sp_grade: None,
            str_bonus: 0,
            agi_bonus: 0,
            bloodline: None,
            gene_stage: 0,
            qi: 0,
            qi_max: 0,
            inner_art: None,
            tech_shield: 0,
            tech_shield_max: 0,
            cultivation_stage: 0,
            cultivation_qi_max: 0,
            treasures: vec![],
            sect: None,
            skills: vec![],
            skill_proficiency: Default::default(),
            category_proficiency: Default::default(),
            equipped_skills: vec![],
            equipment: crate::defs::Equipment::default(),
            bloodline_tier: Default::default(),
            bloodline_proficiency: Default::default(),
            bloodline_contribution: Default::default(),
            mode: Mode::Normal,
            pending_death: None,
            settle_total: 0,
            settle_rank: 'D',
            scaling_enabled: false,
        }
    }
    /// 测试辅助：新建一个关闭动态难度缩放的初始状态（既有 flow 测试数值回归用）。
    pub fn new_no_scaling() -> Self {
        let mut s = Self::new();
        s.scaling_enabled = false;
        s
    }
    pub fn flag(&self, k: &str) -> bool {
        *self.flags.get(k).unwrap_or(&false)
    }
    pub fn set_flag(&mut self, k: &str) {
        self.flags.insert(k.to_string(), true);
    }
    pub fn team_alive(&self, k: &str) -> bool {
        !self.dead_team.iter().any(|d| d == k)
    }
    pub fn kill_team(&mut self, k: &str) {
        if self.team_alive(k) {
            self.dead_team.push(k.to_string());
        }
    }
    pub fn alive_count(&self) -> i32 {
        ["one", "rain", "kaplan", "jd"]
            .iter()
            .filter(|k| self.team_alive(k))
            .count() as i32
    }
}

/// 存档容器（带模式序列化辅助）
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub state: GameState,
}

/// 存档迁移 v1→v2→v3：幂等。返回被重写的 explored key 数量（供 rpc.log 统计）。
/// 设计依据 §3.1/§3.2：R1 世界归属 / R2 探索迷雾前插 / R5 版本号；R3/R6 依设计为 no-op。
/// 战斗体系包 A：R7 基因锁一档迁移——`gene_lock==true && gene_stage==0` → `gene_stage=1`，
/// 并递增 save_version 到 v3。为保持既有 v1→v2 测试断言（save_version==2）不变，
/// 仅在确实执行基因迁移时才升到 v3（gene_lock=false 的旧档停在 v2，语义一致）。
pub fn migrate_save(st: &mut GameState) -> usize {
    let mut rewritten = 0usize;
    // R7 基因锁一档迁移（v2→v3，独立于 v2 屏障；幂等：gene_stage>=1 即跳过）
    if st.gene_lock && st.gene_stage == 0 {
        st.gene_stage = 1;
        st.save_version = 3;
    }
    // 技能熟练度三字段（skill_proficiency/category_proficiency/equipped_skills）：
    // 均为 #[serde(default)]，旧档反序列化时自动回填空 HashMap/Vec，无需显式迁移。
    // 此处不写字段、不留版本号变更，保证既有 migrate_save 测试断言（save_version 语义）不变。
    // R5 幂等屏障：v2 跳过（防二次前插）
    if st.save_version >= 2 {
        return rewritten;
    }
    // R1 世界归属
    if st.world_id.is_empty() {
        st.world_id = crate::worlds::WORLD_BIOHAZARD.to_string();
    }
    // R2 探索迷雾：首段纯数字的 key 前插 "biohazard_ch1:"
    let prefix_id = crate::worlds::WORLD_BIOHAZARD;
    let old: Vec<String> = st.explored.iter().cloned().collect();
    st.explored.clear();
    for key in old {
        let first = key.split(':').next().unwrap_or("");
        if !first.is_empty() && first.chars().all(|c| c.is_ascii_digit()) {
            st.explored.insert(format!("{}:{}", prefix_id, key));
            rewritten += 1;
        } else {
            st.explored.insert(key);
        }
    }
    // R3 world_states 空则留空（顶层即活跃世界镜像，设计合法）——不填充
    // R6 sp_grade 保持（None if 缺省）
    // R5 版本号
    st.save_version = 2;
    rewritten
}
