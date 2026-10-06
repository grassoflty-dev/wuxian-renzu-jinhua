# preview.5 只读核验工具

1. 按发布附件SHA256SUMS.txt核对完整下载文件
2. Python3标准库源码核验：python verify-source.py ../wuxian-renzu-jinhua-preview.5-corresponding-source.zip
3. Windows/Linux/macOS Python3原生静态核验（先安装官方 Brotli Python 包；Linux/macOS 也可使用既有系统库）：python verify-native.py ../wuxian-renzu-jinhua-preview.5-windows-x64.exe ../wuxian-renzu-jinhua-preview.5-corresponding-source.zip

../表示附件放在本工具目录上一层，也可传入下载路径。只读取文件、不联网、不启动EXE、不解包完整源码。请勿使用-O/-OO，工具会拒绝禁用assert的执行。

源码验证独立检查1906条清单、757个工程文件，拒绝缺项/额外/重复归档路径；8份PowerShell按.gitattributes把CRLF规范为LF后计算Git树。提交值是声明的构建身份，Git对象/历史不在ZIP；内容树独立重算。

原生验证绑定确切EXE和确切源码ZIP，核对PE32+ x64 GUI、13系统DLL导入、7个RT_ICON/GROUP_ICON、原PNG、256px默认窗口RGBA、30原生scene和163实际raw/Brotli载荷及clean身份sidecar。PUBLIC-NATIVE-EVIDENCE.json保留当前字节偏移与hash，删除无关本地路径。原bundle-identity.json保持原字节。

构建验收已逐项排除18个陈旧codegen表示，并运行20项自测与独立另加3项旧探针负例。本工具执行当前固定EXE的正向闭包核验，未附旧缓存/历史负样本；不将自身运行说成重新执行这些负例。没有Windows/WebView2执行、视觉/声音/DPI/性能或断电/并发实际验收。windows-dependency-coverage.json记录237个实际normal/build依赖受262份Cargo源包与通知覆盖；toolchain-supplement.json保留SDK/CRT版本及manifest来源限定。

保留GNU GPL version3文本和第三方原许可。无独立可信数字签名；hash证明与清单一致，仍应核对可信下载来源。无绝对法律或字节可复现构建保证。

## Windows Brotli 兼容修复（补充工具版）

本修复只替换 Brotli 解码后端，优先使用 Google Brotli Python 包；Windows 不再尝试加载 Linux .so。没有依赖时明确报错，绝不跳过解码或闭包检查。不会自动下载或安装依赖。固定 EXE、源码 ZIP、PUBLIC-NATIVE-EVIDENCE.json、bundle-identity.json 与所有 PE/图标/场景/载荷/身份断言保持原样。

从本公开仓库 tools/preview5-verification/ 获取补充工具。先按 docs/testing/WINDOWS_CODEX_TEST_GUIDE.md 第 0 节核对固定工具 commit 和所有文件 SHA256，再将这 11 个文件复制到本轮独立运行根目录的 verification-windows-fix（不要覆盖旧工具与旧证据）。进入该运行根目录，用之前运行核验的同一 CPython 3.12 Windows x64 解释器执行以下 PowerShell 命令：

```powershell
$RunRoot = (Get-Location).Path
$Verifier = Join-Path $RunRoot 'verification-windows-fix'
python -m pip install --only-binary=:all: --no-deps --index-url https://pypi.org/simple --target "$Verifier" --require-hashes -r "$Verifier\requirements-windows-py312-x64.txt"
if ($LASTEXITCODE -ne 0) { throw 'Brotli 安装失败；停止核验' }
python "$Verifier\verify-native.py" "$RunRoot\downloads\wuxian-renzu-jinhua-preview.5-windows-x64.exe" "$RunRoot\downloads\wuxian-renzu-jinhua-preview.5-corresponding-source.zip"
if ($LASTEXITCODE -ne 0) { throw '原生静态核验未通过' }
```

只向该隔离工具目录安装依赖，不安装到全局 Python；需要网络的只有明确的 pip 安装步骤。如果同一 bundled Python 不带 pip，停止并报告缺失，不下载来历不明的 DLL，不修改系统防护。官方包来源：https://pypi.org/project/Brotli/ （Google Brotli，MIT）；1.2.0 提供 CPython 3.12 Windows x64 wheel；此处 requirements 文件限定该 wheel 的官方 SHA256，其他解释器版本/架构会安全失败。Linux/macOS 已安装系统 Brotli 解码库时可以不安装 Python 包。

该命令仅重新做 W01 的 PE/bundle 静态子项，不启动 EXE，也不能解决 B02 的执行策略拒绝。保持此前 Windows 记录不变，另开记录写新结果。不可把云端 Linux 的正向闭包/单元测试或模拟 win32 后端选择称为 Windows 实机验收。

### 解码与回归测试边界

Python 后端使用 Brotli 1.2.0 流式 API，分块请求小输出缓冲并在累计长度超出声明时立即拒绝；底层可能向上取整缓冲大小，不声称 64 KiB 严格内存上限。Python 后端也拒绝尾随垃圾和不完整流。系统 C API 保持旧版一次性有界缓冲行为，可能接受有效 Brotli 流后的尾随字节；完整 EXE SHA256 与每个压缩载荷 SHA256 仍保持原断言，会拒绝实际输入字节改动。

test_brotli_decoder.py 属于云端 Linux 维护测试，依赖相邻 original/、frozen-preview.5.exe 与 Brotli 包，不是可独立在 Windows 运行的测试套件；公开补充工具目录不包含该测试文件。Windows 用户运行的是上述完整只读核验命令。
