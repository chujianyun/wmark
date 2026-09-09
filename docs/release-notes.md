本地批量图片水印工具，支持 macOS（Apple Silicon / Intel）和 Windows x64。

- JPG、PNG、静态 WebP 图片导入与批量导出
- 中文文字、透明 Logo、九宫格定位、旋转、平铺、透明度和安全边距
- 原图对比、预览缩放、模板保存与恢复
- 导出参数快照、取消、失败列表与重试
- 原图保护、自动重名编号、EXIF/GPS 移除

此版本为未获平台开发者签名的预览版（Mac 使用 ad-hoc 签名）：没有 Apple Developer ID 签名/公证，也没有 Windows Authenticode 签名，系统可能阻止或提示未知发布者。发布资产附 SHA256 校验值。跨平台 CI 包含自动化测试与打包，不等于已完成所有硬件上的人工验收。

当前范围：单张最多 4000 万像素、200 MiB，批次最多 1000 张；不支持动态图片、HEIC、RAW、PDF 或视频。WebP 输出为无损；仅 JPG 支持质量调节。RGB ICC 转换为 sRGB，非 RGB ICC 会提示转换后重试。

测试报告：[完整用例与验证范围](https://github.com/chujianyun/wmark/blob/main/.spec-to-ship/desktop-v1/20260910_test-report-02.md)。发布结果：[最终发布验收记录](https://github.com/chujianyun/wmark/blob/main/docs/release-verification.md)。
