# Wmark

一个在本地处理图片的跨平台水印工具。使用 Tauri 2、React、TypeScript 和 Rust，支持 macOS 与 Windows。

## 功能

- 导入 JPG、PNG、静态 WebP，支持批量选择、拖放、缩略图、全选和移除。
- 中文文字与透明 Logo 水印：颜色、大小、透明度、旋转、九宫格、自由位置、安全边距和平铺间距。
- 预览与导出共用 Rust 渲染核心；原图对比、预览缩放、拖动定位、方向键微调（Shift 加大步长）。
- 内置摄影署名、证件保护、样片模板；本地保存多个模板并恢复上次设置。
- 批量导出至指定目录，保持原格式或转为 PNG/JPG/WebP；JPG 支持质量及透明背景色设置。
- 导出使用参数快照，支持取消、部分失败与仅重试失败项。
- 不覆盖原图或已有输出；同名自动编号；移除源文件 EXIF/GPS/XMP 元数据。

## 安装

从 [Releases](https://github.com/chujianyun/wmark/releases) 下载适合平台的文件：

| 平台 | 安装包 |
| --- | --- |
| macOS Apple Silicon | `*_aarch64.dmg` |
| macOS Intel | `*_x64.dmg` |
| Windows x64 | `*_x64-setup.exe` |

当前为预览版本。macOS 使用 ad-hoc 签名，没有 Apple Developer ID 签名或公证；Windows 没有 Authenticode 签名。系统信任验证可能阻止运行或提示未知发布者，签名状态不等同于功能测试。发布页附 `SHA256SUMS.txt`，请核对来源与校验值。

## 使用

1. 导入或拖入图片，右侧设置默认应用于整个批次。
2. 选择文字或图片水印，调整参数，在中间查看效果。
3. 点击“导出图片”，设置范围、文件夹、格式与后缀，开始导出。
4. 查看成功、失败、未处理数量，打开输出目录；失败项可修复后单独重试。

水印字体内置 Noto Sans CJK SC，授权见 `assets/fonts/OFL.txt`。未标记色彩空间的输入按 sRGB 解释；RGB ICC 输入转换至 sRGB。非 RGB ICC 输入提示先转换。WebP 输出无损，质量滑块只作用于 JPG。水印不能保证防止盗图或滥用。

### 支持边界

- 单图最多 4000 万像素、宽高各不超过 20000px、编码文件不超过 200MiB；一个批次最多 1000 张。
- 只支持静态图片；动态 PNG/WebP 拒绝，不静默取首帧。
- 暂不支持 HEIC、RAW、PDF、视频；不保留原始 EXIF/GPS/XMP。
- 图片与 Logo 不上传。模板存于平台应用配置目录；Logo 模板引用本地原文件，移动文件后需要重新选择。
- 预览与任务串行处理，导出时预览暂停，设置修改用于下一任务；取消在当前处理阶段完成后生效。
- 不支持关闭应用后恢复未完成任务；已经成功导出的文件保留。

## 本地开发

需要 Node.js 22+、Rust stable，以及 [Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)（Mac 的 Xcode 命令行工具；Windows 的 MSVC C++ 工具和 WebView2）。

```sh
npm ci
npm run tauri dev
```

`npm run dev` 仅启动界面服务；文件操作需要 Tauri 桌面环境。历史交互原型位于 `prototype/index.html`。

## 验证与打包

```sh
npm run check
npm run build
cargo fmt --all -- --check
npm run tauri -- build
```

测试分为 Rust 图像/文件集成测试、桌面输入与配置测试、React 交互测试。React 测试使用 IPC mock；实际图片解码、合成、元数据和文件输出由 Rust 集成测试覆盖，真实桌面链路另行验收，不把 mock 结果称为端到端通过。详细报告见 `.spec-to-ship/desktop-v1/`。

生成合成测试图片：

```sh
cargo run -p wmark-core --example fixtures
```

## 自动发布

`.github/workflows/desktop.yml` 在 main、PR、手动触发时测试并构建三个目标：Mac ARM64、Mac x64、Windows x64。版本标签 `v*` 触发同样检查；全部成功后由单独任务发布预览 Release，上传安装包和 SHA256 校验值。任何目标失败都不发布。

发布前同步 `package.json`、`package-lock.json`、两个 Cargo manifest、`Cargo.lock` 与 `src-tauri/tauri.conf.json` 的版本，更新 Release notes，测试通过后提交，再推送对应标签。检查脚本会阻止标签与工程版本不一致的发布。

不设置应用自动更新；后续需配置签名与可信更新通道后再启用。

## 目录

- `src/`：React 界面、类型与 IPC 客户端。
- `src-tauri/`：桌面窗口、文件授权、任务、配置和打包。
- `crates/wmark-core/`：独立图像引擎、校验、原子输出、批量任务和集成测试。
- `docs/`：原型方案、实施决策和发布说明。
- `.spec-to-ship/`：测试与前端验收记录。

项目代码采用 MIT 许可证；内置字体采用 SIL OFL 1.1，第三方依赖保留其各自许可证。
