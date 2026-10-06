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
