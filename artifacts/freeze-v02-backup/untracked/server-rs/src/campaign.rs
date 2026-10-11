//! 《无限人族进化》的独立三世界试玩战役。
//!
//! 这一层故意不依赖旧的无限世界/场景系统：它是给新 UI 和本地 probe 使用的
//! 小而完整的状态机。地图、任务、战斗和存档都在这里保持确定性，方便前端
//! 以 160ms 移动、250ms tick 的节奏驱动，也方便集成测试复现完整路径。

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// 游戏标题，同时也是存档格式标识：它会被写进 data/campaign-v2.json 的
/// `title` 字段，并由 campaign_store 校验。更名后新存档写新名，
/// campaign_store 同时接受旧标识 "余界：归航协议"，所以老存档仍然可读。
pub const TITLE: &str = "无限人族进化";
const VERSION: u32 = 2;
const WIDTH: i32 = 34;
const HEIGHT: i32 = 24;

const GREY_HIVE: &str = "grey_hive";
const MIST_HARBOR: &str = "mist_harbor";
const CLOCKWORKS: &str = "clockworks";

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PlayerState {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub medkits: i32,
    #[serde(rename = "scrap", alias = "reward_points")]
    pub reward_points: i32,
    #[serde(default)]
    pub side_story_tokens: i32,
    #[serde(rename = "level", default = "legacy_player_level")]
    legacy_level: i32,
    pub build: String,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            name: "岑遥".into(),
            hp: 100,
            max_hp: 100,
            energy: 80,
            max_energy: 80,
            medkits: 3,
            reward_points: 30,
            side_story_tokens: 0,
            legacy_level: 1,
            build: "vanguard".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct WorldState {
    pub id: String,
    pub name: String,
    pub kicker: String,
    pub description: String,
    pub mechanic: String,
    pub accent: String,
    pub cleared: bool,
    pub available: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ExchangeState {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub grade: String,
    #[serde(rename = "cost", alias = "reward_cost", default)]
    pub reward_cost: i32,
    #[serde(default)]
    pub token_cost: i32,
    #[serde(default)]
    pub owned: bool,
    #[serde(rename = "level", default)]
    legacy_level: i32,
    #[serde(rename = "max_level", default = "legacy_max_level")]
    legacy_max_level: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct SkillState {
    pub id: String,
    pub name: String,
    pub key: String,
    pub cost: i32,
    pub cooldown: i32,
    pub ready: bool,
    pub description: String,
    #[serde(default)]
    cooldown_left: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ObjectiveState {
    pub id: String,
    pub title: String,
    pub description: String,
    pub done: bool,
    pub optional: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ChoiceState {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct DialogState {
    pub title: String,
    pub speaker: String,
    pub body: String,
    pub choices: Vec<ChoiceState>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ResultState {
    pub title: String,
    pub world_name: String,
    pub summary: String,
    pub rank: String,
    pub reward: i32,
    #[serde(default)]
    pub side_story_tokens: i32,
    pub all_complete: bool,
    pub stats: Value,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct EffectState {
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct TelegraphState {
    pub x: i32,
    pub y: i32,
    pub radius: i32,
    pub remaining: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct EnemyState {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub alive: bool,
    pub kind: String,
    pub telegraph: Option<TelegraphState>,
    #[serde(default)]
    attack_clock: i32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct NpcState {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct PointState {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub kind: String,
    pub done: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ZoneState {
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub active: bool,
    pub telegraph: Option<TelegraphState>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct PortalState {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub kind: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct GateState {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct PropState {
    pub kind: String,
    pub x: i32,
    pub y: i32,
    pub w: Option<i32>,
    pub h: Option<i32>,
    pub rotation: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct MapState {
    pub world_id: String,
    pub world_name: String,
    pub w: i32,
    pub h: i32,
    pub floor: i32,
    pub tiles: Vec<String>,
    pub px: i32,
    pub py: i32,
    pub revision: i32,
    pub explored: Vec<String>,
    pub enemies: Vec<EnemyState>,
    pub npcs: Vec<NpcState>,
    pub points: Vec<PointState>,
    pub zones: Vec<ZoneState>,
    pub portals: Vec<PortalState>,
    pub gates: Vec<GateState>,
    pub props: Vec<PropState>,
    pub nearby: Vec<Value>,
}

/// Persisted campaign boundary. The public fields are deliberately the same concepts
/// exposed by `view`; the private run fields make replay/reward rules unambiguous.
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CampaignState {
    pub version: u32,
    pub mode: String,
    pub title: String,
    pub subtitle: String,
    pub player: PlayerState,
    pub worlds: Vec<WorldState>,
    #[serde(rename = "upgrades", alias = "exchanges", default = "exchange_catalog")]
    pub exchanges: Vec<ExchangeState>,
    pub skills: Vec<SkillState>,
    pub objectives: Vec<ObjectiveState>,
    pub dialog: Option<DialogState>,
    pub result: Option<ResultState>,
    pub last_event: String,
    pub effect: Option<EffectState>,
    pub map: Option<MapState>,
    /// 首通抉择是战役元进度，不随某一局的 objectives 生命周期结束。
    pub decisions: BTreeMap<String, String>,

    #[serde(default)]
    current_world_id: Option<String>,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    spoils_claimed: Vec<String>,
    /// 补给箱是跨失败/回访的一次性探索资源，避免死亡重试复制奖励。
    #[serde(default)]
    supply_claimed: BTreeSet<String>,
    #[serde(default)]
    run_reward_allowed: bool,
    #[serde(default)]
    run_scrap: i32,
    #[serde(default)]
    enemies_defeated: i32,
    #[serde(default)]
    run_ticks: i32,
    #[serde(default)]
    run_damage_taken: i32,
    #[serde(default)]
    optional_choice: Option<String>,
    #[serde(default)]
    shield: i32,
    #[serde(default)]
    attack_cooldown: i32,
    #[serde(default)]
    steam_phase: i32,
    #[serde(default = "default_true")]
    spawn_protected: bool,
}

fn default_true() -> bool {
    true
}

fn legacy_player_level() -> i32 {
    1
}

impl Default for CampaignState {
    fn default() -> Self {
        let mut state = Self {
            version: VERSION,
            mode: "hub".into(),
            title: TITLE.into(),
            subtitle: "三座失联世界，三枚归航信标；出发之前，先听见自己的呼吸。".into(),
            player: PlayerState::default(),
            worlds: world_catalog(),
            exchanges: exchange_catalog(),
            skills: skill_catalog(),
            objectives: Vec::new(),
            dialog: None,
            result: None,
            last_event: "归航站安全。选择一座世界，沿着灯光走进去。".into(),
            effect: None,
            map: None,
            decisions: BTreeMap::new(),
            current_world_id: None,
            flags: Vec::new(),
            spoils_claimed: Vec::new(),
            supply_claimed: BTreeSet::new(),
            run_reward_allowed: false,
            run_scrap: 0,
            enemies_defeated: 0,
            run_ticks: 0,
            run_damage_taken: 0,
            optional_choice: None,
            shield: 0,
            attack_cooldown: 0,
            steam_phase: 0,
            spawn_protected: true,
        };
        state.recompute_capacity();
        state.player.hp = state.player.max_hp;
        state.player.energy = state.player.max_energy;
        state
    }
}

fn world_catalog() -> Vec<WorldState> {
    vec![
        WorldState {
            id: GREY_HIVE.into(),
            name: "灰巢设施".into(),
            kicker: "冷混凝土 / 暖黄设施照明".into(),
            description: "断电的地下设施把一段求救声锁在隔离区。恢复主电，拆掉封锁，把还能回答的人带回光里。".into(),
            mechanic: "断电 / 封锁 / 救援".into(),
            accent: "#c6a56b".into(),
            cleared: false,
            available: true,
        },
        WorldState {
            id: MIST_HARBOR.into(),
            name: "雾港余烬".into(),
            kicker: "青绿海水 / 空码头信号灯".into(),
            description: "雾把码头切成一格一格的暗色水面。点亮两座信标，安全航道才会在潮声里显形。".into(),
            mechanic: "双信标 / 安全通道 / 浓雾".into(),
            accent: "#54c9bd".into(),
            cleared: false,
            available: true,
        },
        WorldState {
            id: CLOCKWORKS.into(),
            name: "钟骨工厂".into(),
            kicker: "铜色齿轮 / 停摆回路".into(),
            description: "旧工厂仍按无人值守的节拍吐出蒸汽。关掉两处阀门，才能穿过炉心，把停机回路带回归航站。".into(),
            mechanic: "蒸汽危害 / 阀门 / 停机回路".into(),
            accent: "#d08b52".into(),
            cleared: false,
            available: true,
        },
    ]
}

fn exchange_catalog() -> Vec<ExchangeState> {
    vec![
        ExchangeState {
            id: "reinforced_suit".into(),
            name: "织层护甲".into(),
            description: "装备：提高最大生命。".into(),
            category: "equipment".into(),
            grade: "D".into(),
            reward_cost: 24,
            token_cost: 0,
            owned: false,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "capacitor".into(),
            name: "回能导线".into(),
            description: "装备：提高最大能量。".into(),
            category: "equipment".into(),
            grade: "D".into(),
            reward_cost: 22,
            token_cost: 0,
            owned: false,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "field_medkit".into(),
            name: "急救折叠仓".into(),
            description: "装备：增加一枚医疗包。".into(),
            category: "equipment".into(),
            grade: "D".into(),
            reward_cost: 18,
            token_cost: 0,
            owned: false,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "signal_lens".into(),
            name: "侦测棱镜".into(),
            description: "装备：扩大探索半径。".into(),
            category: "equipment".into(),
            grade: "C".into(),
            reward_cost: 20,
            token_cost: 1,
            owned: false,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "vanguard_lineage".into(),
            name: "壁垒血统".into(),
            description: "血统：获得先锋的生命上限倾向。".into(),
            category: "lineage".into(),
            grade: "C".into(),
            reward_cost: 60,
            token_cost: 1,
            owned: true,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "echo_lineage".into(),
            name: "回声血统".into(),
            description: "血统：获得回声的能量上限倾向。".into(),
            category: "lineage".into(),
            grade: "C".into(),
            reward_cost: 60,
            token_cost: 1,
            owned: false,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "pulse".into(),
            name: "脉冲".into(),
            description: "技能：以自身为中心释放震荡。".into(),
            category: "skill".into(),
            grade: "D".into(),
            reward_cost: 16,
            token_cost: 0,
            owned: true,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "guard".into(),
            name: "护盾".into(),
            description: "技能：展开一次短盾。".into(),
            category: "skill".into(),
            grade: "C".into(),
            reward_cost: 28,
            token_cost: 1,
            owned: true,
            legacy_level: 0,
            legacy_max_level: 1,
        },
        ExchangeState {
            id: "pierce".into(),
            name: "穿刺".into(),
            description: "技能：沿八方向贯穿敌人。".into(),
            category: "skill".into(),
            grade: "C".into(),
            reward_cost: 32,
            token_cost: 1,
            owned: true,
            legacy_level: 0,
            legacy_max_level: 1,
        },
    ]
}

fn legacy_max_level() -> i32 {
    1
}

fn display_exchanges(state: &CampaignState) -> Vec<ExchangeState> {
    let mut catalog = exchange_catalog();
    for item in &mut catalog {
        if let Some(saved) = state.exchanges.iter().find(|saved| saved.id == item.id) {
            item.owned = saved.owned || saved.legacy_level > 0;
        }
    }
    catalog
}

impl CampaignState {
    fn migrate_legacy(&mut self) {
        let catalog = display_exchanges(self);
        self.exchanges = catalog;
        if self.player.reward_points == 0
            && self.player.side_story_tokens == 0
            && self.player.build.is_empty()
        {
            self.player = PlayerState::default();
        }
        self.recompute_capacity();
    }
}

fn skill_catalog() -> Vec<SkillState> {
    vec![
        SkillState {
            id: "pulse".into(),
            name: "脉冲".into(),
            key: "Q".into(),
            cost: 12,
            cooldown: 3,
            ready: true,
            description: "以自身为中心释放短促震荡，伤害半径 2 格内的敌人。".into(),
            cooldown_left: 0,
        },
        SkillState {
            id: "guard".into(),
            name: "护盾".into(),
            key: "E".into(),
            cost: 15,
            cooldown: 5,
            ready: true,
            description: "展开一次短盾；受伤时先消耗护盾，生命较低时同时进行小额治疗。".into(),
            cooldown_left: 0,
        },
        SkillState {
            id: "pierce".into(),
            name: "穿刺".into(),
            key: "R".into(),
            cost: 18,
            cooldown: 4,
            ready: true,
            description: "沿八方向贯穿至多 3 格，优先命中第一名敌人。".into(),
            cooldown_left: 0,
        },
    ]
}

fn world_name(id: &str) -> &'static str {
    match id {
        GREY_HIVE => "灰巢设施",
        MIST_HARBOR => "雾港余烬",
        CLOCKWORKS => "钟骨工厂",
        _ => "未知世界",
    }
}

fn objective_catalog(id: &str) -> Vec<ObjectiveState> {
    match id {
        GREY_HIVE => vec![
            ObjectiveState {
                id: "hive_power".into(),
                title: "恢复主电".into(),
                description: "启动南侧配电台，让冷白走廊重新亮起。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "hive_lockdown".into(),
                title: "解除隔离".into(),
                description: "在通电后的终端撤下隔离锁，打开 B 区。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "hive_rescue".into(),
                title: "带回救援信标".into(),
                description: "从救援舱取出仍在闪烁的信标，再从东侧出口撤离。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "hive_choice".into(),
                title: "白芷的请求".into(),
                description: "可选：决定是否把白芷的冷藏样本一起带走。选择会影响本局评价。".into(),
                done: false,
                optional: true,
            },
        ],
        MIST_HARBOR => vec![
            ObjectiveState {
                id: "harbor_beacon_a".into(),
                title: "点亮北堤信标".into(),
                description: "让第一束青绿信号刺穿近岸浓雾。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "harbor_beacon_b".into(),
                title: "点亮旧仓信标".into(),
                description: "启动第二座信标，让两束光校准一条安全水道。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "harbor_signal".into(),
                title: "发送救援坐标".into(),
                description: "在潮闸边发送坐标，等到远处回以三次短鸣。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "harbor_choice".into(),
                title: "陆岑的取舍".into(),
                description: "可选：替陆岑保存信号燃料，或把它留给撤离船队。".into(),
                done: false,
                optional: true,
            },
        ],
        CLOCKWORKS => vec![
            ObjectiveState {
                id: "clock_valve_a".into(),
                title: "关闭一号阀".into(),
                description: "拧到底部铜阀，停掉第一段蒸汽喷口。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "clock_valve_b".into(),
                title: "关闭二号阀".into(),
                description: "沿停摆回路走到高压阀，给炉心留出一条无烟通路。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "clock_core".into(),
                title: "取出停机回路".into(),
                description: "从炉心取出铜色停机回路，保持它在冷却前离开工厂。".into(),
                done: false,
                optional: false,
            },
            ObjectiveState {
                id: "clock_choice".into(),
                title: "孟阙的最后一句话".into(),
                description: "可选：为孟阙解开锁链，或优先带走完整的回路。".into(),
                done: false,
                optional: true,
            },
        ],
        _ => Vec::new(),
    }
}

fn optional_npc_id(id: &str) -> &'static str {
    match id {
        GREY_HIVE => "hive_baizhi",
        MIST_HARBOR => "harbor_lucen",
        CLOCKWORKS => "clock_mengque",
        _ => "",
    }
}

fn optional_objective_id(id: &str) -> &'static str {
    match id {
        GREY_HIVE => "hive_choice",
        MIST_HARBOR => "harbor_choice",
        CLOCKWORKS => "clock_choice",
        _ => "",
    }
}

fn gate_ids(id: &str) -> (&'static str, &'static str) {
    match id {
        GREY_HIVE => ("hive_gate_a", "hive_gate_b"),
        MIST_HARBOR => ("harbor_gate_a", "harbor_gate_b"),
        CLOCKWORKS => ("clock_gate_a", "clock_gate_b"),
        _ => ("", ""),
    }
}

fn gate_positions(world_id: &str) -> ((i32, i32), (i32, i32)) {
    match world_id {
        CLOCKWORKS => ((8, 7), (26, 16)),
        MIST_HARBOR => ((17, 7), (23, 16)),
        _ => ((17, 7), (17, 16)),
    }
}

fn exit_position(world_id: &str) -> (i32, i32) {
    match world_id {
        MIST_HARBOR => (29, 21),
        GREY_HIVE => (30, 20),
        _ => (29, 20),
    }
}

fn base_grid(world_id: &str) -> Vec<String> {
    let mut grid = vec![vec![b'.'; WIDTH as usize]; HEIGHT as usize];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1 {
                grid[y as usize][x as usize] = b'#';
            }
        }
    }

    if world_id == MIST_HARBOR {
        // 雾港不是带水贴图的普通房间：除边框外，初始底面全是不可走水面，
        // 三块码头岛和两条窄连接是唯一的陆地。
        for y in 1..HEIGHT - 1 {
            for x in 1..WIDTH - 1 {
                grid[y as usize][x as usize] = b'~';
            }
        }
        for y in 2..=6 {
            for x in 2..=29 {
                grid[y as usize][x as usize] = b'.';
            }
        }
        for y in 10..=14 {
            for x in 4..=29 {
                grid[y as usize][x as usize] = b'.';
            }
        }
        for y in 20..=22 {
            for x in 20..=30 {
                grid[y as usize][x as usize] = b'.';
            }
        }
        for y in 6..=10 {
            grid[y as usize][17] = b'.';
        }
        for y in 14..=20 {
            grid[y as usize][23] = b'.';
        }
        // 第二信标完成前，x23,y19 是水缺口；不是隐藏的绕路墙。
        grid[19][23] = b'~';
        grid[7][17] = b'#';
        grid[16][23] = b'#';
        return grid
            .into_iter()
            .map(|row| String::from_utf8(row).expect("ASCII map"))
            .collect();
    }

    // 灰巢与钟骨都保留大型分区的空间感，但墙、门和中部结构各自不同。
    for x in 1..WIDTH - 1 {
        grid[7][x as usize] = b'#';
        grid[16][x as usize] = b'#';
    }
    if world_id == CLOCKWORKS {
        // 工厂中部的竖隔墙只留一个可读的缺口 (18,13)。
        for y in 9..=15 {
            grid[y as usize][18] = b'#';
        }
        grid[13][18] = b'.';
    } else {
        // 灰巢的机架墙和冷却间边界，主道路在 y4/y11 留出通路。
        for y in 2..6 {
            if y != 4 {
                grid[y as usize][11] = b'#';
            }
        }
        for y in 9..15 {
            if y != 11 {
                grid[y as usize][11] = b'#';
            }
        }
    }
    for x in 22..30 {
        if x != 26 {
            grid[21][x as usize] = b'#';
        }
    }
    let ((gate_ax, gate_ay), (gate_bx, gate_by)) = gate_positions(world_id);
    grid[gate_ay as usize][gate_ax as usize] = b'#';
    grid[gate_by as usize][gate_bx as usize] = b'#';
    // 工厂蒸汽位于可走地板上，危险由 tick 的预警/落点处理，不伪装成水。
    if world_id == CLOCKWORKS {
        grid[4][6] = b'.';
        grid[11][24] = b'.';
    }
    grid.into_iter()
        .map(|row| String::from_utf8(row).expect("ASCII map"))
        .collect()
}

fn enemy(id: &str, name: &str, x: i32, y: i32, hp: i32, kind: &str) -> EnemyState {
    EnemyState {
        id: id.into(),
        name: name.into(),
        x,
        y,
        hp,
        max_hp: hp,
        alive: true,
        kind: kind.into(),
        telegraph: None,
        attack_clock: 0,
    }
}

fn prop(kind: &str, x: i32, y: i32, w: i32, h: i32, rotation: i32) -> PropState {
    PropState {
        kind: kind.into(),
        x,
        y,
        w: Some(w),
        h: Some(h),
        rotation: Some(rotation.rem_euclid(360)),
    }
}

fn props_for(world_id: &str) -> Vec<PropState> {
    match world_id {
        GREY_HIVE => vec![
            prop("power", 5, 4, 1, 1, 0),
            prop("rack", 10, 3, 1, 2, 90),
            prop("rack", 12, 5, 1, 2, 270),
            prop("crate", 5, 6, 1, 1, 0),
            prop("crate", 9, 5, 1, 1, 90),
            prop("pipe", 14, 6, 3, 1, 0),
            prop("lamp", 19, 9, 1, 1, 0),
            prop("console", 27, 12, 1, 1, 180),
        ],
        MIST_HARBOR => vec![
            // 起点 (3,3) 留作安全读 brief 区；仓库贴北岛边缘，不盖住出生位。
            prop("warehouse", 3, 6, 2, 1, 0),
            prop("crate", 6, 5, 1, 1, 90),
            prop("crane", 27, 3, 2, 3, 0),
            prop("lamp", 22, 6, 1, 1, 180),
            prop("warehouse", 5, 12, 2, 2, 0),
            prop("crate", 28, 13, 1, 1, 270),
            prop("pipe", 21, 14, 3, 1, 90),
            prop("crate", 24, 21, 1, 1, 0),
            prop("crane", 29, 22, 1, 2, 180),
        ],
        CLOCKWORKS => vec![
            // 熔炉靠墙摆放，起点周围保留 2~3 格清空区。
            prop("furnace", 2, 6, 2, 1, 0),
            prop("gear", 10, 5, 2, 2, 90),
            prop("pipe", 12, 3, 4, 1, 0),
            prop("power", 7, 6, 1, 1, 180),
            prop("gear", 14, 12, 2, 2, 270),
            prop("pipe", 19, 10, 3, 1, 90),
            prop("furnace", 28, 12, 2, 2, 0),
            prop("gear", 22, 17, 2, 1, 180),
            prop("crate", 29, 20, 1, 1, 0),
        ],
        _ => Vec::new(),
    }
}

fn map_for(world_id: &str) -> MapState {
    let (gate_a, gate_b) = gate_ids(world_id);
    let (point_1, point_2, point_3) = match world_id {
        GREY_HIVE => (
            PointState {
                id: "hive_power".into(),
                name: "主配电台".into(),
                x: 7,
                y: 4,
                kind: "objective".into(),
                done: false,
            },
            PointState {
                id: "hive_lockdown".into(),
                name: "隔离终端".into(),
                x: 8,
                y: 12,
                kind: "objective".into(),
                done: false,
            },
            PointState {
                id: "hive_rescue".into(),
                name: "救援舱".into(),
                x: 27,
                y: 20,
                kind: "objective".into(),
                done: false,
            },
        ),
        MIST_HARBOR => (
            PointState {
                id: "harbor_beacon_a".into(),
                name: "北堤信标".into(),
                x: 7,
                y: 4,
                kind: "objective".into(),
                done: false,
            },
            PointState {
                id: "harbor_beacon_b".into(),
                name: "旧仓信标".into(),
                x: 25,
                y: 11,
                kind: "objective".into(),
                done: false,
            },
            PointState {
                id: "harbor_signal".into(),
                name: "潮闸信号台".into(),
                x: 26,
                y: 20,
                kind: "objective".into(),
                done: false,
            },
        ),
        CLOCKWORKS => (
            PointState {
                id: "clock_valve_a".into(),
                name: "一号铜阀".into(),
                x: 8,
                y: 4,
                kind: "objective".into(),
                done: false,
            },
            PointState {
                id: "clock_valve_b".into(),
                name: "二号高压阀".into(),
                x: 26,
                y: 11,
                kind: "objective".into(),
                done: false,
            },
            PointState {
                id: "clock_core".into(),
                name: "停机回路".into(),
                x: 26,
                y: 20,
                kind: "objective".into(),
                done: false,
            },
        ),
        _ => (
            PointState::default(),
            PointState::default(),
            PointState::default(),
        ),
    };

    let supplies = match world_id {
        GREY_HIVE => vec![
            PointState {
                id: "hive_supply_a".into(),
                name: "机架补给箱".into(),
                x: 10,
                y: 5,
                kind: "supply".into(),
                done: false,
            },
            PointState {
                id: "hive_supply_b".into(),
                name: "冷却间补给箱".into(),
                x: 23,
                y: 10,
                kind: "supply".into(),
                done: false,
            },
        ],
        MIST_HARBOR => vec![
            PointState {
                id: "harbor_supply_a".into(),
                name: "北堤补给箱".into(),
                x: 26,
                y: 5,
                kind: "supply".into(),
                done: false,
            },
            PointState {
                id: "harbor_supply_b".into(),
                name: "南岛补给箱".into(),
                x: 22,
                y: 21,
                kind: "supply".into(),
                done: false,
            },
        ],
        CLOCKWORKS => vec![
            PointState {
                id: "clock_supply_a".into(),
                name: "检修补给箱".into(),
                x: 14,
                y: 6,
                kind: "supply".into(),
                done: false,
            },
            PointState {
                id: "clock_supply_b".into(),
                name: "炉侧补给箱".into(),
                x: 29,
                y: 17,
                kind: "supply".into(),
                done: false,
            },
        ],
        _ => Vec::new(),
    };

    let (npcs, enemies) = match world_id {
        GREY_HIVE => (
            vec![NpcState {
                id: "hive_baizhi".into(),
                name: "白芷".into(),
                x: 25,
                y: 12,
                kind: "survivor".into(),
            }],
            vec![
                enemy("hive_pursuer", "灰膜追击者", 14, 5, 42, "chaser"),
                enemy("hive_guard", "隔离守卫", 21, 12, 62, "elite"),
                enemy("hive_sentinel", "救援舱哨兵", 26, 20, 78, "elite"),
            ],
        ),
        MIST_HARBOR => (
            vec![NpcState {
                id: "harbor_lucen".into(),
                name: "陆岑".into(),
                x: 24,
                y: 13,
                kind: "signalman".into(),
            }],
            vec![
                enemy("harbor_pursuer", "雾行者", 13, 5, 44, "chaser"),
                enemy("harbor_guard", "码头守卫", 20, 11, 66, "elite"),
                enemy("harbor_sentinel", "潮闸执守", 25, 20, 82, "elite"),
            ],
        ),
        CLOCKWORKS => (
            vec![NpcState {
                id: "clock_mengque".into(),
                name: "孟阙".into(),
                x: 8,
                y: 12,
                kind: "worker".into(),
            }],
            vec![
                enemy("clock_pursuer", "齿噪猎手", 13, 5, 46, "chaser"),
                enemy("clock_guard", "铆钉守卫", 20, 11, 70, "elite"),
                enemy("clock_sentinel", "炉心督军", 25, 20, 88, "elite"),
            ],
        ),
        _ => (Vec::new(), Vec::new()),
    };

    let mut zones = Vec::new();
    if world_id == MIST_HARBOR {
        zones.push(ZoneState {
            kind: "water_fog".into(),
            x: 23,
            y: 19,
            w: 1,
            h: 1,
            active: true,
            telegraph: None,
        });
    } else if world_id == CLOCKWORKS {
        zones.push(ZoneState {
            kind: "steam_vent_a".into(),
            x: 6,
            y: 4,
            w: 1,
            h: 1,
            active: true,
            telegraph: None,
        });
        zones.push(ZoneState {
            kind: "steam_vent_b".into(),
            x: 24,
            y: 11,
            w: 1,
            h: 1,
            active: true,
            telegraph: None,
        });
    }

    let ((gate_ax, gate_ay), (gate_bx, gate_by)) = gate_positions(world_id);
    let (exit_x, exit_y) = exit_position(world_id);
    let mut points = vec![point_1, point_2, point_3];
    points.extend(supplies);

    MapState {
        world_id: world_id.into(),
        world_name: world_name(world_id).into(),
        w: WIDTH,
        h: HEIGHT,
        floor: 0,
        tiles: base_grid(world_id),
        px: 3,
        py: 3,
        revision: 1,
        explored: Vec::new(),
        enemies,
        npcs,
        points,
        zones,
        portals: vec![PortalState {
            id: "exit".into(),
            name: "归航出口".into(),
            x: exit_x,
            y: exit_y,
            kind: "exit".into(),
            active: false,
        }],
        gates: vec![
            GateState {
                id: gate_a.into(),
                name: "隔离门 A".into(),
                x: gate_ax,
                y: gate_ay,
                locked: true,
            },
            GateState {
                id: gate_b.into(),
                name: "隔离门 B".into(),
                x: gate_bx,
                y: gate_by,
                locked: true,
            },
        ],
        props: props_for(world_id),
        nearby: Vec::new(),
    }
}

fn distance_sq(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    let dx = ax - bx;
    let dy = ay - by;
    dx * dx + dy * dy
}

fn in_interact_range(ax: i32, ay: i32, bx: i32, by: i32) -> bool {
    distance_sq(ax, ay, bx, by) <= 2
}

fn sign(v: i32) -> i32 {
    if v < 0 {
        -1
    } else if v > 0 {
        1
    } else {
        0
    }
}

fn parse_i32(args: &Value, key: &str) -> Option<i32> {
    args.get(key)
        .and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
}

fn parse_string(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn parse_direction(args: &Value, allow_two: bool) -> Option<(i32, i32)> {
    let dx = parse_i32(args, "dx")?;
    let dy = parse_i32(args, "dy")?;
    let max = if allow_two { 2 } else { 1 };
    if dx == 0 && dy == 0 || dx.abs() > max || dy.abs() > max {
        return None;
    }
    Some((sign(dx), sign(dy)))
}

fn flag_has(state: &CampaignState, flag: &str) -> bool {
    state.flags.iter().any(|f| f == flag)
}

fn flag_add(state: &mut CampaignState, flag: &str) {
    if !flag_has(state, flag) {
        state.flags.push(flag.into());
    }
}

fn map_tile(map: &MapState, x: i32, y: i32) -> Option<u8> {
    if x < 0 || y < 0 || x >= map.w || y >= map.h {
        None
    } else {
        map.tiles
            .get(y as usize)
            .and_then(|row| row.as_bytes().get(x as usize).copied())
    }
}

fn set_map_tile(map: &mut MapState, x: i32, y: i32, tile: u8) {
    if x < 0 || y < 0 || x >= map.w || y >= map.h {
        return;
    }
    if let Some(row) = map.tiles.get_mut(y as usize) {
        let mut bytes = row.as_bytes().to_vec();
        bytes[x as usize] = tile;
        *row = String::from_utf8(bytes).expect("ASCII map");
    }
}

fn walkable(map: &MapState, x: i32, y: i32) -> bool {
    map_tile(map, x, y) == Some(b'.')
}

fn can_step(map: &MapState, from_x: i32, from_y: i32, dx: i32, dy: i32) -> bool {
    if dx == 0 && dy == 0 {
        return false;
    }
    let nx = from_x + dx;
    let ny = from_y + dy;
    if !walkable(map, nx, ny) {
        return false;
    }
    // 对角移动不得穿过两个相邻角，墙、锁门和水都算阻挡。
    if dx != 0 && dy != 0 {
        walkable(map, from_x + dx, from_y) && walkable(map, from_x, from_y + dy)
    } else {
        true
    }
}

fn has_line_of_sight(map: &MapState, x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    // Bresenham 只检查中间格；锁门、墙和水都会截断视线。
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if !(x == x0 && y == y0) && !(x == x1 && y == y1) && map_tile(map, x, y) != Some(b'.') {
            return false;
        }
        if x == x1 && y == y1 {
            return true;
        }
        let twice = 2 * err;
        if twice >= dy {
            err += dy;
            x += sx;
        }
        if twice <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn reveal(map: &mut MapState, cx: i32, cy: i32, radius: i32) {
    let mut cells: BTreeSet<String> = map.explored.iter().cloned().collect();
    for y in (cy - radius).max(0)..=(cy + radius).min(map.h - 1) {
        for x in (cx - radius).max(0)..=(cx + radius).min(map.w - 1) {
            if distance_sq(cx, cy, x, y) <= radius * radius {
                cells.insert(format!("{x}:{y}"));
            }
        }
    }
    map.explored = cells.into_iter().collect();
}

fn detect_bonus(state: &CampaignState) -> i32 {
    state
        .exchanges
        .iter()
        .find(|u| u.id == "signal_lens" && u.owned)
        .map(|_| 1)
        .unwrap_or(0)
}

fn exploration_radius(state: &CampaignState) -> i32 {
    let base = if state.current_world_id.as_deref() == Some(MIST_HARBOR) {
        if flag_has(state, "harbor_beacon_b") {
            8
        } else if flag_has(state, "harbor_beacon_a") {
            5
        } else {
            3
        }
    } else {
        5
    };
    base + detect_bonus(state)
}

fn optional_positive(world_id: &str, choice: &str) -> bool {
    match world_id {
        GREY_HIVE => choice == "hive_help_baizhi",
        MIST_HARBOR => choice == "harbor_save_fuel",
        CLOCKWORKS => choice == "clock_free_mengque",
        _ => false,
    }
}

fn optional_dialog(world_id: &str) -> DialogState {
    match world_id {
        GREY_HIVE => DialogState {
            title: "冷藏舱边的短对话".into(),
            speaker: "白芷".into(),
            body: "白芷把一枚冻得发白的样本盒推到脚边：\"它不能替我们走路，但它记得这里发生过什么。你来决定。\"".into(),
            choices: vec![
                ChoiceState { id: "hive_help_baizhi".into(), label: "替她带走样本".into(), description: "多带一件东西，回收舱的灯会亮得久一点。".into() },
                ChoiceState { id: "hive_leave_sample".into(), label: "留下样本，先撤离".into(), description: "轻装离开，救援信标不会再等人。".into() },
            ],
        },
        MIST_HARBOR => DialogState {
            title: "雾里的取舍".into(),
            speaker: "陆岑".into(),
            body: "陆岑用手背挡住信号灯：\"燃料只够最后一次长鸣。它可以留给船，也可以让我们的坐标更清楚。\"".into(),
            choices: vec![
                ChoiceState { id: "harbor_save_fuel".into(), label: "替船队保存燃料".into(), description: "救援船能多走一段，评价更看重这份让渡。".into() },
                ChoiceState { id: "harbor_burn_fuel".into(), label: "烧掉燃料校准坐标".into(), description: "自己更容易被看见，但船队少了一次余量。".into() },
            ],
        },
        CLOCKWORKS => DialogState {
            title: "停摆前的请求".into(),
            speaker: "孟阙".into(),
            body: "孟阙的手腕还挂在传送链上：\"回路在我身后。先解开我，还是先把能让全厂停下来的东西带走？\"".into(),
            choices: vec![
                ChoiceState { id: "clock_free_mengque".into(), label: "先解开孟阙".into(), description: "停机回路会更难拿，但有人能一起走出炉心。".into() },
                ChoiceState { id: "clock_take_core".into(), label: "先取停机回路".into(), description: "把全厂的危险先按下去，再回来找人已经来不及。".into() },
            ],
        },
        _ => DialogState::default(),
    }
}

fn choice_is_positive(world_id: &str, choice: &str) -> bool {
    optional_positive(world_id, choice)
}

fn world_is_cleared(state: &CampaignState, world_id: &str) -> bool {
    state.worlds.iter().any(|w| w.id == world_id && w.cleared)
}

fn world_index(state: &CampaignState, world_id: &str) -> Option<usize> {
    state.worlds.iter().position(|w| w.id == world_id)
}

fn main_objectives_done(state: &CampaignState) -> bool {
    !state.objectives.is_empty()
        && state
            .objectives
            .iter()
            .filter(|o| !o.optional)
            .all(|o| o.done)
}

fn set_zone_active(map: &mut MapState, kind: &str, active: bool) {
    if let Some(zone) = map.zones.iter_mut().find(|z| z.kind == kind) {
        zone.active = active;
    }
}

fn open_gate(map: &mut MapState, gate_id: &str) -> bool {
    let Some(index) = map.gates.iter().position(|g| g.id == gate_id) else {
        return false;
    };
    if !map.gates[index].locked {
        return false;
    }
    map.gates[index].locked = false;
    let (x, y) = (map.gates[index].x, map.gates[index].y);
    set_map_tile(map, x, y, b'.');
    map.revision += 1;
    true
}

fn gate_locked(map: &MapState, gate_id: &str) -> Option<bool> {
    map.gates.iter().find(|g| g.id == gate_id).map(|g| g.locked)
}

fn point_position(map: &MapState, id: &str) -> Option<(i32, i32)> {
    map.points.iter().find(|p| p.id == id).map(|p| (p.x, p.y))
}

fn object_position(map: &MapState, id: &str) -> Option<(i32, i32)> {
    point_position(map, id)
        .or_else(|| map.npcs.iter().find(|n| n.id == id).map(|n| (n.x, n.y)))
        .or_else(|| map.gates.iter().find(|g| g.id == id).map(|g| (g.x, g.y)))
        .or_else(|| map.portals.iter().find(|p| p.id == id).map(|p| (p.x, p.y)))
}

fn enemy_view(enemy: &EnemyState) -> Value {
    json!({
        "id": enemy.id,
        "name": enemy.name,
        "x": enemy.x,
        "y": enemy.y,
        "hp": enemy.hp.max(0),
        "max_hp": enemy.max_hp,
        "alive": enemy.alive,
        "kind": enemy.kind,
        "telegraph": enemy.telegraph.as_ref().map(|t| json!({"x": t.x, "y": t.y, "radius": t.radius, "remaining": t.remaining})).unwrap_or(Value::Null),
    })
}

fn zone_view(zone: &ZoneState) -> Value {
    json!({
        "kind": zone.kind,
        "x": zone.x,
        "y": zone.y,
        "w": zone.w,
        "h": zone.h,
        "active": zone.active,
        "telegraph": zone.telegraph.as_ref().map(|t| json!({"x": t.x, "y": t.y, "radius": t.radius, "remaining": t.remaining})).unwrap_or(Value::Null),
    })
}

fn prop_view(prop: &PropState) -> Value {
    let mut value = json!({"kind": prop.kind, "x": prop.x, "y": prop.y});
    if let Some(w) = prop.w {
        value["w"] = json!(w);
    }
    if let Some(h) = prop.h {
        value["h"] = json!(h);
    }
    if let Some(rotation) = prop.rotation {
        value["rotation"] = json!(rotation);
    }
    value
}

fn nearby_value(state: &CampaignState, map: &MapState) -> Vec<Value> {
    let mut entries: Vec<(i32, String, Value)> = Vec::new();
    let (px, py) = (map.px, map.py);
    for enemy in map.enemies.iter().filter(|e| e.alive) {
        let dx = enemy.x - px;
        let dy = enemy.y - py;
        let d = distance_sq(px, py, enemy.x, enemy.y);
        if d <= 2 {
            entries.push((
                d,
                enemy.id.clone(),
                json!({"id": enemy.id, "name": enemy.name, "kind": "enemy", "dx": dx, "dy": dy}),
            ));
        }
    }
    for npc in map
        .npcs
        .iter()
        .filter(|n| n.id != optional_npc_id(&map.world_id) || state.optional_choice.is_none())
    {
        let dx = npc.x - px;
        let dy = npc.y - py;
        let d = distance_sq(px, py, npc.x, npc.y);
        if d <= 2 {
            entries.push((
                d,
                npc.id.clone(),
                json!({"id": npc.id, "name": npc.name, "kind": "npc", "dx": dx, "dy": dy}),
            ));
        }
    }
    for point in map.points.iter().filter(|p| !p.done) {
        let dx = point.x - px;
        let dy = point.y - py;
        let d = distance_sq(px, py, point.x, point.y);
        if d <= 2 {
            entries.push((
                d,
                point.id.clone(),
                json!({"id": point.id, "name": point.name, "kind": point.kind, "dx": dx, "dy": dy}),
            ));
        }
    }
    for gate in &map.gates {
        let dx = gate.x - px;
        let dy = gate.y - py;
        let d = distance_sq(px, py, gate.x, gate.y);
        if d <= 2 {
            entries.push((
                d,
                gate.id.clone(),
                json!({"id": gate.id, "name": gate.name, "kind": "gate", "dx": dx, "dy": dy}),
            ));
        }
    }
    for portal in &map.portals {
        let dx = portal.x - px;
        let dy = portal.y - py;
        let d = distance_sq(px, py, portal.x, portal.y);
        if d <= 2 {
            entries.push((
                d,
                portal.id.clone(),
                json!({"id": portal.id, "name": portal.name, "kind": "portal", "dx": dx, "dy": dy}),
            ));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    entries.into_iter().map(|(_, _, value)| value).collect()
}

fn map_view(state: &CampaignState, map: &MapState) -> Value {
    let enemies: Vec<Value> = map.enemies.iter().map(enemy_view).collect();
    let npcs: Vec<Value> = map
        .npcs
        .iter()
        .map(|n| json!({"id": n.id, "name": n.name, "x": n.x, "y": n.y, "kind": n.kind}))
        .collect();
    let points: Vec<Value> = map.points.iter().map(|p| json!({"id": p.id, "name": p.name, "x": p.x, "y": p.y, "kind": p.kind, "done": p.done})).collect();
    let portals: Vec<Value> = map.portals.iter().map(|p| json!({"id": p.id, "name": p.name, "x": p.x, "y": p.y, "kind": p.kind, "active": p.active})).collect();
    let gates: Vec<Value> = map
        .gates
        .iter()
        .map(|g| json!({"id": g.id, "name": g.name, "x": g.x, "y": g.y, "locked": g.locked}))
        .collect();
    let zones: Vec<Value> = map.zones.iter().map(zone_view).collect();
    let props: Vec<Value> = map.props.iter().map(prop_view).collect();
    json!({
        "world_id": map.world_id,
        "world_name": map.world_name,
        "w": map.w,
        "h": map.h,
        "floor": map.floor,
        "tiles": map.tiles,
        "px": map.px,
        "py": map.py,
        "revision": map.revision,
        "explored": map.explored,
        "enemies": enemies,
        "npcs": npcs,
        "points": points,
        "zones": zones,
        "portals": portals,
        "gates": gates,
        "props": props,
        "nearby": nearby_value(state, map),
    })
}

fn dialog_view(dialog: &DialogState) -> Value {
    json!({
        "title": dialog.title,
        "speaker": dialog.speaker,
        "body": dialog.body,
        "choices": dialog.choices.iter().map(|c| json!({"id": c.id, "label": c.label, "description": c.description})).collect::<Vec<_>>(),
    })
}

fn result_view(result: &ResultState) -> Value {
    json!({
        "title": result.title,
        "world_name": result.world_name,
        "summary": result.summary,
        "rank": result.rank,
        "reward": result.reward,
        "side_story_tokens": result.side_story_tokens,
        "all_complete": result.all_complete,
        "stats": result.stats,
    })
}

/// Produce the complete front-end snapshot. This function has no side effects.
pub fn view(state: &CampaignState) -> Value {
    let worlds: Vec<Value> = state
        .worlds
        .iter()
        .map(|w| {
            json!({
                "id": w.id,
                "name": w.name,
                "kicker": w.kicker,
                "description": w.description,
                "mechanic": w.mechanic,
                "accent": w.accent,
                "cleared": w.cleared,
                "available": true,
            })
        })
        .collect();
    let exchanges: Vec<Value> = display_exchanges(state)
        .iter()
        .map(|u| {
            json!({
                "id": u.id,
                "name": u.name,
                "description": u.description,
                "category": u.category,
                "grade": u.grade,
                "reward_cost": u.reward_cost,
                "token_cost": u.token_cost,
                "owned": u.owned,
            })
        })
        .collect();
    let skills: Vec<Value> = state.skills.iter().map(|s| {
        let exchange = display_exchanges(state).into_iter().find(|item| item.id == s.id);
        json!({
        "id": s.id,
        "name": s.name,
        "key": s.key,
        "cost": s.cost,
        "category": "skill",
        "grade": exchange.as_ref().map(|item| item.grade.as_str()).unwrap_or("D"),
        "reward_cost": exchange.as_ref().map(|item| item.reward_cost).unwrap_or(0),
        "token_cost": exchange.as_ref().map(|item| item.token_cost).unwrap_or(0),
        "owned": exchange.as_ref().map(|item| item.owned).unwrap_or(false),
        "cooldown": s.cooldown_left.max(0),
        "cooldown_max": s.cooldown,
        "ready": exchange.as_ref().map(|item| item.owned).unwrap_or(false) && s.cooldown_left <= 0 && state.player.energy >= s.cost,
        "description": s.description,
    })
    }).collect();
    let objectives: Vec<Value> = state
        .objectives
        .iter()
        .map(|o| {
            json!({
                "id": o.id,
                "title": o.title,
                "description": o.description,
                "done": o.done,
                "optional": o.optional,
            })
        })
        .collect();
    let dialog = state
        .dialog
        .as_ref()
        .map(dialog_view)
        .unwrap_or(Value::Null);
    let result = state
        .result
        .as_ref()
        .map(result_view)
        .unwrap_or(Value::Null);
    let effect = state
        .effect
        .as_ref()
        .map(|e| json!({"kind": e.kind, "x": e.x, "y": e.y, "dx": e.dx, "dy": e.dy}))
        .unwrap_or(Value::Null);
    let map = state
        .map
        .as_ref()
        .map(|m| map_view(state, m))
        .unwrap_or(Value::Null);
    json!({
        "version": VERSION,
        "mode": state.mode,
        "title": TITLE,
        "subtitle": state.subtitle,
        "player": {
            "name": state.player.name,
            "hp": state.player.hp.max(0),
            "max_hp": state.player.max_hp,
            "energy": state.player.energy,
            "max_energy": state.player.max_energy,
            "medkits": state.player.medkits,
            "reward_points": state.player.reward_points,
            "side_story_tokens": state.player.side_story_tokens,
            "build": state.player.build,
            "shield": state.shield.max(0),
        },
        "worlds": worlds,
        "exchanges": exchanges,
        "skills": skills,
        "objectives": objectives,
        "dialog": dialog,
        "result": result,
        "last_event": state.last_event,
        "effect": effect,
        "map": map,
        "decisions": state.decisions,
    })
}

impl CampaignState {
    fn set_event(&mut self, message: impl Into<String>) {
        self.last_event = message.into();
    }

    fn invalid(&mut self, message: impl Into<String>) -> Value {
        self.set_event(message);
        view(self)
    }

    fn recompute_capacity(&mut self) {
        let armor = self
            .exchanges
            .iter()
            .find(|u| u.id == "reinforced_suit" && u.owned)
            .map(|_| 1)
            .unwrap_or(0);
        let capacitor = self
            .exchanges
            .iter()
            .find(|u| u.id == "capacitor" && u.owned)
            .map(|_| 1)
            .unwrap_or(0);
        let (mut max_hp, mut max_energy) = (100 + armor * 12, 80 + capacitor * 12);
        if self.player.build == "vanguard" {
            max_hp += 20;
        } else {
            max_energy += 20;
        }
        self.player.max_hp = max_hp;
        self.player.max_energy = max_energy;
        self.player.hp = self.player.hp.min(max_hp).max(0);
        self.player.energy = self.player.energy.min(max_energy).max(0);
    }

    fn start_world(&mut self, world_id: &str) {
        let Some(world) = self.worlds.iter().find(|w| w.id == world_id && w.available) else {
            self.set_event("这座世界不存在，入口没有回应。");
            return;
        };
        let cleared = world.cleared;
        let world_label = world.name.clone();
        let spoils_claimed = self.spoils_claimed.iter().any(|id| id == world_id);
        self.current_world_id = Some(world_id.into());
        self.flags.clear();
        self.objectives = objective_catalog(world_id);
        self.dialog = None;
        self.result = None;
        self.map = Some(map_for(world_id));
        self.mode = "world".into();
        if let Some(map) = self.map.as_mut() {
            for point in &mut map.points {
                if point.kind == "supply" && self.supply_claimed.contains(&point.id) {
                    point.done = true;
                }
            }
        }
        self.run_reward_allowed = !cleared && !spoils_claimed;
        self.run_scrap = 0;
        self.enemies_defeated = 0;
        self.run_ticks = 0;
        self.run_damage_taken = 0;
        self.optional_choice = None;
        self.shield = 0;
        self.attack_cooldown = 0;
        self.steam_phase = 0;
        self.spawn_protected = true;
        for skill in &mut self.skills {
            skill.cooldown_left = 0;
            skill.ready = true;
        }
        let medkit_level = self
            .exchanges
            .iter()
            .find(|u| u.id == "field_medkit" && u.owned)
            .map(|_| 1)
            .unwrap_or(0);
        self.player.medkits = 3 + medkit_level;
        self.player.hp = self.player.max_hp;
        self.player.energy = self.player.max_energy;
        if let Some(decision) = self.decisions.get(world_id).cloned() {
            self.optional_choice = Some(decision);
            if let Some(objective) = self
                .objectives
                .iter_mut()
                .find(|o| o.id == optional_objective_id(world_id))
            {
                objective.done = true;
            }
        }
        let radius = exploration_radius(self);
        if let Some(map) = self.map.as_mut() {
            let (px, py) = (map.px, map.py);
            reveal(map, px, py, radius);
        }
        let note = match world_id {
            GREY_HIVE => "安全起点：南侧配电台在起点东南，墙门只会在主任务完成后打开。",
            MIST_HARBOR => "安全起点：先找北堤信标；雾中水面不可走，两束信标会生成可见的安全通道。",
            CLOCKWORKS => "安全起点：先关一号铜阀；蒸汽口和炉心墙都是实物阻挡，不是装饰。",
            _ => "安全起点：沿着地面灯带寻找第一个任务点。",
        };
        self.set_event(if cleared {
            format!("复访{}。{} 本次不再重复领取世界奖励。", world_label, note)
        } else {
            format!("进入{}。{}", world_label, note)
        });
    }

    fn current_world(&self) -> Option<&str> {
        self.current_world_id.as_deref()
    }

    fn update_nearby_and_reveal(&mut self) {
        let radius = exploration_radius(self);
        if let Some(map) = self.map.as_mut() {
            reveal(map, map.px, map.py, radius);
            map.nearby.clear();
            map.revision += 1;
        }
    }

    fn enemy_at(&self, x: i32, y: i32) -> Option<usize> {
        self.map
            .as_ref()?
            .enemies
            .iter()
            .position(|e| e.alive && e.x == x && e.y == y)
    }

    fn award_enemy_scrap(&mut self, count: i32) {
        if self.run_reward_allowed && count > 0 {
            self.run_scrap += count * 6;
        }
    }

    fn damage_enemy(&mut self, index: usize, amount: i32) -> bool {
        let mut killed = false;
        if let Some(map) = self.map.as_mut() {
            if let Some(enemy) = map.enemies.get_mut(index) {
                if enemy.alive {
                    enemy.hp -= amount;
                    if enemy.hp <= 0 {
                        enemy.hp = 0;
                        enemy.alive = false;
                        enemy.telegraph = None;
                        killed = true;
                    }
                }
            }
            map.revision += 1;
        }
        if killed {
            self.enemies_defeated += 1;
            self.award_enemy_scrap(1);
        }
        killed
    }

    fn apply_damage(&mut self, mut amount: i32, source: &str) {
        if amount <= 0 || self.mode != "world" {
            return;
        }
        if self.shield > 0 {
            let absorbed = self.shield.min(amount);
            self.shield -= absorbed;
            amount -= absorbed;
        }
        if amount > 0 {
            self.player.hp -= amount;
            self.run_damage_taken += amount;
        }
        self.effect = Some(EffectState {
            kind: "damage".into(),
            x: self.map.as_ref().map(|m| m.px).unwrap_or(0),
            y: self.map.as_ref().map(|m| m.py).unwrap_or(0),
            dx: 0,
            dy: 0,
        });
        if self.player.hp <= 0 {
            self.player.hp = 0;
            self.mode = "dead".into();
            self.dialog = None;
            self.result = None;
            self.set_event(format!(
                "{}击中了你。岑遥倒下了；可以重试本局，或回到归航站。",
                source
            ));
        } else {
            self.set_event(format!("{}擦过护甲，损失 {} 点生命。", source, amount));
        }
    }

    fn progress_tick(&mut self) {
        if self.mode != "world" {
            return;
        }
        self.run_ticks += 1;
        self.player.energy = (self.player.energy + 1).min(self.player.max_energy);
        if self.attack_cooldown > 0 {
            self.attack_cooldown -= 1;
        }
        for skill in &mut self.skills {
            if skill.cooldown_left > 0 {
                skill.cooldown_left -= 1;
            }
            skill.ready = skill.cooldown_left <= 0;
        }

        let (px, py) = self.map.as_ref().map(|m| (m.px, m.py)).unwrap_or((0, 0));
        let mut pending_damage = 0;
        let mut pending_source = String::new();
        let run_ticks = self.run_ticks;
        let steam_active = self.current_world_id.as_deref() == Some(CLOCKWORKS);
        if steam_active {
            self.steam_phase = (self.steam_phase + 1) % 6;
        }
        let steam_phase = self.steam_phase;
        let spawn_safe = self.spawn_protected && distance_sq(px, py, 3, 3) <= 9;
        if let Some(map) = self.map.as_mut() {
            for index in 0..map.enemies.len() {
                if !map.enemies[index].alive {
                    continue;
                }
                if let Some(mut telegraph) = map.enemies[index].telegraph.take() {
                    telegraph.remaining -= 1;
                    if telegraph.remaining <= 0 {
                        if distance_sq(px, py, telegraph.x, telegraph.y)
                            <= telegraph.radius * telegraph.radius
                        {
                            pending_damage += if map.enemies[index].kind == "elite" {
                                14
                            } else {
                                9
                            };
                            pending_source = map.enemies[index].name.clone();
                        }
                    } else {
                        map.enemies[index].telegraph = Some(telegraph);
                    }
                } else {
                    let (ex, ey) = (map.enemies[index].x, map.enemies[index].y);
                    let kind = map.enemies[index].kind.clone();
                    let dist = distance_sq(px, py, ex, ey);
                    let visible =
                        !spawn_safe && dist <= 64 && has_line_of_sight(map, ex, ey, px, py);
                    let melee = dist <= 2 && has_line_of_sight(map, ex, ey, px, py);
                    if kind == "chaser" && visible && melee {
                        // 追击者只有真正贴身（<=1.5 格）才出攻击预警。
                        map.enemies[index].telegraph = Some(TelegraphState {
                            x: px,
                            y: py,
                            radius: 0,
                            remaining: 2,
                        });
                    } else if kind == "elite" && visible && dist <= 16 {
                        // 精英能覆盖 3~4 格，但墙/锁门/水面会完全截断视线。
                        map.enemies[index].telegraph = Some(TelegraphState {
                            x: px,
                            y: py,
                            radius: 1,
                            remaining: 2,
                        });
                    } else if kind == "chaser" && visible && dist > 2 && run_ticks % 2 == 0 {
                        let dx = sign(px - ex);
                        let dy = sign(py - ey);
                        if can_step(map, ex, ey, dx, dy) {
                            map.enemies[index].x += dx;
                            map.enemies[index].y += dy;
                        } else if dx != 0 && can_step(map, ex, ey, dx, 0) {
                            map.enemies[index].x += dx;
                        } else if dy != 0 && can_step(map, ex, ey, 0, dy) {
                            map.enemies[index].y += dy;
                        }
                    }
                }
                map.enemies[index].attack_clock = (map.enemies[index].attack_clock + 1) % 8;
            }
            if steam_active {
                for zone in &mut map.zones {
                    if !zone.active || !zone.kind.starts_with("steam_vent_") {
                        continue;
                    }
                    if let Some(mut telegraph) = zone.telegraph.take() {
                        telegraph.remaining -= 1;
                        if telegraph.remaining <= 0 {
                            if distance_sq(px, py, telegraph.x, telegraph.y)
                                <= telegraph.radius * telegraph.radius
                            {
                                pending_damage += 12;
                                pending_source = if pending_source.is_empty() {
                                    "蒸汽喷口".into()
                                } else {
                                    format!("{}与蒸汽喷口", pending_source)
                                };
                            }
                        } else {
                            zone.telegraph = Some(telegraph);
                        }
                    } else if steam_phase == 1 {
                        zone.telegraph = Some(TelegraphState {
                            x: zone.x,
                            y: zone.y,
                            radius: 1,
                            remaining: 2,
                        });
                    }
                }
            }
            map.revision += 1;
        }
        self.update_nearby_and_reveal();
        if pending_damage > 0 {
            self.apply_damage(pending_damage, &pending_source);
        }
    }

    fn move_player(&mut self, args: &Value) -> Value {
        if self.mode != "world" {
            return self.invalid("当前状态不能移动。");
        }
        let Some((dx, dy)) = parse_direction(args, false) else {
            return self.invalid("移动需要 dx/dy，范围为 -1..1 且不能同时为 0。");
        };
        let (new_x, new_y, left_spawn) = {
            let Some(map) = self.map.as_mut() else {
                return self.invalid("当前世界地图不存在。");
            };
            let (x, y) = (map.px, map.py);
            if !can_step(map, x, y, dx, dy) {
                return self.invalid("前方被墙、锁门或水面挡住了。");
            }
            map.px += dx;
            map.py += dy;
            map.revision += 1;
            (map.px, map.py, distance_sq(map.px, map.py, 3, 3) > 9)
        };
        if left_spawn {
            self.spawn_protected = false;
        }
        self.player.energy = (self.player.energy - 1).max(0);
        self.effect = Some(EffectState {
            kind: "move".into(),
            x: new_x,
            y: new_y,
            dx,
            dy,
        });
        self.update_nearby_and_reveal();
        view(self)
    }

    fn dash_player(&mut self, args: &Value) -> Value {
        if self.mode != "world" {
            return self.invalid("当前状态不能闪避。");
        }
        let Some((dx, dy)) = parse_direction(args, false) else {
            return self.invalid("闪避需要一个八方向 dx/dy。");
        };
        if self.player.energy < 6 {
            return self.invalid("能量不足，无法闪避。");
        }
        let (new_x, new_y, moved, left_spawn) = {
            let Some(map) = self.map.as_mut() else {
                return self.invalid("当前世界地图不存在。");
            };
            let mut moved = 0;
            for _ in 0..2 {
                if can_step(map, map.px, map.py, dx, dy) {
                    map.px += dx;
                    map.py += dy;
                    moved += 1;
                } else {
                    break;
                }
            }
            if moved > 0 {
                map.revision += 1;
            }
            (map.px, map.py, moved, distance_sq(map.px, map.py, 3, 3) > 9)
        };
        if moved == 0 {
            return self.invalid("闪避方向没有可落脚的格子。");
        }
        if left_spawn {
            self.spawn_protected = false;
        }
        self.player.energy -= 6;
        self.effect = Some(EffectState {
            kind: "dash".into(),
            x: new_x,
            y: new_y,
            dx,
            dy,
        });
        self.update_nearby_and_reveal();
        view(self)
    }

    fn attack(&mut self, args: &Value) -> Value {
        if self.mode != "world" {
            return self.invalid("当前状态不能攻击。");
        }
        let Some((dx, dy)) = parse_direction(args, true) else {
            return self.invalid("近战需要一个 1~2 格方向 dx/dy。");
        };
        if self.attack_cooldown > 0 {
            return self.invalid("近战动作还在收势；请等 tick 再出刀。");
        }
        let (px, py) = self.map.as_ref().map(|m| (m.px, m.py)).unwrap_or((0, 0));
        let mut target = None;
        for distance in 1..=2 {
            if let Some(index) = self.enemy_at(px + dx * distance, py + dy * distance) {
                target = Some(index);
                break;
            }
        }
        let Some(index) = target else {
            return self.invalid("挥空了；近战只影响方向上 1~2 格内的敌人。");
        };
        let damage = if self.player.build == "vanguard" {
            30
        } else {
            24
        };
        let (x, y) = self
            .map
            .as_ref()
            .map(|m| (m.enemies[index].x, m.enemies[index].y))
            .unwrap_or((px + dx, py + dy));
        let killed = self.damage_enemy(index, damage);
        self.attack_cooldown = 2;
        self.effect = Some(EffectState {
            kind: if killed { "defeat" } else { "attack" }.into(),
            x,
            y,
            dx,
            dy,
        });
        self.set_event(if killed {
            "近战命中，追击者的轮廓熄灭了。"
        } else {
            "近战命中。"
        });
        view(self)
    }

    fn skill(&mut self, args: &Value) -> Value {
        if self.mode != "world" {
            return self.invalid("当前状态不能释放技能。");
        }
        let Some(slot) = parse_i32(args, "slot").and_then(|n| usize::try_from(n).ok()) else {
            return self.invalid("技能 slot 只能是 0、1、2。");
        };
        if slot >= self.skills.len() {
            return self.invalid("技能 slot 只能是 0、1、2。");
        }
        if !self
            .exchanges
            .iter()
            .any(|item| item.id == self.skills[slot].id && item.owned)
        {
            return self.invalid("该技能尚未兑换。");
        }
        let (cost, cooldown, cooldown_left) = {
            let skill = &self.skills[slot];
            (skill.cost, skill.cooldown, skill.cooldown_left)
        };
        if cooldown_left > 0 {
            return self.invalid("技能仍在冷却中。");
        }
        if self.player.energy < cost {
            return self.invalid("能量不足。");
        }
        let (px, py) = self.map.as_ref().map(|m| (m.px, m.py)).unwrap_or((0, 0));
        let mut hits = 0;
        let mut target_pos = (px, py);
        match slot {
            0 => {
                let targets: Vec<usize> = self
                    .map
                    .as_ref()
                    .map(|map| {
                        map.enemies
                            .iter()
                            .enumerate()
                            .filter(|(_, e)| e.alive && distance_sq(px, py, e.x, e.y) <= 6)
                            .map(|(i, _)| i)
                            .collect()
                    })
                    .unwrap_or_default();
                if targets.is_empty() {
                    return self.invalid("脉冲没有触及敌人。");
                }
                for index in targets {
                    if self.damage_enemy(index, 36) {
                        hits += 1;
                    } else {
                        hits += 1;
                    }
                }
                target_pos = (px, py);
            }
            1 => {
                if self.shield >= 30 && self.player.hp >= self.player.max_hp {
                    return self.invalid("护盾与生命都已充满。");
                }
                self.shield = 30;
                if self.player.hp < self.player.max_hp {
                    self.player.hp = (self.player.hp + 18).min(self.player.max_hp);
                }
                hits = 1;
                self.set_event("护盾展开，暖白的边缘把雾推开了一寸。");
            }
            2 => {
                let Some((dx, dy)) = parse_direction(args, true) else {
                    return self.invalid("穿刺需要一个八方向 dx/dy。");
                };
                for distance in 1..=3 {
                    if let Some(index) = self.enemy_at(px + dx * distance, py + dy * distance) {
                        target_pos = (px + dx * distance, py + dy * distance);
                        self.damage_enemy(index, 58);
                        hits = 1;
                        break;
                    }
                }
                if hits == 0 {
                    return self.invalid("穿刺方向上没有目标。");
                }
            }
            _ => return self.invalid("未知技能。"),
        }
        self.player.energy -= cost;
        self.skills[slot].cooldown_left = cooldown;
        self.skills[slot].ready = false;
        self.effect = Some(EffectState {
            kind: format!("skill_{slot}"),
            x: target_pos.0,
            y: target_pos.1,
            dx: parse_i32(args, "dx").unwrap_or(0),
            dy: parse_i32(args, "dy").unwrap_or(0),
        });
        if slot != 1 {
            self.set_event(format!("{}命中 {} 个目标。", self.skills[slot].name, hits));
        }
        view(self)
    }

    fn heal(&mut self) -> Value {
        if self.mode != "world" {
            return self.invalid("当前状态不能使用医疗包。");
        }
        if self.player.medkits <= 0 {
            return self.invalid("医疗包已经用完。");
        }
        if self.player.hp >= self.player.max_hp {
            return self.invalid("生命值已满，先把医疗包留给真正需要的时候。");
        }
        self.player.medkits -= 1;
        self.player.hp = (self.player.hp + 38).min(self.player.max_hp);
        self.effect = Some(EffectState {
            kind: "heal".into(),
            x: self.map.as_ref().map(|m| m.px).unwrap_or(0),
            y: self.map.as_ref().map(|m| m.py).unwrap_or(0),
            dx: 0,
            dy: 0,
        });
        self.set_event("医疗包打开了，生命线暂时稳住。");
        view(self)
    }

    fn complete_main_point(&mut self, point_id: &str) -> bool {
        let Some(world_id) = self.current_world().map(str::to_owned) else {
            return false;
        };
        let Some(objective_index) = self
            .objectives
            .iter()
            .position(|o| o.id == point_id && !o.optional)
        else {
            return false;
        };
        if self.objectives[objective_index].done {
            return false;
        }
        if self
            .objectives
            .iter()
            .enumerate()
            .any(|(i, o)| !o.optional && i < objective_index && !o.done)
        {
            return false;
        }
        // 每张图的最后一个主目标由末段守卫看守；普通追击者不会阻止探索，
        // 但这一名精英要先被处理，才会把「目标已完成」写入状态。
        if objective_index == 2 {
            let final_guard_alive = self
                .map
                .as_ref()
                .map(|map| {
                    map.enemies
                        .iter()
                        .any(|e| e.alive && e.kind == "elite" && e.id.contains("sentinel"))
                })
                .unwrap_or(false);
            if final_guard_alive {
                return false;
            }
        }
        self.objectives[objective_index].done = true;
        let mut event = String::new();
        let flag = match point_id {
            "hive_power" | "hive_lockdown" | "hive_rescue" => point_id,
            "harbor_beacon_a" | "harbor_beacon_b" | "harbor_signal" => point_id,
            "clock_valve_a" | "clock_valve_b" | "clock_core" => point_id,
            _ => "",
        };
        if !flag.is_empty() {
            flag_add(self, flag);
        }
        if let Some(map) = self.map.as_mut() {
            if let Some(point) = map.points.iter_mut().find(|p| p.id == point_id) {
                point.done = true;
            }
            let (gate_a, gate_b) = gate_ids(&world_id);
            match point_id {
                "hive_power" => {
                    open_gate(map, gate_a);
                    event = "主电恢复。暖黄灯一盏接一盏亮起，隔离门 A 解锁。".into();
                }
                "hive_lockdown" => {
                    open_gate(map, gate_b);
                    event = "隔离锁解除。门后的冷气终于不再像一堵墙。".into();
                }
                "hive_rescue" => {
                    if let Some(portal) = map.portals.iter_mut().find(|p| p.id == "exit") {
                        portal.active = true;
                    }
                    event = "救援信标握在手里，东侧出口收到回应。".into();
                }
                "harbor_beacon_a" => {
                    open_gate(map, gate_a);
                    event = "北堤信标亮起。雾退到脚边，隔离门 A 解锁。".into();
                }
                "harbor_beacon_b" => {
                    open_gate(map, gate_b);
                    set_map_tile(map, 23, 19, b'.');
                    set_zone_active(map, "water_fog", false);
                    event = "第二束信号对准了第一束光，水面上生成一条安全通道。".into();
                }
                "harbor_signal" => {
                    if let Some(portal) = map.portals.iter_mut().find(|p| p.id == "exit") {
                        portal.active = true;
                    }
                    event = "三次短鸣从雾外回来。救援坐标已发送，潮闸出口开放。".into();
                }
                "clock_valve_a" => {
                    open_gate(map, gate_a);
                    set_map_tile(map, 6, 4, b'.');
                    set_zone_active(map, "steam_vent_a", false);
                    event = "一号铜阀停转。第一段蒸汽沉默，隔离门 A 解锁。".into();
                }
                "clock_valve_b" => {
                    open_gate(map, gate_b);
                    set_map_tile(map, 24, 11, b'.');
                    set_zone_active(map, "steam_vent_b", false);
                    event = "二号高压阀关死。蒸汽喷口沉默，炉心前的通路终于安全。".into();
                }
                "clock_core" => {
                    if let Some(portal) = map.portals.iter_mut().find(|p| p.id == "exit") {
                        portal.active = true;
                    }
                    event = "停机回路脱离炉心，整座工厂第一次真正安静下来。".into();
                }
                _ => return false,
            }
            map.revision += 1;
        }
        if self.run_reward_allowed {
            self.run_scrap += 8;
        }
        self.set_event(event);
        true
    }

    fn collect_supply(&mut self, point_id: &str, x: i32, y: i32) -> Value {
        let already_claimed = self.supply_claimed.contains(point_id);
        self.supply_claimed.insert(point_id.to_owned());
        if let Some(map) = self.map.as_mut() {
            if let Some(point) = map.points.iter_mut().find(|p| p.id == point_id) {
                point.done = true;
            }
            map.revision += 1;
        }
        let first_run = self.run_reward_allowed && !already_claimed;
        let (scrap, medkits) = if point_id.ends_with("_a") {
            (10, 1)
        } else {
            (8, 1)
        };
        if first_run {
            self.run_scrap += scrap;
            self.player.medkits += medkits;
            self.set_event(format!(
                "打开补给箱：+{} 本局废料，+{} 医疗包。",
                scrap, medkits
            ));
        } else {
            self.set_event("补给箱已经被上一轮取空，回访不产生新的 meta 资源。");
        }
        self.effect = Some(EffectState {
            kind: "supply".into(),
            x,
            y,
            dx: 0,
            dy: 0,
        });
        view(self)
    }

    fn interact(&mut self, args: &Value) -> Value {
        if self.mode != "world" {
            return self.invalid("当前状态不能交互。");
        }
        let Some(id) = parse_string(args, "id") else {
            return self.invalid("交互需要对象 id。");
        };
        let Some(map) = self.map.as_ref() else {
            return self.invalid("当前世界地图不存在。");
        };
        let Some((x, y)) = object_position(map, &id) else {
            return self.invalid("附近没有这个交互对象。");
        };
        if !in_interact_range(map.px, map.py, x, y) {
            return self.invalid("距离交互对象太远，需要在 1.5 格以内。");
        }
        let Some(world_id) = self.current_world().map(str::to_owned) else {
            return self.invalid("当前世界不存在。");
        };

        if id == optional_npc_id(&world_id) {
            if self.optional_choice.is_some() {
                return self.invalid("这段对话已经做出选择。");
            }
            self.dialog = Some(optional_dialog(&world_id));
            self.mode = "dialog".into();
            self.set_event("对话停在选择之前。没有 tick，等你自己决定。");
            return view(self);
        }
        if id == "exit" {
            let active = self
                .map
                .as_ref()
                .and_then(|m| m.portals.iter().find(|p| p.id == "exit").map(|p| p.active))
                .unwrap_or(false);
            if !active || !main_objectives_done(self) {
                return self.invalid("出口还没有收到完整的归航信号。");
            }
            return self.extract();
        }
        if let Some(locked) = self.map.as_ref().and_then(|m| gate_locked(m, &id)) {
            if locked {
                return self.invalid("门还锁着；先完成它前面的主目标。");
            }
            return self.invalid("门已经开启。");
        }
        let Some((point_x, point_y, point_kind, point_done)) = self.map.as_ref().and_then(|m| {
            m.points
                .iter()
                .find(|p| p.id == id)
                .map(|p| (p.x, p.y, p.kind.clone(), p.done))
        }) else {
            return self.invalid("这个对象没有可用的交互动作。");
        };
        if point_done {
            return self.invalid("这个任务点已经完成。");
        }
        if point_kind == "supply" {
            return self.collect_supply(&id, point_x, point_y);
        }
        if !self.complete_main_point(&id) {
            return self.invalid("主任务必须按地图上的顺序推进。");
        }
        self.effect = Some(EffectState {
            kind: "interact".into(),
            x: point_x,
            y: point_y,
            dx: 0,
            dy: 0,
        });
        self.update_nearby_and_reveal();
        view(self)
    }

    fn choose(&mut self, args: &Value) -> Value {
        if self.mode != "dialog" {
            return self.invalid("当前没有等待选择的对话。");
        }
        let option_id = parse_string(args, "option_id").or_else(|| parse_string(args, "id"));
        let Some(option_id) = option_id else {
            return self.invalid("choice 需要 option_id。");
        };
        let valid = self
            .dialog
            .as_ref()
            .map(|d| d.choices.iter().any(|c| c.id == option_id))
            .unwrap_or(false);
        if !valid {
            return self.invalid("这个选项不属于当前对话。");
        }
        let Some(world_id) = self.current_world().map(str::to_owned) else {
            return self.invalid("对话没有所属世界。");
        };
        let positive = choice_is_positive(&world_id, &option_id);
        self.optional_choice = Some(option_id.clone());
        self.decisions.insert(world_id.clone(), option_id.clone());
        if let Some(objective) = self
            .objectives
            .iter_mut()
            .find(|o| o.id == optional_objective_id(&world_id))
        {
            objective.done = true;
        }
        self.dialog = None;
        self.mode = "world".into();
        flag_add(self, &option_id);
        if positive && self.run_reward_allowed {
            self.run_scrap += 10;
        }
        if let Some(map) = self.map.as_mut() {
            map.revision += 1;
        }
        self.set_event(if positive {
            "选择已记录。有人会因为这份让渡多走一段路。"
        } else {
            "选择已记录。轻装确实更快，但结果会留下空白。"
        });
        view(self)
    }

    fn ending_summary(&self) -> String {
        let mut rescued = Vec::new();
        let mut records = Vec::new();
        match self.decisions.get(GREY_HIVE).map(String::as_str) {
            Some("hive_help_baizhi") => {
                rescued.push("白芷");
                records.push("灰巢样本");
            }
            Some("hive_leave_sample") => records.push("灰巢电文"),
            _ => records.push("未确认的灰巢记录"),
        }
        match self.decisions.get(MIST_HARBOR).map(String::as_str) {
            Some("harbor_save_fuel") => rescued.push("陆岑与返航船队"),
            Some("harbor_burn_fuel") => records.push("精确港口坐标"),
            _ => records.push("未校准的港口信号"),
        }
        match self.decisions.get(CLOCKWORKS).map(String::as_str) {
            Some("clock_free_mengque") => rescued.push("孟阙"),
            Some("clock_take_core") => records.push("停机回路"),
            _ => records.push("未归档的工厂回路"),
        }
        let rescue_text = if rescued.is_empty() {
            "无人被带回".to_owned()
        } else {
            rescued.join("、")
        };
        format!(
            "三界终幕：救援 {}；保全资料 {}。归航站留下的不是同一条答案。",
            rescue_text,
            records.join("、")
        )
    }

    fn extract(&mut self) -> Value {
        if self.mode != "world" {
            return self.invalid("当前状态不能撤离。");
        }
        if !main_objectives_done(self) {
            return self.invalid("三个主目标尚未全部完成。");
        }
        let Some((portal_active, portal_x, portal_y, player_x, player_y)) =
            self.map.as_ref().and_then(|map| {
                map.portals
                    .iter()
                    .find(|p| p.id == "exit")
                    .map(|p| (p.active, p.x, p.y, map.px, map.py))
            })
        else {
            return self.invalid("出口不存在。");
        };
        if !portal_active || !in_interact_range(player_x, player_y, portal_x, portal_y) {
            return self.invalid("需要站在已激活出口附近才能撤离。");
        }
        let Some(world_id) = self.current_world().map(str::to_owned) else {
            return self.invalid("当前世界不存在。");
        };
        let world_name = world_name(&world_id).to_owned();
        let first_clear = !world_is_cleared(self, &world_id);
        let optional_done = self.objectives.iter().any(|o| o.optional && o.done);
        let positive = self
            .optional_choice
            .as_deref()
            .map(|choice| optional_positive(&world_id, choice))
            .unwrap_or(false);
        let base_reward = match world_id.as_str() {
            GREY_HIVE => 42,
            MIST_HARBOR => 48,
            CLOCKWORKS => 54,
            _ => 0,
        };
        let reward = if first_clear {
            base_reward + self.run_scrap + if positive { 10 } else { 0 }
        } else {
            0
        };
        let token_award = if first_clear {
            1 + i32::from(positive)
        } else {
            0
        };
        if first_clear {
            if let Some(index) = world_index(self, &world_id) {
                self.worlds[index].cleared = true;
            }
            self.player.reward_points += reward;
            self.player.side_story_tokens += 1;
            if positive {
                self.player.side_story_tokens += 1;
            }
            self.spoils_claimed.push(world_id.clone());
        }
        let all_complete = self.worlds.iter().all(|w| w.cleared);
        let rank = if !first_clear {
            "C"
        } else if all_complete && optional_done && self.run_damage_taken <= 20 {
            "S"
        } else if optional_done {
            "A"
        } else {
            "B"
        };
        let summary = if all_complete {
            self.ending_summary()
        } else if first_clear {
            format!(
                "{}的灯光留在身后。主目标已完成，归航站又多了一条可用的回路。",
                world_name
            )
        } else {
            format!("复访{}结束；这次没有新的世界奖励。", world_name)
        };
        let stats = json!({
            "objectives_done": self.objectives.iter().filter(|o| o.done).count(),
            "optional_done": optional_done,
            "optional_choice": self.optional_choice,
            "enemies_defeated": self.enemies_defeated,
            "explored": self.map.as_ref().map(|m| m.explored.len()).unwrap_or(0),
            "damage_taken": self.run_damage_taken,
            "mechanic": world_catalog().into_iter().find(|w| w.id == world_id).map(|w| w.mechanic).unwrap_or_default(),
            "replay": !first_clear,
            "decisions": self.decisions,
        });
        self.result = Some(ResultState {
            title: if all_complete {
                "无限人族进化 · 共同终幕".into()
            } else {
                "任务完成 · 返回归航站".into()
            },
            world_name,
            summary,
            rank: rank.into(),
            reward,
            side_story_tokens: token_award,
            all_complete,
            stats,
        });
        self.mode = "result".into();
        self.map = None;
        self.dialog = None;
        self.run_reward_allowed = false;
        self.set_event(if all_complete {
            "三世界全部完成。共同终幕已解锁。"
        } else {
            "撤离成功。奖励已经写入归航站资源。"
        });
        self.effect = Some(EffectState {
            kind: "extract".into(),
            x: portal_x,
            y: portal_y,
            dx: 0,
            dy: 0,
        });
        view(self)
    }

    fn retreat(&mut self) -> Value {
        if self.mode != "world" && self.mode != "dialog" {
            return self.invalid("当前没有可以放弃的任务。");
        }
        let Some(world_id) = self.current_world().map(str::to_owned) else {
            return self.invalid("当前任务不存在。");
        };
        let recovered = if self.run_reward_allowed {
            self.run_scrap / 2
        } else {
            0
        };
        if recovered > 0 {
            self.player.reward_points += recovered;
        }
        if !self.spoils_claimed.iter().any(|id| id == &world_id) {
            self.spoils_claimed.push(world_id.clone());
        }
        self.mode = "hub".into();
        self.map = None;
        self.objectives.clear();
        self.dialog = None;
        self.result = None;
        self.current_world_id = None;
        self.flags.clear();
        self.run_reward_allowed = false;
        self.run_scrap = 0;
        self.optional_choice = None;
        self.shield = 0;
        self.player.hp = self.player.max_hp;
        self.player.energy = self.player.max_energy;
        self.set_event(if recovered > 0 {
            format!(
                "已撤回{}，保留 {} 枚探索废料；本局物资不会重复刷出。",
                world_name(&world_id),
                recovered
            )
        } else {
            format!(
                "已撤回{}。本局未结算的探索物资留在原地。",
                world_name(&world_id)
            )
        });
        view(self)
    }

    fn continue_to_hub(&mut self) -> Value {
        if self.mode != "result" && self.mode != "dead" {
            return self.invalid("只有结算或死亡状态可以回到归航站。");
        }
        let complete = self
            .result
            .as_ref()
            .map(|r| r.all_complete)
            .unwrap_or(false);
        self.mode = "hub".into();
        self.map = None;
        self.objectives.clear();
        self.dialog = None;
        self.result = None;
        self.current_world_id = None;
        self.flags.clear();
        self.run_reward_allowed = false;
        self.run_scrap = 0;
        self.optional_choice = None;
        self.shield = 0;
        self.player.hp = self.player.max_hp;
        self.player.energy = self.player.max_energy;
        self.set_event(if complete {
            "共同终幕已经点亮。归航站现在可以自由复访三座世界。"
        } else {
            "回到归航站。选择下一座世界，或先用奖励点数兑换条目。"
        });
        view(self)
    }

    fn restart(&mut self) -> Value {
        if self.mode != "dead" && self.mode != "result" {
            return self.invalid("只有死亡或结算状态可以重启任务。");
        }
        let Some(world_id) = self.current_world_id.clone() else {
            return self.invalid("没有可重启的任务世界。");
        };
        self.start_world(&world_id);
        self.set_event(format!(
            "重新任务：{}。已兑换项目、世界进度和已结算资源都已保留。",
            world_name(&world_id)
        ));
        view(self)
    }

    fn exchange(&mut self, args: &Value) -> Value {
        if self.mode != "hub" {
            return self.invalid("兑换只能在归航站进行。");
        }
        let Some(id) = parse_string(args, "id") else {
            return self.invalid("exchange 需要 id。");
        };
        let canonical = match id.as_str() {
            "armor" | "reinforced" | "reinforced_suit" => "reinforced_suit",
            "energy" | "battery" | "capacitor" => "capacitor",
            "medkit" | "medkits" | "field_medkit" => "field_medkit",
            "detect" | "lens" | "signal_lens" => "signal_lens",
            _ => id.as_str(),
        };
        let Some(index) = self.exchanges.iter().position(|u| u.id == canonical) else {
            return self.invalid("不存在这个兑换条目。");
        };
        let exchange = self.exchanges[index].clone();
        if exchange.owned {
            return self.invalid("这个条目已经兑换，不能重复兑换。");
        }
        if self.player.reward_points < exchange.reward_cost
            || self.player.side_story_tokens < exchange.token_cost
        {
            return self.invalid("奖励点数或支线凭证不足。");
        }
        self.player.reward_points -= exchange.reward_cost;
        self.player.side_story_tokens -= exchange.token_cost;
        self.exchanges[index].owned = true;
        match canonical {
            "field_medkit" => self.player.medkits += 1,
            _ => {
                let old_hp = self.player.hp;
                let old_energy = self.player.energy;
                self.recompute_capacity();
                if canonical == "reinforced_suit" {
                    self.player.hp = (old_hp + 12).min(self.player.max_hp);
                }
                if canonical == "capacitor" {
                    self.player.energy = (old_energy + 12).min(self.player.max_energy);
                }
            }
        }
        self.set_event(format!(
            "已兑换{}级{}：{}。",
            exchange.grade, exchange.category, exchange.name
        ));
        view(self)
    }

    fn equip(&mut self, args: &Value) -> Value {
        if self.mode != "hub" {
            return self.invalid("战斗配置只能在归航站切换。");
        }
        let Some(id) = parse_string(args, "id") else {
            return self.invalid("equip 需要 id。");
        };
        let build = match id.as_str() {
            "vanguard" | "build_vanguard" | "先锋" | "bulwark" => "vanguard",
            "echo" | "build_echo" | "scout" | "recon" | "回声" | "侦察" => "echo",
            _ => return self.invalid("可选配置只有 vanguard 与 echo。"),
        };
        let lineage_id = if build == "vanguard" {
            "vanguard_lineage"
        } else {
            "echo_lineage"
        };
        if !self
            .exchanges
            .iter()
            .any(|item| item.id == lineage_id && item.owned)
        {
            return self.invalid("该血统尚未兑换。");
        }
        if self.player.build == build {
            return self.invalid("当前已经是这套配置。");
        }
        self.player.build = build.into();
        self.recompute_capacity();
        self.player.hp = self.player.max_hp;
        self.player.energy = self.player.max_energy;
        self.set_event(if build == "vanguard" {
            "装备先锋配置：更厚的生命层，近战伤害更高。"
        } else {
            "装备回声配置：更大的能量池，适合技能和探索。"
        });
        view(self)
    }
}

/// Apply one action and return a complete snapshot. Invalid actions only update
/// `last_event`; they never consume energy, advance cooldowns, move entities or
/// award resources.
pub fn act(state: &mut CampaignState, action: &str, args: &Value) -> Value {
    if action == "resume" {
        state.migrate_legacy();
        return view(state);
    }
    if action == "new" {
        *state = CampaignState::default();
        return view(state);
    }
    state.migrate_legacy();
    match action {
        "enter" => {
            if state.mode != "hub" {
                return state.invalid("只能从归航站选择世界。");
            }
            let Some(world_id) = parse_string(args, "world_id").or_else(|| parse_string(args, "id")) else {
                return state.invalid("enter 需要 world_id。");
            };
            if !state.worlds.iter().any(|w| w.id == world_id && w.available) {
                return state.invalid("这座世界不可用。");
            }
            state.effect = None;
            state.start_world(&world_id);
            view(state)
        }
        "move" => state.move_player(args),
        "attack" => state.attack(args),
        "skill" => state.skill(args),
        "dash" => state.dash_player(args),
        "interact" => state.interact(args),
        "choice" => state.choose(args),
        "tick" => {
            if state.mode != "world" {
                return state.invalid("归航站、对话、结算和死亡状态不会推进 tick。");
            }
            state.effect = None;
            state.progress_tick();
            view(state)
        }
        "heal" => state.heal(),
        "extract" => state.extract(),
        "retreat" => state.retreat(),
        "upgrade" | "exchange" => state.exchange(args),
        "equip" => state.equip(args),
        "restart" => state.restart(),
        "continue" => state.continue_to_hub(),
        _ => state.invalid("未知动作；可用动作：new/enter/move/attack/skill/dash/interact/choice/tick/heal/extract/retreat/exchange/upgrade/equip/restart/continue/resume。"),
    }
}
