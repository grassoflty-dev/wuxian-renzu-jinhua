# Round 3 候选：Windows 构建与测试结果

指南：[WINDOWS_ROUND3_CANDIDATE_GUIDE.md](WINDOWS_ROUND3_CANDIDATE_GUIDE.md)

受测源码固定提交：`bb6c2c3713d1d897a83f641e58b1716f1c5cda8c`
候选分支：`candidate/preview5-exit-return-station-20261006`

本文件与 preview.5 的旧测试记录分开。发布本文时 **尚无 Windows Round 3 Run**，未产生候选 EXE SHA、原生 PID 或实际候选截图；不能把模板或交付前源级测试计入 Windows 统计。

## 交付前源级检查（2026-10-06）

- TypeScript typecheck 与编译：PASS
- 最终聚焦测试：181 PASS / 0 FAIL / 0 SKIPPED（13 个文件；退出能力、归航站结构/ROOM-FIT/resize 与 session 完成、HUD、会话/桥接、surface、camera、Aim、临时 locomotion）
- 独立复核聚焦测试：103 PASS / 0 FAIL（12 个文件，与上项重叠，不累计）
- 新增 session 级 resize 回归覆盖初次进入、运行中重绘、反复 resize/返回原大小、draw 中 resize、取消、context loss 和 epoch 替换
- 额外 world-renderer-demand/renderer-layers 等资产依赖检查在本地恢复环境缺少 AI_ASSET_RELEASE_MANIFEST.json 等 fixture，未作为通过项；完整资产闭包/full web suite 未完成
- Rust 工具链在当前执行环境不可用；Rust 检查、Windows 构建与实机测试均未运行
- 公开源码相对 TypeScript import 闭包无缺口；地图/碰撞/导航、素材、锁文件保持基线身份

源级检查不证明原生 ACL 生效、Windows 构建通过、正常退出/重启读档闭环或真实 HUD/人物画面通过。尚未运行的项目保留该状态。

## 追加模板（保留原模板）

### Run <UTC时间-随机ID>

- 阶段、开始/结束时间（Asia/Tokyo，UTC+09:00；另保留 UTC）：
- 关联历史 Run/本轮先前阶段：
- 文档工作区 HEAD 与 commit URL：
- 受测源码固定 SHA、Git tree SHA、构建前后 clean 状态：
- Node/npm/Rust/MSVC/SDK/Windows/WebView2 版本（实际检查，不猜测）：
- 构建命令、工作目录（脱敏）、退出码、日志证据：
- 本地 EXE 路径（脱敏）、大小、SHA256、生成时间：
- bundle-identity.json SHA256、内嵌 gitSha、sourceTreeDirty、资源/场景身份：
- 实际启动方式/授权范围、PID/映像路径核对、原生窗口、F3 身份：
- 窗口/客户区或渲染逻辑尺寸/DPI；不能观测的项目：
- 本轮隔离 saves/diagnostics 及实际写入检查（脱敏）：
- 本轮实际 PASS / FAIL / BLOCKED / NOT_RUN 数量（合计 10）：

| ID | 状态 | 精确步骤/预期/实际 | 证据编号及限制 |
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

### 本 Run 缺陷与阻塞

逐条记录发现时间、受测 SHA/EXE、复现操作、预期/实际、影响范围、证据和下一步。策略拒绝写原始动作与返回信息，不猜原因、不绕过。

### 本 Run 证据索引

证据编号、种类、SHA256、对应步骤、覆盖范围。原件留本地，未经额外同意不上传；明确公开读者不能访问原件。截图不代替交互，mock 不代替原生，编译成功不代替功能通过。

### 本 Run 写回核验

普通提交/推送的结果、远端 commit URL、结果文档 URL。若被阻塞，写确切原因和已保留的本地结果，不写“已上传”。

## Windows 实际阶段记录（不替代上述模板和交付前记录）

### Run 20261006T064040Z-49ae1ae7

- 阶段：独立固定源码获取、环境检查、完整 Web 构建前置门禁；完整 Web 测试失败后停止依赖步骤。本阶段没有原生试玩。
- 开始：2026-10-06 15:40:40.999 Asia/Tokyo（UTC+09:00；06:40:40.999Z）；测试门禁结束：15:46:46.637（06:46:46.637Z）；收尾身份/环境核验结束：15:49:30.864（06:49:30.864Z）。文档提交/推送为之后的独立写回阶段。
- 本轮独立 Run，不把旧 preview.5、旧 Windows Run 或交付前 181/103 项聚焦测试计入本轮。按用户指定 GPT-6.1 Sol 直接执行，无子代理；模型配置来自用户说明，不是受测程序身份。
- 指南版本及文档 clone 起点：`a197bf9be112de58a2fa5b978c41979f2724eb39`（[提交](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/commit/a197bf9be112de58a2fa5b978c41979f2724eb39)）；写回前 `ls-remote` 候选分支仍为该 SHA。origin 核对为本公开仓库。
- 固定源码：`bb6c2c3713d1d897a83f641e58b1716f1c5cda8c`；Git tree：`d07bed12022b168b808eae3e4744fedf681da30d`。源码 clone 与文档 clone 完全分开；源码 detached HEAD。获取后、Web 门禁失败后及收尾核验 `status --porcelain=v1 --untracked-files=all` 均为空。生成的忽略目录不等于没有产物。
- 脱敏隔离路径：`<USERPROFILE>/WuxianRound3Tests/<RunID>/{source,documents-retry,evidence,saves,diagnostics}`。未使用旧开发区、旧 target、旧 Web dist 或旧 preview.5 EXE；原开发区既有 dirty/untracked 保留，只做状态读取。本轮未清理缓存、共享 Git 数据库、工作树或残留文件。
- 实际调用了本地 PowerShell、Git、Node/npm 及文件读写/哈希工具；没有启动/控制本轮游戏，也未核验本轮原生截图、键鼠、音频能力。没有解除或绕过历史启动策略拒绝。

#### 环境、锁文件与产物身份

- Windows 11 家庭版中文版，10.0.26200，64 位；本轮自动化 shell 为 PowerShell 7.6.5（不是用户之前手动命令的 PowerShell 5.1）。Node v24.14.0，npm 11.9.0，Git 2.53.0.windows.2。
- rustc 1.98.0（`88d9e12ae`，2026-08-18），host `x86_64-pc-windows-msvc`，LLVM 22.1.8；cargo 1.98.0（`797e8a9bc`，2026-08-05）；active toolchain `stable-x86_64-pc-windows-msvc (default)`。只检查已有工具，没有安装或升级工具链。
- 已有 MSVC 工具目录版本 14.44.35207，链接器文件版本 14.44.35228.0；Windows SDK Lib 目录 10.0.22621.0。本轮未实际执行 Rust 编译，以上不是链接成功证据。
- 本机 WebView2 Application 下有 154.0.4258.37、154.0.4258.53；没有候选进程，因此实际候选加载版本未观测。未改全局配置；检查的 `CARGO_TARGET_DIR`、`CARGO_BUILD_TARGET`、`RUSTUP_TOOLCHAIN`、`RUSTFLAGS`、隔离存档/诊断环境变量均未设置。
- C 盘可用空间：开始 27,476,054,016 bytes；收尾 26,172,542,976 bytes。未进行重型 Rust 构建。
- 构建前后锁文件 SHA256 相同：`apps/web/package-lock.json` = `3AFD3C32A4BF0E4AADE42BF17B59AE9AEFC6E9296FB03DB053C6105535E04477`；`server-rs/Cargo.lock` = `DA136A53596938569C9E28D711C5AB63D66643ED2DD4A06E263B1E7A05DCF06D`。`npm ci` 使用固定锁文件，不重写其 registry/integrity 或放宽校验。
- 首次成功 Web build 的 `apps/web/dist/bundle-identity.json` SHA256 = `EA7A0F1BDF9BB16DBB868C0C05A4DDAD3B363001637C6991430426081613BD97`；内嵌 `gitSha` 为固定源码 SHA，`sourceTreeDirty=false`、`hasTrackedDiff=false`、`hasUntrackedFiles=false`；`builtAtUtc=2026-10-06T06:46:22.653Z`，appVersion 1.0.0、contentVersion `campaign-content-v1`、saveV6SchemaVersion 6。
- 该首次 bundle 资源清单身份 `081bebad5eede90e6ed50f2f7cacb930a67f2295961d405f58141d23874ec414`，场景清单身份 `9c2d4e33cd88027ed0d9b97bb23e6242572967f099466016b650b9fc73137fc9`。此 JSON 在测试后仍存在，但测试已向 dist 输出模块，最终生产重建没有运行；**不得将其称为已完成的最终原生 bundle**。
- 候选 `server-rs/target/release/wuxian-horror-ch1.exe` 不存在：大小、SHA256、生成时间均无；原生 PID、窗口、F3 身份、外窗/客户区/渲染尺寸/DPI 均未观测。没有搜索或替用旧 EXE。
- 本轮 saves、diagnostics 收尾均 0 文件；未设置变量启动子进程、未产生或读取本轮游戏存档。浏览器测试临时 profile 位于系统临时目录，未声称系统/浏览器缓存全部隔离。

#### 实际命令及顺序

在 `<RUN>/source/apps/web` 执行下表；开始/结束均为 2026-10-06 Asia/Tokyo（UTC+09:00）。命令耗时包括 npm 包装步骤，不等于测试器内部耗时。

| 命令 | 开始 → 结束 | 退出码/结果 | 证据 |
|---|---|---|---|
| `npm ci` | 15:46:01.082 → 15:46:15.422 | 0；14.338 s，安装 29 个锁定包 | E02、E03 |
| `npm run typecheck` | 15:46:15.426 → 15:46:18.657 | 0；3.231 s | E02、E04 |
| `npm run build`（首次） | 15:46:18.658 → 15:46:25.467 | 0；6.809 s；有大 chunk 提示，不是失败 | E02、E05 |
| `npm test`（完整套件） | 15:46:25.469 → 15:46:46.637 | **1；884 tests：881 PASS、3 FAIL、0 cancelled/skipped/todo**；21.168 s；测试器内部 17,710.6424 ms | E02、E06 |
| `npm run build`（最终生产恢复） | 未执行 | NOT_RUN：前置完整测试失败，遵照指南停止 | 没有 E07 执行日志 |
| `cargo test --locked --lib`（`source/server-rs`） | 未执行 | NOT_RUN：前置门禁失败 | 无 Rust 测试证据 |
| `cargo build --locked --release --features tauri/custom-protocol`（同上） | 未执行 | NOT_RUN：前置门禁失败 | 无原生构建证据 |

#### R01–R10 本轮统计

**PASS 0 / FAIL 1 / BLOCKED 0 / NOT_RUN 9，合计 10。** R01 部分前置检查通过，但完整门禁实际失败，不能记 PASS；R02–R10 是没有执行，不能冒称原生功能已失败或通过。

| ID | 状态 | 精确步骤/预期/实际 | 证据编号及限制 |
|---|---|---|---|
| R01 | FAIL | 独立 clone 固定 SHA、clean、锁身份核对后依序安装、typecheck、首次 build、完整 npm test；预期全部 0 后继续。实际完整测试退出 1、3 项失败，停止最终 Web/Rust/native；没有 EXE hash | E02–E06、E09–E11；仅完成部分构建前置 |
| R02 | NOT_RUN | 未生成本轮 EXE，未冷启动、核对 PID/窗口/F3 | R01 门禁未通过；没有原生截图 |
| R03 | NOT_RUN | 未点击本轮新鲜主菜单退出或重启候选 | 不以自动化 mock close 或旧 preview.5 代替 |
| R04 | NOT_RUN | 未开始本轮旅程、保存、正常退出、重启自动/命名读档 | saves 为空；自动 Web 测试不等于原生闭环 |
| R05 | NOT_RUN | 未观测本轮 24×16 归航站、五终端、脚点或可达边缘 | 没有本轮真实合成画面 |
| R06 | NOT_RUN | 未原生检查内部场景 ID 隐藏、中文动作、HUD 可读性/遮挡 | 下列 HUD fixture 断言失败不直接证明原生 HUD 故障 |
| R07 | NOT_RUN | 未原生检查移动、停步、鼠标 Aim、普攻朝向及残留输入 | mock 不能替代实际操作 |
| R08 | NOT_RUN | 未切换本轮真实窗口大小或核对 resize/DPI/输入 | 自动浏览器缩放套件不计原生 resize 验收 |
| R09 | NOT_RUN | 未按正常路线进入其他世界或返回归航站 | 无候选世界推进证据 |
| R10 | NOT_RUN | 未原生重复暂停、菜单、保存完成退出或重复点击 | 没有强杀、破坏性排空或绕过策略 |

#### 本 Run 缺陷与阻塞

发现时间：完整 `npm test` 阶段 15:46:25.469–15:46:46.637（日志没有每条断言的独立墙钟时间）。共同受测身份为固定源码 `bb6c2c3…`，无 EXE。复现入口均为固定干净源码按上述前置步骤后运行完整 `npm test`，不是仅跑聚焦 181 项。

1. **R3-F01：浏览器测试收尾遇到文件锁。** `tests/accessibility-scale-browser.mjs:36:1`，测试名 `real browser settings panel scales preserve panel layout, focus and restart at supported viewports`。预期测试及 profile 收尾成功，实际删除 `<TEMP>/accessibility-browser-<random>/lockfile` 返回 `EBUSY`（errno -4082、syscall unlink）。只读源码显示 finally 中关闭 CDP、调用 browser.kill、关闭 server 后删除临时 profile；本轮没有另行杀进程/删缓存/重试。不能由收尾错误推断原生设置布局错误，也不能将此 test 改记通过。
2. **R3-F02：旅程生命周期 fixture 的场景断言不匹配。** `tests/hub-lifecycle-regression.mjs:180:1`，测试名 `hub lifecycle: two new journeys, save, return, continue, and recover after IPC failure`。在首次成功 New Journey 后 `:263:12` 的 `current.scene` 预期 `'gh_entry'`，实际 `undefined`，`ERR_ASSERTION` strictEqual。测试停在该断言，后续 save/return/continue 操作没有该 test 的通过证据。是浏览器加 mocked IPC 的回归失败，尚未判定候选产品逻辑与 fixture 哪方需修正，不等于已经原生验证存档损坏。
3. **R3-F03：合成受击标记脚点断言不匹配。** `tests/world-renderer-demand.mjs:493:1`，测试名 `composed contact events tint the displayed walking mesh and clear on expiry, pause and replacement`。在 `:507:10`，断言 `mark.y === sprite.y - 18 * (app.screen.width / 1280)`，说明 `contact uses the displayed elevated footpoint`；实际 `190.99383808044638`，预期 `185.82358695691204`。该 fixture 的后续 expiry/pause/replacement 断言因此没有通过证据；尚不能外推为真实原生画面缺陷。

三项均保留原始失败，不跳断言、不修改代码/测试/锁文件。整体影响：按指南完整 Web 门禁不通过，停止依赖构建与 R02–R10。下一步需由维护者核对失败原因并给出获准的新固定源码或明确的新测试方案；本轮不自行补丁凑通过。

附带获取/采证情况：首次完整文档 clone 返回 `RPC failed; curl 56 schannel: server closed abruptly (missing close_notify)`、early EOF/invalid index-pack；在新 `documents-retry` 目录普通 HTTPS depth-1/filter/sparse clone 同一候选分支成功（15:46:43.528–15:46:47.220），未关闭 TLS 校验、改身份或覆盖旧工程。E09 中一次未加引号的 `HEAD^{tree}` 被 PowerShell 解析拒绝；保留该采证错误，E10 使用引号正常重核完整 HEAD/tree。这不是游戏测试失败，也没有改变源码。

#### 本 Run 证据索引

以下原件只保留 `<RUN>/evidence`（bundle JSON 保留 `source/apps/web/dist`），**没有上传；公开读者不能访问本地原件**。仅公开脱敏文本、精确断言及 SHA256。E01 的外部程序输出在 transcript 中不全，版本以 E11、Web 原始输出以 E03–E06 为准；E09 的原始采证错误未覆盖。无原生截图、录音、存档、EXE 证据。

| 编号/原件 | SHA256 | 覆盖/限制 |
|---|---|---|
| E01 setup transcript | `580ED54F6FA1EAB068DD0B264742D1FE0C9B0EDDAD920F9D6E2CB31B2BDA8104` | 新目录/工具检查命令、初次文档 clone 失败；外部程序输出不全 |
| E02 Web pipeline transcript | `6E94DB0022354CDBA5F9944E3D0D29043A76F0E0707FDC4DDDE13464D7C31E3A` | 每步命令、时间、退出码、失败停止与源码状态 |
| E03 npm ci log | `C3C9E638BD874701ED565EA3330032AEF556E7F311D23752DB3AD30C628568A1` | 实际依赖安装输出 |
| E04 typecheck log | `76B95E6AB60DC2664BB45585431ABE1164F828875C52635A093BB7EAC7E82FEC` | 实际类型检查输出 |
| E05 first Web build log | `A2E1D1998A9641CC52AFBB70B7DC588190781A2D4EEEEA2ABB3C9D99C1598689` | 首次生产构建，不是最终恢复构建 |
| E06 full Web tests log | `D1D680590F6FDE94AC2F9D69D2E7FE4D2CB96B7CB3D5C8DBCB96691F4E867F00` | 全部 884 项结果和三项失败原始堆栈 |
| E08 document clone retry transcript | `AA6C20789322F99E16D35A11365665B94BFE4E83492081A542416476E1A528E4` | 新独立文档 clone 成功 |
| E09 final read-only transcript | `C191BB4745C7C47B010214B1D1D4A2AFEB9D168D9DC6BD487F28D5EBB4B92AF6` | 锁/首次 bundle/无 EXE/旧工程状态/磁盘；含 tree 采证语法错误 |
| E10 identity recheck transcript | `5BEDEEE5E5B3F025C83489582A1B2294299EAC3B2121D85F99F1A85DE70622AB` | 完整源码 HEAD/tree、clean、首次 bundle 身份、隔离目录 0 文件 |
| E11 environment transcript | `749221ABD63A3B8041C843A088A739CEDC714871A920AD7B45BC924DF8174D26` | 本机实际版本与相关环境变量检查 |
| 首次 bundle-identity.json | `EA7A0F1BDF9BB16DBB868C0C05A4DDAD3B363001637C6991430426081613BD97` | 首次生产身份；测试后非最终 native-ready bundle |

#### 本 Run 写回核验

本段随首个文档提交保存。待普通提交/推送及远端核验，**此处不预先声称已上传**；确切远端结果由提交完成后的回复提供。目标仅为候选分支的本文件，提交使用 `[skip ci]`；不改 main、旧结果、游戏代码、工作流/仓库设置，不上传本地原件或创建 Release。
