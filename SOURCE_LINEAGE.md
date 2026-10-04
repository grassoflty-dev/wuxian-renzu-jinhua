# 本源码候选的来源与边界

公开基线为 v0.1.0-preview.2 的对应源码附件 project/：706 个文件，规范化 Git 树 1b32b1b00e663ab9c11d8cad1eb71abf074069b5。该源码 ZIP 的 SHA256 为 97c543429fef7ef3376978af20cbf97bac9b68d4724387269216b100c38b83d8。基线发行记录声明源码提交 0ef917f530f84d2451d718168ab83b7b4938e762；源码附件不含 Git 历史，因此独立核对的是内容树。

本候选在这一公开边界内加入正式代码及测试：输入方向先于战斗边沿、独立鼠标离散攻击、精确信号怨灵受击反馈、音效回归接线、默认自动存档继续、Beacon 交互/持久化与 Sentinel 冲锋。保留正式库模块白名单、正式 Tauri 命令入口和旧网格存档的轻量只读拒绝器。未加入历史故事/角色模块、旧前端、私人工作报告或新增调试截图。

保留 docs/source/deep-research-report.md 的确切 SHA256 fcd027320a01a0fe083b09321deebc85b33598b4efc8642a6e5d624f5aaf51a1，叙事与岑遥工具使用仓库相对来源路径。原本未被 npm test 接线的 soundCueModel.test.ts 作为历史源码保留，等价回归已由 sound-cue-model.mjs 接线执行。独立 Web 验收测试还补齐当前生产协议的 SupportProjection、SentinelEncounter 与 GreyHiveBeacon 依赖。

## 依赖材料继承

server-rs/Cargo.lock 的 SHA256 为 da136a53596938569c9e28d711c5ab63d66643ed2dd4a06e263b1e7a05dcf06d；apps/web/package-lock.json 为 3afd3c32a4bf0e4aade42bf17b59ae9aefc6e9296fb03db053c6105535e04477。两者与公开基线相同。本候选没有新增依赖。

基线对应源码附件包含 262 份 Cargo、13 份 npm 生产依赖及 8 份上游/子模块补充源包，共283份；547条许可/通知文件记录均按原始身份继承。旧 inventory 头部是历史来源证据，不是本候选源码或EXE身份。新外层 SOURCE-MANIFEST、DEPENDENCY-SOURCES 与发布记录必须绑定本候选的新提交/树，并保留原始许可文字。锁文件一致不等于所有平台、feature 或法律义务自动完成；工具链、CRT/SDK和最终原生内容由对应产物另记录。

## 媒体与发布身份

43份原图、114个运行时输出以及 assets/下173媒体文件均与公开基线字节相同。精确身份与派生链在 governance/assets/PUBLIC_ASSET_LINEAGE.json。基线已有的测试golden图另记边界，不将它们混为43份上传原图。本候选的原生图标已另采用指定策划Chat于2026-10-03生成的无文字环形光核徽记；原1254×1254 RGBA PNG随源码保留，ICO只做16/24/32/48/64/128/256尺寸转换，没有使用旧图标或添加字体。精确来源与授权见server-rs/icons/ICON_PROVENANCE.json及governance/assets/PUBLIC_RELEASE_NATIVE_MANIFEST.json。43份游戏原图、114runtime与assets下173媒体仍保持原字节。

本文件描述来源，不能自引用当前提交或源码包hash。最终源码身份由外层 SOURCE-MANIFEST 及干净 Git HEAD 记录。不得用另一个源树构建的 EXE 对应本源码，也不得将本地候选准备状态写成已公开发布。

本图标增量基于已冻结净化源提交379f3054338d6e0c282909570599692cd2ad3938；游戏源码、地图、叙事、玩法与依赖锁未改变。旧候选及其EXE保持独立身份，不充当新图标构建的产物。
