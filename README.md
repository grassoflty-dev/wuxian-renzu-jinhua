# 无限人族进化

## 游戏介绍

《无限人族进化》是一款原创无限流动作角色扮演游戏，采用固定斜视角 2.5D 表现，以工业科幻与多世界探索为背景。

玩家扮演普通人类探索者岑遥，从连接未知世界的小型技术中继站“归航站”出发。每个世界都有自己的环境、危险与规则：在连续场景中探索、战斗、操作设施、完成目标，再返回归航站保存成长，准备下一次旅程。

游戏的成长主轴是“逐步解除普通人的限制”。获得能力会改变你能看到的信息、承受的环境压力，以及理解和穿越世界的方式；新的能力也为有限重访提供不同的解法。

### 核心玩法

- 即时战斗：结合普通攻击、冲刺、Pulse 脉冲、Guard 防御和 Pierce 穿刺，观察敌人预警，选择进攻或防守时机
- 场景探索：恢复供电、解除封锁、调整泵站与阀门，在环境危险和机械结构中寻找通路
- 信息成长：地图、后方感知与敌方状态等玩法信息随能力开放，让感知本身成为成长的一部分
- 跨世界推进：从归航站依次进入灰巢设施、雾港余烬和钟骨工厂，在世界之间保留成长与进度

### 三个世界

**灰巢设施（Grey Hive）**

一座因感染与生物安全事故而封锁的工业设施。玩家从维修入口深入动力间、中央竖井、生物隔离区和深层去污区，恢复主电、解除封锁，并面对重型设施守卫 Sentinel。它建立普通探索者面对未知危险时的生存起点。

**雾港余烬（Mist Harbor）**

被海雾、潮水、失效信号与异常共振笼罩的工业港区。玩家沿港口追踪信标和信号，处理泵站水位，在视野受限、声音线索与水域移动的变化中寻找方向。

**钟骨工厂（Clockworks）**

由熔炉、高压蒸汽、传送带、升降结构和巨大机械组成的自动化工厂。压力控制、热量管理与移动平台共同影响行进和战斗，旅程最终抵达调节器核心。

### 角色成长

灰巢首次通关后，玩家可在本地地图、后视感知和身体再生中选择首次强化。三世界成长还围绕敌方生命体征、声学映射和空踏展开：信息能力帮助理解威胁，移动能力用于特定场景通路，空踏不等于自由飞行。

更多血统、装备、技能和高级能力是长期扩展方向。当前预览版仍有占位表现，完整美术动画、部分剧情与档案、完整能力树界面和无障碍设置尚待完善，不能将设计文档中的全部目标视为本版已交付功能。

## 下载与运行

从[preview.5发布页](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/tag/v0.1.0-preview.5)下载[wuxian-renzu-jinhua-preview.5-windows-x64.exe](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/download/v0.1.0-preview.5/wuxian-renzu-jinhua-preview.5-windows-x64.exe)，先阅读`READ_ME_FIRST.txt`和`KNOWN_LIMITATIONS.md`，按`SHA256SUMS.txt`核对完整文件。

- Windows x64与Microsoft Edge WebView2 Runtime；最低窗口1280×720
- EXE未签名、无安装器，升级前备份应用数据目录`com.wuxian.humanevolution`下的存档
- 这个确切EXE尚未在Windows实机执行；启动、视觉/声音/DPI/性能及真实保存/继续仍待验收

## preview.5 的实际调整

重要：白芷NPC是程序绘制胶囊与“肖像待补·开发占位”文字，正式角色美术、肖像和动画尚未交付。本版没有玩家行走或腿部动作修复。

- 雾港首次完成后返回归航站时，依据权威接受的就绪切换给出首返反馈
- Archive v1从已确认的世界事件展示三条固定档案，可重新阅读，不新增保存字段或领取奖励
- 钟骨工厂炉心既有对白接入权威F交互，保持原世界推进条件
- 灰巢Bio加入可选白芷对话：权威暂停、临时票据、三态选择、当前命名槽原子保存与终局只读复谈
- 白芷Taken仅表示承诺，不表示已跟随或已撤离；三种结果均保留她在Bio，不提供队伍、战斗、奖励或额外Gate B门槛

## 基本操作

WASD移动，鼠标瞄准，鼠标左键或J离散普攻，Shift闪避，Q Pulse，按住/松开E Guard，R Pierce，F交互，空格情境移动，Esc暂停。

## 源码、构建与验证

本仓库提供实际工程Git树；preview.5代码更新保留公共历史，不引入旧私有仓库历史。可直接克隆、浏览、构建和逐次审查代码。项目共757个tracked文件；依赖锁、构建工具、素材与来源清单保留，依赖源码大包、EXE、缓存与测试日志不纳入Git。Actions保持关闭。

```sh
git clone https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git
cd wuxian-renzu-jinhua
```

[BUILDING.md](BUILDING.md)说明工程构建，[KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md)列出工程边界。preview.5外层附件给出本次更完整构建/测试记录；冻结源码中的旧验证文字不可冒充本轮结果。Release另附[wuxian-renzu-jinhua-preview.5-corresponding-source.zip](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/download/v0.1.0-preview.5/wuxian-renzu-jinhua-preview.5-corresponding-source.zip)，含757个冻结工程文件、283份依赖源包、863份通知及独立源码清单。

本版EXE真正构建源提交为`1d8aff05f35b24c49355cf64aa58cd7eba561571`，规范化树为`1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a`。公共代码托管提交为[b85ac67644b6730b268293339f62e37cf73059fa](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/commit/b85ac67644b6730b268293339f62e37cf73059fa)，与上述构建源码的树完全一致。公共README随后单独更新而产生文档提交；这些托管提交均不是冻结EXE的原构建提交。源码ZIP保留构建时README未发布候选文字，外层发布说明记录后续验证及托管状态。自行构建须记录真实HEAD与生成身份。

- Web：859通过、0失败、4项真实浏览器依赖测试跳过；typecheck与最终生产构建通过
- Rust：默认703项非bundle游戏测试通过，replay+legacy-test-rng 725项通过；两套重叠，不能相加。两条原完整路线分别用时1170.924秒、1171.001秒；白芷原生产路线自动回归分别为74.388秒、74.424秒
- 白芷跨语言同fixture在真实TypeScript客户端JSON序列化和Rust serde两端通过；这不是真实Tauri进程IPC执行
- 原headless测试各有6项环境失败，保留失败事实；同12项bundle夹具在真实源码/生产资源环境全部通过，逐项闭合。共有737个唯一Rust测试身份具备适用环境通过证据
- 地图编译器47项、图标7项、真实资源bundle 12项通过；30场景可逐字节重建；真实源native lib/all-targets检查通过
- Windows x64离线交叉编译103.225秒；冻结PE的163项Web载荷、30个原生场景、7帧ICO与默认256px RGBA绑定通过；无缓存独立核验及20项验证器正负自测通过；18项陈旧codegen表示在EXE中逐项排除，独立另加3项旧探针缺失/重复/损坏负例通过

详细统计、headless原失败与真实bundle闭合、供应链限定、未验收项见Release的`VERIFICATION.json`。Windows实机、WebView2、真实浏览器、视觉/声音、断电与并发存档尚未验收；不声称所有设计文档或工具测试已完成。

## 许可与来源

项目保留GNU GPL version3文本；第三方依赖及素材记录保留各自许可、通知和来源。283份依赖源包与863份通知文件继承preview.4原字节，锁文件、美术素材和图标不变。指定生成图已按本项目公开用途授权；不构成CC0、独占权、未知第三方授权、商标清查或版权绝对保证。原始C2PA块保留，但未声称验签。

发布附件hash以同页清单为准，实际托管时间以GitHub Release记录为准。preview.2、preview.3与preview.4保留供比较。
