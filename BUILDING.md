# 构建该测试版

本工程使用 Rust/Tauri 2 和 TypeScript/Vite/Pixi。依赖版本锁定在 server-rs/Cargo.lock 与 apps/web/package-lock.json。

## Windows 构建路径

安装相应的 Rust MSVC 工具链、Node.js、Microsoft C++ 构建工具及 Windows SDK，并遵守各提供方条款。先在 apps/web 执行 npm ci，再执行 npm run build；在 server-rs 执行 cargo build --locked --release --features tauri/custom-protocol。本版构建需要 Git HEAD 和干净工程身份，源码压缩包解压后应先 git init 并提交本地源树，再构建；本地提交 ID 将成为该次构建的身份，不会伪装为官方发布提交。

Web 产物由 Tauri 嵌入 EXE。不要把本地缓存、node_modules、target 或 .git 放入对应源码附件。构建脚本会校验实际地图、资源清单及 Web bundle 身份。

## 验证与资源修改

npm run typecheck 检查 Web 类型。首次测试前必须先执行 npm run build 生成实际资源，因为 npm test 只编译测试模块，不复制资源；随后 npm test 执行 Web 测试。测试产生的 tsc 模块会改变 dist，生成原生程序前必须再次 npm run build，以恢复严格的生产输出集合。Rust 库与选定集成测试保留在 server-rs/tests。tools/cloud-verification 提供明确标记的无原生窗口测试适配器，它不等同于 Tauri 或 Windows 实机验证。

原始已批准 PNG 与派生资源均保留在 assets 中。tools/asset-pipeline、tools/cenyao-runtime-master 和 tools/world-map-compiler 提供对应转换与地图编译工具。Pillow 等图像编码库版本可能影响派生文件的字节；正式替换资源时必须重新验证清单与构建，不得仅凭像素相似覆盖哈希。

## 清理差异

本导出移除了未注册旧命令、未使用的旧剧情模块及历史工作报告。在公开 preview.2 的正式模块边界内，选择性加入了 Beacon、Sentinel、输入/反馈与自动档继续修复。图片字节和依赖锁文件保持 preview.2 身份；旧网格 V2 仍采用仅拒绝头部解析。它不会将旧网格存档迁入当前游戏，且不写原文件。此源码因此需要独立重新编译；不能将先前 EXE 当作此清理后的构建产物。

精确冻结身份、工具链、依赖源码附件与构建结果由各次发布清单列出。源码重新提交或资源改变后，必须重新记录该次构建的身份。

历史目录校验器提示：当前 MH/CW 叙事目录 Python 检查器仍含较早的 staged-marker 断言，在未清理前的源树上也失败。它们属于尚待更新的叙事创作审计，不是当前游戏运行验收；本预览未将此两项声称为通过。GH/回归站目录检查及资源来源/准入检查已分别记录，最终以发布验证说明为准。

默认 Rust 库与集成测试之外，升降平台/垂直移动的受控回放测试还需使用 --features deterministic-replay 单独执行。legacy-test-rng 不是本公开正式运行库的模块依赖。

源码与素材/依赖继承边界见 SOURCE_LINEAGE.md。当前保留的历史叙事与素材创作审计测试有明确未闭合项，见 KNOWN_LIMITATIONS.md；不得将所有工具测试描述为全通过。

## 原生图标重建

原始PNG位于server-rs/icons/source/chatgpt-portal-emblem-20261003.png。使用既有tools/asset-pipeline/requirements.txt锁定的Pillow12.3.0运行python tools/native-icon/build_icon.py --check，可不写文件地重算并验证全部ICO帧。移除--check才重写ICO；python -m unittest discover -s tools/native-icon -v验证源hash、alpha、七种尺寸、帧边界、逐RGBA与标准ICO解码。此工具仅转换格式和尺寸，不生成新美术。
