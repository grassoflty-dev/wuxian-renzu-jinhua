# 公共仓库分支归档与主线规则

日常开发与集成主线为`main`；正式发布沿用现有`v0.1.0-preview.*`标签。需要独立实现时可使用短期分支，完成集成后删除，不再把报告或每次备份长期堆为远端分支。

2026-10-11清理前公开库有26个分支。25个非主线分支均有未合入内容，无关联开放PR。本次不合入或丢弃旧代码：18个独立提交快照先保留为`archive/2026-10-11/*`标签，再删除对应旧分支入口。这些标签不是游戏发布版本，也不是验收通过。

## 恢复映射

| 原分支 | 完整提交 | 可恢复归档标签 |
| --- | --- | --- |
| `candidate/preview5-exit-return-station-20261006` | `5697a4c1fa46b2ef8b3a9e70dffe2da65875a3f5` | `archive/2026-10-11/5697a4c1fa46` |
| `candidate/test-evidence-vite-20261006` | `e4d3a864ae4156e256b4ee5951d1dd4c74bfdb1f` | `archive/2026-10-11/e4d3a864ae41` |
| `codex/backup-browser-profile-consumer-gate-01-impl-20261006` | `3a662d83a0197a0288e8f20d459a116a1475fa59` | `archive/2026-10-11/3a662d83a019` |
| `codex/backup-browser-profile-consumer-gate-01-report-20261006` | `2cefffac3e4efdd0e14700ad51ba1d1ab0a3cfdf` | `archive/2026-10-11/2cefffac3e4e` |
| `codex/backup-edge-browser-connection-safety-04-impl-20261006` | `aebabd80c33f42caabde7d98dfa2e421798cf045` | `archive/2026-10-11/aebabd80c33f` |
| `codex/backup-edge-browser-connection-safety-04-report-20261006` | `20ac3b2f0972ed17453001ec893906d422bf3603` | `archive/2026-10-11/20ac3b2f0972` |
| `codex/backup-edge-browser-lifecycle-01-draft-20261006` | `e647a5b133cd3bdca3170ba65d23454a8d811561` | `archive/2026-10-11/e647a5b133cd` |
| `codex/backup-edge-browser-lifecycle-01-report-20261006` | `cbb92ad771448cca64d2ca585f094479636309a1` | `archive/2026-10-11/cbb92ad77144` |
| `codex/backup-edge-browser-lifecycle-02-impl-20261006` | `04671aba6e76be039589ef8c40d90cbc2fc724ec` | `archive/2026-10-11/04671aba6e76` |
| `codex/backup-edge-browser-lifecycle-02-merge-20261006` | `81ceebf363722b427596feb9c4e6d2172c16d299` | `archive/2026-10-11/81ceebf36372` |
| `codex/backup-edge-browser-lifecycle-02-report-20261006` | `0232cba8f8c0ced234219a24097cdce13c68571a` | `archive/2026-10-11/0232cba8f8c0` |
| `codex/backup-edge-browser-lifecycle-03-impl-20261006` | `3bf66da0a938dff0acbda0958ccd1dbf6c24df51` | `archive/2026-10-11/3bf66da0a938` |
| `codex/backup-edge-browser-lifecycle-03-report-20261006` | `5d42696fe5d01660b06777800f32ebf88695b515` | `archive/2026-10-11/5d42696fe5d0` |
| `codex/backup-save-slot-world-labels-01-impl-20261006` | `66fa8a708f7a03183cbb3054054b76da31b31a25` | `archive/2026-10-11/66fa8a708f7a` |
| `codex/backup-save-slot-world-labels-01-report-20261006` | `fa6abb981f4763b335b66b60cf6997e8e6583343` | `archive/2026-10-11/fa6abb981f47` |
| `codex/backup-vite-creation-resource-retention-01-report-20261006` | `8f6bb522eeca9d1e0352f4938f277000d06538b7` | `archive/2026-10-11/8f6bb522eeca` |
| `codex/backup-vite-creation-resource-retention-01-wip-20261006` | `82d89f091686fd46e8d6b698f8c7f449be008f5e` | `archive/2026-10-11/82d89f091686` |
| `codex/browser-profile-consumer-gate-01-20261006` | `2cefffac3e4efdd0e14700ad51ba1d1ab0a3cfdf` | `archive/2026-10-11/2cefffac3e4e` |
| `codex/edge-browser-connection-safety-04-20261006` | `20ac3b2f0972ed17453001ec893906d422bf3603` | `archive/2026-10-11/20ac3b2f0972` |
| `codex/edge-browser-lifecycle-01-20261006` | `cbb92ad771448cca64d2ca585f094479636309a1` | `archive/2026-10-11/cbb92ad77144` |
| `codex/edge-browser-lifecycle-02-20261006` | `0232cba8f8c0ced234219a24097cdce13c68571a` | `archive/2026-10-11/0232cba8f8c0` |
| `codex/edge-browser-lifecycle-03-20261006` | `5d42696fe5d01660b06777800f32ebf88695b515` | `archive/2026-10-11/5d42696fe5d0` |
| `codex/save-slot-world-labels-01-20261006` | `fa6abb981f4763b335b66b60cf6997e8e6583343` | `archive/2026-10-11/fa6abb981f47` |
| `codex/vite-creation-resource-retention-01-20261006` | `8f6bb522eeca9d1e0352f4938f277000d06538b7` | `archive/2026-10-11/8f6bb522eeca` |
| `migration/preview3-source-20261004` | `4b142794ce64ff8ee5914a72bf1efc91f36df927` | `archive/2026-10-11/4b142794ce64` |

恢复时从对应归档标签建立新的本地分支即可。现有5个正式发布标签、主线历史与本地工程不改写。
