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

Obtain the supplementary verifier from tools/preview5-verification/ at the immutable source commit b4fec8adbae2a4a47dd1f840005ef38393a92a80. Verify its 11 original files against the SHA256 values below before placing them in a new isolated verification-windows-fix directory. These hashes identify that historical tool snapshot, including its original README; they do not identify this revised documentation. Do not overwrite existing tools or evidence. From the run directory, use the same existing CPython 3.12 Windows x64 interpreter for the following PowerShell commands:

```text
3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986  LICENSE
3e73fa781832343ca0f5e63d35252a8a40097528b6d343ad0dc4346e9f22e44d  PUBLIC-NATIVE-EVIDENCE.json
7855f054ceadc23982938362503f72baedcd7f940308205cec9e26066145d2b0  README.md
d2c88bb855559bb53aa9ee86143f06d4c002e69a29165d7749d587379dcdea76  brotli_decoder.py
690c84621d98edef548f73c8f6fa302cf4a523da0b8904c0c02898dbdd049e87  bundle-identity.json
e1eb4056e66f3e43015df65f045e7e10043cf6a6edeb59aa131f66239cceffed  icon_parser.py
24bf1914d30cb8c148462034a092b57870778d98b6884eebbd0cdb9446a1aae0  requirements-windows-py312-x64.txt
cbb32b6cef7923ebb58a9c9c4cdd2a9f84fe58a18e8919fa657b153541347cad  toolchain-supplement.json
af96caae96bab98d6b403b1f6ab5625e4e4d8d235390a5a543d255fed958d5c2  verify-native.py
2a112b00bc316bcfa72c8278fbd6fab0018fd718cf68a37d2e101f30236f63aa  verify-source.py
5737defad4892f141c4ab0a94cbef06a5b0c02412df7d2e03042c95d5649cde0  windows-dependency-coverage.json
```

```powershell
$RunRoot = (Get-Location).Path
$Verifier = Join-Path $RunRoot 'verification-windows-fix'
python -m pip install --only-binary=:all: --no-deps --index-url https://pypi.org/simple --target "$Verifier" --require-hashes -r "$Verifier\requirements-windows-py312-x64.txt"
if ($LASTEXITCODE -ne 0) { throw 'Brotli 安装失败；停止核验' }
python "$Verifier\verify-native.py" "$RunRoot\downloads\wuxian-renzu-jinhua-preview.5-windows-x64.exe" "$RunRoot\downloads\wuxian-renzu-jinhua-preview.5-corresponding-source.zip"
if ($LASTEXITCODE -ne 0) { throw '原生静态核验未通过' }
```

只向该隔离工具目录安装依赖，不安装到全局 Python；需要网络的只有明确的 pip 安装步骤。如果同一 bundled Python 不带 pip，停止并报告缺失，不下载来历不明的 DLL，不修改系统防护。官方包来源：https://pypi.org/project/Brotli/ （Google Brotli，MIT）；1.2.0 提供 CPython 3.12 Windows x64 wheel；此处 requirements 文件限定该 wheel 的官方 SHA256，其他解释器版本/架构会安全失败。Linux/macOS 已安装系统 Brotli 解码库时可以不安装 Python 包。

This command performs only PE/bundle static verification. It does not start the EXE or establish permission to run it. Preserve previous verification evidence and record any new results separately. Linux validation, unit tests, and simulated Windows backend selection are not Windows gameplay acceptance.

### 解码与回归测试边界

Python 后端使用 Brotli 1.2.0 流式 API，分块请求小输出缓冲并在累计长度超出声明时立即拒绝；底层可能向上取整缓冲大小，不声称 64 KiB 严格内存上限。Python 后端也拒绝尾随垃圾和不完整流。系统 C API 保持旧版一次性有界缓冲行为，可能接受有效 Brotli 流后的尾随字节；完整 EXE SHA256 与每个压缩载荷 SHA256 仍保持原断言，会拒绝实际输入字节改动。

test_brotli_decoder.py 属于云端 Linux 维护测试，依赖相邻 original/、frozen-preview.5.exe 与 Brotli 包，不是可独立在 Windows 运行的测试套件；公开补充工具目录不包含该测试文件。Windows 用户运行的是上述完整只读核验命令。

