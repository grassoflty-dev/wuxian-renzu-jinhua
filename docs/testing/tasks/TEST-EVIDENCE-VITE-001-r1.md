# TEST-EVIDENCE-VITE-001 / revision 1

- task_type: test
- code_sha: e4d3a864ae4156e256b4ee5951d1dd4c74bfdb1f
- code_tree: d66b89383079a2098434b84090b310f15621d11d
- source_branch: candidate/test-evidence-vite-20261006
- result_branch: candidate/preview5-exit-return-station-20261006
- result_path: docs/testing/results/TEST-EVIDENCE-VITE-001/<run-id>.md
- executor: 用户本地 Windows Codex，GPT-6.1 Sol

本任务仅在 TEST_QUEUE.md 的同 ID/revision 为 READY、成功领取为 RUNNING 后执行。文档提交与源码提交分别记录；不得把队列分支代码当作受测源码。

## 目的与范围

复核 Node 命令证据记录器与已保存 Vite 资源保留 WIP 的纯 mock 测试。源码在公开 WIP 82d89f091686fd46e8d6b698f8c7f449be008f5e 上仅增加 tools/testing 的三个文件；没有合并 EDGE04、其他独立修复或改动原 Vite 实现。旧 PowerShell 包装器丢失的退出信息仍为 UNKNOWN；新记录器不修复该包装器，也不补造旧结果。

不运行 Edge、真实 Vite 服务、完整 Web、Rust 编译或原生游戏。本任务通过只表示这些固定版本的语法/纯 mock 检查通过，Windows 原生游戏仍待验证。

## 前置与领取

1. 先读取同分支 GITHUB_TEST_LOOP.md；确认没有本轮/其他冲突 RUNNING 或已执行的同 ID/revision。领取时使用唯一 run_id，匿名 claimed_by、JST 时间和上面结果路径，非强推并确认远端成功后执行
2. 使用独立干净源码工作区；保留旧工程、未提交改动、存档、工具。git rev-parse HEAD 必须为 code_sha，tree 必须一致；记录 git status --porcelain 为干净。结果文档工作区、请求 JSON、原始日志和临时输出放源码工作区外
3. Windows 和现有 Node 24.x；记录 node --version、解析后的 Node 可执行文件身份。缺少 Node 24 则 BLOCKED，本任务不安装依赖，不运行 npm install/npm ci。全部指定 mock 只需 Node 内置模块
4. 只读报告证据盘剩余字节、是否存在冲突构建/测试以及实际核查时间；不要公开个人绝对路径、完整进程命令行。至少预留 256 MiB 给本轮小日志/临时输出；不足就 BLOCKED，不自行清缓存。没有 Rust target 清理、全局进程清理或权限修改
5. 校验下面固定输入 hash 与源码来源。manifest 只能证明列出文件，不能替代完整 clean SHA 检查。任务前后再检查工作区和输入未改变

## 记录方式与顺序

阅读 tools/testing/README.md。使用 tools/testing/record-command.mjs：每条请求给 resolved Node executable 加下列 argv、cwd 为隔离源码仓库根、sourceSha 为固定 code_sha、inputs 为下方所有 repository-relative path/hash、timeoutMs 为 30000、outputDir 为源码外本 run 下全新阶段目录。输出目录父目录须存在。请求 JSON 同样仅保存在本地。

每阶段调用：node tools/testing/record-command.mjs <该阶段请求JSON>。不使用 shell 拼接命令；数组参数逐个传递。记录器外层调用上限 40 秒，任务命令总预算 5 分钟。超时或记录器无法返回时标 BLOCKED/UNKNOWN，停止依赖阶段，不自动重试；不能用强杀后的结果算 PASS。

记录器只约束直接子进程，不能保证整个子孙进程树收尾；只可运行这里列出的可信、输出有界命令。日志流式写盘但没有大小上限。本轮不会启动服务器/浏览器；自测含受控短时 Node 子进程和 4 MiB 输出案例。不要按名称终止所有 Node/Edge，也不要删除不明进程资源。正常阶段必须自然退出；自测内部预期的受控 timeout 案例仅是断言成功，不是外层超时。

按以下顺序逐条串行执行；任何阶段失败/阻塞时停止后续依赖阶段并写 NOT_RUN，不修改断言或原生产代码：

1. recorder-self：argv = ["--test","--test-isolation=none","--test-reporter=tap","--test-concurrency=1","tools/testing/record-command.test.mjs"]。Windows 预期总计 10，PASS 9、SKIP 1（missing captured log/unlink Linux 用例），FAIL/CANCELLED/TODO 0。明确保存 skip 原因，不写成 10/10 全通过
2. syntax-hub：["--check","apps/web/tests/hub-lifecycle-regression.mjs"]
3. syntax-adapter：["--check","apps/web/tests/support/vite-session-resource.mjs"]
4. syntax-vite-tests：["--check","apps/web/tests/vite-session-resource.mjs"]
   三条 syntax 均预期 exitCode=0、stdout/stderr 各 0 字节；必须保留自然退出、观测起止时间、耗时和空文件 SHA256。空 stdout 本身不能推断成功
5. vite-12：["--test","--test-isolation=none","--test-reporter=tap","--test-concurrency=1","apps/web/tests/vite-session-resource.mjs"]，12 PASS，FAIL/CANCELLED/SKIP/TODO 均 0
6. combined-42：["--test","--test-isolation=none","--test-reporter=tap","--test-concurrency=1","apps/web/tests/accessibility-browser-cleanup.mjs","apps/web/tests/browser-profile-consumer-gate.mjs","apps/web/tests/vite-session-resource.mjs"]，42 PASS，FAIL/CANCELLED/SKIP/TODO 均 0。这包含前一步 12 项，不能相加成 54 个不同测试

每个外层记录应为 EXIT_ZERO、exitCode 0、naturalExit true、inputsUnchanged true、无记录错误；再单独核对 TAP 统计。退出 0 不替代断言数量校验。没有执行的检查标 NOT_RUN，未知退出码标 UNKNOWN，不能写 PASS。

## 固定输入 SHA256

| Repository path | SHA-256 |
|---|---|
| BUILDING.md | d970d9cd7a6e333a48f0df7b9bc74ed6f9b9a258219061c0371ef353e0c110b0 |
| apps/web/package.json | 3ede498084460fc2a839f52cb3f7e35026dc67d11ab453a2b7fd6fda9f59b3f2 |
| apps/web/tests/vite-session-resource.mjs | bb3b7ebd6f0e198da2608997141552fb3ee522feb5b1067db7e0ed61362c7c1d |
| apps/web/tests/support/vite-session-resource.mjs | 8c13b92cc9190b2dbe7bd9ec8a691de9df226da48e3b56a6ba3fd051fe631b47 |
| apps/web/tests/hub-lifecycle-regression.mjs | ee640761a8276cf05aa084dd155715e0786409ac011e3175cc7916140251ccb8 |
| apps/web/tests/support/browser-session.mjs | 36d753b2cafc1c55016caad1edd973351921d518d50897dc8dcdddf7c8f9683c |
| apps/web/tests/support/browser-cleanup.mjs | 0e8e58e26f0032a734ebd603572d08f3fed95e4f82833261167a6de69dcac74d |
| apps/web/tests/support/browser-process-ownership.mjs | 4f5e3215a1b2ba955f4ee8845f87a766eb3a48201792837110c3ad9751af8de5 |
| apps/web/tests/accessibility-browser-cleanup.mjs | 1f54a8bcb948dcef08bdb8e7781a7841ff1233b3873c9ac2f1754eccb559fb87 |
| apps/web/tests/browser-profile-consumer-gate.mjs | 266bc13ec76693104a470aff5787d9c2f3482314a081a8c36a5d08bb45b2a508 |
| tools/testing/README.md | 07ae72f1a1cb57000194ad62dacddb9fe155cf9875bd4b1a4b1f1d22fcb88060 |
| tools/testing/record-command.mjs | d98bd90b8367f86a0364411fa99f8373e107fa658ff98377c6b57bc03e3d2fd7 |
| tools/testing/record-command.test.mjs | ac039d712f15f03dcb25c3d1c4ad3ccb0b33c1707a5b41d9a22dbc250535fe38 |
## 回传与结束

结果按协议追加到 result_path；记录任务 ID/revision、队列文档 SHA、领取提交、固定源码 SHA/tree、干净状态、Node/OS、每阶段完整 argv（路径脱敏）、JST 起止、耗时、退出码、统计、自然退出及输入身份。只上传简明结果与本地证据相对路径/大小/hash、必要首错摘要；不上传原始日志、metadata 中个人绝对路径、缓存、node_modules、Rust target、EXE 或源码副本。

本地保留所有原始 metadata/stdout/stderr/request 作为证据；不得覆盖旧 run。本轮不删除临时证据/旧产物。成功提交结果后仅当队列仍匹配 task_id/revision/run_id，才非强推改为 PASS/FAIL/BLOCKED 并链接结果。冲突则重读并保留他人领取/新任务，不自动重跑。

验收通过不授权集成、发布新 EXE 或扩大测试。后续由助手读取反馈及主 Chat 审查结果后发布新 revision/任务。
