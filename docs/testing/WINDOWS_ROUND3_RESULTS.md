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
