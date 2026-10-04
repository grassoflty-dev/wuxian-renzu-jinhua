《无限人族进化》V1 游戏玩法与工程实现设计规格

文档类型：Gameplay Design + Engineering Guidance
用途：指导 Codex / 工程实现 / 内容制作 / 测试验收
当前工程基线：release/v1-three-worlds-2d5 @ dc7d8c587bdb7784764de30a1b450d33a28e4c27
日期：2026-09-28
适用范围：V1 三世界首发 + Return Station + 长期无限流扩展基础
权威关系：本文件不替代已有三份总方案/视觉方案/UI 方案；在“玩法与工程实现细则”范围内作为新增统一规格。后续用户明确修改时，以最新明确条款覆盖本文件对应部分。

────────

0. 状态标记

为避免“设计存在”被误报为“游戏已完成”，本文统一使用以下状态：

• FROZEN：玩法方向/规则已冻结，工程必须按此实现，除非后续用户明确修改。
• IMPLEMENTED：当前正式开发线已有权威实现。
• PARTIAL：已有实现，但尚未形成完整可玩闭环。
• STAGED：地图/Marker/数据已登记，但玩法逻辑尚未完成。
• TARGET：V1 必须实现的目标规则。
• TUNE：机制冻结，但数值允许在测试后调整。
• FUTURE：只要求架构支持，V1 不必交付。

任何 registered / staged / web-tested / browser-rendered / native-launched / native-verified / release-approved 状态不得互相替代。

────────

1. 产品核心定义

《无限人族进化》是一款：

> **固定斜视角 2.5D、多世界、长期扩展的无限流 Action RPG。**

玩家从一个受普通人类身体、感知、信息、移动和认知限制的人开始，通过不同世界获得能力、道具、血统、技能、装备、法宝或其他强化，逐步改变自己与世界规则的关系。

核心不是传统“等级越来越高”，而是：

> **越来越多的人类限制被解除。**

典型成长：

• 看不见身后 → 后视感知
• 不知道敌人状态 → 敌人生命/弱点感知
• 没有地图 → 局部地图 → 远程测绘 → 全域地图
• 水会减速 → 悬浮/踏水
• 高温会伤害 → 热抗性
• 悬崖必须绕路 → Air Step → Flight
• 必须呼吸 → No Breath
• 背包有限 → 空间储物
• 只能看到当前威胁 → 危险预测
• 受物理边界约束 → 高阶空间/规则能力

────────

2. 游戏设计支柱

2.1 限制驱动成长

每个世界必须提出新的“普通人限制”。

世界不是简单换皮，而是：

1. 世界制造一种限制；
2. 玩家用当前构筑寻找解法；
3. 世界提供新的强化；
4. 新强化在未来世界产生新的解法；
5. 旧世界重访时出现新的路线/效率/信息优势。

2.2 信息本身就是成长

HUD 和世界信息不能默认全部开放。

例如：

• 未获得 Local Map → 不显示玩法小地图；
• 未获得 Enemy Vitals → 不显示敌人 HP；
• 未获得 Rear View → 不显示后方威胁视窗；
• 未获得特定探测能力 → 隐藏敌人/危险/秘密仍不可见。

Accessibility 永远例外：字幕、字号、色盲辅助、屏幕震动、闪光强度、按键重映射等不得能力门控。

2.3 世界规则优先于剧情空气墙

能力成长必须真实改变世界规则。

例如雾港泵站的深水区域：

• 普通 Ground 玩家必须排水；
• Hover / Water Walk / Flight 等未来效果可以合法绕过；
• 不应再叠一层“没有按泵所以空气墙禁止通行”。

剧情关键边界单独使用：

• boundary.quest
• boundary.world
• rule.sealed

不能与普通地形碰撞混用。

2.4 内容数量与底层效果数量解耦

长期目标可以拥有数百、上千个 Skills / Bloodlines / Items / Weapons / Gear / Treasures / Capabilities，但这些内容应组合有限数量的底层 EffectSpec，而不是每个内容写一段专用 Rust。

长期可维护性的衡量指标应是：

> “有多少基础效果类型”，而不是“有多少内容条目”。

────────

3. V1 战役结构

正式首发战役：

Return Station
    ↓
Grey Hive
    ↓
Return Station
    ↓
Mist Harbor
    ↓
Return Station
    ↓
Clockworks
    ↓
Return Station / V1 Epilogue
    ↓
World Network begins branching

前三世界不是三个独立 Demo，而是一条完整首发战役。

当前世界顺序固定：

1. grey_hive
2. mist_harbor
3. clockworks

Clockworks 完成后，世界网络才开始明确表现为分支结构。

────────

4. Return Station / 归航站

4.1 世界定位

归航站不是巨大天空城市或宇宙首都，而是：

> **人类在未知世界之间建立的小型技术中继站。**

首发版只需一个紧凑核心室：rs_core_room。

核心功能：

• World Gate
• Mission Terminal
• Capability Terminal
• Save / Rest Terminal
• Storage / Inventory
• 世界状态展示

4.2 Hub 功能循环

每次世界返回：

1. 记录世界完成状态；
2. 保存永久成长；
3. 处理能力/奖励；
4. 查看下一世界；
5. 调整装备/主动技能/构筑；
6. 继续下一个世界或有限重访。

4.3 World Gate

World Gate 只允许进入：

• 当前已解锁世界；
• 合法 Revisit 世界；
• 后续开放世界。

Rust RouteState / World Progression 是权威，前端不得自行解锁。

────────

5. 玩家基础操作

|输入    |行为                          |
|------|----------------------------|
|WASD  |固定斜视角下的屏幕直觉移动               |
|Mouse |Aim                         |
|左键 / J|Primary Attack              |
|Shift |Dash                        |
|Q     |Pulse                       |
|E     |Guard                       |
|R     |Pierce                      |
|F     |Interact                    |
|Space |Context Traversal           |
|T     |Terminal / Mission / Meta UI|
|M     |Map（仅在能力允许时具有玩法信息）          |
|Esc   |Pause                       |

相机固定，不允许 RMB 自由旋转。

────────

6. 基础战斗能力

6.1 Primary Attack

要求：明确 windup、短 active、明确 recovery；鼠标方向驱动 facing；命中反馈发生在接触点。

当前数值只作为 TUNE：

windup ≈ 90–180 ms
active ≈ 80–120 ms
recovery ≈ 150–280 ms

6.2 Dash

• Dash 开始时锁定方向；
• 有移动输入 → 使用移动方向；
• 无移动输入 → 使用 facing；
• Dash 中不被普通转向改变；
• Rust/KCC 权威碰撞；
• Dash 不是瞬移。

6.3 Pulse / Q

视觉语言：圆形、感知、扫描。

玩法：短距径向作用、轻伤害、较强 stagger，同时作为后续“扫描类能力”的基础动作。

6.4 Guard / E

视觉语言：弧、防御、稳定。

Guard 成立由 Rust 权威；命中产生局部反馈；未来可扩展 perfect timing。

6.5 Pierce / R

视觉语言：线、穿透、方向性。

直线定向攻击，可附带短突进；与 Pulse 的径向作用严格区分。

────────

7. 能力与成长分类

正式 UI 分类：

PERCEPTION
INFORMATION
BODY
MOBILITY
COGNITION
COMBAT
SPACE
SOUL
RULE
UTILITY

分类只负责设计组织/UI 分组/内容检索/叙事表达。真正玩法由统一 EffectSpec 决定。

────────

8. Unified Gameplay Effect System

8.1 所有强化的统一来源

Capability
Item
Equipment
Treasure
Skill
Bloodline
Gene
Cultivation
Temporary Buff
World Effect
Quest Effect
        ↓
EffectSource
        ↓
EffectResolver
        ↓
EffectivePlayerRules

禁止：

if item_id == "hover_boots" { ... }
if bloodline == "dragon" { ... }

8.2 EffectSource

enum EffectSourceKind {
    InnateCapability,
    Item,
    Equipment,
    Bloodline,
    Skill,
    Treasure,
    Gene,
    Cultivation,
    TemporaryBuff,
    WorldEffect,
    QuestEffect,
}

enum EffectLifetime {
    Permanent,
    Owned,
    Equipped,
    Selected,
    Activated,
    Timed { remaining_ms: u64 },
    Scene,
    World,
}

8.3 EffectSpec

至少支持：

enum EffectSpec {
    GrantCapability(...),
    Stat(...),
    Perception(...),
    MapKnowledge(...),
    Movement(...),
    Terrain(...),
    Hazard(...),
    Combat(...),
    Resource(...),
    Interaction(...),
    Inventory(...),
    Space(...),
    Rule(...),
}

V1 不必实现所有 Future 变体，但结构必须允许扩展。

8.4 EffectivePlayerRules

struct EffectivePlayerRules {
    stats: EffectiveStats,
    perception: PerceptionRules,
    map: MapKnowledgeRules,
    movement: MovementRules,
    terrain: TerrainRules,
    hazards: HazardRules,
    combat: CombatRules,
    interaction: InteractionRules,
    inventory: InventoryRules,
    space: SpaceRules,
    rule: RuleRules,
}

保存 Source，不保存 resolved rules。Load 后重新 Resolve。

────────

9. Map Knowledge / 地图知识系统

9.1 地图知识通道

地图不是 On/Off bool，至少拆为：

Topology
Terrain
Connections
Objectives
Enemies
Hazards
Loot
NPCs
Secrets

enum KnowledgeLevel {
    None,
    ExploredOnly,
    DetectedOnly,
    Known,
    Full,
}

9.2 Local Map I

典型规则：

Topology     = ExploredOnly
Terrain      = ExploredOnly
Connections  = ExploredOnly
Objectives   = Known
Enemies      = None
Hazards      = None
Secrets      = None

9.3 Full-map 类道具

例如“全域地形测绘仪”：

Topology     = Full
Terrain      = Full
Connections  = Full
Objectives   = Known
Enemies      = None
Secrets      = None

它不能自动把敌人、秘密、战利品全部揭露。

9.4 Explored State 与 Map Projection 分离

Rust 始终记录玩家真正探索过的区域。没有地图能力时仍记录但不显示；后来获得地图能力后显示过去真正探索的区域。远程测绘/全图能力只改变 Projection，不伪造 explored=true。

────────

10. Terrain / Movement / Hazard

10.1 Terrain Tags

terrain.normal
terrain.water_shallow
terrain.water_deep
terrain.mud
terrain.ice
terrain.rubble
terrain.hot_floor
terrain.toxic_pool
surface.conveyor
obstacle.low
obstacle.soft
obstacle.wall
obstacle.force_field
boundary.world
boundary.quest

10.2 Movement Modes

enum MovementMode {
    Ground,
    AirStep,
    Flight,
    Swim,
    Phase,
}

V1 正式重点：Ground、Context Traversal、AirStep。Flight / Swim / Phase 主要为未来留接口。

10.3 Hazard Tags

hazard.fire
hazard.heat
hazard.cold
hazard.toxin
hazard.radiation
hazard.electric
hazard.pressure
hazard.vacuum
hazard.drowning
hazard.sanity
hazard.soul

────────

11. 世界一：Grey Hive / 灰巢设施

11.1 世界定位

世界主题：生物研究/维护设施发生感染和生物安全事故，设施进入封锁。

视觉关键词：白灰工业、冷混凝土、暗钢、Cyan 系统灯、Amber 交互、Red 危险、污染和失效设备。

核心限制：信息不足、看不见身后、不知道敌人状态、恢复能力弱、地图未知、战斗经验不足。

这个世界负责建立：

> “玩家原本只是普通人”的基准。

11.2 世界流程

gh_entry_maintenance
→ gh_power_room
→ gh_gate_a
→ gh_central_shaft
→ gh_lockdown
→ gh_bio_isolation
→ gh_gate_b
→ gh_deep_decon
→ gh_sentinel_arena
→ gh_beacon
→ gh_exit

语义：

Entry → Power → Gate A → Central Shaft → Lockdown
→ Bai Zhi / Bio Isolation → Gate B → Deep Decon
→ Sentinel → Beacon → Exit

11.3 Required Events

hive_power
hive_lockdown
hive_extraction

11.4 场景设计

gh_entry_maintenance

• 教移动/基础攻击；
• 第一次遇到 Infected Maintenance Worker；
• tutorial terminal；
• checkpoint；
• 进入 Power Room。

gh_power_room

核心互动 gh_power_console，完成 hive_power。

玩家清理动力间并恢复主电；场景照明从紧急红/低照度切换到冷白/Cyan 工作灯；Gate A 后续合法开启。

gh_gate_a

验证 hive_power，教学“世界事件会改变门/通路”。Gate A 不是独立剧情目标，而是 Power 的结果。

gh_central_shaft

扩大空间尺度，混合维护工与感染安保，通过日志建立事故线索。

gh_lockdown

gh_lockdown_terminal 完成 hive_lockdown，解锁后续 Bio Isolation。

gh_bio_isolation

承载 Bai Zhi 支线。结果允许：

taken
left
unresolved

Bai Zhi 选择不得成为 Gate B 主线门锁。

gh_gate_b

由 hive_lockdown 授权，不检查 Bai Zhi 结果。

gh_deep_decon

当前已有 steam_jet、decon_mist 和两个 decon valve marker。

V1 目标：教学 Hazard Telegraph；Steam 是方向性短时危险；Mist 是区域环境压力；Valve 可作为局部降危险可选互动，不作为主线硬锁。

gh_sentinel_arena

Boss / Elite：enemy.grey_hive.sentinel。

正式形象：双足/类人型重型设施守卫。禁止回退到旧蜘蛛/多足概念。

Boss 强调：清晰 windup、重击、冲压/突进、stagger 窗口、Guard/Pierce 的实际价值。

gh_beacon

Beacon 是便携式任务装置，不是巨型塔。玩法目标：收取/部署/挂载或确认 Beacon；当前 marker 仍需正式逻辑化。

gh_exit

gh_exit_extraction_console 完成 hive_extraction，随后返回 Return Station。

11.5 Grey Hive 敌人生态

1. Infected Maintenance Worker：慢、短距、数量压力、教学基础移动/攻击。
2. Infected Security：更积极、更强 stagger/追击，迫使 Dash/Guard。
3. Swarm（TARGET）：群体单位，Pulse 价值高。
4. Brute（TARGET）：重体型，Pierce/Guard 价值高。
5. Sentinel：世界 Boss/精英核心。

11.6 Grey Hive 第一批能力

首次返回站点后的三选一：

Local Map
Rear View
Regeneration

Enemy Vitals 来源为 Scanner/扫描能力。第一等级只显示：

Healthy
Wounded
Severely Wounded
Critical

不能立刻显示精确 HP 数字。

11.7 Grey Hive 重访

当前：afterFirstClearLimited, maxRevisits = 2。

用途：新能力回到旧限制时产生新体验、Bai Zhi 后续、Scanner/Enemy Vitals、日志/资源。不能无限刷首通奖励。

────────

12. 世界二：Mist Harbor / 雾港余烬

12.1 世界定位

世界主题：被海雾、潮水、失效信号和异常共振吞没的工业港区。

视觉关键词：湿石、旧港设施、锈、海水、浓雾、暖灯、Cyan/Sea-green 信号。

核心限制：可见度、方向感、声音判断、信号干扰、水域移动、地图可靠性。

世界目标：

> 让玩家意识到“看得见”不是唯一感知方式。

12.2 正式九场景路线

mh_fog_pier
→ mh_tidal_warehouse
→ mh_signal_yard
→ mh_drowned_quay
→ mh_breakwater
→ mh_pump_station
→ mh_resonance_tower
→ mh_warden_arena
→ mh_extraction

12.3 Required Events

mist_beacon_west
mist_beacon_east
mist_signal

Pump 不作为 World Required Event。

12.4 地图基础规格

九图当前统一：

24 × 16 tiles
32 px/tile
32 px/m
24m × 16m

Tiled → World：

worldX = tiledX / 32
worldZ = tiledY / 32

V1 walkable 基础范围：

x = 0.5 .. 23.5
z = 0.5 .. 15.5

最终实际可行走：

Walkable
- Collision
- 当前玩家无法通过的 Terrain/Hazard

12.5 Exploration Regions

Rust 必须始终记录真实探索区域，即使玩家尚未获得地图能力。

mh_fog_pier

mh_fp_entry_berth          [0.5,0.5] → [8,15.5]
mh_fp_foghorn_pier         [8,0.5] → [16,15.5]
mh_fp_warehouse_approach   [16,0.5] → [23.5,15.5]

mh_tidal_warehouse

mh_tw_west_loading     [0.5,0.5] → [8.5,15.5]
mh_tw_storage_floor    [8.5,0.5] → [16.5,15.5]
mh_tw_beacon_bay       [16.5,0.5] → [23.5,15.5]

mh_signal_yard

mh_sy_west_yard            [0.5,0.5] → [13,15.5]
mh_sy_interference_field   [13,0.5] → [19,15.5]
mh_sy_quay_approach        [19,0.5] → [23.5,15.5]

mh_drowned_quay

mh_dq_west_quay        [0.5,0.5] → [13,15.5]
mh_dq_flooded_channel  [13,0.5] → [19,15.5]
mh_dq_east_quay        [19,0.5] → [23.5,15.5]

mh_breakwater

mh_bw_west_seawall    [0.5,0.5] → [8.5,15.5]
mh_bw_center_seawall  [8.5,0.5] → [16.5,15.5]
mh_bw_east_beacon     [16.5,0.5] → [23.5,15.5]

mh_pump_station

mh_ps_intake        [0.5,0.5] → [10.5,15.5]
mh_ps_control_hall  [10.5,0.5] → [18,15.5]
mh_ps_east_channel  [18,0.5] → [23.5,15.5]

mh_resonance_tower

mh_rt_entry              [0.5,0.5] → [9.5,15.5]
mh_rt_resonance_floor    [9.5,0.5] → [16,15.5]
mh_rt_signal_section     [16,0.5] → [23.5,15.5]

mh_warden_arena

mh_wa_entry       [0.5,0.5] → [8,15.5]
mh_wa_arena_core  [8,0.5] → [20.5,15.5]
mh_wa_exit        [20.5,0.5] → [23.5,15.5]

mh_extraction

mh_ex_arrival         [0.5,0.5] → [9,15.5]
mh_ex_extraction_pad  [9,0.5] → [19,15.5]
mh_ex_exit_section    [19,0.5] → [23.5,15.5]

12.6 探索判定

• 玩家中心进入 Region 至少 0.35m → explored；
• Spawn 所在 Region 进入场景即 explored；
• Save/Load 保留；
• Return Station 后保留；
• Revisit 保留；
• New Journey 清空。

Acoustic Mapping 不自动永久探索远处地图。

12.7 mh_fog_pier

体验：大雾、Foghorn 声源、基础 Drowned 威胁。玩家从声音、局部灯光和构图找到仓库方向。

12.8 mh_tidal_warehouse

核心：mh_west_beacon，完成 mist_beacon_west。

作用：第一座 Beacon，建立“信号源 = 导航锚点”；Drowned + Signal Wraith 组合首次出现。

12.9 mh_signal_yard

当前 Hazard：mh_signal_interference_region。

规则：进入干扰区时 HUD/信号信息产生有限扰动，但不破坏 Accessibility；声学信息价值提升；Signal Wraith 是主要敌人。

12.10 mh_drowned_quay

当前 Water Region：

mh_water_depth_region
x = 13..19m
z = 5..11m

Pump 未排水时：movement ×0.6（当前正式代码值）。未来 Hover / Water Walk / Flight 等 Terrain/Movement Effect 可以合法绕过该减速。

12.11 mh_breakwater

核心：mh_east_beacon，完成 mist_beacon_east。

完成后正常路线允许进入 Pump Station。该场景负责第二 Beacon、海堤开放空间、强化雾和 Signal Wraith 压力。

────────

13. Mist Harbor 泵站玩法

13.1 主泵控制台

当前静态 Marker：mh_pump_activation_static_marker。

正式转换：

id   = mh_pump_control_primary
kind = pump_control

坐标：

Tiled: (544,224)
World: (17.0, 0.0, 7.0)

交互：F，距离沿用 Scene Runtime 2.5m。

13.2 前置条件

必须：mist_beacon_east == complete。

正常路线进入 Pump Station 时已经满足，但 Rust 仍需二次验证。

13.3 Pump State Machine

READY
  ↓ F
DRAINING
  ↓ 6000 ms
DRAINED

6000ms 为 V1 冻结值。V1 不使用连续水位模拟，也不采用旧示例 targetWaterLevel = 0.25。

13.4 READY

• East Channel = deep water；
• 普通 Ground 玩家不可通过；
• Drowned Quay 水区仍 ×0.6；
• UI：[F] 启动排水泵。

13.5 DRAINING

• 持续 6000ms；
• 玩家可继续走动；
• 不可取消；
• 受击不取消；
• 不可重复启动；
• Rust authority 计时。

视觉：机械启动、管线震动、排水声、Amber → Cyan 指示灯。

13.6 DRAINED

• East Channel 可正常通过；
• Drowned Quay 不再减速；
• 再次交互仅显示完成状态；
• 不重复奖励。

13.7 Pump East Channel

正式 Region：mh_pump_east_channel_water。

[(18.0,5.5),
 (23.5,5.5),
 (23.5,10.5),
 (18.0,10.5)]

排水前：terrain.water_deep，普通 Ground blocked。
排水后：terrain.wet_floor，Movement = 1.0。

13.8 Pump 对 Drowned Quay 的影响

只影响 mh_water_depth_region。

排水前：movement ×0.6。
排水后：movement ×1.0。

视觉仍保留湿地/浅水。

13.9 Pump 影响范围

V1 只影响：

mh_pump_station / mh_pump_east_channel_water
mh_drowned_quay / mh_water_depth_region

不影响其余七个场景。

13.10 Pump 持久化

保留于：Save/Load、Scene transition、Return Station、Mist Harbor Revisit。New Journey 才重置。

DRAINING 中保存：

pumpState = draining
drainCompleteAtWorldTimeMs

Load 时按 Rust world time 恢复。

13.11 泵站捷径

V1 不做捷径。

mh_pump_shortcut_candidate_static_marker 只能保留为 authoring_only，不能成为 runtime shortcut。

────────

14. Mist Harbor 后半程

14.1 mh_resonance_tower

核心互动：mh_signal_console_staged，目标事件 mist_signal。

这里是 Acoustic Mapping 最自然的获取点。推荐正式时序：

1. 激活 Signal Console；
2. 稳定 Resonance Signal；
3. 获得/正式激活 perception.acoustic_mapping_i；
4. 后续 Warden 战立即要求玩家用该能力理解方向/声学攻击。

14.2 Acoustic Mapping

作用：声源方向、距离、短时轮廓、雾中辅助导航。

不等于：永久全图、透墙精确敌人 HP、自动发现秘密。

14.3 mh_warden_arena

Boss：Resonance Warden。

设计：雾中方向性攻击、声音 telegraph、Resonance pulse、真假视觉/声音错位。Acoustic Mapping 提供可靠信息，但没有它也必须存在低效率可玩解；能力提供优势，而不是硬 Key Check。

14.4 mh_extraction

结算 Mist Harbor、返回 Return Station，并保存 Pump、探索、Beacon、Signal 状态，开启 Clockworks。

14.5 Mist Harbor 敌人

• Drowned：水域/近战，和地形组合。
• Signal Wraith：信号干扰、中远距、位置感模糊，Acoustic Mapping 克制。
• Tidebound / 第三普通敌人（TARGET）：把潮水/地形和攻击结合，不能只是 Drowned 加血版。
• Resonance Warden：世界 Boss。

14.6 Mist Harbor Revisit

当前：afterFirstClearLimited, maxRevisits = 2。

Pump、Explored、Beacon 保留；新地图/移动能力可提高效率；首通核心奖励不重复。

────────

15. 世界三：Clockworks / 钟骨工厂

15.1 世界定位

不是“天空钟表城市”。正式方向：

> 高温、高压、蒸汽、铸造、运输、巨大机械和垂直结构组成的工业工厂/熔炉体系。

视觉：dark steel、brass、furnace orange、steam、moving machinery、heavy silhouettes。

核心限制：热、压力、moving surface、timing、vertical traversal、heavy enemies。

15.2 正式九场景路线

cw_entry_foundry
→ cw_pressure_hall
→ cw_conveyor_bridge
→ cw_boiler_chamber
→ cw_gear_shaft
→ cw_furnace_heart
→ cw_forged_guard_arena
→ cw_regulator_core
→ cw_shutdown_exit

15.3 Required Events

clockworks_valves
clockworks_core
clockworks_shutdown

────────

16. Clockworks 场景玩法

当前 Clockworks 大量 Marker 仍是 STAGED，以下为 V1 TARGET。

16.1 cw_entry_foundry

• 建立工厂仍自动运转的压迫感；
• Forged Guard 初遇；
• 不引入复杂 Puzzle；
• 提前展示 Conveyor / Steam / Pressure 的视觉语言。

16.2 cw_pressure_hall

当前三个阀门：

cw_pressure_valve_01_staged
cw_pressure_valve_02_staged
cw_pressure_valve_03_staged

Aggregate：clockworks_valves。

V1：三阀门构成压力稳定流程；全部完成后产生 clockworks_valves。错误操作可以产生临时 steam/pressure hazard，但不得永久锁死。Valve 状态必须存档。整个流程数据化，不做三个专用 if 分支。

16.3 cw_conveyor_bridge

当前已有 moving_surface_staged_marker 与 cw_conveyor_context_vault。

V1：Conveyor 给予水平附加速度；Context Traversal 跨越 authored 安全点；Hover/Flight 等未来 Movement/Terrain Effect 可以抵消 Conveyor；boundary.world 仍不可绕过。

16.4 cw_boiler_chamber

当前 Marker：heat accumulation、coolant valve、boiler vent。

V1：局部热量压力；Coolant Valve 降低 Heat；Boiler Vent 周期 telegraph；热抗血统/装备通过统一 HazardRules 生效。

禁止：

if scene == cw_boiler_chamber && dragon_bloodline { ... }

16.5 cw_gear_shaft

当前 Marker：vertical traversal、lift、moving platform。

目标：建立垂直感、Lift/Platform、Context Traversal，并为 Air Step 奖励做预教学。V1 玩家此时还没有 Air Step，所以主路径不能要求 Air Step。

16.6 cw_furnace_heart

当前 Marker：heat accumulation、cooling switch、coolant valve。

世界观提示：

> 炉心不是失控——它被维持在过载边缘。

作用：说明工厂异常并非单纯事故；玩家处理 Cooling/Heat；准备进入 Arena。

16.7 cw_forged_guard_arena

当前 Marker：forged_guard_elite、arena_shutter、pressure_wave。

玩法：Heavy elite、压力波迫使 Dash/Guard、Arena Shutter 作为战斗边界；敌人死亡后解除，不作为长期剧情锁。

16.8 cw_regulator_core

核心 Boss 区。当前 Marker：core console、boss phase sequence、valve-furnace link。

必须完成：clockworks_core。

Boss：Prime Regulator。

推荐三阶段：

1. Phase 1：机械直接攻击 + pressure wave；
2. Phase 2：控制 valve/furnace，环境参与战斗；
3. Phase 3：核心过载，Heat + moving machinery 同时施压。

Boss 不得要求玩家拥有未必取得的未来血统/道具。

16.9 cw_shutdown_exit

cw_master_shutdown_staged 产生 clockworks_shutdown。

完成后：

• 世界结算；
• 获得/确认 mobility.air_step_i；
• V1 Epilogue：“原来门一直不止三扇。”；
• Return Station 开始显示更多未知节点。

────────

17. Clockworks 敌人

Forged Guard

重型基础兵；pressure/heavy attack；Guard/Pierce 教学价值。

Pressure Drone（TARGET）

小型远程；Pressure shot；与 Conveyor/Steam 组合。

Furnace Hound（TARGET）

高速；热区追击；与移动空间组合。

Prime Regulator

世界 Boss。

────────

18. Clockworks 奖励：Air Step

正式目标：mobility.air_step_i。

V1 Air Step 不是传统无限 Jump：

• 只在 authored traversal markers 上使用；
• 可跨越原本必须绕行的小型垂直/横向缺口；
• Rust 验证 capability；
• Rust 验证 from/to；
• Rust 验证 cooldown；
• 前端不能自行移动玩家。

长期：

Air Step → Free Air Step → Flight

────────

19. 三世界限制—奖励闭环

|世界         |主要限制          |核心学习   |主要奖励                                             |
|-----------|--------------|-------|-------------------------------------------------|
|Grey Hive  |信息、后方、恢复、基础战斗 |普通人局限  |Local Map / Rear View / Regeneration；Enemy Vitals|
|Mist Harbor|雾、声音、信号、水域    |非视觉感知  |Acoustic Mapping                                 |
|Clockworks |热、压力、移动机械、垂直结构|地形/时序控制|Air Step                                         |

整体成长：

Human baseline
→ Information / Perception
→ Multi-sensory perception
→ Mobility beyond normal human limits

────────

20. 道具 / 血统 / 技能与世界机制组合

以下不是单独“作弊开关”，而是 Unified Effect System 的组合范式。

20.1 Rear View 来源组合

可能来源：

• 永久能力 perception.rear_view_i
• 全向视觉装备
• 多眼血统
• 灵觉技能
• 临时侦察 Drone Buff

Resolver：

rearView.enabled = any(source)

移除一个来源，不代表 Rear View 一定消失。

20.2 Map Fog Bypass

例如“全域地形测绘仪”：

Topology    = Full
Terrain     = Full
Connections = Full
Enemies     = None
Secrets     = None

在 Mist Harbor 中可以看到未亲自进入区域的地形，但 explored 仍保持真实，Signal Wraith 也不会自动显示。

20.3 Hover Boots

ignoreMovementPenalty:
- terrain.water_shallow
- terrain.water_deep
- terrain.mud
- terrain.rubble
- surface.conveyor

Mist Harbor：可绕过 Pump East Channel 的深水限制（前提是该效果允许 deep-water traversal），Drowned Quay 不减速。
Clockworks：Conveyor 影响显著降低或无效。

世界系统不需要知道装备名字。

20.4 Heat-resistant Bloodline

Effect：hazard.heat resistance。

Clockworks Boiler / Furnace / Boss Heat phase 自动受益。

20.5 No Breath

immune: drowning
immune: suffocation

不自动获得 toxin / pressure / radiation immunity。

20.6 Flight

未来 MovementMode::Flight 可绕过普通地形，但不能绕过：

boundary.world
boundary.quest
rule.sealed

────────

21. Legacy Content Library

当前正式开发线仍保存大量旧内容数据：

• Skills：约 161
• Weapons：约 82
• Gear：约 50
• Treasures：约 33
• Items：约 34
• Bloodlines：约 22

这些是重要的“无限流内容池”设计素材，但：

> **存在于仓库 ≠ V1 正式可玩 ≠ 可公开发行。**

尤其旧数据包含明显第三方作品/IP 指向内容，不能自动进入公开发行 Content Registry。

工程策略：

Legacy Content
↓
Legacy Adapter
↓
EffectSpec

新内容一律原生使用 EffectSpec。

────────

22. Bloodline 设计原则

血统不能只做数值皮肤。

低阶可以有 Stats，高阶应该逐渐改变规则。

原创龙血体系示意：

Rank 1

• physical resistance
• heat resistance

Rank 2

• No Breath

Rank 3

• short flight

Rank 4

• Rule resistance

这种成长才符合“人族进化”。

────────

23. Skill 设计原则

Skill 分 Active / Passive。

Active：Q/E/R 槽、energy、cooldown、cast/active/recovery、EffectSpec。
Passive：进入 Resolver，不要求前端单独判断。

当前旧字段 flying / detect_bonus 应由 Adapter 迁移，不应继续成为正式底层接口。

────────

24. 主动技能槽长期方案

V1 UI 仍显示：

Q Pulse
E Guard
R Pierce

长期内部演进为：

AbilitySlot0
AbilitySlot1
AbilitySlot2

V1 兼容映射：

slot0 = Pulse
slot1 = Guard
slot2 = Pierce

未来玩家可更换技能而不推翻输入层设计。

────────

25. Synergy / 组合联动

组合系统可以判断：

Bloodline
Weapon
Treasure
Skill
Capability
World State

但结果仍使用 EffectSpec。

例：

机械义体 + 全向传感器
→ Rear View range +50%

修真体系 + 本命飞剑
→ Flight energy cost -20%

热适应血统 + 热隔离装甲
→ 高温区域接近免疫

禁止为每条 Synergy 写世界专用代码。

────────

26. Stacking Rules

正式至少支持：

Any
Add
Multiply
Max
Min
HighestPriority
Replace

例：

• RearView：Any
• Detection Range：Add
• Movement Speed：Multiply
• Hazard Resistance：Multiply Remaining / 专用 Resistance Resolver
• Immunity：Any

────────

27. Generic Actor Runtime

长期不能只依赖 Sentinel 专用状态。

正式应统一：

struct ActorRuntime {
    entity_id: String,
    entity_type: String,
    transform: Transform,
    hp: u32,
    max_hp: u32,
    movement: ...,
    ai: ...,
    combat: ...,
    tags: ...,
    active: bool,
}

普通敌人数据化。Boss 使用：

Generic Actor
+
Boss Controller

Sentinel 特殊逻辑可保留为 Boss-specific module。

────────

28. AI 设计层

普通敌人至少拥有：

Idle
Investigate
Chase
Attack
Stagger
Dead

世界差异由：

• Perception profile
• Movement profile
• Attack profile
• Terrain preference
• Tags

组合。

Signal Wraith 更依赖 Signal/Range；Drowned 更依赖 Water/Melee。

────────

29. Tiled 场景规范

推荐正式层：

visual.floor
visual.floor_detail
visual.props_back
visual.props_dynamic
visual.foreground
visual.vfx_markers

logic.collision
logic.navigation
logic.spawn
logic.interaction
logic.door
logic.trigger
logic.hazard
logic.checkpoint
logic.objective
logic.transition
logic.camera
logic.traversal
logic.exploration
logic.terrain

Tiled 只做 Authoring，编译后生成 Canonical SceneDefinition。Rust 不读取随意的前端临时坐标作为权威玩法规则。

────────

30. Scene Runtime 原则

场景定义应描述：Bounds、Collision、Navigation、Spawn、Interaction、Door、Trigger、Hazard、Checkpoint、Objective、Transition、Camera Zone、Traversal、Exploration Region、Terrain Region。

世界特殊逻辑优先通过：

generic component + data

实现。

例如 Pump 是：

Interaction
+ WorldPersistentState
+ TerrainStateChange

而不是 if scene_id == "mh_pump_station" 的大量特例。

────────

31. Save / Persistence

建议 Save V6 正式拥有：

PlayerProgressionV6 {
    inventory,
    equipment,
    owned_skills,
    equipped_skills,
    skill_proficiency,
    category_proficiency,
    bloodline,
    bloodline_tiers,
    bloodline_proficiency,
    treasures,
    gene_stage,
    cultivation,
    capabilities,
}

世界状态：

• RouteState
• scene state
• enemy state
• world persistent state
• explored state
• Pump state
• Clockworks valve/core/shutdown state

────────

32. 不保存 Resolved Rules

禁止把：

rearView = true
heatResistance = 0.5

作为独立持久真值。

保存：装备了什么、学了什么、血统是什么、永久能力是什么、Buff 剩余多久。

Load：

Sources
→ Validate
→ EffectResolver
→ EffectivePlayerRules

────────

33. Revisit

当前正式：

Grey Hive

afterFirstClearLimited, maxRevisits = 2

Mist Harbor

afterFirstClearLimited, maxRevisits = 2

Clockworks

taskCandidatesLimited, maxRevisits = 4

重访原则：

• 世界变化状态保留；
• 已探索保留；
• 首通核心奖励不重复；
• 新能力允许新路径/效率；
• 可存在 post-clear tasks；
• 前三世界不能无限首通 Farming。

────────

34. UI 与 Gameplay 数据边界

前端只显示 Rust 授权的数据。

Map：

Canonical Map
+ Exploration
+ Effective MapRules
↓
Rust MapProjection
↓
Web

Rear View：

EffectResolver
↓
RearViewProjection
↓
Pixi

Enemy Vitals：

EffectResolver
↓
VitalProjection
↓
HUD

前端不能自行根据装备 ID 推断玩法权限。

────────

35. Visual / Gameplay 分离

正式画面：

Hero Scene Plate
+ Modular Props
+ Sprite Actors
+ Pixi VFX
+ HTML/CSS UI

Scene Plate 视觉不能修改 Gameplay Geometry。若图片看起来有路但 Collision 不允许，应修视觉，不允许前端临时开放碰撞。

────────

36. Audio 与 Gameplay

声音在 Mist Harbor 尤其属于 gameplay。

SoundCue 应包含：

kind
source
world/scene
direction (when authorized)
distance (when authorized)

普通玩家可以听到声音，但没有 Acoustic Mapping 时不直接得到精确 HUD 方位角/距离。获得 Acoustic Mapping 后由 Rust 授权 direction/distance Projection。

────────

37. 世界观连接原则

三个世界不需要属于同一个物理宇宙。

Return Station 提供“多世界连接层”。每个世界应具备：

1. 独立历史；
2. 独立技术/文明逻辑；
3. 独立限制；
4. 可以产出通用于其他世界的强化。

跨世界连续性来自：

> 玩家自己变成了“跨世界连续存在的主体”。

────────

38. V1 世界观节奏

Grey Hive

“事故发生了什么？”

尺度：局部设施。

Mist Harbor

“世界感知本身不可靠。”

尺度：城市港区。

Clockworks

“世界规则可以由巨大系统主动维持。”

尺度：工业体系。

V1 尾声：

> 世界门不止三扇。

────────

39. 未来世界设计模板

新增世界必须填写：

world_id
world_name
world_fantasy
human_limitations
signature_hazards
signature_perception
signature_terrain
signature_enemies
boss
world_required_events
reward_capabilities
revisit_policy
cross_world_synergies

新世界必须至少提出一个旧世界没有强调过的人类限制，否则只是换皮地图。

────────

40. 未来能力层级

PERCEPTION

Rear View → Thermal → Spirit → Occluded Presence → Precognition

INFORMATION

Local Map → Tactical Scan → Full Terrain → Dynamic Enemy Intel → Rule Knowledge

BODY

Regeneration → Poison Resistance → No Breath → Extreme Environment → Rule Body

MOBILITY

Dash → Context Traversal → Air Step → Flight → Phase

SPACE

Storage → Blink → Anchor → Local Phase → Spatial Rewrite

RULE

Gravity Resistance → Time-slow Resistance → Rule Seal Resistance → Rule Interaction

V1 不要求实现高阶能力，但底层不能封死。

────────

41. V1 当前必须完成的玩法闭环

Grey Hive

• Return Station 进入；
• Power；
• Lockdown；
• Bai Zhi 可选；
• Sentinel；
• Beacon；
• Extraction；
• Return；
• Save/Continue；
• 首次能力选择。

Mist Harbor

• World Gate 开放；
• 九场景连续路线；
• West Beacon；
• Signal Interference；
• Water slowdown；
• East Beacon；
• Pump；
• Signal；
• Acoustic Mapping；
• Warden；
• Extraction；
• Return；
• Save/Continue/Revisit。

Clockworks

• 九场景连续路线；
• Valves；
• Conveyor；
• Heat/Steam；
• Vertical traversal；
• Forged Guard；
• Regulator Core；
• Shutdown；
• Air Step；
• V1 Epilogue；
• Return。

────────

42. 当前主要工程阻塞

P0

• Unified Effect / Player Rule System
• Formal Build（Inventory/Equipment/Skill/Bloodline）接入
• Generic Actor Runtime
• Map Knowledge
• Terrain/Hazard
• Save Build state
• 三世界连续通关
• Windows Native Acceptance

内容

• Mist Harbor 后半程正式敌人/Boss/世界入口
• Clockworks staged → playable

美术

• Scene Plates
• Cenyao 动作
• Boss
• Guard/Pierce final VFX
• props
• audio

Release

• public asset eligibility
• installer
• performance
• real native screenshots/interactions

────────

43. 推荐工程工作包

POWER-SYSTEM-V2-01
↓
FORMAL-BUILD-V6-01
↓
GENERIC-ACTOR-RUNTIME-01
↓
MAP-KNOWLEDGE-TERRAIN-01
↓
GH-EFFECT-REGRESSION-01
↓
MIST-HARBOR-GAMEPLAY-01
↓
CLOCKWORKS-GAMEPLAY-01
↓
PUBLIC-ASSET-GATE-01
↓
NATIVE-WINDOWS-ACCEPTANCE-01

美术和音频并行，不等待全部代码完成。

────────

44. V1 Gameplay Acceptance Matrix

Core

• WASD
• Mouse Aim
• Attack
• Dash direction lock
• Pulse
• Guard
• Pierce
• Interact
• Pause
• Save
• Restart
• Continue

Capability

• No Map → no gameplay minimap
• Local Map → explored only
• Rear View → 后视信息
• Enemy Vitals → qualitative tiers
• Regeneration → 延迟恢复
• Acoustic Mapping → direction/distance
• Air Step → authored traversal only

Effect Resolver

• 多来源同效果
• remove one source still active if another remains
• ordering deterministic
• Save/Load re-resolve identical

Terrain

• Water slowdown
• Pump deep-water blocking
• Pump drained state
• future test Hover bypass
• world boundary never bypassed by normal terrain-ignore

Campaign

• RS → GH → RS → MH → RS → CW → RS
• cross-process save
• world state preserved
• revisit policy respected

────────

45. 工程硬规则

1. Rust authoritative gameplay。
2. Pixi/HTML 只负责显示与输入。
3. 世界信息权限由 Rust Projection 决定。
4. 新道具不能靠 ID 特判。
5. 新血统不能靠 ID 特判。
6. 新技能尽量组合 EffectSpec。
7. 新世界环境机制优先复用 Terrain/Hazard/Interaction。
8. Tiled 是 Authoring，不是 Runtime Authority。
9. Resolved Rules 不作为独立存档真值。
10. Registered/Staged 不得被描述为 Playable/Complete。
11. 内部 release_approved 不等于公开发行资格。
12. Web tests 不等于 Windows Native Acceptance。

────────

46. 代码目录目标建议

server-rs/src/

effects/
    mod.rs
    definitions.rs
    source.rs
    resolver.rs
    stacking.rs

player_rules/
    mod.rs
    perception.rs
    map_knowledge.rs
    movement.rs
    terrain.rs
    hazards.rs
    combat.rs
    interaction.rs
    resources.rs
    inventory.rs
    space.rs
    rule.rs

actors/
    runtime.rs
    profiles.rs
    ai.rs
    boss.rs

content/
    adapters/
    registry.rs

world/
    persistent_state.rs
    terrain_state.rs

不要求一次全部拆 crate。

────────

47. 数据定义示例

47.1 Rear View Equipment

{
  "id": "gear.rear_sensor_mk1",
  "slot": "accessory",
  "effects": [
    {
      "type": "perception",
      "kind": "rearView",
      "rank": 1,
      "rangeM": 12.0
    }
  ]
}

47.2 Full Terrain Map

{
  "id": "gear.cartographer_array",
  "effects": [
    {
      "type": "mapKnowledge",
      "topology": "full",
      "terrain": "full",
      "connections": "full",
      "objectives": "known",
      "enemies": "none",
      "secrets": "none"
    }
  ]
}

47.3 Hover Boots

{
  "id": "gear.hover_boots",
  "effects": [
    {
      "type": "terrain",
      "ignoreMovementPenalty": [
        "terrain.water_shallow",
        "terrain.water_deep",
        "terrain.mud",
        "terrain.rubble",
        "surface.conveyor"
      ]
    }
  ]
}

47.4 Heat Bloodline

{
  "id": "bloodline.thermal_adaptation",
  "effects": [
    {
      "type": "hazardResistance",
      "hazard": "heat",
      "value": 0.5
    }
  ]
}

────────

48. 最终玩法定义

《无限人族进化》的长期内容生产必须变成：

提出一个新的“人类限制”
↓
设计一个世界让玩家真正感受到这个限制
↓
让已有构筑提供不同解法
↓
给予新能力/道具/血统/技能
↓
通过 Effect System 改变玩家与世界规则的关系
↓
新强化在未来世界继续产生组合

而不是：

新世界
→ 更高 HP 的怪
→ 更高攻击装备
→ 重复

游戏的核心乐趣应来自：

> **“以前必须遵守的规则，现在我可以用某种方式绕过、理解、承受或改写。”**

三世界 V1 是这套长期结构的第一条完整证明链：

Grey Hive
普通人的信息与身体限制

→ Mist Harbor
普通视觉与方向感不再可靠

→ Clockworks
普通移动和环境承受能力开始不足

→ World Network
玩家已经不是最初那个普通人

这就是后续所有工程代码、内容设计、地图制作、技能设计、道具设计和血统设计应共同服务的核心。

────────

49. 参考的当前工程权威位置

本文件编写时核对的正式开发线：

release/v1-three-worlds-2d5 @ dc7d8c587bdb7784764de30a1b450d33a28e4c27

关键工程位置：

server-rs/src/formal_runtime.rs
server-rs/src/world_v3.rs
server-rs/src/scene_runtime.rs
server-rs/src/scene_registry.rs
server-rs/src/continuous_combat.rs
server-rs/src/continuous_kcc.rs
server-rs/src/capability_v1.rs
server-rs/src/save_v5.rs
server-rs/src/defs.rs
server-rs/src/state.rs
server-rs/src/skills_data.rs
server-rs/src/bloodlines.rs
server-rs/src/weapons.rs
server-rs/src/gear.rs
server-rs/src/treasures.rs
server-rs/src/items.rs
server-rs/data/world_progression_v1.json
server-rs/data/capability_v1.json
server-rs/data/combat_v1.json

design/maps/grey_hive/*.tmj
design/maps/mist_harbor/*.tmj
design/maps/clockworks/*.tmj
design/maps/return_station/rs_core_room.tmj

本文中的“当前实现状态”应继续以代码和真实验收为准；本文描述的是玩法权威目标，不得被用作“已经实现”的证据。

这一份是玩法文件，要和之前的几个文件一起保存