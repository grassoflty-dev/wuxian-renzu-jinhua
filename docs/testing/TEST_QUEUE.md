# GitHub 测试队列

更新时间：2026-10-06 21:44 JST（UTC+09:00）

本文件是唯一执行入口。协议见 [GITHUB_TEST_LOOP.md](GITHUB_TEST_LOOP.md)。普通历史指南、报告、新分支或提交本身均不触发测试。

## 当前队列

- task_id: LOOP-BOOTSTRAP-20261006
- revision: 1
- status: DRAFT
- availability: NO_READY_TASK
- code_sha: null
- task_document: null
- claimed_by: null
- run_id: null
- claimed_at_jst: null
- result_path: null

这是协议初始化记录，不是可执行测试任务。不要构建或重跑旧 Round 3/4，也不要自行选择最新分支。现有本地 Codex 的 EDGE04、consumer gate、Vite 等独立修改尚需汇总审查；Vite 报告明确保留未验证 WIP，缺少新的已审查固定源码及测试计划。因此目前没有 READY 任务。

## 已保存资料（仅供审查，不能执行）

- candidate 的本次文档更新基线：48c2dcbb5ef6f69b7b369c3eec7c321aa3cd53e2
- EDGE04 报告：20ac3b2f0972ed17453001ec893906d422bf3603 / docs/testing/EDGE_BROWSER_CONNECTION_SAFETY_04_REPORT.md；实现 aebabd80c33f42caabde7d98dfa2e421798cf045，报告记录 102/102 clean pure mock，通过不代表真实浏览器或原生通过
- Vite 报告：8f6bb522eeca9d1e0352f4938f277000d06538b7 / docs/testing/VITE_CREATION_RESOURCE_RETENTION_01_REPORT.md；WIP 82d89f091686fd46e8d6b698f8c7f449be008f5e，PARTIAL / FAIL / NOT_READY_FOR_CANDIDATE，12 个新测试未执行，syntax 退出结果 UNKNOWN
- 历史 Round 4 的 904/908 仅对应 a5dcc63cddd491d7e656369bd01ce3421f9eba39；不能与上述局部数量合并

## READY 的必填项

由助手发布新任务时，必须填写唯一 task_id、递增 revision、完整 40 位 code_sha、同仓库固定版本测试文档、前置条件、精确命令/范围/预期、超时和安全清理边界、结果路径及判定标准。code_sha 必须可获取并对应待测试源码；文档提交 SHA 与源码 SHA 分开记录。缺任一项就不执行，报告 BLOCKED。

同一 task_id + revision 只能领取一次。FAIL/BLOCKED/PASS 均不自动重跑；只有助手明确发布新 revision 的 READY 才开始下一次。RUNNING 不因十分钟到期或电脑重启自动失效。详见协议的领取与恢复规则。
