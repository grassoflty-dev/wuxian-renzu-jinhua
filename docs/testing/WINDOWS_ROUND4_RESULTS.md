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
