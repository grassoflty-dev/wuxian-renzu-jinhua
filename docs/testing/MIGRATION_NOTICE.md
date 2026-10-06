# 测试协作渠道迁移通知
migration_id: CHANNEL-MIGRATION-20261006-001
revision: 1
task_id: CHANNEL-MIGRATION-20261006-001
task_type: channel_migration
phase: PREPARE
new_repository: grassoflty-dev/wuxian-renzu-jinhua-coordination
new_branch: main
protocol_path: PROTOCOL.md
protocol_commit: 3158daa5b2f40173250dcc16b8e15c900e578fc0
control_path: CONTROL.md
queue_path: QUEUE.md

用户要求测试通信改为私有渠道。本通知仅扩展旧协议允许这一项迁移控制，不是测试任务，不运行编译/测试、不删除/安装/扩权。旧 TEST-EVIDENCE-VITE-001 r1 BLOCKED 不重跑。
核对上述固定私库协议，按现有授权验证读写，暂停旧编测 intake 并上传 PREPARED。等助手在私库发布引用 ACK 的 COMMIT 后再切换既有十分钟轮询；实际读新入口并确认旧入口不能执行，再上传 ACTIVE。只有 ACTIVE 经核验后才清理旧公开资料。
不能核实访问、发布身份或切换则 BLOCKED，不绕过、不回退公共任务。SHA 占位未替换则不可领取。私人配置、聊天和设备信息不得回传公开库。

本次仅迁移控制，不适用源码构建字段 code_sha；固定身份使用 protocol_commit。普通编测任务的40位源码 SHA 要求仍有效。本地保留旧去重账本与运行证据，发现 RUNNING 或不明运行先报阻碍，不启动重叠任务。
