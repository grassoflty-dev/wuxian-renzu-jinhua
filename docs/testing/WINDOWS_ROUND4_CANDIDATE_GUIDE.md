# Round 4：独立源码候选构建与 Windows 实测

本轮对象是候选源码，不是已发布的 preview.5 EXE。请使用 GPT-6.1 Sol，在本机全新独立目录按本文构建后测试。本文提供任务说明；没有自动启动本机任务，也不表示 Windows 已通过。

## 1. 固定对象与范围

- 公开仓库：<https://github.com/grassoflty-dev/wuxian-renzu-jinhua>
- 候选分支：`candidate/preview5-exit-return-station-20261006`
- **受测源码固定提交：`a5dcc63cddd491d7e656369bd01ce3421f9eba39`**。源码 checkout 必须固定此完整 SHA，不跟随滚动分支。
- 本次测试修复基线：`617560096ab94b48c9e080817a861adbda71c0c0`；上一轮受测产品源码：`bb6c2c3713d1d897a83f641e58b1716f1c5cda8c`
- 本指南/结果文档可以在候选分支继续追加，但文档 HEAD 不能替代上述源码身份。
- 新结果只追加到 [WINDOWS_ROUND4_RESULTS.md](WINDOWS_ROUND4_RESULTS.md)。保留旧 [preview.5 指南](WINDOWS_CODEX_TEST_GUIDE.md) 和 [历史结果](WINDOWS_TEST_RESULTS.md)，不改其 Run、结论或统计。

本轮在上一候选基础上仅调整测试代码，未改生产渲染来迎合旧断言。上一候选仍需 Windows 原生验收的三块产品修改：
1. D01：为本地 `main` 窗口添加唯一 `core:window:allow-close` 能力。菜单仍走正常关闭和已有待处理命令排空；没有强杀进程或扩大其他权限。
2. 归航站：整房间固定 ROOM-FIT 相机、临时结构块面/后墙层次、resize 期间的旧帧与输入保护。结构块面是占位工程表达，未声称完成正式美术。沿用公开运行时已有 idle/临时形变表现；未增加新的 Walk 素材包、加载器或准入接口。
3. HUD：普通界面不显示场景内部 ID；日常 idle 状态留空，动作名中文显示，未知状态不直接暴露原始字符串。

地图、碰撞/导航、存档协议、依赖锁、生产素材及其他世界相机实现未修改。新候选没有发布 EXE、标签或 Release；既有 preview.5 不包含这些修复。不得把旧发布的固定 hash 验证器用于本候选，也不得改它的预期值来使新 EXE 通过。

### 本轮针对上一轮完整 Web 门禁的三项修复

- R3-F01：浏览器测试先请求正常 CDP 关闭、等待本轮创建的子进程退出，再删除本轮临时 profile；有限重试 Windows 临时文件锁；测试主体和清理同时失败时保留两个错误。确定性测试不代替真实 Windows 文件锁复测
- R3-F02：不再查询已删除的 #hud-scene。通过已有 WorldRenderer mock 的实际 render 参数检查两次新旅程与继续存档的世界/场景/epoch；另检查普通 HUD 不泄漏内部 ID。仍保留 IPC 失败恢复、保存、两次返回和继续流程
- R3-F03：受击反馈断言使用真实场景、角色资源和快照构造独立 ROOM-FIT 预期相机，检查标记比例及 x/y，并覆盖宽度/高度限制、resize 和非归航站。保留过期、暂停与替换断言

禁止用删除断言、跳过失败测试、修改生产相机或“更新预期数值到实际值”的方式将本轮门禁改为通过。Round 3 的原始失败记录完整保留。本指南中的 SHA 占位符必须在发布前替换成已验证的代码提交；尚未替换时不得执行构建步骤。

## 2. 保护旧工程，建立新构建目录

先只读核查 Windows x64、可见交互桌面、现有 Git/Node/npm/Rust MSVC/链接器/Windows SDK/WebView2、磁盘空间及正在运行的任务。不要在旧开发区执行 pull、checkout、reset、clean、stash、覆盖、清缓存或改存档；不关闭他人的进程。

按 [BUILDING.md](../../BUILDING.md) 和固定源码中的锁文件执行。缺少工具、安装需要许可、下载/执行被策略阻止时，保留 BLOCKED 并询问用户；不能换 shell/包装器绕过、关闭防护或修改执行策略。不要自动装工具、更新锁文件、改源代码或补丁凑通过。

示例 PowerShell，所有路径必须是本轮新建目录。命令尚未在 Windows 执行验证；逐条保存真实命令、版本、退出码和输出。证据放仓库外，避免污染构建身份。

```powershell
$ErrorActionPreference = 'Stop'
$SourceCommit = 'a5dcc63cddd491d7e656369bd01ce3421f9eba39'
if ($SourceCommit -notmatch '^[0-9a-f]{40}$') { throw '尚未固定 Round 4 源码 SHA，停止' }
$Branch = 'candidate/preview5-exit-return-station-20261006'
$RepoUrl = 'https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git'
$RunId = (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ') + '-' + [guid]::NewGuid().ToString('N').Substring(0,8)
$RunRoot = Join-Path $env:LOCALAPPDATA ('WuxianRound4Tests\' + $RunId)
if (Test-Path -LiteralPath $RunRoot) { throw '运行目录已存在，停止' }
New-Item -ItemType Directory -Path $RunRoot | Out-Null
$Source = Join-Path $RunRoot 'source'
$Evidence = Join-Path $RunRoot 'evidence'
$SaveDir = Join-Path $RunRoot 'saves'
$DiagDir = Join-Path $RunRoot 'diagnostics'
foreach ($Dir in @($Evidence, $SaveDir, $DiagDir)) { New-Item -ItemType Directory -Path $Dir | Out-Null }
git clone --single-branch --branch $Branch $RepoUrl $Source
if ($LASTEXITCODE -ne 0) { throw 'clone 失败' }
git -C $Source checkout --detach $SourceCommit
if ($LASTEXITCODE -ne 0) { throw '固定源码失败' }
if ((git -C $Source remote get-url origin).Trim() -ne $RepoUrl) { throw 'origin 不符' }
if ((git -C $Source rev-parse HEAD).Trim() -ne $SourceCommit) { throw '源码 SHA 不符' }
if (git -C $Source status --porcelain=v1 --untracked-files=all) { throw '源码树不干净' }
```

记录 `node --version`、`npm --version`、`rustc -Vv`、`cargo -V`、`rustup show active-toolchain`；只检查已有工具。依赖来自 `apps/web/package-lock.json` 和 `server-rs/Cargo.lock`，使用正常批准的官方下载路径。若依赖安装需要额外权限，先停在该步骤。

## 3. 构建与对象身份

在 `$Source\apps\web` 依次执行，每一步非零退出即停止依赖它的下一步并记录：

```powershell
Push-Location (Join-Path $Source 'apps\web')
try {
    npm ci
    if ($LASTEXITCODE -ne 0) { throw 'npm ci 失败' }
    npm run typecheck
    if ($LASTEXITCODE -ne 0) { throw 'typecheck 失败' }
    npm run build
    if ($LASTEXITCODE -ne 0) { throw '首次 web build 失败' }
    npm test
    if ($LASTEXITCODE -ne 0) { throw 'web tests 失败，记录并报告，不跳过断言' }
    # 单独复测三个原始失败，完整 npm test 仍为必需门禁，不用聚焦替代
    node --test tests/accessibility-scale-browser.mjs tests/hub-lifecycle-regression.mjs tests/world-renderer-demand.mjs
    if ($LASTEXITCODE -ne 0) { throw '原始失败聚焦复测失败' }
    # 记录浏览器用例是否实际执行；无 Edge 导致 skip 不能算修复已验证
    # 测试会向 dist 输出模块，原生构建前必须恢复严格生产 bundle
    npm run build
    if ($LASTEXITCODE -ne 0) { throw '最终 web build 失败' }
} finally { Pop-Location }
Push-Location (Join-Path $Source 'server-rs')
try {
    cargo test --locked --lib
    if ($LASTEXITCODE -ne 0) { throw 'Rust library tests 失败' }
    cargo build --locked --release --features tauri/custom-protocol
    if ($LASTEXITCODE -ne 0) { throw '原生构建失败' }
} finally { Pop-Location }
if ((git -C $Source rev-parse HEAD).Trim() -ne $SourceCommit) { throw '源码 HEAD 改变' }
if (git -C $Source status --porcelain=v1 --untracked-files=all) { throw '构建后源码不干净' }
```

库测试不代表所有 Rust 集成/回放测试通过。其他测试如实际运行，分别列出命令与结果，不能复制历史数字。完整 web 测试和独立聚焦结果分别记 PASS/FAIL/SKIP，不能叠加重叠计数。记录实际 Edge 版本；两个真实浏览器用例必须执行，skip 不满足 Windows 门禁。可在初次通过后独立重复真实浏览器两用例两次，记录每次结果以检查清理竞态。完整 web 测试若失败，报告确切失败；本轮不要修游戏代码、放宽校验或把失败改成预期成功。

本地 package 二进制名为 `wuxian-horror-ch1.exe`。通常位于 `$Source\server-rs\target\release`；若环境已有 `CARGO_TARGET_DIR` 或工具链 target 覆盖，先核实 Cargo 实际产物路径，不要搜索到旧 EXE 就使用。记录相关环境设置，不改全局配置。禁止混用旧 target 或旧 Web dist。若当前环境变量把 target 指向其他工作区，停止并报告；需要隔离时仅在获准后为本轮进程设置新目录并记录/恢复原值，不修改全局配置。

成功后分别记录：
- 固定源码 SHA、Git tree SHA、构建前后 clean 状态
- Node/npm/Rust/链接器/SDK/Windows/WebView2 实际版本，构建开始/结束时间及精确命令
- `apps/web/dist/bundle-identity.json` 原件及 SHA256，核对其中 buildIdentity.gitSha 为固定源码 SHA、sourceTreeDirty 为 false，并保留其资源/场景清单身份
- 实际 EXE 完整路径（公开文档脱敏）、大小、SHA256、生成时间
- 本地构建没有事先固定 EXE SHA；它是本次实测产生的新身份，不能填写 preview.5 的 SHA 或声称官方发布

构建脚本通过仅证明其静态校验通过；运行时 F3 身份面板、EXE hash 与实际 PID 还须单独核对。启动前确认 Windows 允许执行的路径及当前工具能力；不能把进程列表能力当作已具备截图/键鼠/音频能力。

## 4. 获准后，隔离启动与运行

确认 `$Exe` 是上述新构建产物并记录 SHA。保存/诊断隔离变量只用于当前子进程；不写系统设置。以下是允许启动时的可见窗口示例，不是解除任何先前拒绝的授权：

```powershell
$OldSave = $env:WUXIAN_FORMAL_SAVE_DIR
$OldDiag = $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR
try {
    $env:WUXIAN_FORMAL_SAVE_DIR = $SaveDir
    $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $DiagDir
    $Game = Start-Process -FilePath $Exe -WorkingDirectory (Split-Path -Parent $Exe) -PassThru
    # 记录 PID，核对实际映像路径、窗口与 F3 身份；不要向未核对窗口输入
} finally {
    $env:WUXIAN_FORMAL_SAVE_DIR = $OldSave
    $env:WUXIAN_NATIVE_DIAGNOSTICS_DIR = $OldDiag
}
```

启动/控制遭策略拒绝就停止该动作，记 BLOCKED 并请用户用受支持方式操作。若需要人工启动，必须仍在本轮隔离 saves/diagnostics 环境内；不要直接双击进入默认档。首次保存时核查实际文件落入本轮 saves。系统/WebView2 缓存未声称全部隔离。

## 5. Round 4 专项检查（R01–R10，各自独立状态）

每项只用 PASS / FAIL / BLOCKED / NOT_RUN；每项实际步骤与证据必须齐全才能 PASS，部分完成写明。

| ID | 操作与必须记录的证据 |
|---|---|
| R01 | 独立目录、固定源码、锁文件、web/Rust 命令、构建日志、bundle 身份、EXE hash；身份冲突立即停止 |
| R02 | 冷启动本轮 EXE，核对实际 PID/映像路径/可见窗口/F3 gitSha 与 clean 身份；原生截图留本地 |
| R03 | 新鲜主菜单点正常“退出”，观察本轮 PID 正常结束；再启动同一 EXE；无强杀、无后台残留。此项针对 D01，不以 mock close 通过代替 |
| R04 | 新旅程归航站→保存独立命名测试槽→返回菜单→正常退出并验证 PID 结束→同一隔离 saves 重启→自动档“继续”与明确命名槽分别读取，比较场景/任务/成长；核查真实文件路径。分别记录两个读档分支 |
| R05 | 归航站新候选真实截图：完整 24×16 房间、后墙/环/支撑层次、五个终端、角色脚点与人物可辨识度；移动到可达边缘核对画面裁切及遮挡。不能用静态示意图代替 |
| R06 | HUD 不显示 rs_core_room 等内部场景 ID；idle 日常状态不显示，主动动作状态中文；世界名、HP/能量、交互/目标/技能/暂停可读。截图检查 HUD 是否挡住人物、终端、门或关键目标 |
| R07 | 归航站四向/斜向移动、停步、绕角色鼠标 Aim 与普攻方向；分开观察人物朝向/位移/脚点，无残留输入；当前临时形变不是新 Walk 美术验收 |
| R08 | 使用显示器实际支持的窗口大小（至少默认窗口与另一可用大小），反复 resize 后返回原大小；记录外窗、客户区/渲染逻辑尺寸与 DPI 可测项；resize 中和完成后无旧帧跳动、错误 Aim 或长期输入失效。不把外窗 1280×720 当作 app.screen 1280×720 |
| R09 | 通过正常游戏路线进入可达其他世界，观察原有跟随/瞄准相机；回归航站后整房间固定相机恢复。未到达场景写 NOT_RUN，不能凭代码代替实际验证 |
| R10 | 重复暂停/恢复、菜单返回/取消、正常退出；用安全顺序覆盖保存完成后退出及重复点击不留遮罩/卡死。不要强制崩溃、并发注入存档或人为制造破坏性排空测试 |

旧 W01–W16 是另一个受测对象的历史清单，不并入本轮十项统计。需要扩大三世界路线时另列实际附加结果；此次只完成 R01/R02 也可以先追加真实阶段记录。

## 6. 写回新结果，不改游戏代码

1. 文档工作区另建新 clone，使用候选分支；不要切换源码构建目录来写结果。记录文档 HEAD，重新检查 origin 和远端最新 HEAD。
2. 在 `docs/testing/WINDOWS_ROUND4_RESULTS.md` 末尾追加新 Run；保留模板及所有历史记录。阶段更新用新增段落与关联 Run，不覆盖原始失败。
3. 只公开脱敏文字与最少必要日志节选。截图、存档、完整日志、系统详情和 EXE 留本地；以证据编号和 SHA256 引用，注明原件未上传。不要公开私人路径、用户名、邮箱、令牌或其他项目内容。
4. 核查 diff，仅暂存这一份新结果文档。提交消息建议 `docs: Round 4 candidate Windows results [skip ci]`。只普通非强制推送至 `candidate/preview5-exit-return-station-20261006`，不改 main、不合并、不新建 Release、不上传 EXE，不改工作流/Actions/可见性/付费设置。
5. 若远端推进，先读新增内容并保留他人记录后整合；不强推、不覆盖历史。权限/保护规则阻塞时停下，保留本地文档与补丁并报告，不换身份或改权限绕过。
6. 推送后核对远端 commit 与文件内容，返回提交和结果文档链接。没有成功写回不得声称已发布。

## 7. 当前证据边界

云端本轮检查属于源级/确定性测试。未运行 Windows Edge 清理复测、Windows 完整 Web 门禁、Rust、候选 EXE 或原生交互；这些状态不得从 Linux 测试推断通过。具体源级命令、结果和限制以本轮结果文档为准。生产构建在文本恢复环境因缺少 Git 元数据受阻；资产依赖测试可能因缺少固定 fixture 受阻，这不等于可在 Windows 跳过。新 Windows Run 必须全新固定 SHA checkout、保留原有锁文件与完整资源闭包。
