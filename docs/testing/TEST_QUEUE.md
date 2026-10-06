# GitHub 迁移控制队列

更新时间：2026-10-06 22:40 JST（UTC+09:00）

## 当前唯一控制项

- task_id: CHANNEL-MIGRATION-20261006-001
- migration_id: CHANNEL-MIGRATION-20261006-001
- revision: 1
- task_type: channel_migration
- status: READY
- availability: MIGRATION_CONTROL_ONLY
- operational_dispatch: PAUSED
- protocol_commit: 3158daa5b2f40173250dcc16b8e15c900e578fc0
- new_repository: grassoflty-dev/wuxian-renzu-jinhua-coordination
- new_branch: main
- new_queue_path: QUEUE.md
- task_document: https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/b1b3336e7399371ca36c5bee23e2d601f5516292/docs/testing/MIGRATION_NOTICE.md
- task_document_sha: b1b3336e7399371ca36c5bee23e2d601f5516292
- code_sha: NOT_APPLICABLE_CONTROL_ONLY
- prepared_ack_path: outbox/migrations/CHANNEL-MIGRATION-20261006-001/PREPARED.md
- active_ack_path: outbox/migrations/CHANNEL-MIGRATION-20261006-001/ACTIVE.md

仅按已固定迁移通知执行协议第9节的一次性例外，不编译、不测试、不删除、不安装、不扩权。不自行启动普通工程任务，旧 r1 BLOCKED 不重跑。先暂停旧编测领取，再核实私库与上传 PREPARED；等私库 COMMIT 后才切换，实际轮询新入口且禁用旧执行源后回传 ACTIVE。任何验证阻碍保持暂停，不能回退公共任务。没有正常编测 READY。

## 迁移前队列历史（不可执行）

以下原始记录保留证据，不是并列队列，不触发领取：

# GitHub 测试队列

更新时间：2026-10-06 22:04 JST（UTC+09:00）

本文件是唯一执行入口。协议见 [GITHUB_TEST_LOOP.md](GITHUB_TEST_LOOP.md)。普通历史指南、报告、新分支或提交本身均不触发测试。

## 当前队列

- task_id: TEST-EVIDENCE-VITE-001
- revision: 1
- task_type: test
- status: BLOCKED
- availability: NO_READY_TASK
- code_sha: e4d3a864ae4156e256b4ee5951d1dd4c74bfdb1f
- source_branch: candidate/test-evidence-vite-20261006
- task_document: https://github.com/grassoflty-dev/wuxian-renzu-jinhua/blob/ee815354cc561dee9b1856f77a099568d8e259a8/docs/testing/tasks/TEST-EVIDENCE-VITE-001-r1.md
- task_document_sha: ee815354cc561dee9b1856f77a099568d8e259a8
- claimed_by: local-windows-tester
- run_id: 20261006T131725Z-0f41c6a9
- claimed_at_jst: 2026-10-06T22:17:25+09:00
- result_path: docs/testing/results/TEST-EVIDENCE-VITE-001/20261006T131725Z-0f41c6a9.md

仅按固定任务文档领取一次。Windows Node 24 证据记录器自测、3 条语法检查、Vite 12 项与组合 42 项纯 mock；不启动浏览器、Rust 或原生游戏。只有本地结果回传后才有本轮执行结论。

## 初始化历史

LOOP-BOOTSTRAP-20261006 revision 1 原为 DRAFT / NO_READY_TASK，未执行；由本 READY 任务替代。原历史指南和独立分支不会自动触发测试。

## 已保存资料（仅供审查，不能执行）

- candidate 的本次文档更新基线：48c2dcbb5ef6f69b7b369c3eec7c321aa3cd53e2
- EDGE04 报告：20ac3b2f0972ed17453001ec893906d422bf3603 / docs/testing/EDGE_BROWSER_CONNECTION_SAFETY_04_REPORT.md；实现 aebabd80c33f42caabde7d98dfa2e421798cf045，报告记录 102/102 clean pure mock，通过不代表真实浏览器或原生通过
- Vite 报告：8f6bb522eeca9d1e0352f4938f277000d06538b7 / docs/testing/VITE_CREATION_RESOURCE_RETENTION_01_REPORT.md；WIP 82d89f091686fd46e8d6b698f8c7f449be008f5e，PARTIAL / FAIL / NOT_READY_FOR_CANDIDATE，12 个新测试未执行，syntax 退出结果 UNKNOWN
- 历史 Round 4 的 904/908 仅对应 a5dcc63cddd491d7e656369bd01ce3421f9eba39；不能与上述局部数量合并

## READY 的必填项

由助手发布新任务时，必须填写唯一 task_id、递增 revision、完整 40 位 code_sha、同仓库固定版本测试文档、前置条件、精确命令/范围/预期、超时和安全清理边界、结果路径及判定标准。code_sha 必须可获取并对应待测试源码；文档提交 SHA 与源码 SHA 分开记录。缺任一项就不执行，报告 BLOCKED。

同一 task_id + revision 只能领取一次。FAIL/BLOCKED/PASS 均不自动重跑；只有助手明确发布新 revision 的 READY 才开始下一次。RUNNING 不因十分钟到期或电脑重启自动失效。详见协议的领取与恢复规则。

