# 无限人族进化 本地 Codex 工程接手说明

更新时间：2026-10-06，日本时间。接收方：用户电脑上的 Codex，Work 模型统一 GPT-6.1 Sol。

## 1 接手决定与当前结论

用户已将工程代码修改、构建、测试和调试交给本地 Codex。dot 暂停承担工程开发，只负责运营、在自己的电脑上通过已授权 ChatGPT 网页对话获取美术资源，以及读取 GitHub 汇报进度。本文件是一次性工程交接，不启动任何本机任务。

**目前完整 Windows Web 门禁仍失败，不能宣称三项旧失败全部解决，也没有新候选 EXE 或候选原生试玩通过证据。** 最新完整输出是 908 项，904 PASS / 4 FAIL，且在人工停止挂起测试资源后才收尾。先解决浏览器测试进程生命周期与清理问题，再恢复完整构建及原生验收。

旧 Round 3/4 指南中的“本轮只测试、不改代码”约束属于当时的测试任务。用户本次已将工程修复交给本地 Codex；接手后可在用户授权范围内开展修复，但必须新建自己的修改提交并记录新测试身份，不能把历史受测 SHA 改写成新代码。

## 2 仓库和版本身份

公开仓库：https://github.com/grassoflty-dev/wuxian-renzu-jinhua

- 接手分支：`candidate/preview5-exit-return-station-20261006`
- 本交接文档写入前核实的分支 HEAD：`c63f721fd5a4ba20018bfdb9d2a8faeda51e0a87`，是 Round 4 结果文档提交
- 最新受测源码：`a5dcc63cddd491d7e656369bd01ce3421f9eba39`；Git tree：`25b4ab7f6ebe888c22b5c835eefd819c33df2024`
- Round 4 指南提交：`01f4ff85d30de5df86810f70aee9844a16b7503f`
- Round 3 受测产品源码：`bb6c2c3713d1d897a83f641e58b1716f1c5cda8c`
- 公开 main：`6420272d48ad62be6ae80fa1a129b0a1777ef792`，仍是旧 preview.5 测试记录所在基线
- 最新既有 Release：[v0.1.0-preview.5](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/releases/tag/v0.1.0-preview.5)，标签提交 `b85ac67644b6730b268293339f62e37cf73059fa`
- 新候选尚未合并 main，没有新标签、新 Release 或新公开 EXE。旧 preview.5 EXE 不含本候选修改

分支 HEAD、产品源码 SHA 和实际 EXE SHA 是不同身份。读取文档用最新分支；复现既有失败固定 a5dcc63；修改后必须使用新的完整源码 SHA 重新测试。

## 3 先读这些固定证据

以下是本交接时的固定版本；以后追加结果时保留原文：

- [Round 4 指南](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/c63f721fd5a4ba20018bfdb9d2a8faeda51e0a87/docs/testing/WINDOWS_ROUND4_CANDIDATE_GUIDE.md)
- [Round 4 结果](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/c63f721fd5a4ba20018bfdb9d2a8faeda51e0a87/docs/testing/WINDOWS_ROUND4_RESULTS.md)
- [Round 3 结果](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/c63f721fd5a4ba20018bfdb9d2a8faeda51e0a87/docs/testing/WINDOWS_ROUND3_RESULTS.md)
- [preview.5 历史结果](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/6420272d48ad62be6ae80fa1a129b0a1777ef792/docs/testing/WINDOWS_TEST_RESULTS.md)
- [构建说明](https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/a5dcc63cddd491d7e656369bd01ce3421f9eba39/BUILDING.md)，以及同提交的 SOURCE_LINEAGE.md、KNOWN_LIMITATIONS.md

公开结果文档提供脱敏测试结论；本交接未独立复现 Windows 结果，不能据此宣称候选原生验收完成。

## 4 已交付代码和验证边界

bb6c2c3 的产品修改：

1. D01 退出：`server-rs/tauri.conf.json` 给本地 main 窗口增加 `core:window:allow-close`。正常菜单退出及已有命令排空顺序保留。源级配置/mock 测试通过不证明 Windows 原生 ACL 和进程退出正常
2. 归航站：`ReturnStationArchitecture.ts`、`ReturnStationRoomCamera.ts`、`WorldRenderer.ts` 的临时建筑层次、整房间固定相机、resize/旧帧/Aim 保护和初次场景完成处理
3. HUD：`main.ts`、`ui/Hud.ts`、`visual-authority.css` 隐藏普通 HUD 的内部场景 ID 和日常 idle，受支持动作中文化；未完成全面 UI 美术改版

a5dcc63 只改变以下测试文件，没有为迎合断言改变生产渲染、地图、素材、协议或依赖锁：
- apps/web/tests/accessibility-browser-cleanup.mjs
- apps/web/tests/accessibility-scale-browser.mjs
- apps/web/tests/hub-lifecycle-regression.mjs
- apps/web/tests/support/browser-cleanup.mjs
- apps/web/tests/world-renderer-demand.mjs

对应意图是浏览器正常关闭/等待/有限清理重试、用实际 render 探针替代已删除 HUD 节点，以及按 ROOM-FIT 计算受击脚点预期。源码级检查是局部证据；下面的实际 Windows 结果优先。

## 5 真实测试现状与未解决项

### Round 3 历史

固定 bb6c2c3，884 项：881 PASS / 3 FAIL。
- R3-F01：accessibility profile 清理 EBUSY
- R3-F02：hub lifecycle 读取已删除场景 HUD，current.scene 得到 undefined
- R3-F03：受击标记脚点仍按旧宽度比例预期，和 ROOM-FIT 不符

### Round 4 最新

固定 a5dcc63，2026-10-06 17:05–17:10 JST：
- npm ci、typecheck、首次 Web build 均退出 0
- npm test 最终退出 1：908 项，904 PASS / 4 FAIL，0 SKIP / CANCELLED / TODO
- 三个 worker 挂起超过额外等待，精确核对本轮资源身份并停止后，父测试器才输出最终统计。不能把它描述为自然完整退出
- R3-F03 原 contact 用例及新增 ROOM-FIT 宽/高/resize/普通场景用例在全套中 PASS；仅是合成测试通过
- R3-F01/F02 尚未闭环。报错变化或 render 探针改写不能单独证明修复
- 独立聚焦复测、浏览器重复、最终生产重建、Rust、候选 EXE、R02–R10 原生操作均 NOT_RUN；R01=FAIL

四个真实失败签名：
1. R4-F01，`tests/accessibility-scale-browser.mjs:66:1`：connect `:26:41` 报 `Browser exited 0`，约 245.639 ms；未完成主体断言
2. R4-F02，`tests/hub-lifecycle-regression.mjs:210:1`：清理 `first_party_sets.db-journal` 时 EBUSY（errno -4082），约 39,627.7817 ms；主体是否另有错误仍需原日志确定
3. R4-F03，`tests/hub-viewport-fit.mjs:73:1`：清理 `first_party_sets.db` 时 EBUSY，约 39,304.7847 ms；不能据此推断原生布局故障
4. R4-F04，`tests/inventory-panel-browser.mjs:38:1`：清理 `lockfile` 时 EBUSY，约 10,463.8841 ms；是新增暴露的独立浏览器失败

进程证据观察到携带 `--edge-skip-compat-layer-relaunch`、使用本轮 profile 的新 Edge PID，而原受追踪 launcher 已退出。这只是进程重启/生命周期线索，**不是确认根因**。需调查真实 CDP 浏览器进程、launcher 关系、服务器/socket 收尾和四套件清理覆盖。不通过禁用防护/沙盒/兼容层、全局杀 Edge、吞异常或跳测试解决。

旧 preview.5 曾实际启动，确认菜单退出 ACL 失败；同进程存读档的局部操作有证据。它不能证明新候选正常退出、重启继续、画面或存档闭环通过。

## 6 本地接手顺序

1. 只读核对用户指定工作区、origin、分支、HEAD、dirty/untracked、正在运行进程和磁盘。保存旧工程状态；不要在旧开发目录 reset、clean、覆盖或自动 stash
2. 用新目录复现固定 a5dcc63；保留锁文件、完整资源和 Git 身份。准备修改时从明确基线建立独立工作分支，记录实际改动，避免文档 checkout 与构建 checkout 混用
3. 优先逐项复现四个公开记录的浏览器失败，区分主体错误与 finally/清理错误。复现 launcher 退出但实际浏览器继续运行的关系，确认只操作本轮创建资源
4. 在用户授权范围内修复测试基础设施；保留真实行为断言、主体及清理错误。不得删断言、随实际值放宽预期、改 FAIL 为 SKIP/PASS、只靠聚焦数量宣称全套通过。若发现产品缺陷，提供复现和对应修复，不能无证据将测试错误当产品故障
5. 完整 Web 自然收尾通过，并单独复测原失败、重复真实浏览器流程，才继续最终生产构建和 Rust/native
6. 编译后核对新 EXE、bundle、源码 SHA；按 R02–R10 原生验收，尤其 D01 退出、重启自动/命名档、归航站构图/HUD、移动 Aim/resize 和其他世界相机恢复
7. 回传源码提交、命令、退出码、完整/聚焦各自统计、失败/未执行项和必要脱敏证据。没有实际截图不得宣称视觉达到效果图
8. 本地 Codex 明确向用户确认已接手、当前工作分支和下一步；dot 后续只读这些 GitHub 记录

## 7 构建命令和身份要求

使用项目锁文件。Round 4 实际环境记录：Node 24.14.0、npm 11.9.0、Rust/Cargo 1.98.0（MSVC x64）、Edge 154.0.4258.53；这些是该次环境证据，不要求擅自升级或替换用户工具。项目为 Rust/Tauri 2、TypeScript/Vite/Pixi。缺工具或执行受策略限制时报告，不绕过。

先在全新独立 clone 固定复现提交：
```powershell
git clone --single-branch --branch candidate/preview5-exit-return-station-20261006 https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git <新的源码目录>
git -C <新的源码目录> checkout --detach a5dcc63cddd491d7e656369bd01ce3421f9eba39
git -C <新的源码目录> rev-parse HEAD
git -C <新的源码目录> status --porcelain=v1 --untracked-files=all
```
上面目录占位符必须换成真正全新目录，不整段盲贴。修复后用新提交 SHA 建干净构建对象，不伪造原 SHA。

在 apps/web 顺序执行，每项保存输出及退出码；失败即停止依赖阶段：
```text
npm ci
npm run typecheck
npm run build
npm test
node --test tests/accessibility-scale-browser.mjs tests/hub-lifecycle-regression.mjs tests/world-renderer-demand.mjs
npm run build
```
新暴露的 hub-viewport-fit 和 inventory-panel-browser 也应各自复测；真实浏览器流程必须实际执行，SKIP 不能算通过。最后再次 build 必需，因为 npm test 会向 dist 写测试模块。

在 server-rs：
```text
cargo test --locked --lib
cargo build --locked --release --features tauri/custom-protocol
```
按 BUILDING.md 另行记录集成/受控回放测试范围；lib 不代表全套 Rust 通过。不要改锁、复用旧 target/dist 或把无 Git 的恢复目录伪装成官方源码。

核对 `apps/web/dist/bundle-identity.json` 的 gitSha、clean 状态、资源/场景身份及 hash；记录实际 `wuxian-horror-ch1.exe` 的大小/SHA256/生成时间。若 CARGO_TARGET_DIR 指向旧工程，先处理隔离授权，不覆盖旧产物。首次 Web build 身份不是测试后的最终 native-ready bundle。

允许执行时，仅给本轮游戏子进程设置独立 `WUXIAN_FORMAL_SAVE_DIR` 和 `WUXIAN_NATIVE_DIAGNOSTICS_DIR`，随后恢复调用进程原值。实际检查保存路径、PID、窗口和 F3；不要直接双击进入默认旧存档。权限拒绝记 BLOCKED，不改安全设置或换接口绕过。

## 8 玩法和视觉约束

以用户已确认的主策划 Chat 方案为准；本交接不授权自行改玩法。

- 归航站实体房间使用环形门、工业支撑/管线；蓝色球体属于抽象世界网络 UI，不把两张不同设计混成一个物理大厅
- 当前结构是程序化占位表达，正式美术尚未完成；不能用效果图当运行时整幅背景冒充真实场景
- ROOM-FIT 只作用于 return_station / rs_core_room：整房间固定相机，按真实 XYZ 几何、终端和已准入角色包络适配，横纵各 5% 留边；resize/重新进入重算，离开恢复原相机
- 不套用已废弃的固定像素角色高度或旧固定 ppm 预期；HUD 可读性与遮挡另验
- 保持世界米制、碰撞/导航、交互坐标及距离、移动和存档语义。公共候选沿用既有 idle/临时形变，没有交付新的 Walk 美术包
- 滚轮远近层、感知技能、背后敌人/声音提示、模块化原创技能等仍需按确认方案及实际源码逐项核查，不能从需求描述推断实现完成
- 用户明确指出地图与视觉/UI 设计图差距大。验收必须有对应新构建的无 HUD 场景截图和带 HUD 交互截图，检查层次、裁切、角色/终端可读性与操作；测试数量不替代视觉验收
- 缺素材先用占位并记录需求；dot 承担后续已授权网页 Chat 美术获取，代码负责人提供清楚的素材规格与引用，不自行使用付费生成

## 9 Git 和证据安全

保持 GitHub Actions 关闭，不 dispatch、不新增工作流、不启用额外收费功能、不改仓库可见性。正常提交和非强制推送，先读远端新内容；发生分歧保留双方记录。公开内容仅放已获授权源码和脱敏记录。

保护旧工程未提交改动、存档、原始素材、工具/SDK、唯一 EXE 和测试证据。日志含个人路径或凭据时先脱敏；不要上传完整进程命令行、存档或私人信息。现有测试记录保留失败和干预事实，新 Run 追加，不改写过去。

## 10 接手确认与后续清理

用户表示会在确认接手后再让本地 Codex 删除 GitHub 测试文件夹和交接文档。**本交接不执行删除，也不是立即自动删除指令。**

接手确认至少包括：已读本文件及最新结果、核对基线和本地旧工程保护、保存必要证据、列出四个当前失败及原生未验证项、确定后续工程分支。之后若用户明确下达清理指令，先核对精确范围与依赖，保留必要归档/提交引用再操作。

候选临时交接范围可供用户核对：`docs/testing/` 和 `docs/CODEX_ENGINEERING_HANDOFF.md`。**不要把它理解为删除 `apps/web/tests/`、`server-rs/tests/`、验证工具、源码测试或所有名称含 test 的文件。**常规 Git 删除仍保留历史；本文件不授权重写历史、清空远端或永久删除不可恢复证据。
