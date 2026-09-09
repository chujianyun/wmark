# Wmark v0.1.0 发布验收记录

## 结论

PASS：本地57项自动化测试通过，标签提交在三平台的自动化检查和安装包构建全部成功，GitHub Release 自动发布成功。已从公开发布页下载全部4项资产并检查实际文件大小和SHA256；3个安装包全部与发布的校验清单一致，2个Mac DMG额外通过 `hdiutil verify`。

这是已执行范围的通过结论，不表示所有操作组合绝对正确。Windows与Intel Mac的证据是对应系统CI测试/构建；没有将其写成人工安装验收。

- [下载 v0.1.0](https://github.com/chujianyun/wmark/releases/tag/v0.1.0)
- [完整发布流水线](https://github.com/chujianyun/wmark/actions/runs/34379820020)
- 发布时间：2026-09-09T17:04:03Z（北京时间2026-09-10 01:04:03）
- 标签：`v0.1.0`，提交：`5ee6849defa71c03cb73850b09280629455535fd`
- 功能代码：`e6c5c3a`；标签提交及本记录补充只改文档，未改变发行程序。
- Release 状态：已公开发布、非草稿、预览版。

## 跨平台结果

| 任务 | 结果 | 证据 |
|---|---|---|
| build (macos-14, aarch64-apple-darwin, dmg) | PASS | [运行记录](https://github.com/chujianyun/wmark/actions/runs/34379820020/job/102561620566) |
| build (macos-15-intel, x86_64-apple-darwin, dmg) | PASS | [运行记录](https://github.com/chujianyun/wmark/actions/runs/34379820020/job/102561620857) |
| build (windows-latest, x86_64-pc-windows-msvc, nsis) | PASS | [运行记录](https://github.com/chujianyun/wmark/actions/runs/34379820020/job/102561620979) |
| release | PASS | [运行记录](https://github.com/chujianyun/wmark/actions/runs/34379820020/job/102564243326) |

三个构建任务均执行前端23项、Rust核心29项、桌面契约5项测试（共57项），以及TypeScript、格式、Clippy、标签版本检查和安装包构建。Release任务等待全部成功后，验证2份DMG和1份安装EXE的数量，生成校验文件并自动发布。

## 下载与校验

下表为实际下载文件计算值，已同时比对GitHub资产digest和SHA256SUMS.txt（清单本身仅比对GitHub digest）。

| 资产 | 字节 | SHA256 |
|---|---:|---|
| [SHA256SUMS.txt](https://github.com/chujianyun/wmark/releases/download/v0.1.0/SHA256SUMS.txt) | 268 | `6d4cd069913837f5e23330eef9022f423a4eebaa5c68c6a0f624acb887d2096d` |
| [Wmark_0.1.0_aarch64.dmg](https://github.com/chujianyun/wmark/releases/download/v0.1.0/Wmark_0.1.0_aarch64.dmg) | 18005539 | `d2935d6a08f070ec3960f924d868d5568d916d3a07e194c21e7273d174e8abab` |
| [Wmark_0.1.0_x64-setup.exe](https://github.com/chujianyun/wmark/releases/download/v0.1.0/Wmark_0.1.0_x64-setup.exe) | 14223559 | `8283be38996d2bf392eadcc0d81688500fb6678a4d51d50ea9ad9b22591e9ce4` |
| [Wmark_0.1.0_x64.dmg](https://github.com/chujianyun/wmark/releases/download/v0.1.0/Wmark_0.1.0_x64.dmg) | 18338403 | `57b83acf4fe77983dd5cc7367e15c815231c195fdf2ea99e746f66f9d37ea83f` |

## Mac 本机交付

最新版已安装至应用程序目录，启动正常并恢复上次水印设置。本机包签名完整性检查和DMG校验通过；实际合成图片导入、预览、原生输出选择、导出与像素比对见完整报告。

本机最新版可执行文件与本机构建文件SHA256均为 `40b0b773ba09b310fca6faa2db50766bfba751f442e4059cb703c98ab03e3ffd`。这不是GitHub安装包哈希；CI构建环境不同，发行资产按上表独立校验。最新安装启动截图见 `.spec-to-ship/desktop-v1/screenshots/final-installed-startup.png`。

## 报告和边界

- [完整测试报告](../.spec-to-ship/desktop-v1/20260910_test-report-02.md)：用例目录、需求覆盖、失败修复、性能观察和未覆盖场景。
- [前端门禁](../.spec-to-ship/desktop-v1/20260910_frontend-code-gate-01.md)：初始功能验收PASS。
- [最新界面修复回归](../_reports/bug-fixes/20260910-workspace/fix-report.md)：通知、模板选中、布局修复，新增3项测试通过；本次发布已包含。
- [代码质量审计](../_reports/code-quality-audit-20260910-01.md)：96.5分，未发现发布阻断问题，保留低优先级维护项。

当前未配置Apple Developer ID/公证和Windows Authenticode，Mac仅有ad-hoc签名，系统可能提示未知发布者。未完成Windows/Intel Mac人工视觉验收、全设备缩放测试、断电/磁盘满注入、长期泄漏及100张大图压力测试；完整限制见测试报告与README。
