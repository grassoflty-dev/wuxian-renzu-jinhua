# GitHub 测试队列

更新时间：2026-10-06 22:04 JST（UTC+09:00）

本文件是唯一执行入口。协议见 [GITHUB_TEST_LOOP.md](GITHUB_TEST_LOOP.md)。普通历史指南、报告、新分支或提交本身均不触发测试。

## 当前队列

- task_id: TEST-EVIDENCE-VITE-001
- revision: 1
- task_type: test
- status: RUNNING
- availability: RUNNING_TASK
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
