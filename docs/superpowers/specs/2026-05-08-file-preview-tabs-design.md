# 文件预览面板标签化（File Preview Tabs）设计

- 状态：草稿（修订 v2）
- 日期：2026-05-08
- 分支：zh-cn
- 范围：Warp 客户端 UI（文件型 pane 显示行为）

## 背景与动机

当前 Warp 中，从任何入口（左侧文件浏览器、AI 会话点击文件引用、命令输出点击文件路径等）打开文件，最终都会调用 `Workspace::open_file_with_target` → `open_code` 或 `open_file_notebook`，把一个 `CodePane`（代码）或 `FilePane`（Markdown 等）作为新节点向右 split 进 `PaneGroup`。连续打开多个文件时，右侧出现多个并排的预览面板，宽度被等分压缩，标题被截断，可读性迅速下降。

代码库中已存在 `FeatureFlag::TabbedEditorView` + `code.editor.prefer_tabbed_editor_view` 设置，其配套实现利用 `CodeView`（`app/src/code/view.rs`）作为代码文件的多 tab 容器。但该能力存在三个不足：

1. 被 feature flag 门禁锁住，非默认构建不可用
2. 只覆盖代码文件，Markdown 预览（`FilePane` / `FileNotebookView`）仍走独立 pane
3. 设置项位于 External Editor 页，语义与用户心智模型不匹配；也没有集成测试

## 目标

让"打开多个文件"的体验接近浏览器多 tab：单文件时与现状一致；第二个文件打开时把已打开的文件自动收纳进一个共享的标签容器，主体只显示当前活跃 tab 的内容，通过 tab 切换。覆盖**代码文件和 Markdown 文件**两类。

## 非目标

- 不改变终端 pane（`TerminalPane`）的行为
- 不改变 Warp 顶部"terminal tab"或"vertical tabs"侧边栏
- 不改变已有 `CodeView` 作为代码文件容器的基础行为；仅扩展其内容模型
- 第一版不做"拖 tab 出去 split 成独立 pane"（已有 `remove_tab_for_move` 支撑拖拽到 workspace tab 的路径，但拖出成 split 不做）
- 不做未保存修改 dirty 标记
- 不做保存/恢复 tab 顺序到工作区会话（留待后续）

## 行为契约

**触发**：任何路径打开文件时，包括但不限于：
- 文件浏览器（`CodeSource::FileTree`）
- AI 会话点击文件引用（`CodeSource::Link`）
- 命令输出点击文件路径
- 任何其他最终调用 `open_file_with_target` / `open_code` 的入口

**核心规则**：

1. **当前 terminal tab 没有任何文件 pane**：在 `PaneGroup` 中向右 split 创建 `CodePane`（代码文件）或 `FilePane`（Markdown 文件）（行为不变，但新 `CodePane` 本身已是容器）
2. **目标文件已经在某个容器的 tab 里打开**：聚焦该容器 pane 并激活对应 tab
3. **目标文件已经在某个独立的 `FilePane` / 非容器 `CodePane` 中打开**：聚焦该 pane（若是 lone Markdown `FilePane`，即使设置开启也不强行合并，避免破坏用户手动 split 的布局）
4. **目标文件未打开**：
   - 若当前 terminal tab 已有 `CodePane`（已是容器）——把新文件作为新 tab 加入（代码文件：追加 code tab；Markdown 文件：追加 markdown tab）并激活
   - 若当前 terminal tab 没有容器但有**单个**文件 pane（可能是 `FilePane` Markdown 或不在 tabbed 模式下打开的 `CodePane`）——**就地升级**为容器：
     - 若原 pane 是 `CodePane`，直接在其内部追加 tab
     - 若原 pane 是 `FilePane` Markdown，替换为新建的 `CodePane` 容器，把原 Markdown 作为 tab 1、新文件作为 tab 2
   - 若当前 terminal tab 没有容器但有**多个**文件 pane——只升级**最近聚焦**的那个文件 pane 为容器；其他文件 pane 保持原样；该容器后续接收追加
   - 若当前 terminal tab 完全没有文件 pane——创建新容器 `CodePane`（内含 1 个 tab）
5. **关闭 tab**：
   - 剩余 ≥ 2 → 保持容器形态，激活相邻 tab（走 `CodeView::remove_tab_for_move` 或等价路径）
   - 剩余 1 → 容器形态保留（tab bar 可隐藏也可保留，视 UI 策略），不做强制降级为 `FilePane`（简化实现；用户关掉最后一个 tab 后容器销毁即可）
   - 剩余 0 → 容器销毁，按 `PaneEvent::Close` 路径处理

**位置策略**：
- 每个 terminal tab 默认只创建一个文件容器；后续文件打开都进同一容器
- 容器位置 = 第一个文件首次打开时的 split 位置（默认右侧）
- 容器升级后，用户可手动拖动该 pane 改变其在 PaneGroup 树中的位置；新打开的文件继续进同一容器
- 用户通过"在新 pane 中打开"操作显式创建的独立 pane 不与容器合并

**作用域排除**：
- 终端 pane（`TerminalPane`）不参与
- 仅文件类内容（代码 / Markdown）适用

## 设置项

**新增** `appearance.tabs.group_opened_files_into_tabs: bool`，默认 `true`

- 位置：`Settings → Appearance → Tabs → "Group opened files into tabs"`
- 关闭后退回"每次新 split"行为，完整复原现状
- 切换 ON 时，**已分散**的现有 pane 不自动合并；仅新打开文件走新规则
- 切换 OFF 时，**已存在**的容器保留至用户主动关闭；新打开文件走独立 split 路径

**迁移旧设置** `code.editor.prefer_tabbed_editor_view`：
- 首次启动检测旧字段值，迁移到新字段（旧字段值为 false → 新字段也 false；否则新字段为 true）
- 旧字段读取入口仍保留一个版本，但 UI 仅暴露新字段
- External Editor 设置页移除"Tabbed editor view"开关（已由新位置取代）
- `FeatureFlag::TabbedEditorView` 的门禁移除（默认行为始终启用容器逻辑，实际是否合并由新设置项控制）

## Tab 操作

第一版包含：
- 点击切换活跃 tab
- 每个 tab 上 × 关闭按钮
- 拖拽排序（容器内 tab 间）
- 右键菜单：关闭 / 关闭其他 / 关闭全部 / 在新 pane 中打开（脱离容器，作为独立 split） / 在系统中显示
- Tab 溢出时横向滚动

第一版不含（留作后续迭代）：
- 拖 tab 出容器形成独立 split
- 鼠标中键关闭
- 未保存修改 dirty 标记

## 架构

### 核心复用：扩展现有 `CodeView` 为异构内容容器

不新建容器类型。将 `CodeView`（`app/src/code/view.rs`）的 tab 内容从"仅代码"扩展为"代码 + Markdown"异构。这是本次最大的结构改动。

#### 新增/修改的类型

| 类型 | 位置 | 改动 |
|---|---|---|
| `TabContent` 枚举 | `app/src/code/view.rs`（新增） | `Code(CodeEditorState)` / `Markdown(ViewHandle<FileNotebookView>)` 两种 tab 内容形态 |
| `CodeView` 内部 tab 数据结构 | `app/src/code/view.rs`（修改） | 原先 tab 存储类型替换为 `Vec<TabContent>`（保留原有元数据字段如路径、活跃索引） |
| `CodeView` 渲染方法 | 同上（修改） | 按活跃 tab 的 `TabContent` 类型分发 body 渲染；header 的 `MarkdownToggleView` 仅当活跃 tab 为 Markdown 时显示 |
| `CodeView::open_or_focus_existing` | 同上（修改） | 接受可选 `target: FileTarget` 参数，根据 target 决定创建 `TabContent::Code` 还是 `TabContent::Markdown` |
| `FilePaneRouter` 模块 | `app/src/workspace/file_pane_router.rs`（新增） | 纯函数式路由决策，输出 `RouteAction` |
| `RouteAction` 枚举 | 同上 | `FocusExistingTab { code_pane_id, tab_idx }` / `FocusExistingLone { pane_id }` / `AppendToContainer { container_id, target }` / `PromoteLoneToContainer { lone_pane_id, target }` / `CreateNewContainer { target }` |

#### 路由决策伪码

```rust
fn route(ws: &WorkspaceState, path: &Path, target: FileTarget, settings: &TabSettings) -> RouteAction {
    if !settings.group_opened_files_into_tabs {
        return RouteAction::CreateNewSplitPane; // 完全走旧路径（legacy，绕开容器）
    }
    if let Some((pane_id, tab_idx)) = ws.find_tab_for(path) {
        return RouteAction::FocusExistingTab { code_pane_id: pane_id, tab_idx };
    }
    if let Some(pane_id) = ws.find_lone_non_container_file_pane_for(path) {
        return RouteAction::FocusExistingLone { pane_id };
    }
    if let Some(container) = ws.find_focused_or_first_container() {
        return RouteAction::AppendToContainer { container_id: container, target };
    }
    if let Some(lone) = ws.find_most_recently_focused_lone_file_pane() {
        return RouteAction::PromoteLoneToContainer { lone_pane_id: lone, target };
    }
    RouteAction::CreateNewContainer { target }
}
```

#### 修改的现有类型

| 类型 | 改动 |
|---|---|
| `Workspace::open_file_with_target` / `open_code` / `open_file_notebook` | 统一入口前置调用 `FilePaneRouter::route`，按 `RouteAction` 分派；移除散落的 `if FeatureFlag::TabbedEditorView.is_enabled()` 分支 |
| `OpenedFilesModel`（`app/src/code/opened_files.rs`） | 新增方法 `find_pane_and_tab_for(path) -> Option<(PaneId, Option<TabIndex>)>`；push/remove/promote 时同步 |
| `TabSettings`（`app/src/workspace/tab_settings.rs`） | 新增字段 `group_opened_files_into_tabs`；宏定义参见下方 |
| `app/src/lib.rs` 的 `FeatureFlag::TabbedEditorView` 引用处 | 移除 feature flag 门禁，保留设置项控制 |
| `app/src/settings_view/features/external_editor.rs` | 移除该页面的 tabbed editor view 开关渲染 |

#### 为什么复用 `CodeView` 而不是新建 `TabbedFilePane`

- `CodeView` 已有完整 tab 能力：`tab_at` / `set_active_tab_index` / `open_or_focus_existing` / `remove_tab_for_move` / tab bar 渲染
- Warp 近期 Rust 代码已围绕 `CodeView` 建立 tabbed 路径（见 `workspace/view.rs:7306/11049/13577/13720`），新建平行容器会制造两套维护负担
- Markdown 进容器的最小改动是"把 tab body 内容模型泛化"，而不是"把 Markdown view 再塞进一个新容器"
- `IPaneType` 不新增变体，`PaneContent` 实现不变，PaneGroup 拖拽/resize/close 已适配

#### 为什么路由独立成模块

- `workspace/view.rs` 已经非常大，散落的 `if FeatureFlag::TabbedEditorView` 分支不宜再膨胀
- 路由是纯函数（输入：workspace 状态 + 设置 + 文件路径 + target → 输出 `RouteAction`），单元测试成本低
- 未来扩展（按文件类型分容器、按 split 分组等）改动隔离

## 数据流

### 打开文件

```
入口（FileTreeView / AIBlock / 命令输出 click 等）
    │ 发出 Event::OpenFileWithTarget(path, target, source)
    ▼
Workspace::open_file_with_target
    │ 调用 FilePaneRouter::route(ws_state, path, target, &tab_settings)
    ▼
RouteAction
    │
    ├─ FocusExistingTab { code_pane_id, tab_idx } ──►
    │   focus_pane(code_pane_id) + CodeView::set_active_tab_index(tab_idx)
    │
    ├─ FocusExistingLone { pane_id } ──►
    │   focus_pane(pane_id)
    │
    ├─ AppendToContainer { container_id, target } ──►
    │   let view = load_for_target(path, target, ctx); // code 编辑器 or FileNotebookView
    │   container.open_or_focus_existing(path, target, view)
    │
    ├─ PromoteLoneToContainer { lone_pane_id, target } ──►
    │   // 情形 A：lone 是 FilePane（Markdown 独立 pane）
    │   //   新建一个空 CodeView 容器，依次 push 原 Markdown（作为 TabContent::Markdown）和新文件
    │   //   替换 PaneGroup 中 lone leaf 为新容器
    │   // 情形 B：lone 是 CodePane（单 tab 的容器形态，即已是 CodeView）
    │   //   直接对其调用 open_or_focus_existing 追加 tab
    │
    └─ CreateNewContainer { target } ──►
        新建 CodePane（内含 1 个 TabContent），向右 split 进 PaneGroup
        // 若 target 是 Markdown：新 CodePane 内 tab 为 TabContent::Markdown
        // 若 target 是 Code：新 CodePane 内 tab 为 TabContent::Code
```

### 关闭 tab

```
CodeView 内用户点击 tab × 或触发关闭操作
    │
    ▼
CodeView::close_tab(idx)
    │ 从 Vec<TabContent> 移除第 idx 项
    ▼
match remaining count:
    >= 1 ──► 重算 active_tab_index（夹紧到合法范围），继续渲染
       0 ──► 发出 PaneEvent::Close → PaneGroup 销毁该 leaf
```

### `OpenedFilesModel` 一致性

`Workspace` 在以下时机调用 `OpenedFilesModel` 维护索引：
- 新建容器 pane：`file_opened(repo_path, file_path, ctx)`，记录 `(pane_id, tab_idx=0)`
- 追加 tab：`file_opened`，记录 `(pane_id, tab_idx=new_idx)`
- 升级 lone 为容器：旧 pane 索引更新（pane_id 改为新容器 id，tab_idx=0），新文件 `file_opened`
- tab 拖拽排序：批量更新对应文件的 tab_idx
- 关闭 tab / 销毁容器：移除对应文件记录

（注：`OpenedFilesModel` 当前模型只跟踪 `PathBuf → Instant`。本次为其新增一个并列的结构 `HashMap<PathBuf, (PaneId, Option<TabIndex>)>` 作为"打开位置"索引，不影响现有 Instant 时间戳语义。）

## Markdown tab 渲染适配

现有 `FileNotebookView` 的 header 包含 `MarkdownToggleView`（渲染/原文切换）。容器化后：

1. `MarkdownToggleView` 不再由 `FileNotebookView` 自己渲染，而是**提升到 `CodeView` 的 header**，只有活跃 tab 是 `TabContent::Markdown` 时才显示
2. 切换事件 `FileNotebookAction::ToggleMarkdownDisplayMode` 由 `CodeView` 捕获后转发给当前活跃 Markdown tab 的 `FileNotebookView`
3. 切 tab 不销毁 `FileNotebookView`，`markdown_display_mode`、滚动位置、`file_state` 全部保留
4. Markdown 相关事件（`FileNotebookEvent::OpenFileWithTarget` 等）仍正常冒泡到 `CodeView` → `PaneGroup` → `Workspace`，不改动这条事件路径

## 错误处理

| 场景 | 处理 |
|---|---|
| 路由阶段读不到文件（路径不存在 / 权限） | 走原有错误流程；不创建 pane，不修改容器 |
| `PromoteLoneToContainer` 时原 lone pane 状态损坏（rare） | 记录 error，降级回 `CreateNewContainer` |
| 设置 OFF 时已存在容器 | 容器保留至用户主动关闭，新打开走独立 split |
| 设置 ON 切换瞬间已分散的 pane | 不自动合并；仅对后续打开生效 |
| Tab bar 渲染时 `TabContent` handle 失效 | 自动清理该 tab，活跃索引夹紧到合法范围 |
| Markdown tab 的 `FileNotebookView` 加载失败 | 走原 `FileState::Error` 渲染，不影响其他 tab |
| 旧设置 `prefer_tabbed_editor_view = false` 迁移 | 新设置 `group_opened_files_into_tabs = false`；UI 显示正确 |

## 测试策略

### 单元测试

1. **`FilePaneRouter::tests`**（纯函数）
   - 设置 ON/OFF × 文件已开（在容器内 / lone pane）/ 未开 × workspace 状态（无文件 pane / 单 lone / 多 lone / 有容器）
   - 覆盖全部 `RouteAction` 分支
2. **`CodeView::tests`**（新增 TabContent 相关）
   - 追加 `TabContent::Code`、`TabContent::Markdown` 正确性
   - 切 tab 保留 Markdown 的 `markdown_display_mode` 状态
   - 关闭到剩 0 触发 `PaneEvent::Close`
   - 活跃索引在 remove 后夹紧

### 集成测试（`crates/integration`，复用 Builder/TestStep 模式，参考 `test/file_tree.rs`）

- 打开 1 个代码文件 → 容器 pane、1 个 tab、`pane_count=2`（终端 + 容器）
- 打开第 2 个代码文件 → 同容器 2 个 tab，第 2 个激活
- 打开 Markdown 文件 → 进同一容器，作为 Markdown tab；header 出现渲染/原文切换按钮
- 切换活跃 tab 从 Markdown → 代码 → Markdown：切换按钮按可见/隐藏切换；Markdown 的 `markdown_display_mode` 保留
- 重复打开同一文件 → 激活已存在 tab，不新增
- 关闭一个 tab（剩多 → 容器形态保留）
- 关闭最后一个 tab → 容器消失，`pane_count=1`
- 设置 OFF 后打开新文件 → 走独立 split，不进容器
- 从文件浏览器与 AI block 混合打开 → 进同一容器

### 快照 / 视觉测试（按现有 crate 能力）

- Tab bar 默认渲染（代码 tab、Markdown tab 图标/标题差异）
- Markdown 活跃时的渲染/原文切换按钮
- Tab 溢出时横向滚动

## 遥测

- `file_tab_container_created`：容器首次形成
- `file_tab_added`：新 tab 追加（区分 code/markdown + 来源 source）
- `file_tab_closed`：tab 关闭（标注是否导致容器销毁）
- `file_tab_grouping_setting_toggled`：新设置开关变更（区分 on/off）
- `prefer_tabbed_editor_view_migrated`：一次性，记录旧字段迁移到新字段的用户数

## 影响面与回滚

- 行为变更受设置项 `group_opened_files_into_tabs` 保护，关闭后回归"每次新 split"
- `FeatureFlag::TabbedEditorView` 门禁移除，这是**不可回滚的前向变化**；若需紧急下线，通过默认值调整或远程配置把新设置默认置为 false
- `CodeView` 数据结构从单一类型扩展为异构 `TabContent`，存量用法需全量覆盖（编译期强制）
- 设置 UI 位置从 External Editor 迁到 Appearance → Tabs，需同步更新文档与截图

## 后续迭代

- 拖 tab 出容器形成独立 split
- 鼠标中键关闭 tab
- 未保存编辑 dirty 标记
- 跨 terminal tab 共享容器（如有需要）
- Tab 顺序与活跃 tab 持久化到工作区会话恢复
- Markdown 与代码视图的 tab 图标优化
