# Round 4 候选：测试门禁修复与 Windows 复测结果

指南：[WINDOWS_ROUND4_CANDIDATE_GUIDE.md](WINDOWS_ROUND4_CANDIDATE_GUIDE.md)

受测源码固定提交：`a5dcc63cddd491d7e656369bd01ce3421f9eba39`（发布前必须填入已核验代码提交；不得凭分支 HEAD 猜测）。
候选分支：`candidate/preview5-exit-return-station-20261006`

## 与上一轮的关系

Round 3 的 [原始结果](WINDOWS_ROUND3_RESULTS.md) 保持不变：固定源码 bb6c2c3713d1d897a83f641e58b1716f1c5cda8c 的 Windows 全 Web 测试为 881 PASS / 3 FAIL，后续原生阶段没有执行。本轮修复的是三处测试门禁：浏览器 profile 收尾竞态、已删除 HUD 场景节点的旧断言、ROOM-FIT 场景的旧比例断言。没有为迎合断言修改生产渲染、地图、素材、协议或锁文件。

## 交付前云端源级检查（2026-10-06）

- TypeScript typecheck：PASS；测试模块编译：PASS
- 相关聚焦套件：155 tests，152 PASS / 0 FAIL / 3 SKIP；包含清理的 20 项确定性用例、hub render 探针与既有相机/HUD/session 检查。三项 SKIP 为真实 accessibility 浏览器、hub lifecycle 浏览器和 hub viewport 浏览器，因当前 Linux 未配置 Windows Edge；不能计入 Windows 通过
- 浏览器清理确定性测试覆盖延迟退出、关闭超时、仅本轮 child 终止、进程未退出时保留 profile、短暂/持续 EBUSY/EPERM/ENOTEMPTY、主体失败与清理失败同时保留。Linux 实际 Node child 加真实临时目录清理 smoke 通过，仍不代替 Windows Edge
- 构建命令 npm run build：BLOCKED，文本恢复树没有 Git 元数据，报 E_BUILD_IDENTITY_GIT_COMMAND；未伪造 Git/构建身份，也没有产出可交付原生 bundle
- 完整测试曾在缺少权威资源/叙事 fixture 的恢复树运行；不是完整 checkout 验收。保留报错和超时记录。30 秒进程预算下退出 124，测试输出 742 tests：701 PASS / 36 FAIL / 1 CANCELLED / 4 SKIP。缺失文件及挂起导致该次无法完成全套门禁；不得用此数字与 Windows 884 项或后续聚焦结果叠加
- 渲染资产依赖修复测试：PASS。通过当前资源清单逐项 SHA256 验证，从既有源码归档恢复 114 份 PNG/WebP（242,486,711 字节）到独立测试 overlay；权威 manifest 校验 pinned hash；3 份缺失 scene JSON 来自固定公开提交 617560096ab94b48c9e080817a861adbda71c0c0 并核对 Git blob SHA。完整 world-renderer-demand.mjs 为 49 PASS / 0 FAIL / 0 SKIP；其中四项本轮 camera/contact 用例独立复核亦 4/4 PASS，不与 49 项相加。没有伪造 fixture 或修改生产资源，也不是原生画面验证
- Rust、Windows 构建、Windows Edge、候选 EXE 和原生交互：NOT_RUN

## Windows 追加模板

### Run <UTC时间-随机ID>

- 阶段及开始/结束时间（Asia/Tokyo 与 UTC）：
- 关联前一 Run：
- 文档工作区 HEAD 与 commit URL：
- 受测固定源码 SHA、Git tree、构建前后 clean：
- Node/npm/Rust/MSVC/SDK/Windows/WebView2/Edge 实际版本：
- 构建/完整测试/聚焦复测的精确命令、退出码与日志：
- 完整测试实际总数及 PASS/FAIL/SKIP；聚焦结果单列，不相加：
- 两项真实浏览器用例是否实际执行及重复执行结果：
- EXE 脱敏路径、大小、SHA256、生成时间：
- 最终 bundle-identity.json SHA256、gitSha、sourceTreeDirty、资源/场景清单身份：
- PID/映像路径、窗口/F3、尺寸/DPI 可观测项：
- 隔离 saves/diagnostics 及实际写入检查：

| ID | 状态 | 实际步骤/预期/实际 | 证据与限制 |
|---|---|---|---|
| R01 | NOT_RUN | | |
| R02 | NOT_RUN | | |
| R03 | NOT_RUN | | |
| R04 | NOT_RUN | | |
| R05 | NOT_RUN | | |
| R06 | NOT_RUN | | |
| R07 | NOT_RUN | | |
| R08 | NOT_RUN | | |
| R09 | NOT_RUN | | |
| R10 | NOT_RUN | | |

### 缺陷与阻塞

写清发现时间、固定源码/EXE 身份、复现步骤、预期/实际及证据；不以 mock 通过替代原生验收，不因测试框架修好而将 D01/画面/存读档标为通过。策略拒绝保留原始动作与返回，不绕过。

### 证据索引

编号、种类、SHA256、对应步骤及局限。完整日志/截图/存档/EXE 留在本机；公开文本脱敏，注明读者不能访问原件。

### 写回核验

普通提交和非强制推送后，核对远端 commit、文件内容及链接再填写。只追加本文件；不改历史 Round 3 结果，不上传 EXE、不建 Release、不改 Actions/可见性/付费设置。失败时保留本地文档与补丁，不声称已上传。

## Windows 本机实际记录

### Run 20261006T080316Z-a5992912

- 阶段：全新固定源码获取、工具/锁/资源身份核查及完整 Web 门禁。门禁失败且测试 worker 挂起后，保存证据、停止本轮测试进程，未进行最终 Web 恢复构建、Rust 或原生阶段。
- 开始：2026-10-06 17:03:17.008 Asia/Tokyo（UTC+09:00，08:03:17.008Z）；Web 门禁结束：17:10:19.249（08:10:19.249Z）；收尾核验结束：17:10:54.774（08:10:54.774Z）。写回为后续独立阶段。
- 关联前一 Run：Round 3 `20261006T064040Z-49ae1ae7`，其原始失败、模板和统计未更改。本轮不叠加前轮、云端、重复或聚焦测试数字。用户指定 GPT-6.1 Sol，由当前会话直接执行，没有子代理；模型配置不是程序身份。
- 指南/文档 clone 起点：`01f4ff85d30de5df86810f70aee9844a16b7503f`（[提交](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/commit/01f4ff85d30de5df86810f70aee9844a16b7503f)）；写回前候选分支 `ls-remote` 仍为该 SHA，origin 为本公开仓库。
- 固定源码 `a5dcc63cddd491d7e656369bd01ce3421f9eba39`，Git tree `25b4ab7f6ebe888c22b5c835eefd819c33df2024`；独立源码 clone detached HEAD，下载后、门禁后、收尾的 `status --porcelain=v1 --untracked-files=all` 均为空。忽略目录的编译产物不算未生成。
- 新隔离目录（脱敏）：`<USERPROFILE>/WuxianRound4Tests/<RunID>/{source,documents,evidence,saves,diagnostics}`；独立文档 clone 与源码 clone 分开，没有复用第三轮 target/dist。旧开发区/正式 release 仅只读核状态，既有 dirty/untracked 保持原状；未执行 pull/checkout/reset/clean/stash/清缓存。存档没有复制、删除或覆盖。

#### 工具、环境与对象身份

- Windows 11 家庭版中文版 10.0.26200，64 位；PowerShell 7.6.5；Node v24.14.0，npm 11.9.0，Git 2.53.0.windows.2。
- rustc 1.98.0（`88d9e12ae`，2026-08-18），host `x86_64-pc-windows-msvc`，LLVM 22.1.8；cargo 1.98.0（`797e8a9bc`，2026-08-05）；active toolchain `stable-x86_64-pc-windows-msvc (default)`。只检查已有工具，未安装或升级工具链。
- 既有 MSVC 工具目录 14.44.35207，link.exe 文件版本 14.44.35228.0；SDK Lib 目录 10.0.22621.0；Rust 未执行，因此不声称已通过编译/链接。
- 标准安装路径实际 Edge 文件版本 **154.0.4258.53**；测试没有指定替代浏览器。WebView2 Application 下可见 154.0.4258.37 与 154.0.4258.53；没有候选原生进程，实际 WebView2 加载版本未观测。
- 本轮相关 `CARGO_TARGET_DIR`、`CARGO_BUILD_TARGET`、`RUSTUP_TOOLCHAIN`、`RUSTFLAGS`、WUXIAN 环境覆盖检查为空；未改全局配置。C 盘可用空间：开始 25,486,168,064 bytes；收尾 23,277,563,904 bytes。
- 实际调用本地 shell/Git/Node/npm/文件及哈希工具。使用 Computer Use 仅只读列出窗口，确认接口可用；未选择旧游戏、输入、启动本轮游戏或操作原有应用。本轮没有启动策略拒绝，也没有尝试解除历史拒绝。
- 构建前后锁文件 SHA256 保持相同：`apps/web/package-lock.json` = `3AFD3C32A4BF0E4AADE42BF17B59AE9AEFC6E9296FB03DB053C6105535E04477`；`server-rs/Cargo.lock` = `DA136A53596938569C9E28D711C5AB63D66643ED2DD4A06E263B1E7A05DCF06D`。没有重写 registry、integrity、版本或锁内容。
- **首次** Web build 的 `bundle-identity.json` SHA256 = `E31BF21B923FB6A7795B8C3FC4D33CC37914EA54D7A1606C625A33E301E23CF0`；内嵌 gitSha 为固定源码，`sourceTreeDirty=false`、`hasTrackedDiff=false`、`hasUntrackedFiles=false`，`builtAtUtc=2026-10-06T08:05:29.595Z`，appVersion 1.0.0、contentVersion `campaign-content-v1`、saveV6SchemaVersion 6。
- 资源清单 SHA256 `081bebad5eede90e6ed50f2f7cacb930a67f2295961d405f58141d23874ec414`；场景清单 SHA256 `9c2d4e33cd88027ed0d9b97bb23e6242572967f099466016b650b9fc73137fc9`。测试向 dist 输出模块后，最终生产恢复构建未执行；**该首次身份文件不是最终 native-ready bundle**。
- 本轮 `source/server-rs/target/release/wuxian-horror-ch1.exe` 不存在；无候选 EXE 大小/SHA256/生成时间、原生 PID/映像核对、窗口/F3、外窗/客户区/逻辑尺寸/DPI、原生截图/音频。没有替用旧 preview.5。
- 本轮 saves、diagnostics 均为 0 文件；没有本轮原生启动或存读档。浏览器套件依既有测试代码在 `<TEMP>` 建 profile，不声称系统缓存完全隔离；保留其残留，不另行删除文件。

#### 实际构建与完整测试顺序

在 `<RUN>/source/apps/web` 依指南顺序执行。日期均为 2026-10-06，时间为 Asia/Tokyo（UTC+09:00）。

| 命令 | 开始 → 结束 | 实际结果 | 证据 |
|---|---|---|---|
| `npm ci` | 17:05:04.481 → 17:05:20.768 | 退出 0；16.288 s；29 个锁定包 | E03、E04 |
| `npm run typecheck` | 17:05:20.773 → 17:05:24.570 | 退出 0；3.796 s | E03、E05 |
| `npm run build`（首次） | 17:05:24.571 → 17:05:32.772 | 退出 0；8.201 s；chunk 大小提示不是失败 | E03、E06 |
| `npm test`（完整套件） | 17:05:32.774 → 17:10:19.249 | **退出 1；测试器最终输出 908 tests，904 PASS / 4 FAIL / 0 CANCELLED / 0 SKIP / 0 TODO**；286.475 s，内部 duration_ms 282260.5177。挂起 worker 需人工停止后才收尾，非自然完整退出/非通过门禁 | E03、E07、E12、E13 |
| `node --test tests/accessibility-scale-browser.mjs tests/hub-lifecycle-regression.mjs tests/world-renderer-demand.mjs`（单独聚焦） | 未执行 | NOT_RUN：完整门禁已失败，依指南停止后续步骤 | 无 E08 日志 |
| 两次 `node --test tests/accessibility-scale-browser.mjs tests/hub-lifecycle-regression.mjs` | 未执行 | NOT_RUN：尚无初次通过结果，不运行通过后的重复验证 | 无 E09/E10 日志 |
| `npm run build`（最终生产恢复） | 未执行 | NOT_RUN：完整门禁失败 | 无 E11 日志 |
| `cargo test --locked --lib`（`source/server-rs`） | 未执行 | NOT_RUN：前置 Web 门禁未通过 | 无 Rust 测试日志 |
| `cargo build --locked --release --features tauri/custom-protocol`（同上） | 未执行 | NOT_RUN：前置 Web 门禁未通过 | 无 EXE |

两项指定真实浏览器用例均在完整套件中实际尝试执行，不是因缺 Edge 而 SKIP，但均 FAIL。完整套件中的原 `composed contact events tint the displayed walking mesh and clear on expiry, pause and replacement`（18.3485 ms）、新增 ROOM_FIT width/height/resize-back 及两个普通场景脚点用例均输出 PASS；这是源级/合成测试证据，不等于原生受击画面通过。聚焦复测、浏览器重复均没有独立结果，不与完整统计相加。

#### 本 Run R01–R10

**PASS 0 / FAIL 1 / BLOCKED 0 / NOT_RUN 9，合计 10。** R01 部分前置通过，但完整门禁失败；下列未执行的原生功能没有被标成实际游戏故障或通过。

| ID | 状态 | 实际步骤/预期/实际 | 证据与限制 |
|---|---|---|---|
| R01 | FAIL | 新独立源码/固定 SHA/clean/锁核对后依次安装、typecheck、首次 build、完整 test；预期全 0 后继续。实际 4 个浏览器失败且 worker 挂起，需停止本轮进程后退出 1，无最终 bundle/EXE | E01、E03–E07、E12–E14；不以局部 PASS 凑完整通过 |
| R02 | NOT_RUN | 无本轮 EXE，未冷启动、核对 PID/窗口/F3 | 无原生证据 |
| R03 | NOT_RUN | 未点击本轮主菜单正常退出并重启 | 不以 mock close/旧 EXE 替代 D01 验收 |
| R04 | NOT_RUN | 未新旅程保存、正常退出、重启自动/命名读档 | saves 0 文件；浏览器 mocked IPC 不是存档闭环 |
| R05 | NOT_RUN | 未原生观察归航站 24×16 构图、五终端、脚点及边缘遮挡 | 无候选截图 |
| R06 | NOT_RUN | 未原生检查 HUD ID/idle 隐藏、中文状态/可读性/遮挡 | 源级 HUD PASS 不代替原生 |
| R07 | NOT_RUN | 未原生测试移动/停步/Aim/普攻/残留输入 | 无实际游戏操作 |
| R08 | NOT_RUN | 未调整候选真实窗口大小/检查 resize、Aim、DPI | 浏览器 viewport 测试不是原生窗口验收 |
| R09 | NOT_RUN | 未正常推进其他世界再回归航站 | 没有可达性/相机原生证据 |
| R10 | NOT_RUN | 未原生重复暂停/菜单/保存完成退出/多击 | 停止的是挂起测试 worker，不是游戏退出验收 |

#### 缺陷与阻塞

共同受测源码 `a5dcc63cddd491d7e656369bd01ce3421f9eba39`，无 EXE。复现命令为上述干净独立 checkout 的完整 `npm test`。发现区间为 17:05:32.774–17:10:19.249；各测试器提供耗时而无独立墙钟时间，不猜测具体发现秒数。

1. **R4-F01：accessibility 浏览器连接阶段提前退出。** `tests/accessibility-scale-browser.mjs:66:1`，`real browser settings panel scales preserve panel layout, focus and restart at supported viewports`，245.639 ms。预期连接本轮 Edge 并完成设置/缩放/重启测试，实际 `Error: Browser exited 0`，来自 `connect` 的 `:26:41`，经 `withBrowserCleanup` 传播。没有完成浏览器主体断言。不是上一轮同一个 EBUSY 报错，不能视为修复验证成功。
2. **R4-F02：hub lifecycle profile 清理仍锁文件。** `tests/hub-lifecycle-regression.mjs:210:1`，39,627.7817 ms，实际 `EBUSY`、errno -4082、unlink `<TEMP>/hub-lifecycle-regression-<random>/first_party_sets.db-journal`。预期两次新旅程、保存、两次返回、继续及收尾成功；实际 test FAIL。finally 的清理报错可能遮蔽主体错误；本轮没有足够证据认定新 render 探针流程已经通过，不将它写成原生读档故障。
3. **R4-F03：hub viewport profile 清理锁文件。** `tests/hub-viewport-fit.mjs:73:1`，`hub menu and save picker fit common viewports and remain reachable in short windows`，39,304.7847 ms；实际 `EBUSY`、errno -4082、unlink `<TEMP>/hub-viewport-fit-<random>/first_party_sets.db`。主体布局是否另有错误不能由最终清理异常判定；保留 FAIL。
4. **R4-F04：Inventory 浏览器 profile 清理锁文件。** `tests/inventory-panel-browser.mjs:38:1`，`real browser Inventory panel preserves authoritative rows, keyboard focus, pending state and close/epoch safety`，10,463.8841 ms；实际 `EBUSY`、errno -4082、unlink `<TEMP>/inventory-browser-<random>/lockfile`。这是完整套件另外一项真实浏览器失败，不省略或算作旧三项之一。

**挂起与有限停止的证据边界：** 完整测试父进程已输出失败后，hub lifecycle、hub viewport、Inventory worker 仍存活；代码声明超时分别为 120、90、60 秒。经额外等待，分别超过 180 秒仍未退出，阻碍父测试器输出最终结果。17:09:23.416–17:09:31.877 保存并停止前两项挂起 worker/本轮浏览器；17:10:17.884–17:10:19.851 对仍挂起的 Inventory 同样处理。停止前核对本轮测试父 PID、精确 worker 文件名、创建时间、profile 路径/子进程关系，停止时再次核对进程身份；没有按 Edge/node 名称全局杀进程。完整日志原错误及停止证据全部保留，最终 `904/4` 是干预收尾后的测试器输出，不能包装成无干预完整验收。

进程快照还观察到本轮 Edge 出现带 `--edge-skip-compat-layer-relaunch` 参数、使用本轮临时 profile 的新 PID，而最初受追踪 launcher 已退出；这是维护者需核对的进程生命周期线索，**不是已经确认的根因或修复方案**。没有禁用兼容层、防护、扩展、沙盒、代理，没有修改测试脚本来凑通过。只停止已核身份的本轮挂起测试资源，没有关闭原有用户 Edge/游戏，没有另行删除 profile/缓存/文件。收尾核查测试父进程及指定挂起 worker/浏览器均不存在，原有 Edge 父进程仍在。

影响：完整 Windows Web 门禁不通过，指南要求停止最终生产构建、Rust、候选 EXE 和 R02–R10。下一步需维护者核对真实 Edge 启动/重启与各浏览器清理覆盖，提供获准的新固定源码或测试方案；本轮不修生产代码、不删除断言、不改期望数值、不跳失败/改记预期成功。

#### 证据索引

原件仅在本机 `<RUN>/evidence`；首次 bundle JSON 仍在 `source/apps/web/dist`。**没有上传完整日志、进程命令行、profile、截图、存档或 EXE，公开读者不能访问原件。** 公共文档仅脱敏必要节选及 SHA256。没有 E08–E11 的执行证据；预备但未执行的原生构建脚本不算测试步骤。

| 编号/种类 | SHA256 | 覆盖与限制 |
|---|---|---|
| E01 setup transcript | `D5FA034B222617933532022583661D11DA193523FAE3291D7C481AED8592276F` | 工具/版本/磁盘/旧工程状态、全新固定源码 checkout |
| E02 document clone transcript | `019DB4524B19AA75FB6E14D77EB06796A67AE7CEC29F850D8719F77DF7E2D02C` | 独立 sparse 文档 clone，起点 01f4ff8 |
| E03 Web pipeline transcript | `16F229F79CBA1A0E0CFE2AB9D4F86884EA3A727FCCC7E85F21BEFFE5DDD502A3` | 实际命令、时间、退出码、失败停止、锁/clean/磁盘 |
| E04 npm ci log | `4D94289DB0ED32C505B7B7E594267CB7EA577F9A801696307B91E8F67D9C2DDA` | 实际锁定安装输出 |
| E05 typecheck log | `76B95E6AB60DC2664BB45585431ABE1164F828875C52635A093BB7EAC7E82FEC` | 类型检查输出 |
| E06 first Web build log | `152ADF1FBE7A59CCF414BB6962FBD4C7AA97FE4B98099FD630B7F6EF69073BFD` | 首次生产构建，不是最终 bundle |
| E07 full Web tests log | `905555DFD9D6A560FEDE338B8E72C7A862341D3D75DD0D0749F10730AA810900` | 908 项最终输出、四项失败原始堆栈；非自然退出收尾 |
| E12 hung-test process transcript | `DC7385428A8FF765B2452C76FD9568A572C2970A8608975F815A10E48A4B9339` | 前两挂起 worker/本轮浏览器精确身份与停止记录；不是 Rust pipeline |
| E13 hung-Inventory transcript | `2EF8991C1289B118D2CCD2D134A82B5301C8F2135BC50B8FA435AEF93D4DDBF6` | 第三挂起 worker/profile/精确停止记录；不是 Rust 库测试 |
| E14 final identity transcript | `B0D09A56002D2FF1FBBDCEFDDE5FA834459A0769C85F57AC9CCAEEB62DE968E0` | 固定 HEAD/tree/clean/锁、首次 bundle、无 EXE、saves/diagnostics 空、旧工程状态与空间 |
| 首次 bundle-identity.json | `E31BF21B923FB6A7795B8C3FC4D33CC37914EA54D7A1606C625A33E301E23CF0` | 首次 clean 身份；最终生产恢复未运行 |

#### 本 Run 写回阶段

测试事实先保存到独立文档 clone，普通提交使用 `[skip ci]`，仅本文件；当前段落不预先声称推送成功。后续确认远端后追加写回核验。旧 Round 3、main、游戏代码、锁文件、工作流及仓库设置不变，不创建 Release、不上传 EXE。
