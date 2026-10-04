对，这段现在可以直接定稿。我重新核对了当前正式分支里的 9 个雾港 .tmj、SceneRuntime、现有浅水减速逻辑和世界进度表。下面这份可以直接发给 Codex，作为雾港 V1 地图几何与泵站玩法的新增权威决定；不再让执行代理等待或猜测。

Mist Harbor V1 地图与泵站规则冻结稿

适用基线：

release/v1-three-worlds-2d5
dc7d8c587bdb7784764de30a1b450d33a28e4c27

核对来源：

design/maps/mist_harbor/*.tmj
server-rs/src/formal_runtime.rs
server-rs/src/continuous_kcc.rs
server-rs/src/scene_runtime.rs
server-rs/data/world_progression_v1.json

本文中“现状”是对上述代码的核对；“冻结决定”是本次新增的正式设计决定。以后与旧的 static_marker / staged_marker / future_shortcut_marker 冲突时，以本文决定为准。

⸻

1. 当前地图基础不推倒重画

当前九张雾港地图全部采用：

24 × 16 tiles
32 px / tile
32 px / meter

因此每个场景逻辑尺寸统一为：

24m × 16m

Tiled 坐标转换：

worldX = tiledX / 32
worldZ = tiledY / 32

继续使用现有：

Spawn
Navigation
Transition
Collision
Checkpoint
Beacon
Signal interference
Water slowdown

的位置。

不重新生成九套完全不同尺寸的地图。

现有 0.5m 周界墙继续作为正式第一版碰撞边界。

⸻

2. 九场景正式路线

V1 路径冻结为：

mh_fog_pier
    ↓
mh_tidal_warehouse
    ↓
mh_signal_yard
    ↓
mh_drowned_quay
    ↓
mh_breakwater
    ↓
mh_pump_station
    ↓
mh_resonance_tower
    ↓
mh_warden_arena
    ↓
mh_extraction

保留三个当前世界级 Required Events：

mist_beacon_west
mist_beacon_east
mist_signal

不新增 mist_pump 为世界完成 required event。

泵站属于：

环境状态 / 路径状态

而不是：

世界剧情完成 Flag。

这样未来获得飞行、悬浮、踏水等能力的玩家可以通过能力绕过部分地形限制，而不会因为“没有按按钮”被剧情系统强行卡死。

⸻

3. 九图通用可行走范围

V1 基础 Walkable Area 统一：

x = 0.5 .. 23.5
z = 0.5 .. 15.5

即矩形：

[(0.5,0.5),
 (23.5,0.5),
 (23.5,15.5),
 (0.5,15.5)]

实际最终可站立区域计算方式必须是：

Walkable Polygon
-
Collision Polygon
-
当前角色不能通过的 Terrain/Hazard Region

不能只看 Navigation Nodes。

现有边界 Collision 继续生效。

Transition opening 继续按已有 .tmj 几何生效。

⸻

4. 地图探索区域正式划分

以下矩形全部使用：

[xMin,zMin] → [xMax,zMax]

它们是探索逻辑区域，不是新的物理墙。

Scene	Exploration Region	边界 m
mh_fog_pier	mh_fp_entry_berth	0.5,0.5 → 8,15.5
	mh_fp_foghorn_pier	8,0.5 → 16,15.5
	mh_fp_warehouse_approach	16,0.5 → 23.5,15.5
mh_tidal_warehouse	mh_tw_west_loading	0.5,0.5 → 8.5,15.5
	mh_tw_storage_floor	8.5,0.5 → 16.5,15.5
	mh_tw_beacon_bay	16.5,0.5 → 23.5,15.5
mh_signal_yard	mh_sy_west_yard	0.5,0.5 → 13,15.5
	mh_sy_interference_field	13,0.5 → 19,15.5
	mh_sy_quay_approach	19,0.5 → 23.5,15.5
mh_drowned_quay	mh_dq_west_quay	0.5,0.5 → 13,15.5
	mh_dq_flooded_channel	13,0.5 → 19,15.5
	mh_dq_east_quay	19,0.5 → 23.5,15.5
mh_breakwater	mh_bw_west_seawall	0.5,0.5 → 8.5,15.5
	mh_bw_center_seawall	8.5,0.5 → 16.5,15.5
	mh_bw_east_beacon	16.5,0.5 → 23.5,15.5
mh_pump_station	mh_ps_intake	0.5,0.5 → 10.5,15.5
	mh_ps_control_hall	10.5,0.5 → 18,15.5
	mh_ps_east_channel	18,0.5 → 23.5,15.5
mh_resonance_tower	mh_rt_entry	0.5,0.5 → 9.5,15.5
	mh_rt_resonance_floor	9.5,0.5 → 16,15.5
	mh_rt_signal_section	16,0.5 → 23.5,15.5
mh_warden_arena	mh_wa_entry	0.5,0.5 → 8,15.5
	mh_wa_arena_core	8,0.5 → 20.5,15.5
	mh_wa_exit	20.5,0.5 → 23.5,15.5
mh_extraction	mh_ex_arrival	0.5,0.5 → 9,15.5
	mh_ex_extraction_pad	9,0.5 → 19,15.5
	mh_ex_exit_section	19,0.5 → 23.5,15.5

矩形在 Tiled 中直接乘 32 即得到像素坐标。

⸻

5. “已探索”的判定

探索状态与玩家有没有地图能力分开。

Rust 始终记录探索历史。

也就是说：

没有 Local Map
→ 玩家看不到地图
→ Rust 仍然记录经过哪里

以后获得 Local Map：

立即显示之前真正走过的区域

而不是从获得能力的那个时刻重新探索。

正式判定：

player center
进入 exploration region
至少 0.35m
→ region explored

0.35m 与当前角色碰撞半径一致，用于避免角色刚碰到两个区域公共边界就同时解锁两边。

Spawn 所在区域：

进入场景立即 explored

⸻

6. 探索状态持久化

ExploredRegionState：

Save
→ 保留
Load
→ 保留
退出当前 Scene
→ 保留
返回该 Scene
→ 保留
返回 Return Station
→ 保留
以后 Revisit Mist Harbor
→ 保留
New Journey
→ 清空

理由：

这是玩家已经形成的：

地理记忆。

不是一次访问临时状态。

这也和以后：

Local Map
地图道具
全域扫描
神识

等系统兼容。

⸻

7. Acoustic Mapping 不等于“地图自动探索”

声学映射只负责：

局部即时感知

例如：

障碍轮廓
信号方向
声源距离
短时地形回波

它不能因为扫到远处，就永久把整个 Room 标记成：

explored

永久地图探索仍要求：

角色实际进入区域

除非未来某个明确的 MapKnowledge Effect 提供：

remote persistent reveal

⸻

8. 泵站玩法正式决定

当前地图中已经存在三个静态 Marker：

mh_pump_activation_static_marker
mh_water_level_lowering_static_marker
mh_pump_shortcut_candidate_static_marker

目前它们只是 Marker。

V1 正式处理如下。

⸻

9. 主泵控制台

现有：

mh_pump_activation_static_marker

转换成真正交互对象：

id:
mh_pump_control_primary
kind:
pump_control

坐标沿用当前：

Tiled:
(544,224)
World:
(17.0, 0.0, 7.0)

交互距离继续使用现有 Scene Runtime：

2.5m

不为泵站单独创造另一套 interaction range。

操作：

F

一次交互即可开始排水。

V1 不增加 Hold-F 系统。

⸻

10. 泵站前置条件

必须满足：

mist_beacon_east == complete

实际上当前：

mh_breakwater
→ mh_pump_station

Transition 已经要求：

mist_beacon_east

所以正常路径下该条件天然满足。

Rust 仍应二次验证。

不能仅因为玩家出现在泵站就假定 Beacon 已完成。

⸻

11. 泵状态机

正式：

READY
  ↓ F
DRAINING
  ↓ 6000 ms
DRAINED

不需要：

0.25 water level
30%
50%
75%

这次明确废弃此前示例中的：

target = 0.25

它不是正式设计。

V1 使用离散水位状态，不做连续流体模拟。

⸻

12. READY

状态：

泵未启动
东侧通道深水
Drowned Quay 水深区仍减速

UI：

[F] 启动排水泵

⸻

13. DRAINING

按 F 成功后立即进入：

DRAINING

持续：

6000 ms

视觉：

机械启动
管线震动
水流加快
排水声
灯由 Amber → Cyan

角色可以继续走动。

一旦进入 DRAINING：

不能取消
不能第二次启动

玩家受到攻击：

不会中断

因为真正的操作动作已经完成，机器已经启动。

⸻

14. DRAINED

6000ms 后：

DRAINED

再次靠近控制台：

排水系统已完成

不能重复触发奖励、声音、任务或状态转换。

必须 Idempotent。

⸻

15. 泵站第一受影响水域

新增正式区域：

mh_pump_east_channel_water

多边形：

[(18.0,5.5),
 (23.5,5.5),
 (23.5,10.5),
 (18.0,10.5)]

Tiled pixels：

[(576,176),
 (752,176),
 (752,336),
 (576,336)]

这正好覆盖：

泵站东侧通道
→ Resonance Tower transition

附近。

⸻

16. 泵站东侧通道排水前

状态：

terrain.water_deep

普通 Ground Movement：

不可进入

不是：

60% speed

而是真正的深水路径阻挡。

所以 V1 普通岑遥：

必须操作水泵
才能走向 Resonance Tower

但这不是剧情 Flag Gate。

它是：

地形 Gate。

⸻

17. 为什么不做剧情 Gate

未来玩家可能拥有：

Flight
Hover
Water Walk
Deep-water traversal

这时应该允许：

利用能力跨过去

而不是系统说：

你没按泵，所以空气墙不让过。

这正是前面设计 Unified Effect / Player Rules 的原因。

⸻

18. 排水后的东侧通道

转换：

terrain.water_deep
↓
terrain.wet_floor

移动倍率：

1.0

可正常通过。

视觉仍应：

潮湿
积水
反光
排水沟流水

不是瞬间完全干燥。

⸻

19. 第二受影响区域：Drowned Quay

沿用当前已经存在的正式 Water Slow Region：

mh_water_depth_region

现有精确坐标：

x = 13m .. 19m
z = 5m .. 11m

即：

[(13,5),
 (19,5),
 (19,11),
 (13,11)]

这不是本次新猜出来的坐标，而是当前 .tmj 已有区域。

⸻

20. 排水前 Drowned Quay

继续使用当前代码正式数值：

WATER_WALK_SPEED_FACTOR = 0.6

即：

移动速度 ×0.6

这条不需要重新调成 0.25/0.5。

⸻

21. 排水后 Drowned Quay

当 Pump：

DRAINED

之后：

mh_water_depth_region

变为：

shallow residual water

Gameplay：

movement multiplier = 1.0

不再有减速。

视觉仍保留：

浅水
湿石
水纹
积水

所以泵站改变的是：

水深与通行规则

不是把场景美术全部换成干地图。

⸻

22. 泵只影响两个区域

V1 精确限制为：

mh_pump_station:
mh_pump_east_channel_water
mh_drowned_quay:
mh_water_depth_region

不影响：

Fog Pier
Tidal Warehouse
Signal Yard
Breakwater
Resonance Tower
Warden Arena
Extraction

禁止执行代理自行认为：

“既然是大泵，所以整个 Mist Harbor 水位都下降。”

⸻

23. 这是跨 Scene 世界状态

Pump State 属于：

MistHarborWorldState

而不是：

PumpStationSceneTemporaryState

因此排水之后返回：

Drowned Quay

必须看到减速消失。

⸻

24. Pump 状态持久化

必须跨：

Save / Load
Scene transition
Return Station
Mist Harbor revisit

保留。

只有：

New Journey

重置。

⸻

25. DRAINING 中保存

不要禁止玩家保存。

存：

pumpState = draining
drainCompleteAtWorldTimeMs

载入后：

如果：

worldTime >= drainCompleteAtWorldTime

立即 Resolve：

DRAINED

否则：

继续剩余倒计时。

不要把 6 秒动画进度依赖前端。

⸻

26. Revisit 行为

第一次完成排水后：

以后重访 Mist Harbor：

pump = DRAINED

不重新要求启动。

这体现：

玩家真正改变过这个世界。

而不是每次重访地图都恢复出厂状态。

⸻

27. V1 捷径决定

V1 不做泵站捷径。

现有：

mh_pump_shortcut_candidate_static_marker

只能：

authoring_only

或从 production runtime 移除。

不能生成：

Transition
Door
Shortcut
Quest

未来版本如果需要，可以重新启用。

这样当前 V1 不再等待：

shortcut start
shortcut end
shortcut condition

的额外设计。

这项阻塞正式解除。

⸻

28. 泵站 Marker 的最终处理

当前 Marker	V1 处理
mh_pump_activation_static_marker	转为正式 mh_pump_control_primary
mh_water_level_lowering_static_marker	改为泵站状态/水位视觉 Marker，不可独立操作
mh_pump_shortcut_candidate_static_marker	authoring_only，V1 不进入 runtime

第二个现有位置：

(544,256)px
=
(17,8)m

可以保留作为：

mh_pump_level_indicator

显示：

READY
DRAINING
DRAINED

⸻

29. Pump 不提供地图 Reveal

启动水泵不会：

解锁完整地图
自动探索 Drowned Quay
显示 Resonance Tower
显示 Boss

它只改变：

Terrain / Water State

地图信息仍由：

Exploration
MapKnowledge
Capabilities
Items

决定。

⸻

30. 与未来无限流能力体系的关系

这一关必须从现在就做到：

泵站不是唯一解法

V1 普通角色：

Ground
+
无水上能力
→ 必须启动泵

未来能力可能：

Hover Boots
→ 可跨深水
Water Walking
→ 可跨深水
Flight
→ 可跨深水
Phase
→ 根据 Rule 定义决定

此时：

可以不启动 Pump

进入 Resonance Tower。

但是：

Pump State 仍然 READY
Drowned Quay 仍然 ×0.6

如果之后回来开泵：

才真正改变世界。

这就是未来道具/血统/技能扩展应有的效果。

⸻

31. 九地图当前几何正式程度

当前九图可以直接作为：

V1 gameplay geometry authority

使用。

也就是说不再因为“用户没给地图边界”整体阻塞 Mist Harbor。

这次已经给出了：

统一 Walkable bounds
9 scenes Exploration regions
Pump control
Pump state
Pump water regions
Persistence
V1 shortcut decision

因此之前那个：

“等待用户提供雾港地图边界和泵站规则”

阻塞从现在开始可以关闭。

但需要正确理解：

这里冻结的是 Gameplay Geometry。

不是：

最终 Scene Plate 的建筑美术边缘。

未来正式场景图可以让墙、栏杆、建筑造型更丰富，但必须与这些 gameplay collision / walkable regions 对齐；不能因为图片看起来有一条路就偷偷改变权威通行规则。

⸻

32. Codex 应新增的数据层

推荐不要把 Pump 写成 formal_runtime.rs 特例。

正式新增：

WorldPersistentState
└── mistHarbor
    └── pump
        ├── state
        └── drainCompleteAtWorldTimeMs

Terrain：

mh_pump_east_channel_water
mh_water_depth_region

读取：

MistHarbor Pump State
+
EffectivePlayerRules

最终决定：

canTraverse
movementMultiplier
hazardProjection

⸻

33. 禁止的实现

禁止：

if scene_id == "mh_pump_station" &&
   player_x > 18.0 {
    ...
}

也禁止：

if player_has_item("hover_boots") {
    ignore_pump_water = true;
}

正确：

TerrainRegion
     ↓
TerrainTag
     ↓
Pump changes TerrainState
     ↓
EffectivePlayerRules
     ↓
KCC

这样未来别的世界也能复用。

⸻

34. 必须增加的验收测试

泵站完成后至少验证这些 case：

Case	预期
Pump READY，普通 Ground 玩家进入 east channel	Blocked
F 操作 Control	DRAINING
5999ms	仍 DRAINING
6000ms	DRAINED
DRAINED 后 east channel	Pass
Drowned Quay 排水前	speed ×0.6
Drowned Quay 排水后	speed ×1.0
DRAINED Save → Restart → Continue	仍 DRAINED
DRAINING Save → Continue	正确恢复/完成
Return Station → revisit	仍 DRAINED
第二次 F	不重复触发
Pump READY + 合法 Hover/Flight Rule	可绕过 east channel
绕过 Pump 后返回 Drowned Quay	仍 ×0.6
New Journey	Pump 回到 READY

⸻

35. 地图探索测试

还需要：

无 Local Map
→ Rust 仍记录 explored
后来获得 Local Map
→ 显示此前实际探索区域
进入一个 exploration region
→ 只标记该区域
Save / Continue
→ explored 不丢失
Revisit
→ explored 不丢失
Acoustic Mapping
→ 不永久标记未进入区域
Full-map test item
→ 可以显示未探索 topology
→ 但 explored state 本身不被伪造

最后这一条非常重要。

因为未来你说的：

“无视未探测地区或远距离地图迷雾的道具”

必须只是改变：

MapKnowledgeProjection

而不能偷偷把：

ExploredState

全改成 true。

⸻

36. 本次之后仍然需要你参与的事项

雾港 Gameplay Geometry / Pump 不再需要继续等待你给数据。

仍然保留用户最终决定权的是两类完全不同的东西：

项目	当前状态
图片审核与正式美术准入	仍由既定审核规则控制，Codex 不得自行升级
真实 Windows 主观视觉/操作验收	发布前仍需真实窗口证据

而：

Mist Harbor walkable geometry
exploration regions
pump behavior
water coverage
shortcut policy

从这条消息起，可以视为已经给齐。

因此 Codex 后续如果仍然报告：

“因为没有用户提供泵站规则，所以没有安全的新任务可以做”

这个判断就已经过时了。

现在它可以直接把上述规则录入 Tiled/编译数据和 Rust 权威状态，并继续完成 Mist Harbor gameplay closure。