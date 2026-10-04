可以。下面这份可以直接作为 **Codex 的新增工程方案**。它不是替换你已有三份 V1 方案，而是解决当前三世界开发中的实际阻塞，并补上《无限人族进化》作为“无限流长期游戏”最重要的一层：**道具 / 血统 / 技能 / 能力 / 装备对世界规则的统一扩展架构**。

我刚重新核对了 GitHub，当前正式开发线仍是：

```text
release/v1-three-worlds-2d5
HEAD = dc7d8c587bdb7784764de30a1b450d33a28e4c27
```

与你提供的状态一致，没有新的提交超出这一基线。

---

# 《无限人族进化》V1 阻塞清理 + 无限流能力扩展架构实施方案

**适用基线：** `release/v1-three-worlds-2d5@dc7d8c58`  
**性质：** 对现有三份 V1 方案的工程补充  
**优先级：** P0 架构补强 + V1 发布阻塞清理  
**核心原则：**

> V1 的三个世界必须完成，但不能为了赶完三个世界，把以后几百种道具、血统、技能继续写成特殊分支。

最终要求是：

```text
Capability
Item
Equipment
Treasure
Skill
Bloodline
Temporary Buff
World Effect
        ↓
统一 Effect System
        ↓
EffectivePlayerRules
        ↓
Rust authoritative gameplay
```

以后“后视”“全图”“飞行”“踏水”“免疫高温”“灵视”“空间储物”“时间减速抗性”等都从同一体系组合出来。

---

# 1. 当前代码状态：哪些已经很好，哪些必须现在修

当前正式 Runtime 已经有相当好的基础。

| 当前系统 | 状态 | 结论 |
|---|---|---|
| Rust 独立 60Hz owner | 已实现 | 保留 |
| Input v2 | 已实现 | 保留 |
| Snapshot v3 | 已实现 | 保留 |
| Q/E/R/Dash | 已进入 Formal Runtime | 保留继续完善 |
| SceneDefinition | 已实现 | 扩展，不推翻 |
| Scene Registry | 已实现 | 扩展 |
| Save v5 | 已存在 | 需升级玩家成长字段 |
| Rear View | 已有正式原型 | 迁入统一 Effect |
| Local Map | 已有正式原型 | 升级为 Map Knowledge |
| Enemy Vitals | 已有正式原型 | 迁入统一 Effect |
| Regeneration | 已有正式原型 | 迁入统一 Effect |
| Acoustic Mapping | 已有正式原型 | 迁入统一 Effect |
| Air Step | 有 traversal prototype | **当前注册体系不一致，必须修** |
| 旧技能系统 | 161 Skills | 数据非常有价值，尚未正式接入新 Runtime |
| Weapons | 82 | 同上 |
| Gear | 50 | 同上 |
| Treasures | 33 | 同上 |
| Items | 34 | 同上 |
| Bloodlines | 22 | 同上 |
| Unified Effect System | **没有** | P0 |
| Generic enemy runtime | 不完整 | P0/P1 |
| Formal inventory/loadout | 不完整 | P0 |
| Terrain Rule System | 没有 | P0 |
| Map Knowledge Rule System | 只有原型 | P0 |

---

# 2. 当前代码里已经暴露出的几个结构性问题

## 2.1 Capability ID 已经开始分叉

`capability_v1.rs` 的正式 canonical IDs 目前只有：

```text
information.local_map_i
perception.rear_view_i
information.enemy_vitals_basic
body.regeneration_i
perception.acoustic_mapping_i
```

但 `formal_runtime.rs` 和 `scene_runtime.rs` 的 traversal 已经认识：

```text
mobility.air_step_i
```

也就是说当前已经出现：

```text
Capability Registry A
≠
Traversal Capability Registry B
```

并且 `scene_runtime.rs` 的 `known_capabilities` 甚至没有包含：

```text
perception.acoustic_mapping_i
```

这不是严重 bug，但它说明：

> **如果继续按现在方式增加能力，十几个能力以后会出现多个互相不一致的白名单。**

必须立即消掉所有这种本地 `known_capabilities = [...]`。

---

# 3. 当前旧强化系统与新 Runtime 是“两套世界”

这是现在最大架构债。

旧 `GameState` 已经拥有：

```text
bloodline
bloodline_tier
bloodline_proficiency

skills
equipped_skills
skill_proficiency
category_proficiency

equipment
treasures

inventory

cultivation_stage
gene_stage
```

但新的 Formal Runtime 主要保存：

```text
WorldStateV3
CapabilityState
RouteState
```

当前 Save v5 虽然已经有：

```rust
inventory: InventoryState
dialog_flags: DialogFlags
```

但实际 capture 还是：

```rust
inventory: InventoryState::default()
dialog_flags: DialogFlags::default()
```

这说明：

> **正式 2.5D Runtime 还没有真正继承旧大体系的 Inventory / Skill / Bloodline / Equipment。**

这是 V1 前必须解决的。

否则你以后看到的：

```text
161 skills
22 bloodlines
82 weapons
```

只是仓库里“存在”，并不代表玩家能在正式 2.5D 游戏里使用。

---

# 4. 为什么不能直接把旧系统接回来

旧系统大量是：

```text
Attack +12
Dodge +0.15
Damage Reduce +8
HP +30
```

这些很好迁移。

但是你的游戏长期真正有价值的是：

```text
看见身后
看见灵体
看到墙后危险
显示完整地图
探测远距离地图
忽略迷雾
无视水地减速
踏水
飞行
空气踏步
穿越软障碍
无需呼吸
耐真空
耐高温
耐辐射
空间储物
危险预测
时间减速抗性
规则封锁抗性
```

这些不是一个普通 RPG `StatModifier` 能表达的。

所以需要增加一个新的正式层。

---

# 5. 新核心：Unified Gameplay Effect System

正式架构冻结为：

```text
             Content
                │
 ┌──────────────┼───────────────┐
 │              │               │
Items       Bloodlines        Skills
Gear        Capabilities      Treasures
 │              │               │
 └──────────────┼───────────────┘
                ↓
          Effect Sources
                ↓
          Effect Resolver
                ↓
       EffectivePlayerRules
                ↓
  ┌─────────┬───────┬─────────────┐
  │         │       │             │
Movement  Map   Perception      Combat
Terrain   AI      Hazard       Interaction
  │         │       │             │
  └─────────┴───────┴─────────────┘
                ↓
        Rust 60Hz Runtime
```

---

# 6. Effect Source

新增：

```rust
pub enum EffectSourceKind {
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
```

每个来源都有：

```rust
pub struct EffectSource {
    pub source_kind: EffectSourceKind,
    pub source_id: String,
    pub instance_id: String,
    pub lifetime: EffectLifetime,
    pub effects: Vec<EffectSpec>,
}
```

---

# 7. Effect Lifetime

必须从一开始区分。

```rust
pub enum EffectLifetime {
    Permanent,
    Owned,
    Equipped,
    Selected,
    Activated,
    Timed {
        remaining_ms: u64,
    },
    Scene,
    World,
}
```

例如：

### 永久能力

```text
perception.rear_view_i
```

```text
Permanent
```

### 全向感知镜

```text
Equipped
```

### 主动扫描技能

```text
Activated
```

### 临时药剂

```text
Timed
```

---

# 8. 为什么这个设计重要

假设玩家：

```text
进化能力：Rear View
装备：全向视觉镜
Buff：战术无人机
```

三个来源全部提供：

```text
rear_view
```

然后玩家摘掉眼镜。

正确结果：

```text
Rear View 仍然存在
```

现在的：

```rust
RearViewAuthorization {
    grant_id
}
```

无法优雅表达这种多来源组合。

---

# 9. EffectSpec

不要设计 200 个 bool。

使用封闭 enum。

建议：

```rust
pub enum EffectSpec {
    GrantCapability(CapabilityEffect),

    Stat(StatModifier),

    Perception(PerceptionEffect),

    MapKnowledge(MapKnowledgeEffect),

    Movement(MovementEffect),

    Terrain(TerrainEffect),

    Hazard(HazardEffect),

    Combat(CombatEffect),

    Resource(ResourceEffect),

    Interaction(InteractionEffect),

    Inventory(InventoryEffect),

    Space(SpaceEffect),

    Rule(RuleEffect),
}
```

这就是未来无限能力系统的“语言”。

---

# 10. 能力分类和 Effect 类型不要混为一谈

你已有：

```text
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
```

继续保留。

但是它们是：

> **设计分类 / UI 分类。**

真正决定 gameplay 的是：

```text
EffectSpec
```

例如：

```text
information.local_map_i
```

分类：

```text
INFORMATION
```

效果：

```text
MapKnowledgeEffect
```

而：

```text
perception.rear_view_i
```

分类：

```text
PERCEPTION
```

效果：

```text
PerceptionEffect
```

---

# 11. EffectivePlayerRules

EffectResolver 的最终输出：

```rust
pub struct EffectivePlayerRules {
    pub stats: EffectiveStats,

    pub perception: PerceptionRules,

    pub map: MapKnowledgeRules,

    pub movement: MovementRules,

    pub terrain: TerrainRules,

    pub hazards: HazardRules,

    pub combat: CombatRules,

    pub interaction: InteractionRules,

    pub inventory: InventoryRules,

    pub space: SpaceRules,

    pub rule: RuleRules,
}
```

它代表：

> **玩家此刻在这个世界里究竟拥有哪些规则权限。**

---

# 12. 后视道具的正式实现例

例如：

```text
全向战术视觉模块
```

定义：

```json
{
  "id": "gear.omnidirectional_sensor_i",

  "category": "equipment",

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
```

装备：

```text
Loadout
↓
EffectSource
↓
EffectResolver
↓
PerceptionRules
```

输出：

```json
{
  "rearView": {
    "enabled": true,
    "rank": 1,
    "rangeM": 12.0
  }
}
```

Pixi 才显示 Rear View。

---

# 13. 永久进化 Rear View 使用完全一样的底层规则

```json
{
  "id": "perception.rear_view_i",

  "effects": [
    {
      "type": "perception",
      "kind": "rearView",
      "rank": 1,
      "rangeM": 14
    }
  ]
}
```

只是：

```text
source = InnateCapability
lifetime = Permanent
```

不需要另一套代码。

---

# 14. 地图必须从 ExploredMap 升级为 Map Knowledge

当前：

```text
ExploredMap
```

只有：

```text
rooms
connections
objectives
```

长期不够。

正式地图知识分通道：

```rust
pub struct MapKnowledgeRules {
    pub topology: KnowledgeLevel,
    pub terrain: KnowledgeLevel,
    pub connections: KnowledgeLevel,

    pub objectives: KnowledgeLevel,
    pub enemies: KnowledgeLevel,
    pub hazards: KnowledgeLevel,
    pub loot: KnowledgeLevel,
    pub npcs: KnowledgeLevel,
    pub secrets: KnowledgeLevel,

    pub reveal_radius_m: f32,
}
```

---

# 15. KnowledgeLevel

```rust
pub enum KnowledgeLevel {
    None,

    ExploredOnly,

    DetectedOnly,

    Known,

    Full,
}
```

---

# 16. 普通玩家

没有地图能力：

```text
topology = None
```

所以：

> 不显示 Minimap。

---

# 17. Local Map I

例如：

```text
topology     = ExploredOnly
terrain      = ExploredOnly
connections  = ExploredOnly

objectives   = Known

enemies      = None
hazards      = None
secrets      = None
```

---

# 18. 你说的“无视未探索地图迷雾”

例如：

```text
全域地形扫描器
```

效果：

```json
{
  "type": "mapKnowledge",

  "topology": "full",
  "terrain": "full",
  "connections": "full"
}
```

注意：

它**不会自动显示**：

```text
enemy
secret
loot
quest target
```

否则一件地图道具就把所有探索价值毁掉。

---

# 19. 可以继续做更高级地图装备

例如：

## 战术雷达

```text
terrain:
exploredOnly

enemies:
detectedOnly

revealRadius:
25m
```

---

## 轨道扫描器

```text
topology:
full

terrain:
full

hazards:
known

enemies:
detectedOnly
```

---

## 预知感知

```text
hazards:
full

enemies:
predicted
```

未来可增加：

```text
Predicted
```

知识层级。

---

# 20. MapProjection 必须由 Rust 生成

不能：

```text
前端有完整地图
然后自己隐藏一部分
```

正式：

```text
Canonical World Map
        ↓
Exploration State
        ↓
Effective Map Rules
        ↓
Rust MapProjection
        ↓
Web UI
```

这样即使以后存在：

```text
全知
神识
透视
卫星
```

仍然由权威端决定玩家“能知道什么”。

---

# 21. Terrain System

这是你问的“不受地形影响”的基础。

所有地面/障碍必须有 tags。

例如：

```text
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
```

---

# 22. TerrainRules

```rust
pub struct TerrainRules {
    pub ignored_move_penalties: BTreeSet<TerrainTag>,

    pub ignored_surface_hazards: BTreeSet<TerrainTag>,

    pub traversable_obstacles: BTreeSet<ObstacleTag>,

    pub movement_speed_overrides:
        BTreeMap<TerrainTag, f32>,
}
```

---

# 23. 踏水靴

```json
{
  "effects": [
    {
      "type": "terrain",
      "kind": "ignoreMovementPenalty",
      "tags": [
        "terrain.water_shallow",
        "terrain.water_deep"
      ]
    }
  ]
}
```

---

# 24. 悬浮装置

```text
ignore:
water
mud
rubble
conveyor
```

但是：

```text
wall
quest barrier
world boundary
```

仍然阻挡。

---

# 25. 不要做 `ignore_collision=true`

这是未来非常危险的设计。

因为它会导致：

```text
穿墙
跳过 Boss
越过剧情 Gate
离开地图
进入未加载区域
```

应该做：

```text
Collision Category
+
Movement Rule
```

---

# 26. Movement Modes

新增：

```rust
pub enum MovementMode {
    Ground,

    AirStep,

    Flight,

    Swim,

    Phase,
}
```

---

# 27. Air Step

当前已有 traversal marker prototype。

以后正式变成：

```text
MovementMode::AirStep
```

但仍然依赖：

```text
air_step traversal points
```

V1 不需要自由空中移动。

---

# 28. Flight

真正 Flight：

```text
MovementMode::Flight
```

允许：

```text
y
```

连续移动。

但仍受：

```text
WorldBoundary
FlightRestrictedZone
QuestBarrier
```

控制。

---

# 29. Phase

以后空间/灵体类能力：

```text
MovementMode::Phase
```

可以穿：

```text
obstacle.soft
obstacle.normal_wall
```

但不穿：

```text
boundary.world
boundary.quest
rule.sealed
```

---

# 30. Hazard System

新增统一 Hazard Tag：

```text
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
```

---

# 31. HazardResistance

例如龙血：

```text
heat resistance 0.5
fire resistance 0.4
```

不是：

```text
if bloodline == dragon
```

---

# 32. No Breath

未来：

```text
body.no_breath_i
```

效果：

```text
drowning immunity
vacuum suffocation immunity
toxic-gas breathing immunity
```

但：

> 不代表毒液免疫。

机制必须分开。

---

# 33. Bloodline V2

当前 22 种血统大部分是数值被动。

保留当前：

```text
atk
leech
dodge
dmg_reduce
...
```

使用 Adapter 自动转：

```text
StatModifier
```

然后新增：

```text
effects[]
```

---

# 34. 示例：机械义体

现在：

```text
Attack +8
Damage Reduce +8
Dodge +0.08
```

以后可以：

```json
{
  "effects": [
    {
      "type":"perception",
      "kind":"enemyVitalScan",
      "rank":1
    },

    {
      "type":"hazard",
      "kind":"resistance",
      "hazard":"electric",
      "value":0.25
    }
  ]
}
```

---

# 35. 血统进阶更应该改变规则而不是只变数字

例如 Dragon：

### Rank 1

```text
Physical Resistance
Heat Resistance
```

### Rank 2

```text
No Breath
```

### Rank 3

```text
Short Flight
```

### Rank 4

```text
Rule Resistance
```

这样玩家真的感觉生命形态改变。

---

# 36. Skill V2

现在 161 Skills 已有：

```text
SkillEffect
flying
detect_bonus
```

不删除。

先做 Legacy Adapter：

```text
SkillDef
↓
LegacySkillEffectAdapter
↓
Vec<EffectSpec>
```

---

# 37. flying 自动迁移

如果：

```rust
skill.flying
```

则生成：

```text
MovementEffect::Enable(Flight)
```

以后再逐步把旧字段废弃。

---

# 38. detect_bonus 自动迁移

转换：

```text
Map reveal range
或者
Detection range
```

不要继续让前端直接读取：

```text
detect_bonus
```

---

# 39. Item V2

当前 `ItemEffect`：

```text
Heal
San
Ammo
Throw
Charm
None
```

保留，作为：

```text
useEffect
```

增加：

```text
passiveEffects
```

---

# 40. 道具例子

```json
{
  "id": "item.recon_eye",

  "useEffect": null,

  "passiveEffects": [
    {
      "lifetime": "equipped",

      "effect": {
        "type":"perception",
        "kind":"rearView"
      }
    }
  ]
}
```

---

# 41. Inventory 与 Equipment 必须分开

当前 Formal Save 的 inventory 还没真正接。

建议正式结构：

```rust
PlayerBuildState {
    inventory,

    equipment,

    owned_skills,

    equipped_skills,

    active_bloodline,

    bloodline_progress,

    capabilities,

    treasures,

    gene,

    cultivation,
}
```

---

# 42. Equipment Slot

可以先保留当前：

```text
weapon
armor
accessory
treasure ×3
```

未来扩展：

```text
sensor
utility
implant
```

不要现在过度扩张。

---

# 43. Active / Passive 必须分开

例如：

```text
装备：
全向传感器
```

自动生效。

而：

```text
技能：
精神扫描
```

需要玩家使用。

统一 Effect 但 Activation 不同。

---

# 44. Active Ability Runtime

建议：

```rust
ActiveAbilityState {
    ability_id,
    source_id,
    cooldown_remaining,
    charges,
    duration_remaining,
}
```

Q/E/R 只是：

> Slot 绑定。

不是技能本身。

---

# 45. Q/E/R 以后必须可换技能

现在：

```text
Q Pulse
E Guard
R Pierce
```

V1 是默认技能。

以后玩家可以：

```text
Q = 灵识扫描
E = 空间盾
R = 雷电穿刺
```

Input 仍然只发：

```text
UseAbilitySlot(0)
UseAbilitySlot(1)
UseAbilitySlot(2)
```

Rust 决定槽位里是什么。

这会比：

```text
ActionKind::Pulse
ActionKind::Pierce
```

更适合长期。

---

# 46. 但 V1 不需要立即破坏当前 Action v2

迁移顺序：

### V1 当前

继续：

```text
Pulse
Guard
Pierce
```

### 新增内部映射

```text
Pulse → ability slot 0
Guard → slot 1
Pierce → slot 2
```

以后协议升级时再变成：

```text
ActivateAbilitySlot
```

避免当前三世界被重构拖死。

---

# 47. Synergy

当前已经有 10 个：

```text
Bloodline × Weapon × Treasure × Skill
```

联动规则。

这是好东西。

但现在只有：

```text
FlatDamage
Leech
```

以后：

```rust
SynergyEffect {
    Effects(Vec<EffectSpec>)
}
```

于是可以做：

```text
机械义体 + 全向传感器
→ Rear View range +50%

飞剑 + 修真
→ Flight cost -20%

火系血统 + 火抗甲
→ Heat hazard immunity
```

---

# 48. Stacking Rules

必须在扩内容前定义。

建议：

```rust
pub enum StackRule {
    Any,

    Add,

    Multiply,

    Max,

    Min,

    HighestPriority,

    Replace,
}
```

---

# 49. 例如 Rear View

```text
Any
```

只要一个来源即可。

---

# 50. Detection Range

```text
Add
```

例如：

```text
基础 10m
+ 装备 5m
+ 血统 3m
```

---

# 51. Hazard Resistance

推荐：

```text
MultiplyRemaining
```

而不是无限相加。

例如：

```text
50% + 50%
```

应该是：

```text
75%
```

而不是：

```text
100%
```

除非效果明确是：

```text
Immune
```

---

# 52. Effect Conditions

未来需要条件效果：

```rust
EffectCondition {
    world_tags,
    terrain_tags,
    hp_below,
    energy_above,
    combat_state,
    time_state,
    target_tags,
}
```

但 V1 第一版先支持：

```text
Always
WorldTag
TerrainTag
HPBelow
```

足够。

---

# 53. Generic Actor Runtime

这是另一个当前 P0 阻塞。

目前 `WorldStateV3` 的正式 Actor Runtime 仍然核心保存：

```rust
sentinels: Vec<Sentinel>
```

Snapshot actor 也主要由 Sentinel 转出。

这不能支撑：

```text
Drowned
Signal Wraith
Forged Guard
各种未来敌人
```

需要改为通用：

```rust
ActorRuntime {
    entity_id,
    entity_type,

    transform,

    hp,
    max_hp,

    movement,

    ai,

    combat,

    tags,

    active,
}
```

---

# 54. AI Profile

数据：

```text
enemy.grey_hive.worker
enemy.mist.drowned
enemy.clockworks.guard
```

引用：

```text
aiProfile
```

例如：

```json
{
  "id":"ai.drowned",

  "perception":"soundBiased",

  "moveSpeed":2.2,

  "states":[
    "idle",
    "investigate",
    "chase",
    "attack",
    "stagger",
    "dead"
  ]
}
```

---

# 55. 不要为每个敌人创建 Rust struct

`Sentinel` 可以继续作为：

> Boss-specific behavior module。

但普通敌人应该走通用 Actor。

Boss 可以：

```text
Generic Actor
+
BossController
```

---

# 56. 三世界当前阻塞如何和这套架构对应

## Grey Hive

现阶段：

代码闭环最完整。

新增 Effect System 不应阻塞 Grey Hive 美术生产。

要做：

```text
Capability prototype
→ Effect Resolver
```

但保持行为完全兼容。

然后继续完成：

```text
Grey Hive native E2E
```

---

# 57. Mist Harbor

当前 9 scene 已注册：

```text
mh_fog_pier
mh_tidal_warehouse
mh_signal_yard
mh_drowned_quay
mh_breakwater
mh_pump_station
mh_resonance_tower
mh_warden_arena
mh_extraction
```

但世界入口仍关闭。

这部分要分成：

### 已经能继续开发

不依赖地图最终边界：

```text
Generic Actor
Drowned
Signal Wraith
Boss framework
SoundCue
Acoustic Mapping
Fog presentation
water terrain tag
MapKnowledge
```

### 暂停

用户保留决策权：

```text
最终场景 polygon
最终 explored-room boundaries
Pump Station 精确对象
water-level coverage
shortcut topology
```

不要让 Codex 猜。

---

# 58. 雾港地图边界输入格式

以后用户只需要提供：

```json
{
  "sceneId":"mh_pump_station",

  "walkablePolygons":[...],

  "rooms":[...],

  "waterRegions":[...],

  "pumpControls":[...],

  "shortcuts":[...]
}
```

最好直接：

```text
Tiled .tmj
```

成为唯一权威。

---

# 59. 泵站规则也不要写死

建议 Pump Interaction 数据：

```json
{
  "id":"mh_pump_control_a",

  "stateMachine":[
    "offline",
    "ready",
    "running",
    "complete"
  ],

  "requirements":[
    "event.mh_power_restored"
  ],

  "effects":[
    {
      "type":"waterLevel",
      "region":"mh_pump_lower",
      "target":0.25
    }
  ]
}
```

具体：

```text
region
target
前置
shortcut
```

等用户提供。

---

# 60. Clockworks

当前 staged。

它反而非常适合成为：

> 新 Effect / Terrain / Hazard System 的第一个正式使用者。

例如：

```text
heat
pressure
conveyor
steam
```

全部应该是通用：

```text
Hazard
Terrain
Movement
```

而不是 Clockworks 专用代码。

---

# 61. 例如 Conveyor

Terrain：

```text
surface.conveyor
```

默认：

```text
apply velocity
```

悬浮能力：

```text
ignore surface conveyor
```

于是以后其他世界：

```text
ice
river current
gravity floor
```

都复用。

---

# 62. Heat

Clockworks：

```text
hazard.heat
```

以后：

```text
lava
desert
reactor
starship
```

全部复用。

---

# 63. 当前美术阻塞

你提供的当前状态：

```text
30/30 正式 Scene Plate 缺失
```

这是独立于 Effect System 的发布阻塞。

不能等待代码全部完成才做。

美术 Slot 可以并行生产：

```text
Grey Hive scenes
Cenyao animation
Mist Harbor
Clockworks
```

只要不会触碰未确定的雾港精确几何。

---

# 64. 岑遥 88 格缺失怎么处理

当前规格：

```text
12 states × 8 directions
```

但不要理解成：

> 必须 AI 独立生成 96 张画。

Runtime Contract 可以仍然要求：

```text
state × direction
```

但内容可以来自：

```text
Cutout rig
procedural motion
derived frame
hand key pose
```

最终 atlas 能解析到每个：

```text
state/direction
```

即可。

---

# 65. 建议首发动画优先级

P0：

```text
Idle
Walk
Run
Attack
Dash
Hit
Death
```

P1：

```text
Pulse
Guard
Pierce
Interact
Traversal
```

不能为了 88 张图片阻塞整个 Gameplay。

---

# 66. Guard / Pierce VFX

当前正式图缺失。

但逻辑可以继续。

使用 temporary procedural Pixi VFX：

```text
Guard:
arc + impact ripple

Pierce:
line + streak
```

明确：

```text
temporary_visual = true
```

正式资产到位再替换。

---

# 67. 公布/发布资产阻塞

这个必须与内部 `release_approved` 分开。

当前：

```text
43 release_approved
```

只表示：

> 内部生产准入。

V1 对外还需要：

```text
PUBLIC_RELEASE_ASSET_MANIFEST
```

例如：

```json
{
  "assetId":"actor.cenyao",

  "runtimeSha256":"...",

  "sourceType":"ai_generated",

  "provider":"OpenAI",

  "redistributionStatus":"approved",

  "licenseNote":"..."
}
```

---

# 68. CI 最终禁止

任何 runtime asset：

```text
没有 public manifest entry
```

时做公开 release。

内部 dev build 可以允许。

---

# 69. Windows Native 验收阻塞

251/251 Web tests：

只证明：

```text
Web/component/protocol
```

不能代表：

```text
Tauri runtime
WebView2
actual input
fullscreen/window
save filesystem
installer
GPU
```

---

# 70. 建议添加 Native Acceptance Harness

用户本机可以运行：

```text
tools/native-e2e/
```

测试：

```text
launch
window detect
keyboard
mouse
save
close
restart
continue
```

优先使用本机已有的 Windows UI Automation 能力。

如果自动截图仍不可用：

生成：

```text
acceptance checklist
+
automatic state log
```

最后只让用户：

```text
看 3 个分辨率
走 3 个世界
确认视觉/手感
```

---

# 71. Native E2E 必须绑定 Build Identity

游戏 F3 / logs：

```text
Git SHA
Build Version
Content Version
Save Version
Asset Manifest SHA
Scene Bundle SHA
```

否则：

> “我测试的是不是刚构建的版本”

永远不确定。

---

# 72. 分支问题

当前：

```text
release/v1-three-worlds-2d5
```

继续作为唯一正式 integration line。

不要现在强行变 main。

以后：

```text
v1 Release Gate passed
↓
tag
↓
promote main
```

---

# 73. 历史分支清理

不影响 gameplay。

所以优先级：

```text
P3
```

规则：

```text
dirty worktree
→ 不删

untracked unique files
→ 不删

unique commit
→ tag/archive

fully superseded + clean
→ delete candidate
```

不要花正式内容开发时间凑分支数量。

---

# 74. Save V6

我建议这次升级。

原因不是为了数字好看，而是：

> 这是第一次 Formal Runtime 真正拥有完整角色 Build。

新增：

```rust
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
```

---

# 75. 不存 EffectivePlayerRules

Save：

存：

```text
Sources
```

不存：

```text
resolved output
```

Load：

```text
Save
↓
validate
↓
resolve effects
↓
EffectivePlayerRules
```

这是必须遵守的。

---

# 76. v5 → v6

迁移：

现有：

```text
capability
inventory
```

保留。

新增内容：

默认：

```text
no equipment
no bloodline
no skills
```

如果能从 legacy GameState 迁移：

才迁。

不能猜。

---

# 77. Content Registry

现在旧数据还在 Rust 静态表。

V1 不要求一次性重写 382 条数据。

先做：

```text
Legacy Content Adapter
```

让：

```text
SkillDef
GearDef
BloodlineDef
...
```

转换成新模型。

---

# 78. 后续再逐渐数据化

未来：

```text
content/
    skills/
    bloodlines/
    items/
    gear/
```

JSON。

不要为了架构美观阻塞 V1。

---

# 79. 新内容必须使用新格式

规则：

```text
旧内容
→ Adapter allowed

新内容
→ EffectSpec native
```

这样技术债不会继续增长。

---

# 80. 示例：后视装备

```json
{
  "schemaVersion": 1,

  "id": "gear.rear_sensor_mk1",

  "name": "全向感知镜片",

  "slot": "accessory",

  "effects": [
    {
      "type":"perception",
      "kind":"rearView",
      "rank":1,
      "rangeM":12
    }
  ]
}
```

---

# 81. 示例：完整地图装备

```json
{
  "id":"gear.cartographer_array",

  "effects":[
    {
      "type":"mapKnowledge",

      "topology":"full",
      "terrain":"full",
      "connections":"full",

      "objectives":"known",
      "enemies":"none",
      "secrets":"none"
    }
  ]
}
```

---

# 82. 示例：悬浮靴

```json
{
  "id":"gear.hover_boots",

  "effects":[
    {
      "type":"terrain",

      "ignoreMovementPenalty":[
        "terrain.water_shallow",
        "terrain.water_deep",
        "terrain.mud",
        "terrain.rubble",
        "surface.conveyor"
      ]
    }
  ]
}
```

---

# 83. 示例：无需呼吸

```json
{
  "id":"body.no_breath_i",

  "effects":[
    {
      "type":"hazardImmunity",
      "hazard":"drowning"
    },
    {
      "type":"hazardImmunity",
      "hazard":"suffocation"
    }
  ]
}
```

---

# 84. 示例：危险预测

```json
{
  "id":"perception.danger_prediction_i",

  "effects":[
    {
      "type":"perception",
      "kind":"hazardPrediction",
      "leadTimeMs":500
    }
  ]
}
```

Rust 提前 500ms 给：

```text
authorized telegraph
```

不是前端自己读敌人未来状态。

---

# 85. 示例：空间储物

```json
{
  "id":"space.storage_i",

  "effects":[
    {
      "type":"inventory",
      "kind":"capacityAdd",
      "slots":20
    }
  ]
}
```

后续高阶：

```text
weightIgnore
remoteAccess
```

也能扩。

---

# 86. 前端应该得到什么

不要发：

```text
player has dragon blood
player has hover boots
```

然后让 Web 推导。

Web 应该得到：

```json
{
  "perception":{
    "rearView":true
  },

  "map":{
    "topology":"full"
  },

  "movement":{
    "modes":["ground"]
  },

  "terrain":{
    "ignoredPenalties":[
      "water",
      "mud"
    ]
  }
}
```

以及独立：

```text
LoadoutProjection
```

给 UI 显示装备名。

---

# 87. UI 仍然可以告诉玩家来源

比如 Character：

```text
后视感知
来源：
全向传感器
```

但来源来自：

```text
SourceProjection
```

不参与 gameplay 判断。

---

# 88. Effect Resolver 测试

必须做 table-driven。

例如：

```text
RearView source A on
→ enabled

A + B
→ enabled

remove A
→ enabled

remove B
→ disabled
```

---

# 89. Map tests

```text
No map
→ no rooms

Local map
→ explored only

Full topology
→ all rooms

Full topology + enemy none
→ no enemies

Enemy detector
→ only detected enemies
```

---

# 90. Terrain tests

```text
normal player + water
→ slowed

hover boots + water
→ normal

hover boots + wall
→ blocked

phase + normal wall
→ pass

phase + world boundary
→ blocked
```

---

# 91. Effect order determinism

输入 Sources 顺序改变：

```text
A B C
C B A
```

ResolvedRules：

必须 byte-equivalent。

防止未来装备顺序改变结果。

---

# 92. 当前主线重新调整

现在近期主线建议从：

```text
直接完成 Mist Harbor
```

改成：

```text
POWER SYSTEM V2
↓
Formal Build Integration
↓
Generic Actor
↓
Terrain / Map Rules
↓
Grey Hive regression
↓
Mist Harbor
↓
Clockworks
```

---

# 93. 但不能因此暂停美术

两个 Slot：

## Slot A

```text
架构 / gameplay
```

## Slot B

```text
美术 / content / audio
```

继续并行。

---

# 94. Work Package 01

# `POWER-SYSTEM-V2-01`

Branch：

```text
codex/power-system-v2-01
```

实现：

```text
EffectSpec
EffectSource
EffectLifetime
StackRule
EffectResolver
EffectivePlayerRules
```

修改：

```text
server-rs/src/effects/*
server-rs/src/player_rules/*
```

不要改世界内容。

---

# 95. DoD

必须仅用数据完成：

```text
Rear View item
Full Map item
Hover Terrain item
Heat Resistance bloodline
```

且禁止出现：

```rust
if source_id == "gear.xxx"
```

---

# 96. Work Package 02

# `FORMAL-BUILD-V6-01`

内容：

迁入：

```text
inventory
equipment
skills
bloodlines
treasures
proficiency
```

Save V6。

---

# 97. DoD

New：

```text
equip item
save
close
continue
```

Rules 一致。

Unequip：

Effect 立即失效。

---

# 98. Work Package 03

# `LEGACY-CONTENT-ADAPTER-01`

处理：

```text
161 skills
82 weapons
50 gear
33 treasures
34 items
22 bloodlines
```

旧数据。

不要求全部首发可获得。

目标：

> 能统一进入 Effect Resolver。

---

# 99. IP 风险数据

这里需要特别注意：

当前旧内容包含：

```text
Saiyan
Sharingan
Shinigami
Quincy
...
```

这些可以：

```text
保留在历史/archive设计中
```

但不能自动因为 Effect Adapter 做好了，就进入 V1 公开发行内容。

正式 Content Registry 增加：

```text
releaseEligible
```

或由 public content manifest 控制。

---

# 100. Work Package 04

# `GENERIC-ACTOR-RUNTIME-01`

把：

```text
sentinels only
```

升级为：

```text
actors
```

实现普通敌人 AI。

Sentinel 保持 Boss controller。

---

# 101. Work Package 05

# `MAP-KNOWLEDGE-TERRAIN-01`

实现：

```text
MapKnowledgeRules
TerrainTags
MovementModes
HazardTags
```

然后把：

```text
Mist Harbor water
Clockworks heat/conveyor
```

接入。

---

# 102. Work Package 06

# `GH-EFFECT-REGRESSION-01`

保证：

```text
Local Map
Rear View
Enemy Vitals
Regeneration
```

行为不退化。

现有 Scenario 必须继续全部通过。

---

# 103. Work Package 07

# `MIST-HARBOR-GAMEPLAY-01`

不需要用户输入即可完成：

```text
Drowned
Signal Wraith
Warden
Fog
Sound
Acoustic
Water rules
```

---

# 104. Mist Harbor User Blocker

等待用户输入：

```text
精确 Tiled Geometry
Pump Interaction
Water Level Regions
Shortcut topology
```

一旦到位：

```text
只填数据
```

不应再次写新的 Pump 专用架构。

---

# 105. Work Package 08

# `CLOCKWORKS-GAMEPLAY-01`

实现：

```text
Heat
Pressure
Conveyor
Steam
Moving machinery
```

全部使用通用：

```text
Terrain
Hazard
Interaction State Machine
```

---

# 106. Work Package 09

# `PUBLIC-ASSET-GATE-01`

实现：

```text
internal approved
≠
public approved
```

CI：

发布 build 必须只有：

```text
publicReleaseApproved
```

资产。

---

# 107. Work Package 10

# `NATIVE-WINDOWS-ACCEPTANCE-01`

完成：

```text
install
launch
New Journey
Grey Hive
Return
Mist Harbor
Return
Clockworks
Save
Exit
Continue
Quit
uninstall
```

三个分辨率。

---

# 108. 并行计划

建议现在：

| Slot | 任务 |
|---|---|
| A | POWER-SYSTEM-V2-01 |
| B | 正式美术生产 / Scene Plate / Cenyao 动作 / Audio |

POWER 完成：

| Slot | 任务 |
|---|---|
| A | FORMAL-BUILD-V6 |
| B | Generic Actor |

之后：

| Slot | 任务 |
|---|---|
| A | Map/Terrain |
| B | Mist Harbor gameplay/art |

然后：

```text
Clockworks
```

---

# 109. V1 不需要实现的未来能力

架构要支持，但不必 V1 做：

```text
time stop
causality
rule rewrite
soul possession
dimension travel
full phase
free flight
teleport
precognition
```

不要为了证明架构而增加 Scope。

---

# 110. V1 应该做的 Effect 覆盖样本

为了证明系统真的适合无限流，V1 至少正式存在：

```text
Rear View
Local Map
Enemy Vitals
Regeneration
Acoustic Mapping
Air Step

Map Fog Bypass test item
Terrain Penalty Ignore test item
Heat Resistance
Water Movement modifier
```

前六个是正式成长。

后四个可作为：

```text
内部测试内容
```

不一定交给玩家。

---

# 111. 当前所有发布阻塞的最终清单

## P0 工程阻塞

```text
旧强化体系未接 Formal Runtime
没有 Unified Effect Resolver
没有 Generic Actor Runtime
Map Knowledge 不够通用
Terrain/Hazard 规则未统一
Save v5 不保存完整 Build
```

---

## P0 内容阻塞

```text
Mist Harbor 世界入口关闭
Mist Harbor Boss/敌人闭环不足
Mist Harbor Pump/边界等待用户
Clockworks staged
三世界连续 Campaign 尚未成立
```

---

## P0 美术阻塞

```text
30 Scene Plates
Cenyao animations
Boss assets
Guard/Pierce final VFX
key props
audio
```

---

## P0 发布阻塞

```text
public asset eligibility
native Windows E2E
installer
performance
3 resolution native screenshots
```

---

# 112. 明确哪些东西不是阻塞

这些暂时不应该阻塞当前开发：

```text
main 尚未提升
历史分支很多
第四世界不存在
未来全部 161 Skill 尚未实现
所有 22 Bloodline 尚未可玩
全部 UI 最终 polish
```

---

# 113. 核心工程原则

以后 Codex 必须遵守：

> **不得为了一个新道具直接在 `formal_runtime.rs` 中按 item id 写特殊分支。**

禁止：

```rust
if item == "hover_boots" {
    ...
}
```

允许：

```text
hover_boots
↓
EffectSpec
↓
TerrainRules
```

---

# 114. 同理 Bloodline

禁止：

```rust
if bloodline == "dragon" {
    heat_damage /= 2;
}
```

必须：

```text
dragon
↓
HazardResistance
↓
Heat 0.5
```

---

# 115. 同理 UI

禁止：

```text
装备某道具
↓
前端直接显示完整地图
```

必须：

```text
Item
↓
Rust Rule
↓
MapProjection
↓
UI
```

---

# 116. 最终目标

完成这一轮架构后，《无限人族进化》的内容生产应该变成：

```text
设计一个新强化
↓
定义 Source
↓
组合现有 Effects
↓
配置数据
↓
添加视觉/音频
↓
自动测试
```

只有真正出现一个此前无法表达的新“规则类型”时：

> 才新增新的 Effect Variant。

而不是每加入一个道具都写代码。

---

# 117. 对未来无限流规模的实际意义

你当前库里已经有几百条旧内容定义。

以后真正可能变成：

```text
500 Skills
100 Bloodlines
300 Items
100 Weapons
几十个世界
```

这时可维护性取决于：

```text
“有多少效果类型”
```

而不是：

```text
“有多少内容”
```

理想情况是：

```text
1000 个内容
```

只组合：

```text
30–60 种基础 Effect
```

这就是整个系统长期能够继续发展的关键。

---

## 最终执行优先级

从当前 `dc7d8c58` 开始，我建议 Codex 把近期主线正式调整为：

```text
P0
Unified Effect / Player Rule System

P0
Formal Build + Save V6

P0
Generic Actor Runtime

P0
Map Knowledge + Terrain/Hazard

P0
Grey Hive Regression

P1
Mist Harbor 完整 gameplay
（几何/泵站严格等待用户权威数据）

P1
Clockworks 完整 gameplay

并行 P0/P1
三世界正式美术 + Audio

P0 Release
Public Asset Gate

P0 Release
Real Windows Native Acceptance
```

这个顺序既不会推翻当前已经做好的三世界 V1，也能避免为了完成前三个世界，把项目架构永久固定成只能支持几个硬编码能力的短期版本。

**三世界是 V1 内容目标；Unified Effect / Player Rules 是这个“无限流”游戏能继续开发十几个、几十个世界的基础。**