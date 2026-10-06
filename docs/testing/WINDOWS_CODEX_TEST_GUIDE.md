# Windows 实机测试交接：preview.5

本文给本地 Windows Codex 使用。目标是测试已发布的确切 EXE，并把实际结果追加到 [WINDOWS_TEST_RESULTS.md](WINDOWS_TEST_RESULTS.md)。首轮已完成前置核查并记录两项阻塞，**尚未启动游戏或完成 Windows 实机试玩**。下一轮从第 0 节继续；本次文档更新不代表任何复测已经执行。

## 0. 下一轮：Round 2 复测交接（2026-10-06 更新）

先读结果文档的 Run `20261006T005149Z-9001ab28`。该轮为 **PASS 0 / FAIL 0 / BLOCKED 2 / NOT_RUN 14**，没有产生游戏 PID 或观察到原生窗口。下一轮必须新建 Run ID，引用该历史 Run，不修改其命令、证据或结论。

### 当前阻塞与继续条件

- **B01 / W01：补充验证工具现已提供，等待 Windows 实际复测。** 使用本仓库 [tools/preview5-verification](../../tools/preview5-verification/README.md)，按下节固定 commit 并逐文件核对 SHA256。该版优先使用官方 Brotli 1.2.0 Python 后端，不在 Windows 加载 Linux `.so`，保留同一 preview.5 EXE、对应源码与完整 PE/bundle 断言。云端 Linux 的两种解码后端正向闭包、15 项维护测试和 16 项额外边界/随机探针已通过；**未执行 Windows Python 或游戏验收**。本轮实际通过前 W01 仍为 BLOCKED，不能把补充工具 hash 写成第 2 节原发布工具 ZIP 的 hash。
- **B02 / W02：需要明确允许的启动方式。** 上轮工具拒绝 `Start-Process`，未生成 PID；具体策略未公开，不能归因于 SmartScreen、杀毒软件、游戏损坏或权限不足。实际命令带 `-WindowStyle Hidden`，与第 3 节可见窗口示例不同；这只是已知差异，不能声称它导致拒绝或删去后就能成功。先由用户/平台明确确认受支持、获允许的启动路径；确认后使用可见游戏窗口。仍被拒绝就保留 BLOCKED，请用户在隔离环境亲自操作，不换 shell、别名、包装器或其他接口重放被拒绝的动作，不绕过防护或修改执行策略。
- **工具能力按本轮实际检查。** 上轮本地 Codex 已成功通过 `node_repl` 导入 `@oai/sky` 并调用 `list_apps`，因此不能沿用“本机没有桌面工具”的假设。但列出应用不证明已能截图、聚焦、控制游戏、测音频或性能。复测时分别检查当前实际可用能力；只有在获准启动、核实游戏 PID/窗口后，才对目标游戏使用受支持的截图/输入工具。

### 补充验证器：固定来源与 W01 命令

这是独立补充工具，不替换 preview.5 的标签、EXE、对应源码 ZIP 或原 verification-tools 发布附件；第 2 节原始身份表保持不变。新工具本身不启动 EXE、不联网，只有下列明确下载与 pip 步骤需要网络。不得以核验通过推断 B02 已解除。

先按第 3 节只建立本轮全新运行目录，**暂不执行启动代码块**，下载并核对第 2 节固定 EXE 和源码 ZIP。再进入本轮全新、已核对 origin 的文档 clone（不是旧开发区）。以下命令固定当前指南所在的完整 commit；记录该值及对应 GitHub commit URL，所有工具仅从这个不可变 ref 下载，不从滚动 main 混用文件。若当前 clone 过旧而没有补充工具，停止并在新文档工作区获取已发布版本，不更改旧开发区。

```powershell
$ErrorActionPreference = 'Stop'
$VerifierCommit = (git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $VerifierCommit -notmatch '^[0-9a-f]{40}
1. **只读准备并冻结对象。** 保护旧开发工作区和旧存档，新建本轮独立运行目录及独立文档工作区。按第 2 节重新核对 preview.5 的标签、完整 EXE/源码 ZIP 大小和 SHA256、发布清单。不要改测新版、重建 EXE或覆盖上轮证据。记录本指南 commit、文档工作区 HEAD、受测源码身份和 EXE hash，它们不能相互替代。
2. **先重做 W01。** 使用已经交付、核实的 Windows 验证方法，保存精确命令、工具版本、输出、退出码和证据 SHA256。文件 hash、源码验证和 PE/bundle 闭包分别记录；全部所需检查实际完成后才将本轮 W01 标为 PASS。缺项为 BLOCKED；身份冲突立即停止，不自行改期望值。
3. **再重做 W02。** 满足 B02 的许可条件后，使用第 3 节的子进程隔离方式：`WUXIAN_FORMAL_SAVE_DIR` 指向本轮 saves，`WUXIAN_NATIVE_DIAGNOSTICS_DIR` 指向本轮 diagnostics，启动后恢复调用方原值，不修改系统环境。使用可见窗口，记录精确启动命令、时间、PID、真实窗口及截图；进程存在不等于窗口通过。若需人工启动，应请用户在本轮已准备的隔离 PowerShell 环境中执行已核对的启动步骤，不直接双击 EXE进入默认存档环境；记清人工操作范围。仍不能安全启动就停止相关 UI测试并写回阻塞。
4. **先走最小真实闭环。** W02 通过且控制方式受支持后，依次做 W03 主菜单页面/返回/取消，W04 新旅程到归航站和暂停/恢复，再优先做 W09 保存测试槽→返回菜单→正常退出→使用同一隔离 saves 重启→继续及指定槽读取。首次保存就核查实际文件落在本轮 saves；无法确认则暂停存档操作，不能将隔离写入或保存闭环标为 PASS。每项仍须完成第 5 节全部细目；只做部分步骤要写明，不能以一次新旅程成功替代整个 W04。
5. **闭环完成后扩展。** 再按可达场景做 W05–W08 移动、遮挡、战斗与焦点；W10–W12 三世界；满足事件前置条件后做 W13 Archive；用独立旅程/槽位做 W14 白芷三分支；实际观察 W15 尺寸/DPI 与 W16 音频/性能/稳定性。不要因启动成功或菜单脚本通过宣称三世界通过。场景未到达写 NOT_RUN及前置原因；工具、权限或环境确实阻止已尝试步骤写 BLOCKED；观察到与预期不符才写 FAIL。
6. **分段追加并核对远端。** 即使只完成 W01/W02，也追加一份本轮真实记录。每个 W 项仅用 PASS / FAIL / BLOCKED / NOT_RUN，合计 16 项；写清开始/结束时间及 `Asia/Tokyo (UTC+09:00)`，可同时保留 UTC。记录 Windows/WebView2 实际观察、工具版本、窗口/DPI、源码版本、EXE hash、精确命令和各证据范围。原始证据留本地，公开文本脱敏；保留原模板和全部历史 Run。按第 7 节仅提交测试文档、普通非强制推送并核对远端，不修改游戏代码，不触发 Actions，不启用收费功能，不公开私有仓库地址或私人路径。

### 可直接交给本地 Codex 的简短任务

> 先阅读本指南第 0 节和首轮 Run 20261006T005149Z-9001ab28。创建全新 Round 2 Run，继续测试固定 preview.5 EXE。先按第 0 节从固定 commit 获取补充验证器并核对全部 SHA256，用同一 CPython 3.12 Windows x64 解释器安装官方 hash 固定的 Brotli 包，再重做源码与 PE/bundle 核验；启动策略仍须先确认允许的路径。先完成 W01、W02，再做菜单、新旅程、保存退出重启继续，最后扩大到其余条目。保护旧目录和存档，保持子进程隔离。遇策略拒绝停止并请求用户操作，不换工具绕过。实际能用 @oai/sky 要逐项核查，不把 list_apps 当作游戏控制已验证。只把真实结果追加到 WINDOWS_TEST_RESULTS.md，保留历史，按文档限定范围非强制提交；尚未执行的不得写 PASS。

## 1. 范围与安全边界

- 公开仓库：https://github.com/grassoflty-dev/wuxian-renzu-jinhua
- 固定测试版本：[v0.1.0-preview.5](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/tag/v0.1.0-preview.5)。不要自动改测更新版本，不把本机旧 EXE 或自行构建 EXE混入本轮。
- 本机可能有尚未提交的开发工作。不要在旧工作区 pull、checkout、reset、clean、stash、覆盖文件、清缓存或改存档。先只读记录现状；测试、下载、证据及文档提交使用全新独立目录。不要启动或终止其他工作区的进程。
- 只测试和写测试结果；不修改游戏源码、素材、依赖、二进制、工作流、仓库可见性、权限或收费设置。保持 GitHub Actions 关闭，不触发工作流。
- 本轮只授权发布脱敏后的测试文档。原始截图、存档、系统信息和日志留在本地；未经额外确认不上传二进制、截图或完整日志。
- 完全访问权限不等于有原生窗口控制能力。先检查当前 Codex 实际可用的终端、截图、键鼠/可访问性工具。没有受支持的控制方法就记录 BLOCKED，完成可做的只读/身份检查；需要用户操作时给出具体一步。不要操作 Codex 自身 UI，不绕过安全警告、系统权限、前台焦点限制或防护软件。若出现 SmartScreen/安全拦截，请用户自行判断和处理。
- 安装缺失的 WebView2、PowerShell、构建工具或其他依赖前说明缺项并取得必要授权；不要为了测试关闭防护或改变执行策略。

## 2. 冻结身份（2026-10-06 核对发布页）

| 项目 | 期望值 |
|---|---|
| 公共版本标签 | v0.1.0-preview.5 |
| 标签对应公共源码提交 | b85ac67644b6730b268293339f62e37cf73059fa |
| 发布清单记录的原构建源码提交 | 1d8aff05f35b24c49355cf64aa58cd7eba561571 |
| 规范化源码树 | 1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a |
| EXE 文件名 | wuxian-renzu-jinhua-preview.5-windows-x64.exe |
| EXE 大小 | 255988736 字节 |
| EXE SHA256 | e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da |
| 对应源码 ZIP | wuxian-renzu-jinhua-preview.5-corresponding-source.zip |
| 源码 ZIP 大小 | 428215921 字节 |
| 源码 ZIP SHA256 | 2b54c7f18452d7eae5739107f57a9841f7c050162551872118e37ecf30c9b033 |
| 独立验证工具 ZIP SHA256 | cd739060819ac38ea8397822b06ee31537b4753264ef0da76d126baafe0d163f |

公共源码托管提交与原构建源码提交不同；发布页说明两者对应的源码树相同。main 上后续文档提交也不是 EXE 构建身份。必须分别记录“文档工作区 HEAD”“受测源码身份”“实际 EXE SHA256”。下载后若与此表、同版 SHA256SUMS.txt 或发布元数据冲突，停止启动并报告，不能自行改期望值。

先从上面的固定发布页下载并阅读 READ_ME_FIRST.txt、KNOWN_LIMITATIONS.md、BUILDING.md、RELEASE-MANIFEST.json、VERIFICATION.json、SHA256SUMS.txt。发布附件记录当前发布事实；冻结源码 README 中仍有较早候选/历史验证文字，不能当作本次测试结果。

身份验证分开记：
1. 本机对完整下载的 EXE 和源码 ZIP 计算 SHA256、大小。
2. 检查标签指向上表公共提交；独立源码 checkout 必须固定此提交，而不是当前 main。
3. 阅读同版 verification-tools ZIP 的说明与工具参数，验证对应源码清单、规范化树、PE 内嵌资源闭包，逐项保存真实命令/退出码。不要猜工具参数；工具不可用则这项 BLOCKED。仅哈希一致不足以证明全部 bundle 闭包。
4. 可使用 F3 身份面板和现有 identity-run.ps1 辅助观察运行身份。面板或脚本的窄范围成功不等于完整 bundle 校验成功。

## 3. 建立全新隔离目录并启动

要求：Windows x64、可见交互桌面、WebView2；最低游戏窗口 1280×720。先核实磁盘空间、旧游戏进程和工具版本。不要退出用户已有游戏；若已有实例导致无法区分窗口，暂停并请用户处理。

以下 PowerShell 示例建立独立运行根目录，路径可调整为新空目录；不要复用旧游戏或旧测试目录：

```powershell
$ErrorActionPreference = 'Stop'
$RunRoot = Join-Path $env:LOCALAPPDATA ('WuxianPreviewTests\' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Path $RunRoot | Out-Null
$Downloads = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'downloads')
$SaveDir = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'saves')
$DiagDir = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'diagnostics')
$EvidenceDir = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'evidence')
$BaseUrl = 'https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/download/v0.1.0-preview.5'
$ExeName = 'wuxian-renzu-jinhua-preview.5-windows-x64.exe'
$Exe = Join-Path $Downloads.FullName $ExeName
Invoke-WebRequest -Uri "$BaseUrl/$ExeName" -OutFile $Exe
$Expected = 'e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da'
$Actual = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash.ToLowerInvariant()
if ((Get-Item -LiteralPath $Exe).Length -ne 255988736 -or $Actual -ne $Expected) {
    throw 'EXE identity mismatch: stop here'
}
```

下载其他附件亦只使用固定版本页列出的确切文件名；核对 SHA256SUMS 与发布 digest。先完成前置检查，再从该 PowerShell 启动，环境变量只作用于本次子进程，不写系统设置：

```powershell
$OldSave = $env:WUXIAN_FORMAL_SAVE_DIR
$OldDiag = $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR
try {
    $env:WUXIAN_FORMAL_SAVE_DIR = $SaveDir.FullName
    $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $DiagDir.FullName
    $Game = Start-Process -FilePath $Exe -WorkingDirectory $Downloads.FullName -PassThru
    # 记录 $Game.Id；使用受支持的工具检查这个 PID 的真实窗口
} finally {
    $env:WUXIAN_FORMAL_SAVE_DIR = $OldSave
    $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $OldDiag
}
```

这两个隔离变量已在公开冻结源码 server-rs/src/main.rs 核对：前者覆盖正式存档目录，后者用于原生新旅程阶段诊断。游戏仍可能使用 WebView2/OS 自有缓存；不要声称所有系统写入都被隔离。首次新建存档后核查文件确实写到本轮 saves 下，避免触碰默认应用数据。保留测试目录与证据，不自动清理。

文档工作区另建全新 clone，保持 main 用于追加文档。若需要源码测试，可在另一全新 checkout 固定 b85ac676；不要切换文档工作区来混淆身份。

## 4. 操作与本版已知边界

公开 InputController.ts 与 README 的操作：WASD 移动（W 对应屏幕向上），鼠标瞄准，鼠标左键或 J 单次普攻，Shift 闪避，Q Pulse，按住 E Guard / 松开结束，R Pierce，F 交互，空格情境移动，Esc 暂停。鼠标瞄准和移动朝向需分别观察，不能默认人物始终朝行走方向。情境移动不是无限跳跃或自由飞行。

本版为早期预览：白芷使用程序绘制胶囊与“肖像待补·开发占位”；正式肖像、美术和动画未交付。本版没有玩家行走/腿部动作修复。测试仍需记录滑步、朝向、遮挡等可见问题，但不要把占位画面描述成完整美术或伪造修复完成。

白芷位于灰巢 Bio；在真实交互范围内使用 F。三态为 taken（“我会帮你离开这里。”）、left（“我不能带你走。”）、unresolved（“我还需要确认一些事情。”）。taken 只是承诺，不会跟随、撤离或组队；三态均留在 Bio，没有额外奖励或 Gate B 门槛。taken/left 有确认步骤，确认后终局复谈只读；unresolved 可以之后再谈。测试三个分支须使用独立测试旅程/槽位，不能覆盖用户旧存档。

## 5. Windows 验收清单

每项单独填写 PASS / FAIL / BLOCKED / NOT_RUN。PASS 必须有该项实际步骤和证据；发现失败继续可独立进行的项，不修代码。无法安全继续时保留现场并说明阻塞。

| ID | 实际操作 | 预期/记录重点 |
|---|---|---|
| W01 | 完成文件哈希、源码与 bundle 身份检查 | 分别记录结果，冲突停止；不能用 main HEAD 代替 EXE 身份 |
| W02 | 从隔离目录冷启动确切 EXE | 真实游戏窗口出现、可读、无白屏/长期黑屏/启动崩溃；记录耗时、窗口标题/PID、报错 |
| W03 | 在主菜单打开/返回可见页面，取消操作，重复进入 | 文本与按钮可用，无卡死/重复遮罩；没有存档时“继续”的真实状态准确记录 |
| W04 | 新旅程 → 归航站，暂停/恢复，返回菜单后再次新旅程 | 无长时间停滞、残留输入、错误会话或重复奖励；记录到达画面和实际阶段 |
| W05 | 四向/斜向移动、松键停止，鼠标绕角色瞄准 | W 向屏幕上；位移/瞄准可辨，记录朝向、滑步、脚部表现与现有占位；不要仅凭源码判通过 |
| W06 | 绕设施、门框、前后景、台阶/平台移动，切场景 | 玩家/敌人/设施遮挡顺序合理；地面可见，无整幅黑地图、背景丢失或明显穿层；截图标明地点 |
| W07 | 在安全可达战斗场景分别试左键/J、Shift、Q、E按下/松开、R | 区分每次动作是否实际接受、冷却/资源限制、受击/命中反馈；记录敌人预警、伤害反馈、声音，不把无目标未命中当失败 |
| W08 | 战斗中暂停/恢复；失去焦点后回到游戏 | 无持续移动或 Guard 卡住；只操作游戏和授权测试工具，不向其他窗口发送按键 |
| W09 | 归航站可见保存/命名槽入口；存测试槽；返回菜单；正常退出再启动；继续与指定槽读取 | 使用同一隔离 saves；分别检查自动档继续和明确槽位读取；记录场景、成长、任务状态前后是否一致，不改原始文件伪造状态 |
| W10 | 实际游玩灰巢：入口、供电/封锁、竖井、Bio、深层与 Sentinel，完成后返回归航站 | 按真实提示推进，记录实际路线/阻塞点；在进化终端领取可见首通强化；取消不提前发放、正常领取后保存可恢复 |
| W11 | 雾港：沿信标/信号、泵站水位提示推进，首次完成后返回归航站 | 核对实际接受的首返反馈及后续入口状态；没完成全程不得写三世界通过 |
| W12 | 钟骨：压力/热量、输送/升降结构和核心流程；炉心附近 F 交互 | 记录真实对白与推进条件；情境移动仅在提示/能力允许处，无法到达明确写阻塞 |
| W13 | 已获相应世界事件后打开 Archive、重读并关闭 | 本版三条固定档案按已确认事件显示；重读不额外发奖励；尚未解锁写对应前置条件 |
| W14 | Bio 白芷：打开/关闭；对话/选项/确认页返回；unresolved；独立旅程测 taken 与 left | 对话时暂停可靠，取消不提交终局；确认后保存，重启/读槽仍一致；终局复谈只读、NPC留原场景且不影响Gate B |
| W15 | 1280×720 与显示器实际支持的其他窗口尺寸，记录当前 DPI/缩放 | 无关键 UI 裁切；不强行要求显示器无法提供的尺寸，不为测试修改系统安全/显示设置 |
| W16 | 真实游戏音效/音乐、连续游玩、切场景与正常退出 | 记录是否实际听到、卡顿/掉帧/崩溃和观测方法；没有音频/性能测量能力写 BLOCKED，不能凭代码或进程存在判通过 |

三世界全流程可能较长。先提交已完成部分和每个未完成项的确切状态，再继续；不得把时间不足或未到达场景写成 PASS。危险的断电、强制崩溃、并发写档注入不属本轮默认操作，另列 NOT_RUN，需单独安全计划和授权。

## 6. 现成工具与证据等级

先阅读固定源码中的 [tools/native-e2e/README.md](../../tools/native-e2e/README.md)，检查脚本和参数。可用 PowerShell 7 时：
- run.ps1：可见窗口截图证据；它不发送游戏操作，也不证明游戏流程通过。用已隔离启动的 -ProcessId，避免它另启未隔离进程。
- identity-run.ps1：给确切 EXE、期望 SHA、全新空证据目录；脚本会自行设置隔离变量，观察 F3 窄范围身份。不要同时开另一游戏实例。
- interaction-run.ps1：同样给确切 EXE、期望 SHA、新空证据目录，分别跑 JourneyRepeat 或 SaveContinue。它只证明对应菜单流程，不是三世界全流程验收。
- 每个脚本使用不同 evidence 子目录，先结束本轮自己启动的前一实例；不要终止其他实例。焦点/窗口身份/UIA 控件检查失败就保留 BLOCKED，不改脚本绕过检查。

证据必须分层：源码静态检查、自动单元测试、构建成功、浏览器/mock 测试、真实原生窗口截图、真实交互试玩、完整三世界路线，分别报告。模拟 IPC 或浏览器截图不能替代 Windows EXE 证据。历史 Release 测试数字只能作背景，不能复制成你的本轮结果。

源码构建/自动测试属可选补充，先读 BUILDING.md；在独立冻结源码目录进行，使用现有工具，不安装新依赖或改锁文件来凑通过。每条命令记录 cwd（脱敏）、工具版本、退出码与耗时；重建的 EXE 是另一个受测对象，另开 Run ID，不能覆盖下载的官方 EXE。自动测试即使全绿也不能替代 W02–W16。

## 7. 写回结果与交付

1. 在全新文档 clone 中检查 origin 确实为本公开仓库；保存初始 git status 与 HEAD。不要使用旧开发目录。
2. 在 WINDOWS_TEST_RESULTS.md 文末复制模板并填本轮结果；原模板、既有记录和失败证据保留。每个版本/环境/重新测试使用新 Run ID；修正先前结论时保留更正缘由与关联记录。
3. 每项写清前置状态、精确操作、预期、实际、状态、证据编号与限制。日志先脱敏：用户名、主机名、私人路径、账户、令牌、邮箱及其他项目内容不要公开。只写最小必要日志片段；原文件留本地，以编号和 SHA256 引用，明确公共读者未拿到原件。
4. 写明当前仅为局部结果或完整结果，以及已测/失败/阻塞/未运行数量；没有测到的全部列出。
5. 检查 git diff，仅暂存 docs/testing/WINDOWS_TEST_RESULTS.md（如确需更正文档可加本清单并解释）。禁止 git add -A、夹带其他改动、force push 或修改工作流。提交说明包含 docs: Windows preview.5 test results [skip ci]。只在已授权且正常权限允许时普通提交/非强制推送到 main；遇到远端竞争先检查新 HEAD，在干净文档工作区保留并整合他人追加，绝不覆盖历史。
6. 推送后通过远端 HEAD 和文件内容核实，再提供 commit URL、结果文档 URL、主要缺陷与本地原始证据位置。权限/保护规则阻止推送时，保留本地结果，报告具体原因与文件位置/补丁；不得声称已写回远端，也不要改仓库权限绕过。
) { throw '无法固定文档/工具 commit' }
# $RunRoot 必须是本轮新建运行目录，downloads 中已有核对过的固定附件
if (-not $RunRoot -or -not (Test-Path -LiteralPath (Join-Path $RunRoot 'downloads'))) { throw '先准备本轮运行目录' }
$Verifier = Join-Path $RunRoot 'verification-windows-fix'
if (Test-Path -LiteralPath $Verifier) { throw '工具目录已存在；不要覆盖旧证据' }
New-Item -ItemType Directory -Path $Verifier | Out-Null
$ExpectedTools = @{
    'LICENSE' = '3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986'
    'PUBLIC-NATIVE-EVIDENCE.json' = '3e73fa781832343ca0f5e63d35252a8a40097528b6d343ad0dc4346e9f22e44d'
    'README.md' = '7855f054ceadc23982938362503f72baedcd7f940308205cec9e26066145d2b0'
    'brotli_decoder.py' = 'd2c88bb855559bb53aa9ee86143f06d4c002e69a29165d7749d587379dcdea76'
    'bundle-identity.json' = '690c84621d98edef548f73c8f6fa302cf4a523da0b8904c0c02898dbdd049e87'
    'icon_parser.py' = 'e1eb4056e66f3e43015df65f045e7e10043cf6a6edeb59aa131f66239cceffed'
    'requirements-windows-py312-x64.txt' = '24bf1914d30cb8c148462034a092b57870778d98b6884eebbd0cdb9446a1aae0'
    'toolchain-supplement.json' = 'cbb32b6cef7923ebb58a9c9c4cdd2a9f84fe58a18e8919fa657b153541347cad'
    'verify-native.py' = 'af96caae96bab98d6b403b1f6ab5625e4e4d8d235390a5a543d255fed958d5c2'
    'verify-source.py' = '2a112b00bc316bcfa72c8278fbd6fab0018fd718cf68a37d2e101f30236f63aa'
    'windows-dependency-coverage.json' = '5737defad4892f141c4ab0a94cbef06a5b0c02412df7d2e03042c95d5649cde0'
}
$ToolBase = "https://raw.githubusercontent.com/grassoflty-dev/wuxian-renzu-jinhua/$VerifierCommit/tools/preview5-verification"
foreach ($Name in $ExpectedTools.Keys) {
    $Destination = Join-Path $Verifier $Name
    Invoke-WebRequest -Uri "$ToolBase/$Name" -OutFile $Destination
    $Hash = (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Hash -ne $ExpectedTools[$Name]) { throw "补充工具 SHA256 不符：$Name" }
}
Write-Output "Verifier commit: $VerifierCommit"
```

核对并阅读新 README 后，使用上轮核验的**同一解释器**。下面的 `python` 必须解析为该解释器；若之前用了完整路径，应把 `$Python` 设为同一路径。仅支持现有 CPython 3.12 Windows x64；其他环境、无 pip 或依赖安装未经必要授权时停止并记 BLOCKED，不擅自安装 Python 或下载 DLL。官方包来自 [PyPI Brotli 1.2.0](https://pypi.org/project/brotli/1.2.0/#files)，requirements 限定其 Windows x64 wheel SHA256，只安装到本轮工具目录。

```powershell
$Python = (Get-Command python -CommandType Application -ErrorAction Stop).Source
& $Python -c "import sys, struct; print(sys.executable, sys.version); sys.exit(0 if sys.platform == 'win32' and sys.implementation.name == 'cpython' and sys.version_info[:2] == (3,12) and struct.calcsize('P') == 8 else 1)"
if ($LASTEXITCODE -ne 0) { throw '需要同一 CPython 3.12 Windows x64 解释器' }
& $Python -m pip install --only-binary=:all: --no-deps --index-url https://pypi.org/simple --target "$Verifier" --require-hashes -r "$Verifier\requirements-windows-py312-x64.txt"
if ($LASTEXITCODE -ne 0) { throw 'Brotli 安装失败；停止核验' }
& $Python "$Verifier\verify-source.py" "$RunRoot\downloads\wuxian-renzu-jinhua-preview.5-corresponding-source.zip"
if ($LASTEXITCODE -ne 0) { throw '源码核验未通过' }
& $Python "$Verifier\verify-native.py" "$RunRoot\downloads\wuxian-renzu-jinhua-preview.5-windows-x64.exe" "$RunRoot\downloads\wuxian-renzu-jinhua-preview.5-corresponding-source.zip"
if ($LASTEXITCODE -ne 0) { throw 'PE/bundle 闭包核验未通过' }
```

不得加 `-O/-OO`、改期望值或删断言。分别保存下载身份核验、源码核验、原生静态核验的精确命令、解释器版本、输出、退出码及证据 SHA256；只在本轮检查全部实际通过后改变本轮 W01 状态。旧 Run 保持原样。此处 PowerShell 命令尚未在 Windows 执行验证；任何失败据实记录，不能照抄云端通过结论。

### 按这个顺序执行

1. **只读准备并冻结对象。** 保护旧开发工作区和旧存档，新建本轮独立运行目录及独立文档工作区。按第 2 节重新核对 preview.5 的标签、完整 EXE/源码 ZIP 大小和 SHA256、发布清单。不要改测新版、重建 EXE或覆盖上轮证据。记录本指南 commit、文档工作区 HEAD、受测源码身份和 EXE hash，它们不能相互替代。
2. **先重做 W01。** 使用已经交付、核实的 Windows 验证方法，保存精确命令、工具版本、输出、退出码和证据 SHA256。文件 hash、源码验证和 PE/bundle 闭包分别记录；全部所需检查实际完成后才将本轮 W01 标为 PASS。缺项为 BLOCKED；身份冲突立即停止，不自行改期望值。
3. **再重做 W02。** 满足 B02 的许可条件后，使用第 3 节的子进程隔离方式：`WUXIAN_FORMAL_SAVE_DIR` 指向本轮 saves，`WUXIAN_NATIVE_DIAGNOSTICS_DIR` 指向本轮 diagnostics，启动后恢复调用方原值，不修改系统环境。使用可见窗口，记录精确启动命令、时间、PID、真实窗口及截图；进程存在不等于窗口通过。若需人工启动，应请用户在本轮已准备的隔离 PowerShell 环境中执行已核对的启动步骤，不直接双击 EXE进入默认存档环境；记清人工操作范围。仍不能安全启动就停止相关 UI测试并写回阻塞。
4. **先走最小真实闭环。** W02 通过且控制方式受支持后，依次做 W03 主菜单页面/返回/取消，W04 新旅程到归航站和暂停/恢复，再优先做 W09 保存测试槽→返回菜单→正常退出→使用同一隔离 saves 重启→继续及指定槽读取。首次保存就核查实际文件落在本轮 saves；无法确认则暂停存档操作，不能将隔离写入或保存闭环标为 PASS。每项仍须完成第 5 节全部细目；只做部分步骤要写明，不能以一次新旅程成功替代整个 W04。
5. **闭环完成后扩展。** 再按可达场景做 W05–W08 移动、遮挡、战斗与焦点；W10–W12 三世界；满足事件前置条件后做 W13 Archive；用独立旅程/槽位做 W14 白芷三分支；实际观察 W15 尺寸/DPI 与 W16 音频/性能/稳定性。不要因启动成功或菜单脚本通过宣称三世界通过。场景未到达写 NOT_RUN及前置原因；工具、权限或环境确实阻止已尝试步骤写 BLOCKED；观察到与预期不符才写 FAIL。
6. **分段追加并核对远端。** 即使只完成 W01/W02，也追加一份本轮真实记录。每个 W 项仅用 PASS / FAIL / BLOCKED / NOT_RUN，合计 16 项；写清开始/结束时间及 `Asia/Tokyo (UTC+09:00)`，可同时保留 UTC。记录 Windows/WebView2 实际观察、工具版本、窗口/DPI、源码版本、EXE hash、精确命令和各证据范围。原始证据留本地，公开文本脱敏；保留原模板和全部历史 Run。按第 7 节仅提交测试文档、普通非强制推送并核对远端，不修改游戏代码，不触发 Actions，不启用收费功能，不公开私有仓库地址或私人路径。

### 可直接交给本地 Codex 的简短任务

> 先阅读本指南第 0 节和首轮 Run 20261006T005149Z-9001ab28。创建全新 Round 2 Run，继续测试固定 preview.5 EXE。先按第 0 节从固定 commit 获取补充验证器并核对全部 SHA256，用同一 CPython 3.12 Windows x64 解释器安装官方 hash 固定的 Brotli 包，再重做源码与 PE/bundle 核验；启动策略仍须先确认允许的路径。先完成 W01、W02，再做菜单、新旅程、保存退出重启继续，最后扩大到其余条目。保护旧目录和存档，保持子进程隔离。遇策略拒绝停止并请求用户操作，不换工具绕过。实际能用 @oai/sky 要逐项核查，不把 list_apps 当作游戏控制已验证。只把真实结果追加到 WINDOWS_TEST_RESULTS.md，保留历史，按文档限定范围非强制提交；尚未执行的不得写 PASS。

## 1. 范围与安全边界

- 公开仓库：https://github.com/grassoflty-dev/wuxian-renzu-jinhua
- 固定测试版本：[v0.1.0-preview.5](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/tag/v0.1.0-preview.5)。不要自动改测更新版本，不把本机旧 EXE 或自行构建 EXE混入本轮。
- 本机可能有尚未提交的开发工作。不要在旧工作区 pull、checkout、reset、clean、stash、覆盖文件、清缓存或改存档。先只读记录现状；测试、下载、证据及文档提交使用全新独立目录。不要启动或终止其他工作区的进程。
- 只测试和写测试结果；不修改游戏源码、素材、依赖、二进制、工作流、仓库可见性、权限或收费设置。保持 GitHub Actions 关闭，不触发工作流。
- 本轮只授权发布脱敏后的测试文档。原始截图、存档、系统信息和日志留在本地；未经额外确认不上传二进制、截图或完整日志。
- 完全访问权限不等于有原生窗口控制能力。先检查当前 Codex 实际可用的终端、截图、键鼠/可访问性工具。没有受支持的控制方法就记录 BLOCKED，完成可做的只读/身份检查；需要用户操作时给出具体一步。不要操作 Codex 自身 UI，不绕过安全警告、系统权限、前台焦点限制或防护软件。若出现 SmartScreen/安全拦截，请用户自行判断和处理。
- 安装缺失的 WebView2、PowerShell、构建工具或其他依赖前说明缺项并取得必要授权；不要为了测试关闭防护或改变执行策略。

## 2. 冻结身份（2026-10-06 核对发布页）

| 项目 | 期望值 |
|---|---|
| 公共版本标签 | v0.1.0-preview.5 |
| 标签对应公共源码提交 | b85ac67644b6730b268293339f62e37cf73059fa |
| 发布清单记录的原构建源码提交 | 1d8aff05f35b24c49355cf64aa58cd7eba561571 |
| 规范化源码树 | 1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a |
| EXE 文件名 | wuxian-renzu-jinhua-preview.5-windows-x64.exe |
| EXE 大小 | 255988736 字节 |
| EXE SHA256 | e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da |
| 对应源码 ZIP | wuxian-renzu-jinhua-preview.5-corresponding-source.zip |
| 源码 ZIP 大小 | 428215921 字节 |
| 源码 ZIP SHA256 | 2b54c7f18452d7eae5739107f57a9841f7c050162551872118e37ecf30c9b033 |
| 独立验证工具 ZIP SHA256 | cd739060819ac38ea8397822b06ee31537b4753264ef0da76d126baafe0d163f |

公共源码托管提交与原构建源码提交不同；发布页说明两者对应的源码树相同。main 上后续文档提交也不是 EXE 构建身份。必须分别记录“文档工作区 HEAD”“受测源码身份”“实际 EXE SHA256”。下载后若与此表、同版 SHA256SUMS.txt 或发布元数据冲突，停止启动并报告，不能自行改期望值。

先从上面的固定发布页下载并阅读 READ_ME_FIRST.txt、KNOWN_LIMITATIONS.md、BUILDING.md、RELEASE-MANIFEST.json、VERIFICATION.json、SHA256SUMS.txt。发布附件记录当前发布事实；冻结源码 README 中仍有较早候选/历史验证文字，不能当作本次测试结果。

身份验证分开记：
1. 本机对完整下载的 EXE 和源码 ZIP 计算 SHA256、大小。
2. 检查标签指向上表公共提交；独立源码 checkout 必须固定此提交，而不是当前 main。
3. 阅读同版 verification-tools ZIP 的说明与工具参数，验证对应源码清单、规范化树、PE 内嵌资源闭包，逐项保存真实命令/退出码。不要猜工具参数；工具不可用则这项 BLOCKED。仅哈希一致不足以证明全部 bundle 闭包。
4. 可使用 F3 身份面板和现有 identity-run.ps1 辅助观察运行身份。面板或脚本的窄范围成功不等于完整 bundle 校验成功。

## 3. 建立全新隔离目录并启动

要求：Windows x64、可见交互桌面、WebView2；最低游戏窗口 1280×720。先核实磁盘空间、旧游戏进程和工具版本。不要退出用户已有游戏；若已有实例导致无法区分窗口，暂停并请用户处理。

以下 PowerShell 示例建立独立运行根目录，路径可调整为新空目录；不要复用旧游戏或旧测试目录：

```powershell
$ErrorActionPreference = 'Stop'
$RunRoot = Join-Path $env:LOCALAPPDATA ('WuxianPreviewTests\' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8))
New-Item -ItemType Directory -Path $RunRoot | Out-Null
$Downloads = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'downloads')
$SaveDir = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'saves')
$DiagDir = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'diagnostics')
$EvidenceDir = New-Item -ItemType Directory -Path (Join-Path $RunRoot 'evidence')
$BaseUrl = 'https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/download/v0.1.0-preview.5'
$ExeName = 'wuxian-renzu-jinhua-preview.5-windows-x64.exe'
$Exe = Join-Path $Downloads.FullName $ExeName
Invoke-WebRequest -Uri "$BaseUrl/$ExeName" -OutFile $Exe
$Expected = 'e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da'
$Actual = (Get-FileHash -LiteralPath $Exe -Algorithm SHA256).Hash.ToLowerInvariant()
if ((Get-Item -LiteralPath $Exe).Length -ne 255988736 -or $Actual -ne $Expected) {
    throw 'EXE identity mismatch: stop here'
}
```

下载其他附件亦只使用固定版本页列出的确切文件名；核对 SHA256SUMS 与发布 digest。先完成前置检查，再从该 PowerShell 启动，环境变量只作用于本次子进程，不写系统设置：

```powershell
$OldSave = $env:WUXIAN_FORMAL_SAVE_DIR
$OldDiag = $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR
try {
    $env:WUXIAN_FORMAL_SAVE_DIR = $SaveDir.FullName
    $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $DiagDir.FullName
    $Game = Start-Process -FilePath $Exe -WorkingDirectory $Downloads.FullName -PassThru
    # 记录 $Game.Id；使用受支持的工具检查这个 PID 的真实窗口
} finally {
    $env:WUXIAN_FORMAL_SAVE_DIR = $OldSave
    $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $OldDiag
}
```

这两个隔离变量已在公开冻结源码 server-rs/src/main.rs 核对：前者覆盖正式存档目录，后者用于原生新旅程阶段诊断。游戏仍可能使用 WebView2/OS 自有缓存；不要声称所有系统写入都被隔离。首次新建存档后核查文件确实写到本轮 saves 下，避免触碰默认应用数据。保留测试目录与证据，不自动清理。

文档工作区另建全新 clone，保持 main 用于追加文档。若需要源码测试，可在另一全新 checkout 固定 b85ac676；不要切换文档工作区来混淆身份。

## 4. 操作与本版已知边界

公开 InputController.ts 与 README 的操作：WASD 移动（W 对应屏幕向上），鼠标瞄准，鼠标左键或 J 单次普攻，Shift 闪避，Q Pulse，按住 E Guard / 松开结束，R Pierce，F 交互，空格情境移动，Esc 暂停。鼠标瞄准和移动朝向需分别观察，不能默认人物始终朝行走方向。情境移动不是无限跳跃或自由飞行。

本版为早期预览：白芷使用程序绘制胶囊与“肖像待补·开发占位”；正式肖像、美术和动画未交付。本版没有玩家行走/腿部动作修复。测试仍需记录滑步、朝向、遮挡等可见问题，但不要把占位画面描述成完整美术或伪造修复完成。

白芷位于灰巢 Bio；在真实交互范围内使用 F。三态为 taken（“我会帮你离开这里。”）、left（“我不能带你走。”）、unresolved（“我还需要确认一些事情。”）。taken 只是承诺，不会跟随、撤离或组队；三态均留在 Bio，没有额外奖励或 Gate B 门槛。taken/left 有确认步骤，确认后终局复谈只读；unresolved 可以之后再谈。测试三个分支须使用独立测试旅程/槽位，不能覆盖用户旧存档。

## 5. Windows 验收清单

每项单独填写 PASS / FAIL / BLOCKED / NOT_RUN。PASS 必须有该项实际步骤和证据；发现失败继续可独立进行的项，不修代码。无法安全继续时保留现场并说明阻塞。

| ID | 实际操作 | 预期/记录重点 |
|---|---|---|
| W01 | 完成文件哈希、源码与 bundle 身份检查 | 分别记录结果，冲突停止；不能用 main HEAD 代替 EXE 身份 |
| W02 | 从隔离目录冷启动确切 EXE | 真实游戏窗口出现、可读、无白屏/长期黑屏/启动崩溃；记录耗时、窗口标题/PID、报错 |
| W03 | 在主菜单打开/返回可见页面，取消操作，重复进入 | 文本与按钮可用，无卡死/重复遮罩；没有存档时“继续”的真实状态准确记录 |
| W04 | 新旅程 → 归航站，暂停/恢复，返回菜单后再次新旅程 | 无长时间停滞、残留输入、错误会话或重复奖励；记录到达画面和实际阶段 |
| W05 | 四向/斜向移动、松键停止，鼠标绕角色瞄准 | W 向屏幕上；位移/瞄准可辨，记录朝向、滑步、脚部表现与现有占位；不要仅凭源码判通过 |
| W06 | 绕设施、门框、前后景、台阶/平台移动，切场景 | 玩家/敌人/设施遮挡顺序合理；地面可见，无整幅黑地图、背景丢失或明显穿层；截图标明地点 |
| W07 | 在安全可达战斗场景分别试左键/J、Shift、Q、E按下/松开、R | 区分每次动作是否实际接受、冷却/资源限制、受击/命中反馈；记录敌人预警、伤害反馈、声音，不把无目标未命中当失败 |
| W08 | 战斗中暂停/恢复；失去焦点后回到游戏 | 无持续移动或 Guard 卡住；只操作游戏和授权测试工具，不向其他窗口发送按键 |
| W09 | 归航站可见保存/命名槽入口；存测试槽；返回菜单；正常退出再启动；继续与指定槽读取 | 使用同一隔离 saves；分别检查自动档继续和明确槽位读取；记录场景、成长、任务状态前后是否一致，不改原始文件伪造状态 |
| W10 | 实际游玩灰巢：入口、供电/封锁、竖井、Bio、深层与 Sentinel，完成后返回归航站 | 按真实提示推进，记录实际路线/阻塞点；在进化终端领取可见首通强化；取消不提前发放、正常领取后保存可恢复 |
| W11 | 雾港：沿信标/信号、泵站水位提示推进，首次完成后返回归航站 | 核对实际接受的首返反馈及后续入口状态；没完成全程不得写三世界通过 |
| W12 | 钟骨：压力/热量、输送/升降结构和核心流程；炉心附近 F 交互 | 记录真实对白与推进条件；情境移动仅在提示/能力允许处，无法到达明确写阻塞 |
| W13 | 已获相应世界事件后打开 Archive、重读并关闭 | 本版三条固定档案按已确认事件显示；重读不额外发奖励；尚未解锁写对应前置条件 |
| W14 | Bio 白芷：打开/关闭；对话/选项/确认页返回；unresolved；独立旅程测 taken 与 left | 对话时暂停可靠，取消不提交终局；确认后保存，重启/读槽仍一致；终局复谈只读、NPC留原场景且不影响Gate B |
| W15 | 1280×720 与显示器实际支持的其他窗口尺寸，记录当前 DPI/缩放 | 无关键 UI 裁切；不强行要求显示器无法提供的尺寸，不为测试修改系统安全/显示设置 |
| W16 | 真实游戏音效/音乐、连续游玩、切场景与正常退出 | 记录是否实际听到、卡顿/掉帧/崩溃和观测方法；没有音频/性能测量能力写 BLOCKED，不能凭代码或进程存在判通过 |

三世界全流程可能较长。先提交已完成部分和每个未完成项的确切状态，再继续；不得把时间不足或未到达场景写成 PASS。危险的断电、强制崩溃、并发写档注入不属本轮默认操作，另列 NOT_RUN，需单独安全计划和授权。

## 6. 现成工具与证据等级

先阅读固定源码中的 [tools/native-e2e/README.md](../../tools/native-e2e/README.md)，检查脚本和参数。可用 PowerShell 7 时：
- run.ps1：可见窗口截图证据；它不发送游戏操作，也不证明游戏流程通过。用已隔离启动的 -ProcessId，避免它另启未隔离进程。
- identity-run.ps1：给确切 EXE、期望 SHA、全新空证据目录；脚本会自行设置隔离变量，观察 F3 窄范围身份。不要同时开另一游戏实例。
- interaction-run.ps1：同样给确切 EXE、期望 SHA、新空证据目录，分别跑 JourneyRepeat 或 SaveContinue。它只证明对应菜单流程，不是三世界全流程验收。
- 每个脚本使用不同 evidence 子目录，先结束本轮自己启动的前一实例；不要终止其他实例。焦点/窗口身份/UIA 控件检查失败就保留 BLOCKED，不改脚本绕过检查。

证据必须分层：源码静态检查、自动单元测试、构建成功、浏览器/mock 测试、真实原生窗口截图、真实交互试玩、完整三世界路线，分别报告。模拟 IPC 或浏览器截图不能替代 Windows EXE 证据。历史 Release 测试数字只能作背景，不能复制成你的本轮结果。

源码构建/自动测试属可选补充，先读 BUILDING.md；在独立冻结源码目录进行，使用现有工具，不安装新依赖或改锁文件来凑通过。每条命令记录 cwd（脱敏）、工具版本、退出码与耗时；重建的 EXE 是另一个受测对象，另开 Run ID，不能覆盖下载的官方 EXE。自动测试即使全绿也不能替代 W02–W16。

## 7. 写回结果与交付

1. 在全新文档 clone 中检查 origin 确实为本公开仓库；保存初始 git status 与 HEAD。不要使用旧开发目录。
2. 在 WINDOWS_TEST_RESULTS.md 文末复制模板并填本轮结果；原模板、既有记录和失败证据保留。每个版本/环境/重新测试使用新 Run ID；修正先前结论时保留更正缘由与关联记录。
3. 每项写清前置状态、精确操作、预期、实际、状态、证据编号与限制。日志先脱敏：用户名、主机名、私人路径、账户、令牌、邮箱及其他项目内容不要公开。只写最小必要日志片段；原文件留本地，以编号和 SHA256 引用，明确公共读者未拿到原件。
4. 写明当前仅为局部结果或完整结果，以及已测/失败/阻塞/未运行数量；没有测到的全部列出。
5. 检查 git diff，仅暂存 docs/testing/WINDOWS_TEST_RESULTS.md（如确需更正文档可加本清单并解释）。禁止 git add -A、夹带其他改动、force push 或修改工作流。提交说明包含 docs: Windows preview.5 test results [skip ci]。只在已授权且正常权限允许时普通提交/非强制推送到 main；遇到远端竞争先检查新 HEAD，在干净文档工作区保留并整合他人追加，绝不覆盖历史。
6. 推送后通过远端 HEAD 和文件内容核实，再提供 commit URL、结果文档 URL、主要缺陷与本地原始证据位置。权限/保护规则阻止推送时，保留本地结果，报告具体原因与文件位置/补丁；不得声称已写回远端，也不要改仓库权限绕过。
