# 《无限人族进化》V1 UI / Visual System Implementation Specification

**Document Type:** Codex Execution Specification  
**Target:** GPT-6 Sol High Planning Agent + GPT-6 Luna High Execution Agent  
**Status:** Frozen Visual Direction  
**Version:** V1.0

---

# 1. 总体视觉定位

## 1.1 产品视觉定义

《无限人族进化》采用：

> **固定斜视角 2.5D 高精度电影感 Action RPG**

视觉组成：

```
Hero Scene Plate
+
Modular Props
+
Sprite Actor
+
Industrial Tactical UI
+
World-specific Theme Skin
```

目标效果：

- 不是传统 2D 游戏；
- 不是 3D TPS；
- 不是像素 RPG；
- 不是纯 Tile 地图；
- 不使用整屏 AI UI 图片作为运行时 UI。

最终视觉目标：

```
成熟商业 2.5D 动作 RPG
+
工业科幻世界观
+
高信息可读性
+
可扩展多世界结构
```

---

# 2. UI 总体架构

## 2.1 核心原则

UI 必须：

- 代码化；
- 数据驱动；
- 主题化；
- 世界可换皮；
- 不绑定具体图片。

禁止：

```
一个世界 = 一套完全独立 UI
```

必须：

```
Core UI Framework
        |
        |
Theme Layer
        |
--------------------------------
Grey Hive
Mist Harbor
Clockworks
Future Worlds
```

---

# 3. UI 技术实现

## 3.1 屏幕 UI

使用：

```
HTML
CSS
TypeScript
```

负责：

- Main Menu
- Save Slot
- HUD
- Pause
- Inventory
- Character
- Capability
- Mission
- World Network
- Settings

---

## 3.2 世界 UI

使用：

```
PixiJS
```

负责：

- Interaction Prompt
- Enemy HP
- Damage Number
- World Marker
- Telegraph
- VFX UI
- Floating Text

---

# 4. UI 数据结构

目录：

```
apps/web/src/ui/

components/

    HUD.ts
    SkillBar.ts
    ObjectivePanel.ts
    InteractionPrompt.ts
    Inventory.ts
    CapabilityTree.ts
    WorldNetwork.ts
    SaveSlots.ts
    PauseMenu.ts


theme/

    ThemeManager.ts
    ThemeSchema.ts


themes/

    base/
    grey_hive/
    mist_harbor/
    clockworks/


assets/ui/

    icons/
    frames/
    textures/
```

---

# 5. Theme 系统

## 5.1 Theme Schema

```json
{
  "id":"grey_hive",

  "colors":{
    "background":"#0B121A",
    "panel":"#16202A",
    "border":"#344652",
    "system":"#58D7F0",
    "accent":"#D9AD58",
    "danger":"#E1504C"
  },

  "panel":{
    "cornerRadius":8,
    "borderWidth":1
  },

  "iconStyle":"industrial",

  "mapStyle":"facility",

  "interactionStyle":"terminal"
}
```

---

# 6. 全局 UI 主风格

## 6.1 主风格

采用：

```
工业战术终端
```

参考：

- 灰巢工业 UI；
- 科幻 HUD；
- Return Station Terminal。

---

## 6.2 颜色规范

### 主背景

```
#0B121A
```

用途：

- 页面背景；
- HUD 背板。

---

### 面板

```
#16202A
```

---

### 边框

```
#344652
```

---

### 科技信息

```
#58D7F0
```

用途：

- Energy
- Scanner
- System
- Map

---

### 当前交互

```
#D9AD58
```

用途：

- 当前选项；
- F Interaction；
- Objective；
- Reward。

---

### 危险

```
#E1504C
```

用途：

- Damage；
- Enemy Warning；
- Error。

---

# 7. UI 材质规范

所有 UI：

采用：

```
工业终端
```

特点：

- 深色背景；
- 细边框；
- 小切角；
- 少量发光；
- 克制。

禁止：

- 大面积玻璃透明；
- 霓虹 Cyberpunk；
- 哥特花纹；
- 大量装饰。

---

# 8. HUD 规范

## 8.1 左上角

内容：

```
Character Portrait

Name

HP

Energy

Status
```

不显示：

- 等级；
- XP；
- RPG 属性。

---

## 8.2 右上角

内容：

```
Current Objective
```

最多：

```
主目标
+
两个子目标
```

---

## 8.3 底部中央

固定：

```
Q Pulse

E Guard

R Pierce

Shift Dash
```

尺寸：

```
48~56 px
```

---

## 8.4 Interaction

只有靠近时出现：

```
[F]

启动主电控制台
```

---

# 9. Skill UI

## 9.1 Pulse

视觉语言：

```
扫描
感知
扩散
```

Icon：

圆形波纹。

---

## 9.2 Guard

视觉语言：

```
防御
稳定
屏障
```

Icon：

弧盾。

---

## 9.3 Pierce

视觉语言：

```
速度
穿透
方向
```

Icon：

线性裂解。

---

## 9.4 Dash

视觉语言：

```
移动
残影
速度
```

Icon：

箭头残影。

---

# 10. 世界主题 Skin

---

# 10.1 Grey Hive Theme

定位：

```
工业设施
生物隔离
电力恢复
```

颜色：

```
Dark Steel
Blue Grey
Cyan
Amber
```

UI：

- 工业终端；
- 电力控制；
- 生物安全。

特殊组件：

```
Power Status
Containment Status
Facility Map
```

---

# 10.2 Mist Harbor Theme

定位：

```
雾
海
信号
导航
```

颜色：

```
Deep Blue
Sea Green
Warm Lamp
```

新增：

```
Sonar Map

Signal Strength

Navigation UI
```

特殊组件：

```
Signal Terminal
Acoustic Scan
Water Level
```

---

# 10.3 Clockworks Theme

定位：

```
机械
压力
熔炉
工业文明
```

颜色：

```
Dark Steel
Brass
Orange
Red Warning
```

新增：

```
Pressure Gauge

Heat Meter

Steam Warning
```

特殊组件：

```
Pressure Control
Valve Status
Heat Warning
```

---

# 11. UI 页面规范

---

# 11.1 Main Menu

结构：

左侧：

```
新的旅程

继续

设置

退出
```

背景：

Return Station。

要求：

- 小型工业基地；
- World Gate；
- 少量动态灯光。

---

# 11.2 Return Station Terminal

顶部：

```
世界网络
能力
任务
档案
系统
```

---

内容：

## 世界网络

显示：

```
Grey Hive
Mist Harbor
Clockworks
Unknown
```

状态：

```
Unknown
Unlocked
Completed
Revisit
```

---

## 能力

显示：

```
Human Evolution Tree
```

分类：

```
Perception

Information

Body

Mobility

Combat

Space

Rule
```

---

# 11.3 Inventory

结构：

左：

```
Item Grid
```

中：

```
Equipment
```

右：

```
Description
```

---

# 11.4 Character

显示：

```
Portrait

Body State

Capabilities

Equipment
```

禁止：

传统：

```
STR
DEX
INT
```

---

# 11.5 Capability

核心：

```
从普通人
到超越限制
```

节点：

```
Perception

Information

Body

Mobility

Space

Rule
```

---

# 12. 岑遥角色规范

## 12.1 基础设定

姓名：

```
岑遥
```

定位：

```
普通人类探索者
```

---

## 12.2 外观

必须：

- 短黑发；
- 成年女性；
- 深炭灰短外套；
- 青灰内层；
- 暗金单肩结构；
- 胸前金色菱形信标；
- 深色工装裤；
- 工业靴。

---

禁止：

- 长发；
- 红披风；
- 性感幻想服；
- 高跟鞋；
- 魔法少女风。

---

# 13. 角色动画规范

## 必须实现：

```
Idle

Walk

Run

Primary Attack

Dash

Pulse

Guard

Pierce

Hit

Death

Interact

Context Traversal
```

---

# 14. 动画技术

采用：

```
8方向 Sprite/Cutout
+
关键动作帧
+
程序化移动
```

---

普通：

```
Idle
Walk
Run
```

使用：

Cutout。

---

战斗：

```
Attack
Dash
Pulse
Guard
Pierce
```

使用：

关键动作帧。

---

# 15. Scene Visual Pipeline

正式场景：

```
Background

Floor

Back Props

Actors

Dynamic Props

Front Props

Occluders

VFX

World UI
```

---

# 16. Camera

固定：

```
2.5D Isometric
```

禁止：

```
RMB旋转
自由TPS视角
```

---

Camera：

支持：

- Smooth Follow；
- Dead Zone；
- Aim Offset；
- Boss Zoom。

---

# 17. Occlusion

前景遮挡：

进入：

```
alpha 1.0
```

玩家经过：

```
alpha 0.25
```

退出：

恢复。

---

# 18. Lighting

采用：

```
Baked Scene Lighting

+
Pixi Additive Light

+
Fog

+
Particles
```

禁止：

重新做 3D PBR。

---

# 19. Asset 使用规则

AI 图片：

用途：

允许：

```
Concept

Texture

Icon

Portrait

Scene Reference
```

禁止：

```
整张 AI UI 图片直接进入 Runtime
```

---

# 20. 资产结构

```
assets/

source/

    ai_generated/


runtime/

    actors/
    worlds/
    props/
    ui/
    vfx/
```

---

# 21. Asset Pipeline

流程：

```
Source PNG

↓

SHA Verify

↓

Release Manifest Check

↓

Crop

↓

Alpha Cleanup

↓

Resize

↓

Atlas

↓

Runtime Asset
```

---

# 22. 新世界扩展规则

增加第四世界：

只需要：

新增：

```
themes/new_world/

world assets/

scene data/
```

不用重写：

- HUD；
- Menu；
- Inventory；
- Skill UI；
- Save UI。

---

# 23. Codex 执行要求

实现顺序：

## Phase 1

UI Framework

完成：

- Theme System
- HUD
- Menu
- Terminal

---

## Phase 2

Grey Hive Skin

完成：

- Power UI
- Gate UI
- Facility Interaction

---

## Phase 3

Mist Harbor Skin

完成：

- Sonar
- Signal
- Navigation

---

## Phase 4

Clockworks Skin

完成：

- Pressure
- Heat
- Steam Warning

---

## Phase 5

Polish

完成：

- Animations
- VFX
- Sound Binding
- Resolution Adaptation

---

# 24. Definition of Done

视觉系统完成标准：

- [ ] 所有 UI 使用组件实现；
- [ ] 无整屏 AI UI 图片；
- [ ] 三世界共享 UI Framework；
- [ ] Theme 可热切换；
- [ ] 1280×720 正常；
- [ ] 1920×1080 正常；
- [ ] 2560×1440 可用；
- [ ] Grey Hive 有完整工业 Skin；
- [ ] Mist Harbor 有导航 Skin；
- [ ] Clockworks 有机械 Skin；
- [ ] 新世界增加只需新增 Theme；
- [ ] Codex 不需要重新设计 UI。

---

# 最终冻结结论

《无限人族进化》V1：

```
一个 UI Framework

+
三个 World Theme

+
无限未来 Theme
```

而不是：

```
三个世界
三套游戏
三套代码
```

这样才能匹配“无限世界进化”的长期设计目标。