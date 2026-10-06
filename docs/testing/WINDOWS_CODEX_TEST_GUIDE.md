# Windows 实机测试交接：preview.5

本文给本地 Windows Codex 使用。目标是测试已发布的确切 EXE，并把实际结果追加到 [WINDOWS_TEST_RESULTS.md](WINDOWS_TEST_RESULTS.md)。本次提交只提供测试文档，**没有执行 Windows 测试，也没有宣称任何项目通过**。

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
