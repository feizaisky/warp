# 文件预览面板标签化（File Preview Tabs）设计

- 状态：草稿
- 日期：2026-05-08
- 分支：zh-cn
- 范围：Warp 客户端 UI（文件型 pane 显示行为）

## 背景与动机

当前在 Warp 中，从任何入口（左侧文件浏览器、AI 会话点击文件引用、命令输出点击文件路径等）打开文件，最终都会调用 `Workspace::open_file_with_target` → `open_code`，把一个 `FilePane`（Markdown 等）或 `CodePane`（代码）作为新节点向右 split 进 `PaneGroup`。

当用户连续打开多个文件时，右侧会出现多个并排的预览面板，宽度被等分压缩，标题被截断，可读性迅速下降。Warp 现有横向 tab 栏不解决该问题（也无溢出滚动），垂直 tab 栏改的是 terminal tab 而非 pane 内文件视图。

## 目标

让"打开多个文件"的体验接近浏览器多 tab：单文件时与现状一致；第二个文件打开时把已打开的文件自动收纳进一个共享的标签容器，主体只显示当前活跃 tab 的内容，通过 tab 切换。

## 非目标

- 不改变终端 pane（`TerminalPane`）的行为
- 不改变 Warp 顶部"terminal tab"或"vertical tabs"侧边栏
- 不引入文件编辑能力变化（沿用现有 FilePane/CodePane 的可读/可编辑性）
- 第一版不做"拖 tab 出去 split 成独立 pane"（保留扩展空间）
- 不做未保存修改 dirty 标记（取决于底层 pane 的编辑能力，留待后续）

## 行为契约

**触发**：任何路径打开文件时，包括但不限于：
- 文件浏览器（`CodeSource::FileTree`）
- AI 会话点击文件引用（`CodeSource::Link`）
- 命令输出点击文件路径
- 任何其他最终调用 `open_file_with_target` / `open_code` 的入口

**核心规则**：

1. **当前 terminal tab 没有任何文件 pane**：在 `PaneGroup` 中向右 split 创建普通 `CodePane` / `FilePane`（行为不变）
2. **目标文件已经在某个 pane 里打开**：聚焦该 pane（无论它在标签容器内还是普通 pane）；若在容器内，同时把对应 tab 切换为活跃
3. **目标文件未打开**：
   - 若当前 terminal tab 已有标签容器——追加 tab 到末尾并激活
   - 若当前 terminal tab 没有容器但有**单个**文件 pane——**就地升级**该 pane 为 `TabbedFilePane`（原文件 = tab 1，新文件 = tab 2，新文件激活）
   - 若当前 terminal tab 没有容器但有**多个**文件 pane（用户曾手动 split）——只升级**最近聚焦**的那个文件 pane 为容器，其他文件 pane 保持原样（避免破坏用户主动建立的并排布局）；该容器后续接收追加
   - 若当前 terminal tab 完全没有文件 pane——创建新的普通 `CodePane` / `FilePane`（同规则 1）
4. **关闭 tab**：
   - 剩余 ≥ 2 → 保持容器形态，激活相邻 tab
   - 剩余 1 → **降级**回普通 `CodePane` / `FilePane`，去掉 tab bar
   - 剩余 0 → 容器销毁，按原 `PaneEvent::Close` 路径处理

**位置策略**：
- 全局每个 terminal tab 至多一个文件标签容器
- 容器位置 = 第一个文件首次打开时的 split 位置（默认右侧）
- 容器升级后，用户可手动拖动该 pane 改变其在 PaneGroup 树中的位置；新打开的文件继续进同一容器

**作用域排除**：
- 终端 pane（`TerminalPane`）不参与
- 仅文件类 pane（`FilePane` / `CodePane`）适用

## 设置项

`Settings → Appearance → Tabs → "Group opened files into tabs"`

- 默认：ON
- 关闭后退回原行为（每次打开文件都新 split），与现状完全一致
- 切换 ON 时，**已分散**的现有 pane 不自动合并；仅新打开文件走新规则
- 切换 OFF 时，**已存在**的标签容器保留直到用户关闭；新打开文件走旧 split 路径

## Tab 操作

第一版包含：
- 点击切换活跃 tab
- 每个 tab 上 × 关闭按钮
- 拖拽排序（容器内）
- 右键菜单：关闭 / 关闭其他 / 关闭全部 / 在新 pane 中打开（脱离容器，作为独立 split） / 在系统中显示
- Tab 溢出时横向滚动

第一版不含（留作后续迭代）：
- 拖 tab 出容器形成独立 split
- 鼠标中键关闭
- 未保存修改 dirty 标记

## 架构

### 新增类型

| 类型 | 位置 | 职责 |
|---|---|---|
| `TabbedFilePane` | `app/src/pane_group/pane/tabbed_file_pane.rs` | 容器型 pane：持有一组子 `FilePane`/`CodePane`、活跃索引、tab bar UI、tab 操作事件处理 |
| `FileTab` | 同上 | 单个 tab 元数据：底层 pane handle、显示标题、文件路径 |
| `FilePaneRouter` | `app/src/workspace/file_pane_router.rs` | 纯函数式路由决策模块 |
| `RouteAction` 枚举 | 同上 | `FocusExisting { pane_id, tab_idx: Option<usize> }` / `AppendToContainer { container_id }` / `PromoteToContainer { lone_pane_id }` / `CreateNewSplitPane` |

### 修改的现有类型

| 类型 | 改动 |
|---|---|
| `Pane` 枚举（`app/src/pane_group/pane/mod.rs`） | 新增变体 `Tabbed(TabbedFilePane)`；`PaneGroup` 的 layout/resize/drag 逻辑将其视为普通 leaf |
| `Workspace::open_file_with_target` / `open_code` | 在创建底层 pane 之前先调用 `FilePaneRouter::route`，按 `RouteAction` 决定后续动作 |
| `OpenedFilesModel`（`app/src/code/opened_files.rs`） | 增加 `find_pane_for(path) -> Option<(PaneId, Option<TabIndex>)>`；维护文件→（pane_id, tab_idx）索引，在 push/remove/promote/degrade 时同步更新 |
| Tabs settings（`app/src/settings/...`） | 新增字段 `group_opened_files_into_tabs: bool`，默认 `true`，绑定到 Settings UI |

### 设计决策

**为什么 `Pane` 枚举新变体而不是 `PaneGroup` 节点新形态？**

- `PaneGroup` 的 split 树语义是"空间划分"（多个 pane 同时可见）；tab 是"时间复用"（同一空间多内容轮换）。把 tab 做成节点会污染 split 树语义。
- `TabbedFilePane` 在布局里就是一个矩形，与普通 `CodePane` 等价，作为 leaf 嵌入最自然。
- 改动面小：`PaneGroup` 树代码不动，resize / drag / close 等已有逻辑自动适用。
- 后续若需"拖 tab 出去 split"，可直接抽出子 pane 再 split 进 PaneGroup。

**为什么路由放新模块而不是塞进 `Workspace`？**

- `Workspace::view` 已经较大，不该再耦合决策逻辑
- 路由是纯函数（输入：当前 workspace 状态 + 设置 + 待打开文件路径 → 输出：`RouteAction`），单元测试容易写
- 未来扩展（如"按文件类型分容器"）改动隔离

## 数据流

### 打开文件

```
入口（FileTreeView / AIBlock / 命令输出 click 等）
    │ Event::OpenFileWithTarget(path, target, source)
    ▼
Workspace::open_file_with_target
    │ 调用 FilePaneRouter::route(workspace_state, path, settings)
    ▼
RouteAction
    │
    ├─ FocusExisting(pane_id, tab_idx?) ──► focus_pane + 若 tab_idx.is_some() 则 set_active_tab
    ├─ AppendToContainer(container_id) ───► load FilePane/CodePane → container.push_tab(pane) + activate
    ├─ PromoteToContainer(lone_pane_id) ──► load 新 pane → 用 TabbedFilePane::new([old, new], active=1) 替换 PaneGroup 中原 leaf；当存在多个 lone 文件 pane 时，`lone_pane_id` 取最近聚焦者
    └─ CreateNewSplitPane ────────────────► 旧路径不变（向右 split 一个 leaf 进 PaneGroup）
```

### 关闭 tab / 降级 / 销毁

```
TabbedFilePane 内子 pane 发出 PaneEvent::Close(child_pane_id)
    │
    ▼
TabbedFilePane::handle_close
    │ 移除 tab，重新计算活跃索引
    ▼
match remaining count:
    >= 2 ──► 激活相邻 tab，容器形态保留
       1 ──► 发出 TabbedFilePaneEvent::Degrade(remaining_pane)
                ▼ Workspace 用 remaining_pane 替换 PaneGroup 中的容器节点（OpenedFilesModel 同步更新索引）
       0 ──► 发出 PaneEvent::Close 给 PaneGroup（按现有销毁路径处理）
```

### `OpenedFilesModel` 一致性

`Workspace` 在以下时机调用 `OpenedFilesModel` 维护索引：
- 创建新 leaf pane：`add(path, pane_id, tab_idx=None)`
- 升级容器：旧 leaf 的索引更新为 `(container_id, Some(0))`，新文件 `add` 为 `(container_id, Some(1))`
- 追加 tab：`add(path, container_id, Some(new_idx))`
- tab 拖拽排序：批量更新 tab_idx
- 关闭 tab / 降级 / 销毁：`remove(path)` 或 `update(path, new_pane_id)`

## 错误处理

| 场景 | 处理 |
|---|---|
| 路由阶段读不到文件（路径不存在 / 权限） | 走原有 `open_code` 错误流程；不创建 pane，不修改容器 |
| `PromoteToContainer` 时原 lone pane 状态损坏（rare） | 记录 error，降级回 `CreateNewSplitPane` |
| 设置 OFF 时已存在容器 | 容器保留至用户主动关闭，新打开走旧 split 路径 |
| 设置 ON 切换瞬间已分散的 pane | 不自动合并；仅对后续打开生效（避免布局突变） |
| Tab bar 渲染时子 pane handle 失效 | 自动清理该 tab，活跃索引夹紧到合法范围 |

## 测试策略

### 单元测试

1. **`FilePaneRouter::tests`**（纯函数）
   - 设置 ON/OFF × 文件已开/未开 × workspace 状态（无文件 pane / 有 lone pane / 有容器） 共 ~12 个用例覆盖 4 种 `RouteAction`
2. **`TabbedFilePane::tests`**
   - `push_tab` / `remove_tab` / `set_active` 正确性
   - 关闭到剩 1 触发 `Degrade` 事件
   - 关闭到剩 0 触发 `Close` 事件
   - 活跃索引在 remove 后正确夹紧

### 集成测试（`crates/integration`，复用 Builder/TestStep）

- 打开 1 个文件 → 普通 pane（无 tab bar）
- 打开第 2 个不同文件 → 容器形态、2 个 tab、第 2 个激活
- 重复打开同一文件 → 焦点切到已存在 tab，无新 tab 添加
- 关闭 1 个 tab（剩 2）→ 容器保留，相邻 tab 激活
- 关闭到剩 1 → 自动降级为普通 pane，tab bar 消失
- 关闭最后 1 → 容器消失
- 设置切换 OFF 后打开新文件 → 走新 split，不进容器
- 文件浏览器与 AI block 来源混用 → 进同一容器（设置 ON 时）

### 快照 / 视觉测试（如该 crate 现有此能力）

- Tab bar 默认渲染
- 活跃 tab 高亮
- Tab 溢出时横向滚动
- 主题适配

## 遥测

- `file_tab_container_created`：容器首次形成
- `file_tab_added`：新 tab 追加（区分来源 source）
- `file_tab_closed`：tab 关闭（标注是否触发降级 / 销毁）
- `file_tab_grouping_setting_toggled`：设置开关变更（区分 on/off）

## 影响面与回滚

- 行为变更受设置项 `group_opened_files_into_tabs` 保护，关闭后回归现状
- `Pane` 枚举新增变体属于增量改动，不破坏现有匹配（在所有 `match` 处增加分支）
- 若需紧急回滚，可通过远程配置或 feature flag 关闭默认值，无需代码 revert

## 后续迭代

- 拖 tab 出容器形成独立 split
- 鼠标中键关闭 tab
- 未保存编辑 dirty 标记
- 跨 terminal tab 共享标签容器（如有需要）
- Tab 顺序与活跃 tab 持久化到工作区会话恢复
