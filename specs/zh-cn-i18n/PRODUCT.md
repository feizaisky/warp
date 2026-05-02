# 简体中文完整汉化（zh-cn 分支）

## Summary

将 Warp 客户端所有用户可见的 UI 字符串替换为简体中文，完成非官方个人 fork（zh-cn 分支）的汉化收尾工作。前期已翻译约 235 个文件，本次覆盖剩余约 35 个含用户可见字符串的文件，达到整体 UI 全中文的目标。

## Goals / Non-goals

**Goals:**
- 所有用户在操作界面中直接看到的字符串均为简体中文
- 已翻译文件保持一致的术语风格（见"术语表"）
- 不破坏现有已翻译内容

**Non-goals:**
- 不翻译后端日志、telemetry key、错误上报字符串（这些不显示给用户）
- 不翻译代码注释、文档字符串
- 不翻译设置项的内部 key（如 `CanResumeConversation`）
- 不做官方 i18n 框架改造；继续沿用直接替换字符串字面量的方式
- 不翻译外部链接 URL 本身

## 术语表（与已翻译文件保持一致）

| 英文 | 中文 |
|------|------|
| Agent / agent | 智能体 |
| Conversation | 对话 |
| Workflow | 工作流 |
| Notebook | 笔记本 |
| Drive | Drive |
| Tab | 标签页 |
| Pane | 面板 |
| Block | 块 |
| Settings | 设置 |
| Command Palette | 命令面板 |
| Slash command | 斜杠命令 |
| Skill | 技能 |
| Rule | 规则 |
| Compact | 精简 |
| Fork | 分叉 |
| Context | 上下文 |
| Attach | 附加 |
| Prompt | 提示词 |
| Execution profile | 执行配置文件 |
| Credit | 额度 |
| Diff | 差异 |
| Code review | 代码审查 |
| Shared session | 共享会话 |
| Hotkey window | 快捷键窗口 |
| Launch configuration | 启动配置 |
| Environment | 环境 |
| Project explorer | 项目浏览器 |
| Secret | 密钥 |
| Env var | 环境变量 |

## Behavior

以下列出所有待翻译文件及每条字符串的翻译要求，按功能模块分组。翻译原则：

1. 所有面向用户的字符串（按钮文字、提示、错误信息、菜单项、描述文字）替换为简体中文。
2. 带格式占位符的字符串（`{}`、`{n}`、`{name}`）保留占位符原样，仅翻译其余文字部分。
3. 快捷键名称（如 `Shift + ctrl + space`）保持英文/符号形式，不翻译按键本身。
4. 技术性专有名词（如 `jq`、`Docker`、`Gemini CLI`、`Claude Code`、`TOML`、`YAML`）保持原文。
5. 单个英文单词作为 UI 标签但有通用中文对应的（如 `Settings` → `设置`），按术语表翻译。
6. 不确定的文案不得擅自意译；宁可直译保留意思，避免歧义。

---

### 模块 1：工作区命令与导航（workspace/mod.rs）

文件中包含大量 action label 和 description 字符串，这些出现在命令面板、菜单、快捷键说明中。

典型字符串（需翻译）：
- `Switch to next tab` → `切换到下一个标签页`
- `Create New Window` → `新建窗口`
- `Zoom In` / `Zoom Out` / `Reset Zoom` → `放大` / `缩小` / `重置缩放`
- `Toggle project explorer` → `切换项目浏览器`
- `Open theme picker` → `打开主题选择器`
- `Save new launch configuration` → `保存新的启动配置`
- `Switch to 1st tab` / `2nd tab` / `3rd tab` / `last tab` → `切换到第 1 个标签页` 等
- `Activate previous tab` / `Activate next tab` → `激活上一个标签页` / `激活下一个标签页`
- `Toggle Mouse Reporting` → `切换鼠标上报`
- `Create a new team notebook` → `新建团队笔记本`
- `Create a new personal workflow` → `新建个人工作流`
- `New Terminal Tab` / `New Agent Tab` / `New Cloud Agent Tab` → `新建终端标签页` / `新建智能体标签页` / `新建云端智能体标签页`
- `Open Left Panel` → `打开左侧面板`
- `Toggle code review` → `切换代码审查`
- `Left Panel: Agent conversations` → `左侧面板：智能体对话`
- `Toggle Warp Drive` / `Warp Drive` → `切换 Warp Drive` / `Warp Drive`
- `Toggle command palette` / `Command Palette` → `切换命令面板` / `命令面板`
- `Rename the current tab` → `重命名当前标签页`
- `Quit Warp` → `退出 Warp`
- `Close Window` → `关闭窗口`
- `Close the current tab` → `关闭当前标签页`
- `Close other tabs` → `关闭其他标签页`
- `Close tabs to the right` → `关闭右侧标签页`
- `Install update and relaunch` → `安装更新并重启`
- `Check for updates` → `检查更新`
- `Log out` → `退出登录`
- `View latest changelog` → `查看最新更新日志`
- `Toggle Warp AI` → `切换 Warp AI`
- `Open Settings` → `打开设置`
- `Open Settings: Account` → `打开设置：账户`
- `Invite People...` → `邀请成员...`
- `Send feedback (opens external link)` → `发送反馈（打开外部链接）`
- `Dump debug info` → `导出调试信息`（仅开发调试用，可保留英文）
- `Crash the app (for testing ...)` → 不翻译（仅测试用）

---

### 模块 2：根视图（root_view.rs）

- `Screen edge to pin the hotkey window to.` → `快捷键窗口吸附的屏幕边缘。`
- `Hide All Windows` → `隐藏所有窗口`
- `Show Dedicated Hotkey Window` → `显示专用快捷键窗口`
- `Hide Dedicated Hotkey Window` → `隐藏专用快捷键窗口`
- `Toggle fullscreen` → `切换全屏`
- `Create Environment` → `新建环境`
- `Resource not found or access denied` → `资源未找到或访问被拒绝`
- `Web auth handoff is unavailable` → `网页身份验证跳转不可用`

---

### 模块 3：编辑器视图（editor/view/mod.rs）

- Voice 限额错误：`You have hit the limit for Voice requests. Your limit will be refreshed as a part of your next cycle.` → `您已达到语音请求的使用上限，将在下个计费周期重置。`
- `An error occurred while processing your voice input.` → `处理语音输入时发生错误。`
- 编辑器 action 名称（出现在键位说明）：
  - `Select one word to the left` → `向左选择一个词`
  - `Select one word to the right` → `向右选择一个词`
  - `Select one character to the left` → `向左选择一个字符`
  - `Select one character to the right` → `向右选择一个字符`
  - `Select all` → `全选`
  - `Add selection for next occurrence` → `为下一处匹配添加选区`
  - `Accept autosuggestion` → `接受自动建议`
  - `Clear command editor` → `清空命令编辑器`
  - `Add cursor above` / `Add cursor below` → `在上方添加光标` / `在下方添加光标`
  - `Exit Vim insert mode` → `退出 Vim 插入模式`
- 图片附件错误提示：
  - `Image attachment isn't supported by this model` → `该模型不支持图片附件`
  - `Loading...` → `加载中...`
  - `Attach images` → `附加图片`
  - `The selected model does not support images as context.` → `所选模型不支持将图片作为上下文。`
  - `1 image wasn't attached - {limit_reason}.` → `1 张图片未附加——{limit_reason}。`
  - `{num_excess_images} images weren't attached - {limit_reason}.` → `{num_excess_images} 张图片未附加——{limit_reason}。`
  - `Failed to read file: {e}` → `读取文件失败：{e}`
  - `Image cannot be attached - failed to read file.` → `图片无法附加——读取文件失败。`
  - `1 image wasn't attached - failed to read file.` → `1 张图片未附加——读取文件失败。`
  - `Image cannot be attached - file is too large.` → `图片无法附加——文件过大。`
  - `1 image wasn't attached — file is too large.` → `1 张图片未附加——文件过大。`
  - `{num_oversized_images} images weren't attached — files are too large.` → `{num_oversized_images} 张图片未附加——文件过大。`
  - `Image cannot be attached - error processing.` → `图片无法附加——处理出错。`
- `Unable to select next occurrence` → `无法选中下一处匹配`
- `Search files and directories` → `搜索文件和目录`
- `Pasting: {}` → `正在粘贴：{}`

---

### 模块 4：代码编辑器操作（code/editor/view/actions.rs）

均为编辑器快捷键 action 名称，出现在键位提示中：

- `Move Backward One Word` / `Move Forward One Word` → `向后移动一个词` / `向前移动一个词`
- `Move cursor up/down/left/right` → `光标上移/下移/左移/右移`
- `Move to line start` / `Move to line end` → `移至行首` / `移至行尾`
- `Cursor at buffer start` / `Cursor at buffer end` → `光标移至缓冲区开头` / `光标移至缓冲区末尾`
- `Select one word to the left/right` → `向左/右选择一个词`
- `Select up/down` → `向上/下选择`
- `Select all` → `全选`
- `Select to start/end of line` → `选至行首/行尾`
- `Remove the previous character` → `删除前一个字符`
- `Toggle comment` → `切换注释`
- `Cut word left/right` → `剪切左/右侧词`
- `Delete word left/right` → `删除左/右侧词`
- `Cut all left/right` → `剪切光标左/右侧全部内容`
- `Delete all left/right` → `删除光标左/右侧全部内容`
- `Exit Vim insert mode` → `退出 Vim 插入模式`
- `Find in code editor` → `在代码编辑器中查找`
- `Go to line` → `跳转到行`

---

### 模块 5：斜杠命令菜单描述（search/slash_command_menu/static_commands/commands.rs）

这些描述显示在斜杠命令列表的副标题中：

- `Start a new conversation` → `开始新对话`
- `Start a new cloud agent conversation` → `开始新的云端智能体对话`
- `Pull GitHub PR review comments` → `拉取 GitHub PR 审查评论`
- `Create an Oz environment (Docker image + repos) via guided setup` → `通过引导向导创建 Oz 环境（Docker 镜像 + 仓库）`
- `Create a new docker sandbox terminal session` → `创建新的 Docker 沙盒终端会话`
- `Have Oz walk you through creating a new coding project` → `让 Oz 引导你创建新的编程项目`
- `Open a skill's markdown file in Warp's built-in editor` → `在 Warp 内置编辑器中打开技能的 Markdown 文件`
- `Invoke a skill` → `调用技能`
- `Add new Agent prompt` → `添加新的智能体提示词`
- `Add a new global rule for the agent` → `为智能体添加新的全局规则`
- `Open a file in Warp's code editor` → `在 Warp 代码编辑器中打开文件`
- `Rename the current tab` → `重命名当前标签页`
- `Set the color of the current tab` → `设置当前标签页的颜色`
- `Fork the current conversation in a new pane or a new tab` → `在新面板或新标签页中分叉当前对话`
- `Open code review` → `打开代码审查`
- `Index this codebase` → `索引此代码库`
- `Open the latest changelog` → `打开最新更新日志`
- `Send feedback` → `发送反馈`
- `Switch to another indexed repository` → `切换到另一个已索引的仓库`
- `View all of your global and project rules` → `查看所有全局和项目规则`
- `Switch the base agent model` → `切换基础智能体模型`
- `Switch the cloud agent execution host` → `切换云端智能体执行主机`
- `Switch the cloud agent harness` → `切换云端智能体运行框架`
- `Switch the cloud agent environment` → `切换云端智能体环境`
- `Switch the active execution profile` → `切换当前执行配置文件`
- `Prompt the agent to do some research and create a plan for a task` → `提示智能体进行调研并制定任务计划`
- `Break a task into subtasks and run them in parallel with multiple agents` → `将任务拆分为子任务并通过多个智能体并行执行`
- `Free up context by summarizing convo history` → `通过精简对话历史释放上下文空间`
- `Compact conversation and then send a follow-up prompt` → `精简对话后发送后续提示词`
- `Queue a prompt to send after the agent finishes responding` → `在智能体完成响应后发送排队的提示词`
- `Fork current conversation and compact it in the forked copy` → `分叉当前对话并在分叉副本中精简`
- `Continue this cloud conversation locally` → `在本地继续此云端对话`
- `Open billing and usage settings` → `打开账单和用量设置`
- `Start remote control for this session` → `为此会话启动远程控制`
- `Toggle credit usage details` → `切换额度用量详情`
- `Open conversation history` → `打开对话历史`
- `Search saved prompts` → `搜索已保存的提示词`
- `Rewind to a previous point in the conversation` → `回退到对话中的某个节点`
- `Export current conversation to clipboard in markdown format` → `以 Markdown 格式将当前对话导出到剪贴板`
- `Export current conversation to a markdown file` → `将当前对话导出为 Markdown 文件`
- 占位符提示（如 `<describe what you want to build>`）→ `<描述你想构建的内容>`，`<path/to/file[:line[:col]]> or "@" to search` → `<文件路径[:行[:列]]> 或 "@" 搜索`，`<tab name>` → `<标签页名称>`

---

### 模块 6：共享会话错误信息（terminal/shared_session/viewer/network.rs）

这些出现在共享会话连接失败时的提示气泡或状态文字中：

- `This channel does not support session-sharing.` → `此频道不支持会话共享。`
- `Failed to join shared session.` → `加入共享会话失败。`
- `Failed to connect. Please try again later.` → `连接失败，请稍后重试。`
- `Shared session not found.` → `未找到共享会话。`
- `Invalid session sharing link.` → `共享会话链接无效。`
- `The maximum number of participants for this shared session has been reached.` → `此共享会话已达到最大参与人数上限。`
- `You don't have access to this link.` → `您无权访问此链接。`
- `Something went wrong. Please ask sharer to reshare to continue.` → `出现错误，请要求分享者重新共享以继续。`
- `Sharing ended due to sharer inactivity` → `因分享者长时间无操作，共享已结束`
- `Session ended.` → `会话已结束。`
- `Your access to the session was removed. Please ask sharer to reshare to continue.` → `您的会话访问权限已被移除，请要求分享者重新共享以继续。`
- `Insufficient permissions. Please request edit access.` → `权限不足，请申请编辑权限。`
- `Failed to execute command. Please try again.` → `执行命令失败，请重试。`
- `Failed to make edit. Please try again.` → `编辑失败，请重试。`
- `Invalid conversation. Please try again.` → `对话无效，请重试。`
- `A long running command is currently in progress. Please wait for it to complete before sending an agent prompt.` → `当前有长时命令正在运行，请等待其完成后再发送智能体提示词。`
- `Failed to perform action. Please try again.` → `操作失败，请重试。`

---

### 模块 7：代码审查状态（code_review/diff_state.rs）

- `Uncommitted changes` → `未提交的更改`
- `No local changes to save` → `没有可保存的本地更改`
- `{} files` → `{} 个文件`
- `Changes vs. {main_branch_name}` → `与 {main_branch_name} 的差异`
- `Changes vs. {branch}` → `与 {branch} 的差异`

---

### 模块 8：AI 内联视图（多个文件）

**blocklist/agent_view/shortcuts/mod.rs**
- `input shell command` → `输入 Shell 命令`
- `for slash commands` → `用于斜杠命令`
- `for file paths and attaching other context` → `用于文件路径和附加其他上下文`
- `open code review` → `打开代码审查`
- `toggle conversation list` → `切换对话列表`

**blocklist/block/view_impl/orchestration.rs**
- `Generating title...` → `正在生成标题...`
- `Send message to {recipients} cancelled.` → `向 {recipients} 发送消息已取消。`
- `Sending message to ` → `正在发送消息至`
- `Started agent ` → `已启动智能体`

**blocklist/inline_action/ask_user_question_view.rs**
- `Type your answer and press Enter` → `输入答案并按 Enter 确认`
- ` (select all that apply)` → `（可多选）`
- `Questions skipped` → `已跳过问题`
- `Q: {}` → `Q：{}`

**blocklist/inline_action/search_codebase.rs**
- `Searched for "{}" in {}` → `已在 {} 中搜索"{}"` 
- `Searched for "{}"` → `已搜索"{}"`
- `No results found` → `未找到结果`
- `Searching for "{}" in {}` → `正在 {} 中搜索"{}"`
- `Searching codebase for "{}"` → `正在代码库中搜索"{}"`

**blocklist/inline_action/web_search.rs**
- `Searching the web for "{q}"` → `正在网络中搜索"{q}"`
- `Searching the web` → `正在搜索网络`
- `Searched the web` → `已完成网络搜索`
- `Searched the web for "{query}"` → `已在网络中搜索"{query}"`

**blocklist/summarization_cancel_dialog.rs**
- `Cancel summarization` → `取消精简`
- `Continue summarization` → `继续精简`

**blocklist/usage/conversation_usage_view.rs**
- `Credits spent (total)` → `已消耗额度（总计）`
- `Credits spent` → `已消耗额度`
- `Tool calls` → `工具调用次数`
- `Context window used` → `已使用上下文窗口`
- `Files changed` → `修改的文件`

**blocklist/view_util.rs**
- `Attach as agent context` → `附加为智能体上下文`
- `Follow up with existing conversation` → `在现有对话中继续`

---

### 模块 9：设置与外部编辑器（settings_view 文件）

**settings_view/features/external_editor.rs**
- `Group files into single editor pane` → `将文件合并到单个编辑器面板`
- `When this setting is on, any files opened in the same tab will be automatically grouped into a single editor pane.` → `开启此设置后，在同一标签页中打开的文件将自动合并到单个编辑器面板。`
- `Split Pane` → `拆分面板`
- `New Tab` → `新标签页`
- `Default App` → `默认应用`
- `Choose an editor to open file links` → `选择打开文件链接的编辑器`
- `Choose an editor to open files from the code review panel, project explorer, and global search` → `选择从代码审查面板、项目浏览器和全局搜索中打开文件的编辑器`
- `Choose a layout to open files in Warp` → `选择在 Warp 中打开文件的布局方式`
- `Open Markdown files in Warp's Markdown Viewer by default` → `默认在 Warp Markdown 查看器中打开 Markdown 文件`

**settings_view/settings_page.rs**
- `This setting is not synced to your other devices` → `此设置不会同步到其他设备`
- `Reset to default` → `恢复默认`

**settings/import/view.rs**
- `Reset to Warp defaults` → `恢复 Warp 默认设置`
- `1 other setting` → `另外 1 项设置`
- `{} other settings` → `另外 {} 项设置`

**settings_view/platform/create_api_key_modal.rs**
- `1 day` / `30 days` / `90 days` → `1 天` / `30 天` / `90 天`
- `This secret key is shown only once. Copy and store it securely.` → `此密钥仅显示一次，请复制并妥善保管。`
- `Create key` → `创建密钥`
- `Secret key copied.` → `密钥已复制。`

---

### 模块 10：环境变量视图（env_vars/view 文件）

**env_var_collection.rs**
- `Add secret or command. Warp never stores external secrets` → `添加密钥或命令。Warp 不存储外部密钥`
- `Add a title` → `添加标题`
- `Add a description` → `添加描述`
- `Close Env Var Collection` → `关闭环境变量集合`
- `This environment variable cannot be created due to conflicts with your enterprise's secret redaction settings. Contact a team admin for details.` → `因与企业密钥脱敏设置冲突，无法创建此环境变量，请联系团队管理员了解详情。`
- `This environment variable cannot be created due to conflicts with your secret redaction settings. Save the secret as an environment variable...` → `因与密钥脱敏设置冲突，无法创建此环境变量，将密钥另存为环境变量...`
- `Env var not found and could not be invoked` → `未找到环境变量，无法调用`
- `An error occurred while trying to invoke the env var` → `尝试调用环境变量时发生错误`
- `No env var to invoke` → `没有可调用的环境变量`

**env_vars/view/menus.rs**
- `Clear secret` → `清除密钥`
- `Split pane right/left/down/up` → `向右/左/下/上拆分面板`
- `Minimize pane` / `Maximize pane` / `Close pane` → `最小化面板` / `最大化面板` / `关闭面板`
- `Copy link` → `复制链接`

---

### 模块 11：终端输入与提示（terminal/input 文件）

**input/message_bar/attached_context.rs**
- `` `{}` attached as context `` → `` 已附加 `{}` 作为上下文 ``
- `` `{}` and 1 other command attached as context `` → `` 已附加 `{}` 和另外 1 条命令作为上下文 ``
- `` `{}` and {} other commands attached as context `` → `` 已附加 `{}` 和另外 {} 条命令作为上下文 ``
- ` to remove` → ` 以移除`
- `selected text attached as context` → `已选择文字附加为上下文`

**input/models/view.rs**
- `Full Terminal Use` → `完整终端使用`
- `Manage defaults` → `管理默认设置`

**input/skills/data_source.rs**
- `No skills found` → `未找到技能`
- ` to dismiss` → ` 以关闭`

**input/terminal_message_bar.rs**
- ` attach '{}' output as agent context` → ` 附加 '{}' 输出作为智能体上下文`
- ` new conversation` → ` 新建对话`
- ` plan with agent` → ` 与智能体制定计划`
- ` to continue conversation` → ` 继续对话`
- ` new /agent conversation` → ` 新建 /agent 对话`

**input/inline_history/search_item.rs**
- `Conversation: {title}` → `对话：{title}`
- `Command: {command}` → `命令：{command}`
- `AI prompt: {query_text}` → `AI 提示词：{query_text}`

---

### 模块 12：代码编辑器 gutter 按钮（code/editor/element/gutter_button.rs）

- `Add diff hunk as context` → `将差异块附加为上下文`
- `Save changes to attach as context.` → `保存更改以附加为上下文。`
- `Add comment on line` → `在此行添加评论`
- `Save changes to add comment` → `保存更改以添加评论`
- `Show saved comment` → `显示已保存的评论`

---

### 模块 13：代码审查评论列表（code_review/comment_list_view.rs）

- `1 Comment` → `1 条评论`
- `CLI agent` → `CLI 智能体`
- `Send diff comments to {label}` → `将差异评论发送至 {label}`
- `Send to Agent` → `发送给智能体`
- `Copy text` → `复制文本`

---

### 模块 14：AI 助手面板（ai_assistant/panel.rs）

- `Shift + ctrl + space a block or text selection to ask Warp AI.` → `按 Shift + Ctrl + Space 选中块或文字以向 Warp AI 提问。`
- `How do I find all files containing specific text?` → `如何查找包含特定文字的所有文件？`
- `Copy transcript to clipboard` → `复制对话记录到剪贴板`

---

### 模块 15：CLI 智能体插件安装（terminal/cli_agent_sessions/plugin_manager/）

以下文件中包含安装引导步骤文字，出现在插件安装对话框中：

**claude.rs**
- `Install Warp Plugin for Claude Code` → `安装 Warp Claude Code 插件`
- `Ensure that jq is installed on your machine. Then, run these commands.` → `请确保系统已安装 jq，然后运行以下命令。`
- `Add the Warp plugin marketplace repository` → `添加 Warp 插件市场仓库`
- `Install the Warp plugin` → `安装 Warp 插件`
- `Update Warp Plugin for Claude Code` → `更新 Warp Claude Code 插件`

**codex.rs**
- `Enable Warp Notifications for Codex` → `为 Codex 启用 Warp 通知`
- `Update Codex to the latest version, then enable in-focus notifications so Warp can display them while you work.` → `更新 Codex 至最新版本，然后启用前台通知，以便 Warp 在您工作时显示通知。`
- `Set the notification condition to "always" in your Codex config.` → `在 Codex 配置中将通知条件设置为 "always"。`

**gemini.rs**
- `Install Warp Plugin for Gemini CLI` → `安装 Warp Gemini CLI 插件`
- `Run the following command, then restart Gemini CLI.` → `运行以下命令，然后重启 Gemini CLI。`
- `Install the Warp extension` → `安装 Warp 扩展`
- `Update Warp Plugin for Gemini CLI` → `更新 Warp Gemini CLI 插件`

**opencode.rs**
- `Install Warp Plugin for OpenCode` → `安装 Warp OpenCode 插件`
- `Update Warp Plugin for OpenCode` → `更新 Warp OpenCode 插件`

---

### 模块 16：其他零散文件

**input_suggestions.rs**
- `Ignore this suggestion` → `忽略此建议`
- `Command suggestions.` → `命令建议。`

**launch_configs/save_modal.rs**
- `Saved successfully to ` → `已成功保存至`
- `Save Config Modal` → `保存配置`（对话框标题）

**notebooks/link.rs**
- `New session` → `新建会话`
- `Open a new terminal session in this directory` → `在此目录中打开新的终端会话`
- `Open in editor` → `在编辑器中打开`
- `No base directory` → `无基准目录`

**search/action/search_item.rs**
- `Press enter to confirm.` → `按 Enter 确认。`
- `Press enter to confirm. Use {} binding to run this action in the future.` → `按 Enter 确认。以后可使用 {} 快捷键执行此操作。`

**settings_view/show_blocks_view.rs**
- `Copy link` → `复制链接`
- `You don't have any shared blocks yet.` → `您还没有共享的块。`
- `Getting blocks...` → `正在获取块...`
- `Unshare block` → `取消共享块`

**editor/accept_autosuggestion_keybinding_view.rs**
- `Accept Autosuggestion` → `接受自动建议`
- `Change keybinding` → `更改快捷键绑定`

**context_chips/display_menu.rs**
- `Search directories...` → `搜索目录...`
- `Search branches...` → `搜索分支...`
- `Search environments...` → `搜索环境...`

**ai/blocklist/passive_suggestions/static_prompt_suggestions.rs**（智能体对话零状态建议卡片）
- `Code a feature or fix a bug in {1}` → `在 {1} 中编写功能或修复 Bug`
- `Help me create a pull request.` → `帮我创建一个 Pull Request。`
- `Help me start a new project` → `帮我启动一个新项目`
- `Help me create a new React app` → `帮我创建一个新的 React 应用`
- `Help me start a Rust project for {1}` → `帮我启动一个 {1} 的 Rust 项目`
- `Help me start a Poetry project for {1}` → `帮我启动一个 {1} 的 Poetry 项目`
- `Help me start a Django project for {1}` → `帮我启动一个 {1} 的 Django 项目`
- `Help me start a Rails app for {1}` → `帮我启动一个 {1} 的 Rails 应用`
- `Help me start a Go project for {1}` → `帮我启动一个 {1} 的 Go 项目`
- `Help me start a Swift project` → `帮我启动一个 Swift 项目`
- `Help me start a Terraform configuration` → `帮我创建一个 Terraform 配置`
- `Help me set up Prisma in this project` → `帮我在此项目中配置 Prisma`
- `Help me install dependencies for {1}.` → `帮我为 {1} 安装依赖。`
- `Help me understand resource utilization in my cluster.` → `帮我了解集群的资源使用情况。`
- `Help me inspect Kubernetes resources.` → `帮我查看 Kubernetes 资源。`
- `Help me manage running containers.` → `帮我管理运行中的容器。`
- `Help me manage Docker images.` → `帮我管理 Docker 镜像。`
- `Help me search code across files for {1}.` → `帮我在文件中搜索 {1} 相关代码。`

---

## 验收标准

1. 以上所有文件中的用户可见字符串均已替换为对应中文译文。
2. 占位符（`{}`、`{n}`、`{name}`）在翻译后保持原样，翻译不改变其位置和数量。
3. 所有翻译与术语表一致，与已翻译的 235 个文件风格统一。
4. 不引入 Rust 编译错误（字符串类型不变，仅值改变）。
5. 不破坏现有已翻译文件（仅修改新增目标文件）。
6. 更新 `README.zh-CN.md` 进度清单，标记新增的已翻译模块。
