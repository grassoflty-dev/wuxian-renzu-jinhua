# Windows 实机测试结果

对应操作指南：[WINDOWS_CODEX_TEST_GUIDE.md](WINDOWS_CODEX_TEST_GUIDE.md)

当前已有首轮 Run `20261006T005149Z-9001ab28`：**PASS 0 / FAIL 0 / BLOCKED 2 / NOT_RUN 14**。已完成下载身份与源码静态核查；Windows 原生验证器和启动策略阻塞，尚未启动游戏或完成实机交互。下一轮步骤见操作指南第 0 节；本次文档更新没有执行复测。已有 Release 自动化/交叉编译记录不等于 Windows 实机验收。

请将新记录追加到文末，保留模板和历史运行。状态仅用 PASS / FAIL / BLOCKED / NOT_RUN；空白模板不是通过证据。公开文件只包含脱敏文字；原始截图/日志/存档留本地，不默认上传。

## 可复制的单次运行模板

### Run: <UTC时间-唯一后缀>

- 测试开始/结束时间与时区：
- 执行者/方式：Codex 实际工具名称；人工参与的具体步骤；模型（若可确认）：
- 本轮状态：部分完成 / 完成清单 / 阻塞（选择并解释）：
- 操作指南文档 commit：
- 文档工作区 origin、初始 HEAD、是否干净：
- 固定 Release/tag：
- 实际受测源码 commit / 规范化树：
- 发布清单原构建源码身份：
- EXE 文件名、完整大小、计算出的 SHA256：
- 源码 ZIP 文件名、大小、计算出的 SHA256：
- manifest / SHA256SUMS 与下载时间：
- 源码/PE/bundle 验证结果及局限：
- Windows 版本/架构、WebView2 版本：
- 显示分辨率、游戏实际窗口尺寸、DPI/缩放：
- CPU/GPU/RAM（仅必要型号/容量，无序列号/主机名）：
- PowerShell / Git / 其他实际用到的工具版本：
- 旧开发目录保护：只读检查；本轮未改动其文件/存档/进程的核查方法：
- 隔离根目录：使用 <RUN_ROOT> 脱敏；saves / diagnostics / evidence 相对路径：
- 测试保存隔离实际核查：
- 启动方式、受测 PID 与真实窗口核对方法：
- 可用/不可用的屏幕、键鼠、可访问性、音频、性能工具：

#### 命令记录

| 编号 | 时间 | 脱敏工作目录 | 精确命令/参数 | 工具版本 | 退出码 | 耗时 | 日志证据编号 |
|---|---|---|---|---|---|---|---|
| C01 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 | 待填 |

#### 逐项结果

每项实际运行后填写，未运行保留 NOT_RUN 并写理由。可在下表后增加该项详细步骤，不能只打勾。

| ID | 状态 | 前置状态与实际步骤 | 预期 | 实际观察 | 证据编号/限制 |
|---|---|---|---|---|---|
| W01 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W02 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W03 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W04 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W05 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W06 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W07 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W08 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W09 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W10 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W11 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W12 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W13 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W14 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W15 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |
| W16 | NOT_RUN | 待填写 | 见指南对应项 | 尚未执行 | 无 |

#### 证据索引（本地原件不默认公开）

| 编号 | 类型/发生时间 | 本地相对路径 | SHA256 | 证明的具体事实 | 脱敏公开摘要 |
|---|---|---|---|---|---|
| E01 | 待填写 | evidence/... | 待填写 | 待填写 | 待填写 |

截图须说明实际游戏窗口与场景；系统截图中私人信息不要公开。原始文件未发布时明确注明，不能写成可公开复核的附件。

#### 缺陷/阻塞

每个问题分别填写：
- 问题编号 / 严重程度：
- 对应 W 编号 / 复现次数：
- 前置条件与精确操作：
- 预期与实际：
- 报错最小脱敏片段：
- 证据编号：
- 是否阻塞后续项 / 最小下一步：
- 是否涉及已知占位/内容限制：
- 没有修改代码：

#### 覆盖结论

- PASS / FAIL / BLOCKED / NOT_RUN 数量（总数与实际清单一致）：
- 真实原生窗口已观察：
- 真实交互完成范围：
- 三世界实际到达的最后场景/步骤：
- 保存 → 退出 → 重启 → 继续/槽位读取的实际覆盖：
- 白芷分支分别覆盖：
- 音频、DPI、性能实际覆盖与限制：
- 静态/自动化/浏览器/原生交互结果分别说明，禁止混称：
- 断电/崩溃/并发存档注入：NOT_RUN（除非另有安全授权和独立记录）：
- 仍未测试项目与原因：
- 是否建议继续扩大试玩；依据与已知风险：
- 远端写回状态：未尝试 / 成功核实 / 被阻止（原因）：
- 文档提交 SHA / 远端文件 URL（仅在实际验证后填写）：

---

## 实际运行记录

以下保留实际运行历史。新一轮在文末追加完整 Run，不删除以上模板或改写既有结果。

维护备注（2026-10-06）：已从公开远端核实，以下首轮记录随 commit `79d6c356e5be8147808e03922016dc15bdb6e99a` 写回 main。历史 Run 文末的“写成时尚未推送”保留其当时语义；这次核对不新增游戏测试结果。

### Run: 20261006T005149Z-9001ab28

- 测试开始/结束：2026-10-06 00:51–00:55 UTC（09:51–09:55 Asia/Tokyo）；本轮仅前置检查，随后写回文档。
- 执行者/方式：本地 Codex 直接执行，无子代理，无人工游戏操作。实际工具为 exec_command/PowerShell、Git、GitHub connector、node_repl + `@oai/sky`。用户指定 GPT-6.1 Sol，但本轮没有模型切换/身份验证接口，实际模型身份未独立确认，不能宣称已切换。
- 本轮状态：**BLOCKED，未启动 Windows 游戏**。下载与源码核验成功；完整原生 bundle 验证受环境阻塞；隔离启动命令被执行策略拒绝。没有浏览器、mock 或自行构建游戏替代试玩。
- 指南与文档工作区初始 commit：`a90225b9f0a7abc4bd732cc3f02441e4306a6875`。新建 `<RUN_ROOT>/documents` clone，origin 为 `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`，main 初始干净；不是原开发工作区。
- 固定 tag：`v0.1.0-preview.5`；`git ls-remote` 实测标签指向 `b85ac67644b6730b268293339f62e37cf73059fa`。未切换或修改旧目录。
- 发布清单原构建 commit：`1d8aff05f35b24c49355cf64aa58cd7eba561571`；ZIP 独立计算规范化树：`1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a`。本轮没有另建源码 checkout 或重新构建 EXE；ZIP 核验不证明 Git 历史对象。
- EXE：`wuxian-renzu-jinhua-preview.5-windows-x64.exe`，完整大小 **255988736** 字节，实算 SHA256 **e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da**。文件版本/产品版本字段均为 `1.0.0`，不把它误认成正式版或替代 preview.5 哈希身份。
- 源 ZIP：`wuxian-renzu-jinhua-preview.5-corresponding-source.zip`，**428215921** 字节，SHA256 **2b54c7f18452d7eae5739107f57a9841f7c050162551872118e37ecf30c9b033**。
- 验证工具 ZIP：65508 字节，SHA256 `cd739060819ac38ea8397822b06ee31537b4753264ef0da76d126baafe0d163f`。
- 固定 Release 附件于 00:51–00:54 UTC 下载，完整读取 READ_ME_FIRST、KNOWN_LIMITATIONS、BUILDING、RELEASE-MANIFEST、VERIFICATION、SHA256SUMS；实际下载的九个文件均重新计算 hash，匹配发布 API digest 与对应 SHA256SUMS。未下载 LICENSE/third-party-notices；未将未下载项称为核验通过。manifest 的 prepared-not-published 字段属于生成时状态，当前下载事实由固定 GitHub Release/API 确认。
- 源码验证：`verify-source.py` 实际 PASS，1906 清单项、757 工程文件、8 个 PowerShell 文件 CRLF 规范化；未借用历史自动化数字。原生完整验证：`verify-native.py` 退出 1，缺 `libbrotlidec.so.1`，记为环境 BLOCKED，不记游戏 FAIL/完整闭包 PASS。
- 环境：Windows 11 Home 中文版 10.0.26200 x64；本机 WebView2 安装目录有 154.0.4258.37 / 154.0.4258.53，**游戏未启动，实际使用版本未观察**。主 GPU NVIDIA GeForce RTX 5060 Ti，CPU i5-12490F，RAM 25572040704 字节；显示查询为 2560×1440。注册表 LogPixels 未返回值、Win8DpiScaling=0，不能据此推断实际窗口 DPI/缩放。
- 工具版本：PowerShell 7.6.5、Git 2.53.0.windows.2、现有 bundled Python 3.12.14；未安装新依赖。终端权限为 unrestricted/approval never，但实际调用仍可能被执行策略拒绝。`@oai/sky` 导入、list_apps 成功；未找到游戏实例，未发送游戏键鼠输入。音频/性能测量未验证。
- 原目录保护：只读比较旧开发目录开始/结束的 Git status，未提交删除及 5 个 untracked 条目保持；正式 release 仍仅既存 `server-rs/gen/` untracked。测试未执行旧目录 pull/checkout/reset/clean/stash，没有写原存档或启动/终止原游戏。未对所有 untracked 文件逐字节快照，不能声称全盘无其他进程写入。
- 隔离目录：全新 `<RUN_ROOT>`；downloads / saves / diagnostics / evidence / verification / documents 分离，留本地不清理。启动预定 child-only `WUXIAN_FORMAL_SAVE_DIR=<RUN_ROOT>/saves`、`WUXIAN_NATIVE_DIAGNOSTICS_DIR=<RUN_ROOT>/diagnostics`；命令执行前被拒绝，**隔离存档实际写入尚未验证**。结束时 saves、diagnostics 为空，游戏进程列表为空；无受测 PID、窗口、截图或存档。

#### 命令记录

时间按本轮 UTC 范围记录；除工具返回的单元耗时外未逐条计时，不虚构下载/子命令耗时。以下路径均脱敏。

| 编号 | 时间 UTC | cwd | 实际命令/参数 | 结果 / 耗时 | 证据 |
|---|---|---|---|---|---|
| C01 | 00:51 | 原开发目录（只读） | `git status --porcelain=v1`、`git rev-parse HEAD`；`Get-PSDrive C`；`Get-CimInstance Win32_Process` 过滤游戏名 | 成功；C 盘剩余 30568787968 字节；未见游戏实例；单元 1.919s | E01、E06 |
| C02 | 00:51–00:54 | `<RUN_ROOT>` | `Invoke-RestMethod https://api.github.com/repos/grassoflty-dev/wuxian-renzu-jinhua/releases/tags/v0.1.0-preview.5`；`Invoke-WebRequest -Uri <固定附件URL> -OutFile <RUN_ROOT>/downloads/<附件>`；`Get-FileHash -Algorithm SHA256` | 首次工具 ZIP 请求 EOF；curl 替代下载报 CRYPT_E_REVOCATION_OFFLINE；未用 insecure/撤销检查绕过；之后标准 Invoke-WebRequest 成功，逐文件 hash 一致；未逐条计时 | E06；早期传输错误仅会话工具输出 |
| C03 | 00:53–00:54 | `<RUN_ROOT>` | `git clone --depth 1 --branch main https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git <RUN_ROOT>/documents` | 成功；另一次重复 clone 因目标非空退出 128，未覆盖目录；没有原工作区修改 | 初始 HEAD 与后续差异检查 |
| C04 | 00:53 | `<RUN_ROOT>` | `python <RUN_ROOT>/verification/verify-source.py <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-corresponding-source.zip` | 验证输出 pass，Python 退出 0；合并单元含后续重复 clone，单元退出 1，不混称整个单元通过；未单独计时 | E03 |
| C05 | 00:54 | `<RUN_ROOT>` | `python <RUN_ROOT>/verification/verify-native.py <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-windows-x64.exe <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-corresponding-source.zip` | Python 退出 1；缺 Linux so，完整 bundle BLOCKED；命令单元 1.284s | E04 |
| C06 | 00:54 | `<RUN_ROOT>` | 预定设置两个 child-only 隔离变量，`Start-Process -FilePath <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-windows-x64.exe -WorkingDirectory <RUN_ROOT>/downloads -PassThru -WindowStyle Hidden`，finally 恢复调用方变量 | **工具拒绝：blocked by policy**；无子进程退出码、耗时或 PID；未通过其他接口重试 | E05 |
| C07 | 00:55 | `<RUN_ROOT>/documents` | `git remote -v`、`git rev-parse HEAD`、`git status --porcelain=v1`、`git ls-remote origin refs/heads/main refs/tags/v0.1.0-preview.5`；本地只读环境/文件/进程记录脚本 | main 与标签确认；旧 status 相同；saves/diagnostics/gameProcesses 为空；结束记录脚本单元退出 0、1.634s | E06 |

#### 逐项结果

| ID | 状态 | 前置状态与实际步骤 | 预期 | 实际观察 | 证据/限制 |
|---|---|---|---|---|---|
| W01 | BLOCKED | 下载完整 EXE/source/tools；计算大小/hash；查 tag；执行两验证器 | 文件、源码、PE/bundle 身份分项完整核对 | 文件与源码树一致；原生验证缺 Linux so，闭包未完成 | E03/E04/E06；不是 hash 冲突 |
| W02 | BLOCKED | 预定隔离启动通过 hash 的 EXE | 真实窗口可见、无白屏/崩溃 | 启动调用被策略拒绝，未执行，没有游戏窗口/PID | E05/E06；不是游戏崩溃 |
| W03 | NOT_RUN | W02 未启动，未打开菜单页面 | 页面/取消/继续状态正确 | 无观察 | 启动阻塞 |
| W04 | NOT_RUN | 未发新旅程/暂停/再次新旅程输入 | 到达归航站且无残留 | 无观察 | 启动阻塞 |
| W05 | NOT_RUN | 未进行四向/斜向移动及鼠标瞄准 | 位移/朝向/停止可辨 | 无观察，未把源码占位说明当实测 | 启动阻塞 |
| W06 | NOT_RUN | 未走设施/门框/前后景/平台 | 遮挡及地面合理 | 无实际地图截图 | 启动阻塞 |
| W07 | NOT_RUN | 未使用 J/左键/Shift/Q/E/R | 接受动作与反馈可辨 | 未观察命中、受击、Guard 按下/松开或声音 | 启动阻塞 |
| W08 | NOT_RUN | 未进战斗或切换游戏焦点 | 无卡键/Guard 卡住 | 无观察；未向其他窗口发送按键 | 启动阻塞 |
| W09 | NOT_RUN | saves 为空，未保存、退出重启、Continue/读槽 | 同隔离目录状态一致 | 未建立存档，不证明隔离写入/继续 | E06；启动阻塞 |
| W10 | NOT_RUN | 未到灰巢 | 灰巢/首通强化流程 | 最后场景：无 | 启动阻塞 |
| W11 | NOT_RUN | 未到雾港 | 信标/泵站/首次返回反馈 | 无观察 | 启动阻塞 |
| W12 | NOT_RUN | 未到钟骨 | 压力/热量/炉心 F 推进 | 无观察 | 启动阻塞 |
| W13 | NOT_RUN | 无已确认世界事件 | Archive 解锁/重读无额外奖励 | 无观察 | 启动阻塞 |
| W14 | NOT_RUN | 未到 Bio，未建立独立旅程/槽位 | 白芷取消、三态、确认、保存/复谈 | taken/left/unresolved 全未覆盖 | 启动阻塞 |
| W15 | NOT_RUN | 仅查询显示 2560×1440；无游戏窗口 | 多尺寸 UI 无裁切 | 未测试 1280×720 或其他游戏尺寸；DPI 未确认 | E02；环境查询不算 PASS |
| W16 | NOT_RUN | 未启动/连续游玩/正常退出 | 音频/性能/稳定性可观察 | 没有音频或性能测量，未宣称无崩溃 | 启动阻塞 |

#### 证据索引（原件仅留本地，不是公开附件）

| ID | 本地相对路径 | SHA256 | 证明范围 |
|---|---|---|---|
| E01 | evidence/E01-preflight.txt | 0d8b340b6a70fcaab59dddc1cac2073bdc26c7cc3889f2e3abb0b8d5db9d92e7 | 开始时旧 Git status；格式化表输出不全，环境以 E02 为准 |
| E02 | evidence/E02-environment.json | 1e983213b0d12aab32afbe6cad218eafad390e4f9a5fcf6937cab0c57354fb61 | OS/硬件/显示/WebView2 安装目录，只读查询 |
| E03 | evidence/E03-source-verification.txt | ce7fa27b448668a9ac155a1133216453d965a24943772b985a903a00904f8281 | 实际 ZIP 验证 pass 输出，不证明原生运行 |
| E04 | evidence/E04-native-static-verification.txt | 186393c1261ef837c21b0f5e90464bacb390f619846913e1df05be460879d46e | 原生静态验证器缺 libbrotlidec.so.1 traceback |
| E05 | evidence/E05-launch-policy.txt | 7fef8df5267e7d18436f4062b63108d17c3c0a98f5f1174858e08c8fbedcf0a9 | 对实际执行策略拒绝的文字转录；不是 Windows 启动日志 |
| E06 | evidence/E06-final-preflight.json | 5fb1a0d983a14b030305de1c1d7667fec79c6b1045bd8303688a1d7e7d2403b4 | 九个实际下载文件大小/hash、结束 Git status、空存档/诊断/游戏进程 |

无原生截图；原文件可能包含私人路径，未上传。哈希索引不能让公共读者获得未发布原件。

#### 缺陷/阻塞

- B01 / 测试阻塞 / W01 / 一次：既有发布验证器使用 `ctypes.CDLL('libbrotlidec.so.1')`，在 Windows/Python 3.12.14 报 `FileNotFoundError: Could not find module 'libbrotlidec.so.1'`。E04。完整 PE/bundle 未闭合；没有安装依赖、替换验证器或改断言。最小下一步是维护方提供支持 Windows 的原样验证路径，另开复测；不是游戏运行缺陷。
- B02 / 测试阻塞 / W02 / 一次：通过 hash 后提交隔离 Start-Process 命令，工具返回 `rejected: blocked by policy`，未产生 PID。E05/E06。具体拒绝规则未暴露；不能推断 SmartScreen、杀毒软件、EXE 损坏或权限不足。遵循“不绕过安全限制”，停止启动及后续 UI 输入。最小下一步是由用户/平台确认允许的启动方式或由用户在隔离环境手动启动，再另开实际试玩记录；本轮没有再走其他工具规避拒绝。
- 下载中 EOF/撤销服务器不可用属于传输尝试失败；标准 HTTPS 下载随后成功且 hash 一致，不列为游戏 FAIL。没有使用 `--insecure`、关闭防护或修改执行策略。
- 没有修改游戏代码、素材、原工程、依赖、仓库设置或工作流。

#### 覆盖结论

- W01–W16：**PASS 0 / FAIL 0 / BLOCKED 2 / NOT_RUN 14**。两项成功的前置子检查不把 W01 完整身份项升级为 PASS。没有观察到游戏行为，因此 FAIL 0 不代表游戏无缺陷。
- 原生窗口/新旅程/移动与遮挡/战斗/三世界/白芷/保存后重启继续均未完成；最后到达场景：无。没有真实截图、音频、DPI或性能结果。
- 本轮只有实际下载身份与源码静态核验；无 Web/浏览器测试、无构建、无历史自动化结果移植，无原生交互结果。
- 断电/强制崩溃/并发存档注入：NOT_RUN，未执行。
- 先解决启动策略和 Windows 验证器阻塞再扩大试玩；本记录不支持 native-verified、三世界通过或发行验收结论。
- 远端写回：下述记录写成时尚未推送；只提交本文件，说明含 `[skip ci]`，将按普通非强制推送及远端内容核对结果交付。仓库当前没有被 Git 跟踪的 `.github/workflows` 文件；未调用 Actions、未改变设置。

### Run: 20261006T031535Z-626073de (Round 2 / W01)

- 实际时间：2026-10-06 12:15:35–12:17:06 Asia/Tokyo (UTC+09:00)，03:15:35–03:17:06 UTC。关联历史 Run 20261006T005149Z-9001ab28；旧记录未改写。
- Codex 直接执行，无子代理、无人工游戏操作；用户确认配置 GPT-6.1 Sol medium，未独立查询模型身份。实际工具：PowerShell/exec_command、Git、GitHub connector；本轮未使用桌面控制工具，截图、键鼠、音频、性能能力未复核。
- 范围：仅 W01 静态复测。指南和补充工具固定 commit `11130bf9b09fd1016818daeda0eed7f4a3b4cab8`。全新 documents clone 的初始 main HEAD 同值，初始干净；origin `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`。未修改旧开发目录。
- 固定标签 `v0.1.0-preview.5`，本轮 ls-remote 确认公共标签提交 `b85ac67644b6730b268293339f62e37cf73059fa`。原构建源码声明 `1d8aff05f35b24c49355cf64aa58cd7eba561571`，独立重算规范化树 `1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a`；ZIP 不包含原 Git 提交历史对象。
- EXE `wuxian-renzu-jinhua-preview.5-windows-x64.exe`：255988736 bytes，SHA256 `e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da`。文件/产品版本字段 1.0.0 不替代 preview.5 身份。
- source ZIP `wuxian-renzu-jinhua-preview.5-corresponding-source.zip`：428215921 bytes，SHA256 `2b54c7f18452d7eae5739107f57a9841f7c050162551872118e37ecf30c9b033`。
- 上述完整附件与六份发布说明从首轮 downloads 复制到本轮新目录，并重新计算完整大小/hash、对照当前 Release API digest 和同版 SHA256SUMS/manifest；不是重新下载大附件。源目录未写入。11 份补充工具从指定不可变 ref 的 raw URL 新下载，逐文件 SHA256 全部匹配指南第 0 节；未替换原发布工具 ZIP，未改断言、期望值或使用 -O/-OO。
- Windows 11 家庭版中文版 10.0.26200 x64；PowerShell 7.6.5、Git 2.53.0.windows.2、同一 bundled CPython 3.12.14 x64、pip 26.2.1。开始 C 盘空闲 28896116736 bytes。WebView2 游戏实际版本、游戏窗口/分辨率/DPI、音频、性能无运行观察；本轮不重复硬件测量，也不移植历史环境为新实测。
- 新建 `<RUN_ROOT>`，downloads/saves/diagnostics/evidence/verification-windows-fix/documents 隔离。官方 Brotli 1.2.0 cp312-win_amd64 wheel 仅安装至本轮 verification-windows-fix，要求 hash `b35c13ce241abdd44cb8ca70683f20c0c079728a36a996297adb5334adfc1c44`，来自 PyPI，pip --require-hashes 成功。未改全局 Python 或系统设置。
- 开始与结束只读 Git status 一致：旧桌面目录既存删除和 5 个 untracked 条目保留，正式 release 仍仅 server-rs/gen/ untracked。没有逐字节快照全盘；不声称其他进程未写入。结束未见游戏进程，saves/diagnostics 为空，无 PID、游戏窗口、截图、存档或诊断日志；存档隔离实际写入尚未验证。

#### 命令记录

`<PYTHON>` 为首轮相同 bundled CPython 的绝对路径（公开记录脱敏），`<V>` 为 `<RUN_ROOT>/verification-windows-fix`。E01 中保留完整命令与私人路径，仅留本地。

| 编号 | 时间 Asia/Tokyo | 实际命令/参数 | 实际结果/耗时 | 证据 |
|---|---|---|---|---|
| C01 | 12:15:35–12:16:29 | 执行本轮 w01.ps1：Get-PSDrive C、旧目录 git status、Get-CimInstance Win32_OperatingSystem/Win32_Process；git clone --depth 1 --branch main <公开origin> <RUN_ROOT>/documents；git ls-remote origin refs/heads/main refs/tags/v0.1.0-preview.5；逐文件 Invoke-WebRequest <固定ref raw URL>；Copy-Item 旧下载至新 downloads；Get-FileHash；Release API；Python 平台/版本检查及 -m pip --version | 命令单元最终退出 0；11 工具及8附件身份一致；复制/下载子步骤未独立计时 | E01 |
| C02 | 12:16:35–12:16:47 区间 | <PYTHON> -m pip install --only-binary=:all: --no-deps --index-url https://pypi.org/simple --target <V> --require-hashes -r <V>/requirements-windows-py312-x64.txt | pip 退出 0；Brotli 1.2.0 安装成功；单元 3.617s（包含前置文件阅读，非纯 pip 耗时） | E02 |
| C03 | 12:16:47–12:16:50 | <PYTHON> <V>/verify-source.py <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-corresponding-source.zip | 实际 result pass，退出 0，2.309s | E03 |
| C04 | 12:16:50–12:16:56 | <PYTHON> <V>/verify-native.py <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-windows-x64.exe <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-corresponding-source.zip | 实际 result pass，退出 0，6.576s；没有执行 EXE | E03 |
| C05 | 12:17:06 | 旧目录 git status、游戏名进程查询、saves/diagnostics 列表、EXE VersionInfo、Get-FileHash 本轮证据 | 成功；无本轮游戏进程/存档/诊断 | E04 |

#### 逐项结果

| ID | 状态 | 实际步骤/观察及限制 |
|---|---|---|
| W01 | PASS | 完整文件/hash/标签/工具来源检查；源码验证1906清单项、757工程文件、8份PS规范化；Windows Python 实际完成完整 PE32+ x64 GUI/13系统DLL/图标/场景/raw+Brotli/sidecar 正向闭包：7图标帧、256×256默认图标、30原生scene、163Web载荷。没有重跑云端维护测试/负样本，不代表游戏运行通过。E01–E03 |
| W02 | BLOCKED | 历史策略拒绝未解除，用户明确没有平台允许启动的证据；本轮没有提交启动命令，没有重试被拒绝动作或更换工具。待用户在本轮隔离环境亲自启动；无窗口/PID。E04与历史B02 |
| W03 | NOT_RUN | 没有原生窗口，菜单页面未操作 |
| W04 | NOT_RUN | 新旅程/暂停/再次新旅程未操作 |
| W05 | NOT_RUN | 移动/停止/鼠标朝向未观察 |
| W06 | NOT_RUN | 遮挡/地图未观察 |
| W07 | NOT_RUN | J/左键/Shift/Q/E/R及受击未观察 |
| W08 | NOT_RUN | 暂停/恢复及焦点未操作 |
| W09 | NOT_RUN | 保存退出重启继续未操作；saves为空 |
| W10 | NOT_RUN | 未到灰巢；最后场景无 |
| W11 | NOT_RUN | 未到雾港 |
| W12 | NOT_RUN | 未到钟骨 |
| W13 | NOT_RUN | 未确认世界事件/Archive |
| W14 | NOT_RUN | 白芷unresolved/taken/left均未操作 |
| W15 | NOT_RUN | 游戏窗口尺寸/DPI未观察 |
| W16 | NOT_RUN | 音频/性能/连续游玩/正常退出未观察 |

#### 证据索引（原件仅本地，不是公开附件）

| ID | 相对路径 | SHA256 | 证明范围 |
|---|---|---|---|
| E01 | preflight-transcript.txt | b50a160124c2da709c82c6c91502684cb04ba4357b2eaa93750e73577c34f857 | 来源、工具/附件身份、版本、初始旧Git状态 |
| E02 | evidence/E02-pip.txt | 172f343476b0b7c583d5dfdc606e80ed312252a46f3e70c45d997ff4302a7eea | hash限定的本轮隔离依赖安装输出 |
| E03 | evidence/E03-verifications.txt | f2f10e8ed2830ef12421f62e2bd9e843890efd270a5a71a96c73497052762408 | 两项实际Windows静态核验输出/退出码/耗时 |
| E04 | evidence/E04-final.txt | 5bf84b6171b58ce32354b2cdd6d6738203e27a5f8e53154bc22567e4ff71b010 | 结束旧Git状态、无游戏进程、空隔离保存/诊断 |

#### 阻塞与覆盖结论

- 历史 B01 在本轮 Windows 完整静态核验范围已解决，旧 Run 的 BLOCKED 原样保留。验证器输出 WindowsExecuted=false 表示未运行 Windows 游戏，不否认本轮验证器确实在 Windows Python 执行。
- B02 保留：未获取平台允许启动的证据，不再次提交 Start-Process。将向用户展示本轮隔离环境的可见窗口启动步骤；不是游戏崩溃或损坏证据。
- **PASS 1 / FAIL 0 / BLOCKED 1 / NOT_RUN 14**，共16项。FAIL 0不代表游戏无缺陷；未进行浏览器/构建/原生试玩，无正式原生验收结论。
- 原始截图（无）、存档（无）和完整工具日志保留本地；不上传原始证据。断电/崩溃/并发存档注入 NOT_RUN。
- 仅追加本测试文档，保留模板和历史；提交含 [skip ci]，普通非强制推送main后独立核远端。此句写成时尚未推送，交付时另报实际远端结果。不改游戏、旧工程、仓库设置或工作流，不启动 Actions。

#### Round 2 后续路径诊断（2026-10-06 12:32 Asia/Tokyo / 03:32 UTC）

- 上述 W01 记录已随 commit `e26f39c4d0d739d53f406d1b8bd1b9801f6c3881` 普通推送，远端 HEAD、文件内容及历史保留均核实。本段只追加路径诊断，不是新的游戏试玩或重复静态核验。
- 用户随后在桌面 Windows PowerShell 5.1.26100.9549 手动执行此前提供的隔离启动命令，报告 WorkingDirectory 的 DirectoryNotFoundException；三个 Test-Path 均 False。此为用户报告，助手未直接观察该终端；没有证据证明 EXE 被执行，不记游戏 FAIL。
- 助手最初将差异描述为可能不同执行环境，证据不足。后续只读检查找到同一测试目录在应用包 `<LOCALAPPDATA>/Packages/<CODEX_PACKAGE>/LocalCache/Local/WuxianPreviewTests/<RUN>` 下的实际可见路径。原逻辑路径与包缓存路径的 EXE 文件 ID 完全一致，SHA256 均为固定 preview.5 期望值。该证据支持应用包文件重定向，而不是两份不同 EXE；不据 PowerShell 5.1/7.6.5 差异断言不同电脑。桌面工具仅列出应用，用户确认当前应用吻合；截图/输入能力及游戏环境仍未验收。
- 本段依据 E05 `evidence/E05-path-alias.txt`，SHA256 `a9b05409f33bea29bf824557c0f1553ea53fbd43445f25c84048f388351e9548`；原始私人路径和文件 ID 留本地，不上传。包缓存真实路径是否在用户终端可访问，仍待用户 Test-Path 确认。
- W01 的本机 Windows Python 静态核验结果保持；W02 仍 BLOCKED，W03–W16 仍 NOT_RUN。未重试被拒绝启动、未使用桌面/其他接口绕过策略，无游戏 PID 或窗口。将向用户提供包缓存实际路径的隔离手动步骤，禁止默认环境双击；安全提示由用户处理。合计仍 PASS 1 / FAIL 0 / BLOCKED 1 / NOT_RUN 14。

### Run: 20261006T033723Z-12828 (人工启动后的原生交互)

- 实际观察：2026-10-06 12:37:23–12:42:56 Asia/Tokyo (UTC+09:00)，03:37:23–03:42:56 UTC；后续整理证据与文档不计作试玩。用户先报告“好了，游戏启动了”；查询该进程创建时间为12:36:37，没有测量冷启动到首帧耗时。
- 直接执行、无子代理；用户确认配置 GPT-6.1 Sol medium，未独立查询模型接口。人工参与仅隔离启动；助手随后使用 node_repl + @oai/sky 截图、聚焦、点击、Escape/Return及输入框 set_value；只针对唯一返回的游戏窗口。不通过浏览器或源码替代原生试玩。
- 指南固定 `11130bf9b09fd1016818daeda0eed7f4a3b4cab8`。文档工作区沿用 Round2 独立 clone，初始 main HEAD `c2c10e1cef155edfe31b86d1e0e6b9d4311ce01f`、干净，origin `https://github.com/grassoflty-dev/wuxian-renzu-jinhua.git`，写入前 ls-remote 同值；未操作旧开发目录 Git 写命令。
- 本记录为新 Run ID、接续用户已启动的 Round2 隔离环境，**没有另建全新运行根目录**；saves/diagnostics仍属测试目录，不是用户原存档。新建 evidence/native-session 独立证据子目录，不覆盖前轮证据。保留这一偏离与前轮关联，不声称重新冷启动或新建全空根目录。
- 受测固定 `v0.1.0-preview.5`：公共源码身份 `b85ac67644b6730b268293339f62e37cf73059fa`，清单构建身份 `1d8aff05f35b24c49355cf64aa58cd7eba561571`，规范化树 `1bd25e0bcb32d5e4ae1d1ff3625d11abd958780a`；本段未重新下载/查询tag/执行源码或bundle验证，引用独立 Round2 W01，不重复算为本 Run PASS。
- 实际进程路径核为 `<RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-windows-x64.exe`，255988736 bytes；本段重新计算完整 SHA256 `e97d0c07d0ceb393ba3d6775f765c7546a55a7f66d8799535617cd23bca602da`。版本字段1.0.0不改变preview.5身份。关联源 ZIP 428215921 bytes、SHA256 `2b54c7f18452d7eae5739107f57a9841f7c050162551872118e37ecf30c9b033`，本段未重算源ZIP。
- Windows实查：Windows11家庭版中文版10.0.26200 x64，PowerShell7.6.5、Git2.53.0.windows.2。游戏PID12828，真实窗口标题“无限人族进化”，窗口ID1180472；子进程PID20372实际路径位于WebView2 `154.0.4258.53` 目录。不是仅据安装目录推测游戏版本。
- 截图外框1282×751，画面可读；未测实际client尺寸、DPI/缩放、其他窗口尺寸，也未重测CPU/GPU/RAM/显示器。没有验证音频采集、FPS或掉帧测量能力；UIA“音频正在播放”和设置“临时合成音频就绪”不等于实际听到声音。
- 原目录仅只读结束status：桌面既存1个ZIP删除与5个untracked条目、正式release既存server-rs/gen/均保留；未对所有文件逐字节快照，不能声称全盘无其他进程写入。未改游戏代码/素材/原工程/系统设置/ACL，未启动或终止其他实例。

#### 实际步骤与命令

- 用户手动启动所提供的子进程隔离步骤（助手未执行或观察用户终端的完整输出）：先设置`WUXIAN_FORMAL_SAVE_DIR=<RUN_ROOT>/saves`及`WUXIAN_NATIVE_DIAGNOSTICS_DIR=<RUN_ROOT>/diagnostics`，`Start-Process -FilePath <RUN_ROOT>/downloads/wuxian-renzu-jinhua-preview.5-windows-x64.exe -WorkingDirectory <RUN_ROOT>/downloads -PassThru`，finally恢复调用方原环境值；实际路径使用前段核实的应用包缓存真实路径。用户报告启动，加上实际PID/路径/游戏窗口及新文件落点证据，不是断言平台已授权助手重放启动。
- 助手12:37–12:42按游戏窗口逐步观察→输入→刷新：初始菜单（继续disabled）→设置→关闭→新旅程→归航站→Escape暂停→命名`Codex-preview5-native-test`→保存→恢复→返回菜单→第二次新旅程→归航站→返回→继续选择命名槽→指定槽读取→归航站→返回菜单→退出。没有对每次点击独立计时；不虚构首帧/加载时延。
- 第一次存档实际落在`<RUN_ROOT>/saves/slots/<TEST_SLOT>/slot-v6.json`，5295 bytes，保存提示“第826帧”；场景rs_core_room、HP/energy100、位置(4,0,8)，无世界完成事件、奖励或成长。菜单返回后还观察到隔离saves/formal-save-v6.json自动档4756 bytes；不能据文件存在声称自动档重启读取已通过。
- 下拉指定槽的indexed click一次被工具拒绝：point over non-target msedgewebview2 “Chrome Legacy Window”。保留窗口身份保护；激活原游戏并刷新，使用Return选中已观察的高亮槽，再点Continue Selected成功。同进程实际加载完成；没有操作其他窗口或绕过启动策略。此为已恢复的工具子步骤阻碍，不是游戏崩溃。
- 结束只读命令：`Get-CimInstance Win32_Process -Filter 'ProcessId=12828'`及ParentProcessId查询；`Get-FileHash -Algorithm SHA256`；`Get-ChildItem <RUN_ROOT>/saves -Recurse -File`；读取测试槽/自动档JSON和诊断；`git status --porcelain=v1`（旧目录只读），`git remote -v/rev-parse HEAD/ls-remote`（独立文档clone）。日志/存档只复制到新证据文件，不修改原件；查询单元退出0，未逐命令计时。

#### 逐项结果

| ID | 状态 | 实际步骤、观察、未覆盖部分及证据 |
|---|---|---|
| W01 | NOT_RUN | 本Run仅重核实际EXE路径/hash。完整静态核验在前述Round2已PASS，未重跑，不移植为本Run PASS |
| W02 | PASS | 用户隔离启动后，唯一对应PID/路径的真实原生主菜单可读，新旅程也呈现地图，无持续黑/白屏或本段启动崩溃。冷启动耗时未知，未观测安全提示；不是助手启动权限已解除。E01/E03/E09/E13 |
| W03 | FAIL | 设置可打开/关闭、Continue初始disabled且保存后可选择槽；主菜单退出按钮实际报ACL错误且未关闭。尚未覆盖全部页面/取消/重复组合。此FAIL对应下述唯一D01，不算第二个独立缺陷。E01/E02/E08 |
| W04 | NOT_RUN | 部分完成两次新旅程、归航站、暂停/恢复和返回；两次HUD baseline100/100。未做held-input残留或有进度/奖励状态的reset验收，按指南不将一次/两次新旅程呈现升为整项PASS。E03–E06/E12 |
| W05 | NOT_RUN | 未做四向/斜向持续移动、松键、绕角色鼠标瞄准；支持单次按键不证明可完成held-key验收 |
| W06 | NOT_RUN | 只见归航站地面/设施，不曾绕行、穿门、走平台或切世界；未判遮挡PASS |
| W07 | NOT_RUN | 未进入战斗、未试J/左键/Shift/Q/E按下松开/R和受击反馈 |
| W08 | NOT_RUN | 非战斗暂停/恢复已观察，但战斗焦点/持续移动/Guard释放未覆盖 |
| W09 | FAIL | 命名槽隔离写入及同PID明确槽读取成功；尝试正常退出实际失败，重启后的自动档Continue/指定槽及进度一致性未完成。E05/E07–E11；不能将同进程加载当重启恢复 |
| W10 | NOT_RUN | 最后到达归航站rs_core_room；未进灰巢/首通强化 |
| W11 | NOT_RUN | 未到雾港/泵站/首次完成 |
| W12 | NOT_RUN | 未到钟骨/炉心推进 |
| W13 | NOT_RUN | 尚无世界完成事件，未做Archive解锁/重读验收 |
| W14 | NOT_RUN | 未到Bio，白芷unresolved/taken/left三独立分支均未操作 |
| W15 | NOT_RUN | 只有单一外框1282×751截图；其他尺寸/client尺寸/DPI未验证 |
| W16 | BLOCKED | 当前受支持工具未验证音频/FPS观测方法，不以UIA音频标记/进程存在判通过；没有连续切场景测稳定性。退出实际缺陷另见D01，不将它改称工具阻塞 |

#### 本地证据索引

以下相对路径均位于`<RUN_ROOT>/evidence/native-session/`。原始截图、完整日志和存档未上传；哈希只索引本地原件，不是公开可下载附件。

| ID | 文件 | SHA256 | 证明范围 |
|---|---|---|---|
| E01 | E01-menu.png | 46d5b835f8eb90e6e59dd4f0b2077e1c7da979559b3870e59eedafbe3fad6d02 | 真实初始主菜单 |
| E02 | E02-settings.png | ab89c4b016f6e02ce6024e0414790b9e5babb3b37f0162cf13b0ff3484073fc8 | 设置页面，非实际声音证据 |
| E03 | E03-station.png | 47e922a5054c80c47611974c93015e10e82fcf952f8b1af76ce31c0d5256654f | 第一次新旅程归航站 |
| E04 | E04-paused.png | 2a1125b754446dc520067fa7ff2010140374382214247291830c8255adc4e367 | 暂停覆盖层 |
| E05 | E05-saved.png | ac7fa3e13f0175b5cf0cd1d123458a603eca16869c7b132adcfb9cb2e1aacec9 | 命名槽保存成功提示 |
| E06 | E06-second-journey.png | f11a72f3c3f8caac0550d312340d1b71d44da39d3dfe9916638a3e5659a745ea | 第二次新旅程 |
| E07 | E07-loaded-slot.png | a288372dad725c3d535cfaf80107c56a129030720008448002153b144ed1f296 | 同进程指定槽加载，不是重启 |
| E08 | E08-exit-failed.png | 7f6dcd5b558e08087d68b8bc8420ed3301417f26ed2622c3c2191f0d275c48ef | 退出ACL报错 |
| E09 | E09-native-final.txt | f5f64bcb5b03362ec52af68244eafb80130503206231f92baf8c958f9c2ad716 | 本地只读结束查询transcript，部分表列显示省略，不据其虚构值 |
| E10 | E10-auto-save.json | e9252b73eb7ea559257dc8bae4d01ad9f15bf0bd92dc1d9291d9418c44ba4f27 | 实际隔离自动档原样快照 |
| E11 | E11-named-slot.json | 131e898cfe8a6fc29636fd597faeed1aa0c8cfcaa770859c2c87f38ae593ae62 | 实际隔离命名槽原样快照 |
| E12 | E12-new-journey-stages.log | c3701761ea6da9a670ef85137e065de3fc200b741532de406eb52b9039d8d7c9 | 两次新旅程command/reset/receipt阶段，非FPS证据 |
| E13 | E13-observation-notes.md | c95b9ab46b9ae3d11a66c2c2541c200f7f507218d3588bb8484b8c8241301912 | 观察顺序与补充查询转录，非独立自动化测试输出 |

#### 缺陷、限制与接续

- **D01 / 正常退出失败 / W03、W09 / 实际1次。** 主菜单点“退出”，预期正常关闭受测进程；实际可见`退出失败：Command plugin:window|close not allowed by ACL`，PID12828仍存在。E08/同进程查询。提示与window-close命令被ACL拒绝一致，但未审查源码确定根因。没有修改权限/安全设置、换接口强退或借其他工具重放启动。
- 后续需用户通过普通窗口关闭操作结束本实例，再亲自在**同一隔离saves/diagnostics环境**重新启动，另段验证自动档Continue与指定槽、场景/成长/任务状态。即使人工标题栏关闭可用，D01仍是退出按钮FAIL，不能改写成正常退出通过。当前保留失败窗口及本地证据。
- **PASS1 / FAIL2 / BLOCKED1 / NOT_RUN12，共16项**；FAIL2是两项验收受同一个D01影响，不是两项不同游戏缺陷。W04等NOT_RUN中记录了部分实际步骤，完整细目未通过。旧Run和模板完整保留；前轮W01 PASS与本段W02 PASS不等于正式native-verified或三世界通过。
- 仅追加脱敏测试文档，普通非强制推送main并核对远端；提交含[skip ci]。没有改游戏/原工程/仓库设置/工作流，没有调用Actions。此段写成时尚未推送；上传情况以交付核验为准。断电/强制崩溃/并发写档注入均NOT_RUN。
