# Warp 中文汉化版（非官方个人 fork）

> ⚠️ **这是一个非官方的个人汉化 fork**，并非 Warp 官方发布版本。
> 官方仓库：https://github.com/warpdotdev/warp
> 本 fork：https://github.com/feizaisky/warp

## 说明

本仓库基于 [warpdotdev/warp](https://github.com/warpdotdev/warp) 的源码，将客户端 UI 中的英文界面文案逐步替换为简体中文，仅供个人使用与学习。

- 本汉化**未与 Warp 官方合作**，不代表官方立场
- 翻译质量、稳定性与官方版本可能存在差异
- 如需官方功能、客服支持，请使用 [官方版本](https://www.warp.dev/download)
- 同步官方上游更新可能造成翻译丢失或冲突，需要手动维护

## 当前汉化进度

- [x] `onboarding` 模块（首次启动引导：欢迎、意向、主题、项目、智能体配置等）
- [ ] 系统菜单（macOS/Linux/Windows）
- [ ] 设置面板
- [ ] 命令面板
- [ ] 终端 UI 提示
- [ ] 错误与通知

## 构建与运行

需要按官方 [WARP.md](WARP.md) 准备 Rust 工具链与平台依赖。

```bash
./script/bootstrap   # 安装平台依赖
./script/run         # 编译并运行
./script/presubmit   # fmt、clippy、测试（开发自检）
```

## 同步官方上游

```bash
git fetch upstream
git checkout zh-cn
git rebase upstream/master
git push --force-with-lease
```

冲突多为「英文原文 vs 中文译文」场景，肉眼可解。新增字符串需要同步翻译。

## 协议

完全沿用上游协议：

- `warpui_core` 与 `warpui` crate：[MIT](LICENSE-MIT)
- 其余代码：[AGPL v3](LICENSE-AGPL)

依据 AGPL v3：

- 本 fork 的所有修改源码已在 GitHub 公开
- 任何基于本 fork 的衍生分发必须保留 AGPL 协议并公开源码
- 协议、版权声明、免责条款均未做改动

## 贡献

这是个人自用 fork，原则上不接受外部 PR。如希望参与正式的中文本地化工作，请到[官方仓库](https://github.com/warpdotdev/warp/issues)发起 i18n 相关 issue。

## 反馈

仅限技术性问题：在本仓库的 [Issues](https://github.com/feizaisky/warp/issues) 中提交。功能、账号、订阅等问题请联系官方。
