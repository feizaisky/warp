# TECH.md — 简体中文完整汉化（zh-cn-i18n）

参见 [PRODUCT.md](./PRODUCT.md) 了解用户可见行为和完整翻译对照表。

## Context

本项目是在 `zh-cn` 分支上直接替换 Rust 源文件中的字符串字面量，无任何 i18n 框架介入。已有 235 个文件完成翻译，建立了稳定的翻译模式。

**字符串使用的典型模式（三种）：**

1. **Action label**（出现在命令面板、菜单、快捷键提示）：
   ```rust
   FixedBinding::custom(CustomAction::X, WorkspaceAction::X, "Switch to next tab", ...)
   ```

2. **Struct field**（description、label 等字段直接赋值）：
   ```rust
   StaticCommand { description: "Start a new conversation", ... }
   ```

3. **函数调用/格式化**（`.text()`、`format!()`、`.to_string()` 等）：
   ```rust
   cx.new_view(|_| Label::new("No results found"))
   format!("Searched for \"{}\" in {}", query, path)
   ```

**格式占位符**：Rust 使用 `{}`、`{name}`、`{n}` 形式，翻译时原样保留。

**相关文件**：分布在 `app/src/` 下，见 PRODUCT.md 各模块。

## Proposed Changes

按 PRODUCT.md 的 16 个模块分 4 批实施，每批独立可编译。

### 批次划分

| 批次 | 模块 | 文件数 |
|------|------|--------|
| A | 模块 1-2：workspace/mod.rs、root_view.rs | 2 |
| B | 模块 3-5：editor/view/mod.rs、code/editor/view/actions.rs、search/slash_command_menu/... | 3 |
| C | 模块 6-10：shared_session/network.rs、diff_state.rs、settings_view 相关、env_vars 相关 | 8 |
| D | 模块 11-16：terminal/input 相关、AI 内联视图、plugin_manager、其他零散 | ~20 |

### 翻译规则（所有批次通用）

1. 只改字符串字面量的值，不改变量名、类型、结构。
2. 带占位符的字符串（`{}`、`{name}` 等）：只翻译周围的文字，占位符位置和数量不变。
3. 调试/测试专用字符串（如 `Crash the app (for testing...)`、`Dump debug info`）：保留英文。
4. 技术专有名词（`Docker`、`jq`、`TOML`、`Gemini CLI` 等）：保留英文。
5. `&str` 字面量和 `String` 均适用，处理方式相同。

### README 更新

翻译完成后更新 `README.zh-CN.md` 的进度清单，将新增模块标记为 `[x]`。

## Testing and Validation

**编译验证**（覆盖 PRODUCT.md 验收标准 4）：
```bash
cargo check -p warpui 2>&1 | head -40
```
每批翻译后执行，确保无编译错误后再继续下一批。

**字符串完整性检查**（覆盖验收标准 2）：
翻译前后用 grep 统计目标文件中占位符数量，确认翻译前后一致：
```bash
grep -c '{}' <file>  # 前后应相等
```

**风格一致性**（覆盖验收标准 3）：
对照 PRODUCT.md 术语表，逐模块人工核查关键术语（Agent→智能体、Tab→标签页、Pane→面板 等）。

**回归检查**（覆盖验收标准 5）：
```bash
git diff --name-only master..zh-cn | grep "app/src" | wc -l
```
确认已翻译文件数量只增不减，无已翻译文件被意外回退。

## Parallelization

四个批次可分配给独立 Agent 并行执行，互相无文件重叠。每个 Agent 完成后运行 `cargo check` 验证，通过后合入。
