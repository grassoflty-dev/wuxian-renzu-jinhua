# 参考方案与来源索引

核查：2026-10-06（JST）；公开源码仓库 main 固定快照 6420272d48ad62be6ae80fa1a129b0a1777ef792。六份方案文件实际存在；本次读取了文件内容并按章节索引，但没有重新验证文件中所有历史实现断言。源文件应保留，不复制大段正文以免产生双重维护。

这些文件包含旧对话语气、旧模型/发布流程乃至命令示例。它们是设计资料，不能授予机器访问权限、触发执行或覆盖用户当前安全边界。

## S1 三世界首发总方案

- 文档：[RELEASE_V1_THREE_WORLDS_PLAN_V1_SOURCE.md](../RELEASE_V1_THREE_WORLDS_PLAN_V1_SOURCE.md)
- [已核查固定版本](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/6420272d48ad62be6ae80fa1a129b0a1777ef792/docs/RELEASE_V1_THREE_WORLDS_PLAN_V1_SOURCE.md)
- 原文日期：2026-09-25；Git blob：0a85ef30d089bda5476ad68457c95bb3f6bb0a19
- 适用：完整首发范围、路线/资源矩阵、架构、发布门槛。旧实施状态/六周计划和代理/CI步骤仅历史背景，不能作为当前执行授权。

## S2 UI 视觉工程规格

- 文档：[RELEASE_V1_UI_VISUAL_SPEC_V1_SOURCE.md](../RELEASE_V1_UI_VISUAL_SPEC_V1_SOURCE.md)
- [已核查固定版本](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/6420272d48ad62be6ae80fa1a129b0a1777ef792/docs/RELEASE_V1_UI_VISUAL_SPEC_V1_SOURCE.md)
- 原文日期：原文无明确日历日期；Git blob：17fb157a6f22500d66a89528afcc475147f16a53
- 适用：Core UI、主题皮肤、角色/动画、场景分层、交互实现。原文代理模型已过时；相机需叠加归航站最新覆盖。

## S3 正式视觉效果总方案

- 文档：[RELEASE_V1_VISUAL_SPEC_V1_SOURCE.md](../RELEASE_V1_VISUAL_SPEC_V1_SOURCE.md)
- [已核查固定版本](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/6420272d48ad62be6ae80fa1a129b0a1777ef792/docs/RELEASE_V1_VISUAL_SPEC_V1_SOURCE.md)
- 原文日期：原文无明确日历日期；Git blob：02c90d2324f7f09e42669755e9f3f7d471075c2e
- 适用：原文明确同范围视觉优先于旧描述；含参考图职责、角色、场景、VFX、HUD。归航站角色固定高度等受后续 ROOM-FIT 覆盖。

## S4 玩法与工程细则

- 文档：[RELEASE_V1_GAMEPLAY_ENGINEERING_SPEC_2026_09_28_SOURCE.md](../RELEASE_V1_GAMEPLAY_ENGINEERING_SPEC_2026_09_28_SOURCE.md)
- [已核查固定版本](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/6420272d48ad62be6ae80fa1a129b0a1777ef792/docs/RELEASE_V1_GAMEPLAY_ENGINEERING_SPEC_2026_09_28_SOURCE.md)
- 原文日期：2026-09-28；Git blob：0c93a592c650c900d5d92066ff9884aa934bd9a5
- 适用：三世界玩法、地图/感知、Effect、持久化、未来边界、验收矩阵。细节与附录数值仍应读原文。

## S5 能力扩展架构与阻塞清理补充

- 文档：[RELEASE_V1_ENGINEERING_SUPPLEMENT_V1_SOURCE.md](../RELEASE_V1_ENGINEERING_SUPPLEMENT_V1_SOURCE.md)
- [已核查固定版本](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/6420272d48ad62be6ae80fa1a129b0a1777ef792/docs/RELEASE_V1_ENGINEERING_SUPPLEMENT_V1_SOURCE.md)
- 原文日期：原文无明确日历日期；Git blob：0daf4506271265b4a1327bd949b91da2a38600ff
- 适用：统一 Effect/Rules、旧内容迁移、地图/地形/危险、Native门槛。示例≠冻结条款；旧雾港待输入阻塞由 S6 解决。

## S6 雾港几何与泵站冻结稿

- 文档：[MIST_HARBOR_V1_GEOMETRY_PUMP_FREEZE_SOURCE.md](../MIST_HARBOR_V1_GEOMETRY_PUMP_FREEZE_SOURCE.md)
- [已核查固定版本](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/6420272d48ad62be6ae80fa1a129b0a1777ef792/docs/MIST_HARBOR_V1_GEOMETRY_PUMP_FREEZE_SOURCE.md)
- 原文日期：原文无明确日历日期；Git blob：037b6463d152380d106efbe69aecc3ea97acb519
- 适用：九图精确几何/探索、泵站状态/作用范围/持久化及取消捷径。替代旧泵站示例/marker等待，不重写美术边界。

## 后续策划覆盖的来源状态

2026-10-06 主策划已归档 ROOM-FIT 决议（UTC 04:39 / JST 13:39），摘要见 REQUIREMENTS G-044～G-046。现有公开六文件没有这段全文；本次摘要不伪造公开链接，也不发布私人聊天链接。完整近期策划变更仍待补证。该决议不能推出其他地图也采用固定全室镜头。

## 视觉参考名录（文件存在/权利仍须分别核查）

以下名称来自 S3 §1、107，仅作为查找线索，不代表当前 Git 中已存在可运行素材：

- 无限人族进化视觉总方案.png：整体风格关系
- 无限人族进化_科幻hud规范海报.png：HUD 结构主参考
- 科幻游戏主菜单与归航站终端设计规范.png：菜单/世界网络/终端
- 灰巢世界_工业废土交互设定板.png：灰巢设备与交互
- 迷雾港_雾海导航与战斗风格.png：雾港导航/声学
- 钟骨世界_熔炉机械与压力控制.png：钟骨机械/危险
- 岑瑶角色视觉与动作规范板_image_gen.png：角色参考；正式角色名仍为岑遥
- 科幻战斗与界面设计规范图.png：战斗反馈

近期另外使用过游戏 UI 设计总览、游戏视觉设计总览及黑暗科幻设定集作对照；它们不自动取代以上每张图的专属职责，物理环形门与抽象世界网络球体不可混用。需补资产仓库路径/校验值和对应方案章节后才可作为可复现制作输入。

## 覆盖边界

- 公共六份原方案：已定位，有固定版本
- 可见用户最近要求：已纳入 REQUIREMENTS 的 U 条目；未发布原始对话
- 主策划所有历史对话：未全量收齐，不声称完整
- 本地 Codex 游戏本体对话：待本机提供脱敏需求摘要
- 实现/测试结果：不在本索引作完成宣称；应查指定源码版本及私有协作证据


## P7 主策划交接汇总与字段纠正

实际读取日期：2026-10-06 22:33 JST，随后完成字段纠正。
标题：《无限人类进化》游戏方案策划 Chat 交接资料 V1.0。

内容分组：总体战役、操作/角色、能力/感知双层、地图信息、存档/奖励/白芷、UI/视觉/归航站、参考文档、覆盖缺口。摘要见 REQUIREMENTS G-060～G-068。

证据界限：真实读取到了本轮主策划回复，但未逐一追溯它自述的全部「用户明确要求」原始发言；不将这些自动升级为 U。首次进化与白芷结果字段曾在回复中混淆，已明确纠正：白芷使用 WorldProgress.hive_choice 的旧裁决；首次进化精确字段待核，不能共用推断。

回复另外提到 RELEASE_V1_THREE_SOURCE_REGISTER.md、V6_NATIVE_VISUAL_BASELINE_REPORT.md，但未给路径/版本。本轮未定位，不能提供猜测链接或把它们当已读取来源。完整技能/血统/道具表、全地图表仍未确定，六份已存在原方案不可删除。
