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
- [x] `onboarding` 弹出提示（认识输入框、智能体对话引导）
- [x] macOS 系统菜单标准动作（关闭、最小化、退出、粘贴、全屏等）
- [x] 顶层菜单结构（File/Edit/View/Tab/Blocks/Drive/Window/Help 及所有菜单项）
- [x] 登录与认证界面（登录、注册、SSO、隐私设置、离线模式）
- [x] 退出/关闭确认对话框
- [x] 设置面板分区标题与账户信息
- [x] 设置 → 外观（主题、图标、字体、窗口、光标等）
- [x] 设置 → AI（智能体、模型、权限、MCP 允许/拒绝列表等）
- [x] 设置 → 功能（通用、会话、按键、终端、通知、工作流等）
- [x] 设置 → 隐私（密钥隐藏、分析、数据管理等）
- [x] 设置 → 键盘快捷键
- [x] 设置 → MCP 服务器（列表、编辑、安装、更新、删除对话框）
- [x] 设置 → 计费与用量
- [x] 标签页上下文菜单与类型标签
- [x] 命令面板（搜索、零状态）
- [x] 终端零状态块、工具提示
- [x] 内联横幅（匿名用户 AI、AWS Bedrock/CLI、通知、SSH、共享会话等）
- [x] 智能体提示语（Tips）
- [x] 共享会话（停止共享、无活动警告、对话结束视图）
- [x] Drive 面板（索引、新建/重命名/删除对话框、导入、回收站）
- [x] 工作流编辑器（标题、参数、别名、环境变量）
- [x] 笔记本编辑器（上下文菜单、块类型、查找栏、链接编辑）
- [x] 自动更新提示
- [x] 欢迎/开始使用视图

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
