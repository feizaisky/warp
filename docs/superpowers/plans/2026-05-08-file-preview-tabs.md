# File Preview Tabs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the existing `FeatureFlag::TabbedEditorView` behavior to a default-on, user-toggleable feature, extend the `CodeView` container to host both code and Markdown tabs, and consolidate the routing logic into a `FilePaneRouter` module — so opening any file routes into a single tabbed container per terminal tab.

**Architecture:**
1. Reuse `CodeView` (`app/src/code/view.rs`) as the universal file tab container; extend `TabData` with a `TabContent::{Code, Markdown}` enum so a single `CodeView` can host heterogeneous tabs.
2. Lift `MarkdownToggleView` rendering from `FileNotebookView::render_header_content` into `CodeView::render_header_content` so the Rendered/Raw control follows the active tab.
3. Centralize "where does this file open" decisions into a pure `FilePaneRouter::route()` function in `app/src/workspace/file_pane_router.rs`, replacing the three scattered `if FeatureFlag::TabbedEditorView.is_enabled()` branches in `workspace/view.rs`.
4. Migrate setting `code.editor.prefer_tabbed_editor_view` to `appearance.tabs.group_opened_files_into_tabs` and remove the feature-flag gate.

**Tech Stack:** Rust, Warp's custom UI framework (`warpui`), `define_settings_group!` macro, `crates/integration` Builder/TestStep harness for end-to-end tests.

---

## Phase 0: Investigation & guard

### Task 0: Confirm baseline before touching code

**Files:** read-only

- [ ] **Step 1: Verify current state matches design assumptions**

Run:
```bash
cd /opt/soft/warp
grep -n "FeatureFlag::TabbedEditorView" app/src/workspace/view.rs app/src/lib.rs app/src/settings_view/features/external_editor.rs
grep -n "prefer_tabbed_editor_view" app/src/util/file/external_editor/settings.rs
grep -n "tab_group: Vec<TabData>" app/src/code/view.rs
grep -n "display_mode_segmented_control" app/src/notebooks/file/mod.rs
```

Expected: 4 references in `workspace/view.rs` (around lines 7306, 11049, 13577, 13720); 1 definition in `lib.rs:2641`; settings field at `external_editor/settings.rs:109`; `tab_group` at `view.rs:226`; `display_mode_segmented_control` initialized in `notebooks/file/mod.rs:103,273`.

If any reference is missing, STOP — codebase has drifted from the spec. Re-read `docs/superpowers/specs/2026-05-08-file-preview-tabs-design.md` and adjust the plan.

- [ ] **Step 2: Build the workspace clean to catch unrelated breakage**

Run:
```bash
cd /opt/soft/warp
cargo check -p warp_app
```

Expected: clean build (no errors). If broken, fix or stash before continuing — we want a green baseline.

- [ ] **Step 3: No commit (read-only step)**

---

## Phase 1: Settings — new field, migration, UI relocation

### Task 1: Add `group_opened_files_into_tabs` to TabSettings

**Files:**
- Modify: `app/src/workspace/tab_settings.rs`

- [ ] **Step 1: Read current TabSettings to confirm macro form**

Run:
```bash
sed -n '1,80p' /opt/soft/warp/app/src/workspace/tab_settings.rs
```

Note exact `define_settings_group!` invocation pattern, especially how `bool` fields with `default`/`toml_path` are declared.

- [ ] **Step 2: Add the new field**

Inside the `define_settings_group!(TabSettings, settings: [...])` block, add (alphabetical ordering not required by the macro; place near `use_vertical_tabs`):

```rust
group_opened_files_into_tabs: GroupOpenedFilesIntoTabs {
    type: bool,
    default: true,
    toml_path: "appearance.tabs.group_opened_files_into_tabs",
},
```

- [ ] **Step 3: Build to verify the macro accepts the new field**

Run:
```bash
cd /opt/soft/warp
cargo check -p warp_app 2>&1 | head -40
```

Expected: clean build. The macro should auto-generate the accessor `tab_settings.group_opened_files_into_tabs.value()` and a `GroupOpenedFilesIntoTabs::storage_key()` const.

- [ ] **Step 4: Commit**

```bash
git add app/src/workspace/tab_settings.rs
git commit -m "feat(settings): add appearance.tabs.group_opened_files_into_tabs"
```

---

### Task 2: One-shot migration from `prefer_tabbed_editor_view`

**Files:**
- Create: `app/src/workspace/tab_settings/migration.rs`
- Modify: `app/src/workspace/tab_settings.rs` (add `mod migration;` + call site)

- [ ] **Step 1: Write the failing test**

Create `app/src/workspace/tab_settings/migration.rs`:

```rust
//! One-shot migration: prefer_tabbed_editor_view -> group_opened_files_into_tabs.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_false_to_false() {
        let mut store = MockSettingsStore::default();
        store.set_legacy_prefer_tabbed_editor_view(Some(false));
        store.set_new_group_opened_files_into_tabs(None);

        run_migration(&mut store);

        assert_eq!(store.get_new_group_opened_files_into_tabs(), Some(false));
        assert!(store.migration_marker_set());
    }

    #[test]
    fn migrates_true_to_true() {
        let mut store = MockSettingsStore::default();
        store.set_legacy_prefer_tabbed_editor_view(Some(true));
        store.set_new_group_opened_files_into_tabs(None);

        run_migration(&mut store);

        assert_eq!(store.get_new_group_opened_files_into_tabs(), Some(true));
    }

    #[test]
    fn skips_when_new_value_already_set() {
        let mut store = MockSettingsStore::default();
        store.set_legacy_prefer_tabbed_editor_view(Some(false));
        store.set_new_group_opened_files_into_tabs(Some(true));

        run_migration(&mut store);

        assert_eq!(store.get_new_group_opened_files_into_tabs(), Some(true));
    }

    #[test]
    fn idempotent_when_marker_set() {
        let mut store = MockSettingsStore::default();
        store.set_legacy_prefer_tabbed_editor_view(Some(false));
        store.set_new_group_opened_files_into_tabs(None);
        store.set_migration_marker();

        run_migration(&mut store);

        assert_eq!(store.get_new_group_opened_files_into_tabs(), None);
    }
}
```

The mock store and trait are defined alongside; see Step 3.

- [ ] **Step 2: Run the test to verify it fails**

Run:
```bash
cd /opt/soft/warp
cargo test -p warp_app workspace::tab_settings::migration 2>&1 | tail -30
```

Expected: compile errors — `run_migration` and `MockSettingsStore` not defined.

- [ ] **Step 3: Implement the migration**

Add to the same file (above the `#[cfg(test)] mod tests`):

```rust
pub trait SettingsStore {
    fn legacy_prefer_tabbed_editor_view(&self) -> Option<bool>;
    fn new_group_opened_files_into_tabs(&self) -> Option<bool>;
    fn set_new_group_opened_files_into_tabs(&mut self, value: bool);
    fn migration_marker_set(&self) -> bool;
    fn set_migration_marker(&mut self);
}

pub fn run_migration<S: SettingsStore>(store: &mut S) {
    if store.migration_marker_set() {
        return;
    }
    if store.new_group_opened_files_into_tabs().is_none() {
        if let Some(legacy) = store.legacy_prefer_tabbed_editor_view() {
            store.set_new_group_opened_files_into_tabs(legacy);
        }
    }
    store.set_migration_marker();
}

#[cfg(test)]
#[derive(Default)]
struct MockSettingsStore {
    legacy: Option<bool>,
    new_value: Option<bool>,
    marker: bool,
}

#[cfg(test)]
impl MockSettingsStore {
    fn set_legacy_prefer_tabbed_editor_view(&mut self, v: Option<bool>) { self.legacy = v; }
    fn get_new_group_opened_files_into_tabs(&self) -> Option<bool> { self.new_value }
}

#[cfg(test)]
impl SettingsStore for MockSettingsStore {
    fn legacy_prefer_tabbed_editor_view(&self) -> Option<bool> { self.legacy }
    fn new_group_opened_files_into_tabs(&self) -> Option<bool> { self.new_value }
    fn set_new_group_opened_files_into_tabs(&mut self, value: bool) { self.new_value = Some(value); }
    fn migration_marker_set(&self) -> bool { self.marker }
    fn set_migration_marker(&mut self) { self.marker = true; }
}
```

In `app/src/workspace/tab_settings.rs`, add at the top:

```rust
pub mod migration;
```

- [ ] **Step 4: Run tests to verify they pass**

Run:
```bash
cd /opt/soft/warp
cargo test -p warp_app workspace::tab_settings::migration -- --nocapture 2>&1 | tail -20
```

Expected: 4 tests pass.

- [ ] **Step 5: Wire migration into app startup**

Find the workspace settings init site:
```bash
grep -rn "TabSettings::initialize\|TabSettings::default\|setup_settings" /opt/soft/warp/app/src/ | head
```

In whichever module owns settings bootstrap (typically `app/src/lib.rs` or `app/src/settings/mod.rs`), call `migration::run_migration(...)` once after settings are loaded but before the first read. Use the actual settings store handle as the `SettingsStore` impl — implement the trait against it inline (or in `migration.rs`).

The exact wiring depends on what the engineer finds; follow existing settings-bootstrap patterns. The unit-tested `run_migration` function is the contract; the impl wiring is mechanical.

- [ ] **Step 6: Commit**

```bash
git add app/src/workspace/tab_settings.rs app/src/workspace/tab_settings/migration.rs
git commit -m "feat(settings): migrate prefer_tabbed_editor_view -> group_opened_files_into_tabs"
```

---

### Task 3: Move setting UI from External Editor page to Tabs page

**Files:**
- Modify: `app/src/settings_view/features/external_editor.rs` (remove)
- Modify: appearance/tabs settings UI (add) — locate via grep below

- [ ] **Step 1: Locate existing tabs settings UI**

Run:
```bash
grep -rn "use_vertical_tabs\|UseVerticalTabs" /opt/soft/warp/app/src/settings_view/ | head
```

Identify the file rendering the Tabs settings page (likely `app/src/settings_view/features/tabs.rs` or under `appearance/`). Read its render method to learn the row pattern.

- [ ] **Step 2: Add the new toggle row**

In the Tabs settings page, copy an existing toggle row (`use_vertical_tabs` is the closest analog) and adapt:
- Title: "Group opened files into tabs"
- Description: "When opening multiple files, group them into a single tabbed container instead of opening each in a new pane."
- Storage key: `GroupOpenedFilesIntoTabs::storage_key()`
- Sync flag: `GroupOpenedFilesIntoTabs::sync_to_cloud()`
- Action: dispatch a new `TabsAction::ToggleGroupOpenedFilesIntoTabs` (add the variant to that page's action enum, mirroring how `ToggleUseVerticalTabs` is handled)
- Mouse state handle: add `group_opened_files_into_tabs_mouse_state: SwitchStateHandle` to the page view struct, initialize via `Default::default()`

Implement the action handler analogously to `toggle_use_vertical_tabs` (read current value, flip, write).

- [ ] **Step 3: Remove the old External Editor toggle**

In `app/src/settings_view/features/external_editor.rs`:
- Delete the `if FeatureFlag::TabbedEditorView.is_enabled() { ... }` block that renders the tabbed editor toggle (around line 332).
- Delete `tabbed_editor_view_mouse_state` field, its initializer, and the `ExternalEditorAction::ToggleTabbedEditorView` arm + its handler `toggle_prefer_tabbed_editor_view`.
- Delete `PreferTabbedEditorView` from the imports.

- [ ] **Step 4: Build & visually verify**

Run:
```bash
cd /opt/soft/warp
cargo check -p warp_app
```

Expected: clean build. Then run the app (`cargo run -p warp_app`), open Settings → Appearance → Tabs, confirm the new toggle is there. Open Settings → External Editor, confirm the old toggle is gone.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(settings-ui): move tabbed-editor toggle from External Editor to Tabs"
```

---

### Task 4: Remove FeatureFlag gate (keep flag definition for now)

**Files:**
- Modify: `app/src/workspace/view.rs` (4 places)
- Modify: `app/src/settings_view/features/external_editor.rs` (already done in Task 3)

> Note: We KEEP the `FeatureFlag::TabbedEditorView` definition in `lib.rs` for one release cycle as a kill switch. Final removal happens in Task 13.

- [ ] **Step 1: Replace `FeatureFlag::TabbedEditorView.is_enabled() && ...prefer_tabbed_editor_view.value()` with the new setting**

In `app/src/workspace/view.rs`, find these four locations (line numbers are approximate; use the search above):

```rust
let grouping_on = FeatureFlag::TabbedEditorView.is_enabled()
    && *EditorSettings::as_ref(ctx).prefer_tabbed_editor_view.value();
```

Replace with:

```rust
let grouping_on = *TabSettings::as_ref(ctx)
    .group_opened_files_into_tabs
    .value();
```

Repeat for all 4 occurrences (around lines 7306, 11049, 13577, 13720). Add `use crate::workspace::tab_settings::TabSettings;` at the top if not present.

- [ ] **Step 2: Verify EditorSettings::prefer_tabbed_editor_view is still readable**

We are not deleting the legacy field yet (needed for migration to read it). Just stop using it for routing. Run:
```bash
cd /opt/soft/warp
grep -n "prefer_tabbed_editor_view" app/src/
```

Expected: only references inside the migration module + the legacy settings struct field itself. No references in `workspace/view.rs` or `external_editor.rs` (UI removed in Task 3).

- [ ] **Step 3: Build**

```bash
cd /opt/soft/warp
cargo check -p warp_app
```

Expected: clean build.

- [ ] **Step 4: Commit**

```bash
git add app/src/workspace/view.rs
git commit -m "refactor(workspace): drive grouping from TabSettings instead of FeatureFlag"
```

---

## Phase 2: FilePaneRouter — pure routing module

### Task 5: Define `RouteAction` enum and `RouteContext` trait

**Files:**
- Create: `app/src/workspace/file_pane_router.rs`
- Modify: `app/src/workspace/mod.rs` (add `pub mod file_pane_router;`)

- [ ] **Step 1: Write the failing test**

```rust
//! Pure routing decisions for "open file" requests.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileKind {
    Code,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteAction {
    /// Focus the existing tab inside a CodeView container.
    FocusExistingTab { container_pane_id: u64, tab_index: usize },
    /// Focus an existing lone (non-container) file pane that already shows the path.
    FocusExistingLone { pane_id: u64 },
    /// Append the file as a new tab into an existing container.
    AppendToContainer { container_pane_id: u64, kind: FileKind },
    /// Promote a lone file pane (Markdown FilePane or single-tab CodePane) into a
    /// container that holds both the original content and the new file.
    PromoteLoneToContainer { lone_pane_id: u64, kind: FileKind },
    /// Create a brand new CodeView container with one tab.
    CreateNewContainer { kind: FileKind },
    /// Bypass the container logic entirely (setting OFF). Falls back to legacy split.
    CreateNewSplitPane { kind: FileKind },
}

pub trait RouteContext {
    fn grouping_enabled(&self) -> bool;
    fn find_tab_for(&self, path: &Path) -> Option<(u64, usize)>;
    fn find_lone_pane_for(&self, path: &Path) -> Option<u64>;
    fn find_focused_or_first_container(&self) -> Option<u64>;
    fn find_most_recently_focused_lone_file_pane(&self) -> Option<u64>;
}

pub fn route(ctx: &dyn RouteContext, path: &Path, kind: FileKind) -> RouteAction {
    if !ctx.grouping_enabled() {
        return RouteAction::CreateNewSplitPane { kind };
    }
    if let Some((container, idx)) = ctx.find_tab_for(path) {
        return RouteAction::FocusExistingTab { container_pane_id: container, tab_index: idx };
    }
    if let Some(pid) = ctx.find_lone_pane_for(path) {
        return RouteAction::FocusExistingLone { pane_id: pid };
    }
    if let Some(c) = ctx.find_focused_or_first_container() {
        return RouteAction::AppendToContainer { container_pane_id: c, kind };
    }
    if let Some(lone) = ctx.find_most_recently_focused_lone_file_pane() {
        return RouteAction::PromoteLoneToContainer { lone_pane_id: lone, kind };
    }
    RouteAction::CreateNewContainer { kind }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[derive(Default)]
    struct StubCtx {
        grouping: bool,
        existing_tab: Option<(u64, usize)>,
        existing_lone: Option<u64>,
        container: Option<u64>,
        recent_lone: Option<u64>,
    }

    impl RouteContext for StubCtx {
        fn grouping_enabled(&self) -> bool { self.grouping }
        fn find_tab_for(&self, _: &Path) -> Option<(u64, usize)> { self.existing_tab }
        fn find_lone_pane_for(&self, _: &Path) -> Option<u64> { self.existing_lone }
        fn find_focused_or_first_container(&self) -> Option<u64> { self.container }
        fn find_most_recently_focused_lone_file_pane(&self) -> Option<u64> { self.recent_lone }
    }

    fn p() -> PathBuf { PathBuf::from("/tmp/foo.md") }

    #[test]
    fn off_creates_split_pane() {
        let r = route(&StubCtx { grouping: false, ..Default::default() }, &p(), FileKind::Markdown);
        assert_eq!(r, RouteAction::CreateNewSplitPane { kind: FileKind::Markdown });
    }

    #[test]
    fn focus_existing_tab_takes_priority() {
        let ctx = StubCtx {
            grouping: true,
            existing_tab: Some((42, 3)),
            existing_lone: Some(99),
            container: Some(7),
            ..Default::default()
        };
        assert_eq!(
            route(&ctx, &p(), FileKind::Code),
            RouteAction::FocusExistingTab { container_pane_id: 42, tab_index: 3 }
        );
    }

    #[test]
    fn focus_existing_lone_when_no_tab_match() {
        let ctx = StubCtx {
            grouping: true,
            existing_lone: Some(99),
            container: Some(7),
            ..Default::default()
        };
        assert_eq!(
            route(&ctx, &p(), FileKind::Code),
            RouteAction::FocusExistingLone { pane_id: 99 }
        );
    }

    #[test]
    fn append_when_container_exists() {
        let ctx = StubCtx { grouping: true, container: Some(7), ..Default::default() };
        assert_eq!(
            route(&ctx, &p(), FileKind::Markdown),
            RouteAction::AppendToContainer { container_pane_id: 7, kind: FileKind::Markdown }
        );
    }

    #[test]
    fn promote_lone_when_no_container() {
        let ctx = StubCtx { grouping: true, recent_lone: Some(11), ..Default::default() };
        assert_eq!(
            route(&ctx, &p(), FileKind::Code),
            RouteAction::PromoteLoneToContainer { lone_pane_id: 11, kind: FileKind::Code }
        );
    }

    #[test]
    fn create_new_when_workspace_empty() {
        let ctx = StubCtx { grouping: true, ..Default::default() };
        assert_eq!(
            route(&ctx, &p(), FileKind::Code),
            RouteAction::CreateNewContainer { kind: FileKind::Code }
        );
    }
}
```

- [ ] **Step 2: Run the test to verify it fails (file doesn't exist yet)**

Run:
```bash
cd /opt/soft/warp
cargo test -p warp_app workspace::file_pane_router 2>&1 | tail -10
```

Expected: error E0583 "file not found for module".

- [ ] **Step 3: Make tests compile by registering the module**

In `app/src/workspace/mod.rs`, add:
```rust
pub mod file_pane_router;
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /opt/soft/warp
cargo test -p warp_app workspace::file_pane_router -- --nocapture 2>&1 | tail -20
```

Expected: 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add app/src/workspace/file_pane_router.rs app/src/workspace/mod.rs
git commit -m "feat(workspace): add FilePaneRouter pure-function routing module"
```

---

### Task 6: Implement `RouteContext` against the real workspace state

**Files:**
- Modify: `app/src/workspace/file_pane_router.rs` (add adapter struct)
- Modify: `app/src/workspace/view.rs` (add helper `current_route_context()`)

- [ ] **Step 1: Add the workspace adapter**

At the bottom of `file_pane_router.rs` (outside `#[cfg(test)]`), add a real-world adapter:

```rust
#[cfg(feature = "local_fs")]
pub struct WorkspaceRouteContext<'a> {
    pub grouping: bool,
    pub active_pane_group: &'a crate::pane_group::PaneGroup,
    pub opened_files: &'a crate::code::opened_files::OpenedFilesModel,
    pub app_ctx: &'a warpui::AppContext,
}

#[cfg(feature = "local_fs")]
impl<'a> RouteContext for WorkspaceRouteContext<'a> {
    fn grouping_enabled(&self) -> bool { self.grouping }

    fn find_tab_for(&self, path: &Path) -> Option<(u64, usize)> {
        // Walk every CodeView in active_pane_group; find tab matching path.
        // Return (PaneId.creation_order_id().as_u64(), tab_idx).
        // Implementation reads CodeView::tab_at iteratively.
        for (pane_id, code_view) in self.active_pane_group.code_panes(self.app_ctx) {
            let view = code_view.read(self.app_ctx, |v, _| v.clone_tab_paths_for_router());
            if let Some(idx) = view.iter().position(|p| p.as_deref() == Some(path)) {
                return Some((pane_id_to_u64(pane_id), idx));
            }
        }
        None
    }

    fn find_lone_pane_for(&self, path: &Path) -> Option<u64> {
        // Scan FilePane (Markdown) panes for the path. CodePane is always a container in the new model.
        // Iterate active_pane_group's file_panes; FilePane::file_view->location matches.
        // Return its PaneId as u64 if found.
        unimplemented!("see Step 2 for a step-by-step")
    }

    fn find_focused_or_first_container(&self) -> Option<u64> {
        // Pick the focused CodePane if any, else the first visible CodePane.
        let focused = self.active_pane_group.active_pane_id();
        if focused.is_some_and(|p| p.is_code_pane()) {
            return focused.map(pane_id_to_u64);
        }
        self.active_pane_group
            .code_panes(self.app_ctx)
            .next()
            .map(|(pid, _)| pane_id_to_u64(pid))
    }

    fn find_most_recently_focused_lone_file_pane(&self) -> Option<u64> {
        // Among FilePane (Markdown) panes, return the most recently focused.
        // Use PaneGroup's existing focus history (whatever API it exposes; see view.rs).
        unimplemented!("see Step 2")
    }
}

fn pane_id_to_u64(p: crate::pane_group::pane::PaneId) -> u64 {
    // Use creation_order_id().as_u64() or whatever stable u64 representation exists.
    // The actual call is determined by reading PaneId's API.
    p.creation_order_id().as_u64()
}
```

- [ ] **Step 2: Implement the `unimplemented!()` bodies**

The engineer must read the existing `PaneGroup` API to fill in:
- `find_lone_pane_for(path)`: iterate `PaneGroup.file_panes()` (Markdown FilePane) and compare `FileNotebookView.location` paths.
- `find_most_recently_focused_lone_file_pane()`: the workspace already tracks recent focus order; reuse that. If no API exists, scan panes ordered by `PaneId::creation_order_id()` descending and return the first FilePane.

Add a helper on `CodeView`:
```rust
pub fn clone_tab_paths_for_router(&self) -> Vec<Option<PathBuf>> {
    self.tab_group.iter().map(|t| t.path()).collect()
}
```

- [ ] **Step 3: Build to verify**

```bash
cd /opt/soft/warp
cargo check -p warp_app
```

Expected: clean build.

- [ ] **Step 4: Commit**

```bash
git add app/src/workspace/file_pane_router.rs app/src/code/view.rs
git commit -m "feat(workspace): wire RouteContext against PaneGroup state"
```

---

## Phase 3: Heterogeneous TabContent in CodeView

> This is the highest-risk phase: it changes `CodeView`'s internal tab model. We do it in small TDD steps. After each step, ensure `cargo check -p warp_app` is clean.

### Task 7: Introduce `TabContent` enum (Code-only variant first)

**Files:**
- Modify: `app/src/code/view.rs`

- [ ] **Step 1: Add the enum alongside existing TabData**

At the top of `view.rs` near the `TabData` definition (line ~204), add:

```rust
#[derive(Clone)]
pub enum TabContent {
    Code(ViewHandle<LocalCodeEditorView>),
    #[cfg(feature = "local_fs")]
    Markdown(ViewHandle<crate::notebooks::file::FileNotebookView>),
}

impl TabContent {
    pub fn is_markdown(&self) -> bool {
        #[cfg(feature = "local_fs")]
        return matches!(self, TabContent::Markdown(_));
        #[cfg(not(feature = "local_fs"))]
        return false;
    }
}
```

- [ ] **Step 2: Refactor `TabData` to wrap a `TabContent` instead of a bare `editor_view`**

```rust
#[derive(Clone)]
pub struct TabData {
    path: Option<PathBuf>,
    content: TabContent,
    mouse_state_handles: TabDataMouseStateHandles,
    preview: bool,
}

impl TabData {
    pub fn path(&self) -> Option<PathBuf> { self.path.clone() }

    pub fn content(&self) -> &TabContent { &self.content }

    /// Backward-compat accessor for code editors only. Panics on Markdown tabs —
    /// callers must check first via `content().is_markdown()`.
    pub fn editor_view(&self) -> &ViewHandle<LocalCodeEditorView> {
        match &self.content {
            TabContent::Code(v) => v,
            #[cfg(feature = "local_fs")]
            TabContent::Markdown(_) => panic!("editor_view() called on a Markdown tab"),
        }
    }
}
```

- [ ] **Step 3: Replace every `tab.editor_view` access in `view.rs` with `tab.editor_view()` (or content-aware logic)**

Run:
```bash
grep -n "\.editor_view" /opt/soft/warp/app/src/code/view.rs
```

For each match: if the code is code-only, switch to `tab.editor_view()` (the panicking accessor — but tabs are only Code at this stage so it's safe).

- [ ] **Step 4: Build to confirm refactor compiles**

```bash
cd /opt/soft/warp
cargo check -p warp_app
```

Expected: clean build. No behavior change yet — every tab is still `TabContent::Code`.

- [ ] **Step 5: Commit**

```bash
git add app/src/code/view.rs
git commit -m "refactor(code-view): wrap tab editor in TabContent enum"
```

---

### Task 8: Add `TabContent::Markdown` constructor path

**Files:**
- Modify: `app/src/code/view.rs`

- [ ] **Step 1: Write the failing test**

Add to `view.rs`'s `#[cfg(test)] mod tests`:

```rust
#[cfg(all(test, feature = "local_fs"))]
mod tabbed_markdown_tests {
    use super::*;

    #[test]
    fn open_markdown_creates_markdown_tab() {
        // Pseudo-code — depends on test harness available in this crate.
        // Ideal shape:
        let mut view = CodeView::new_for_test();
        view.open_markdown_tab(PathBuf::from("/tmp/x.md"), &mut test_ctx());
        assert_eq!(view.tab_count(), 1);
        assert!(view.tab_at(0).unwrap().content().is_markdown());
    }
}
```

If the existing `view.rs` has no in-file unit test harness, place this test in a new `app/src/code/view/tab_content_tests.rs` module and gate behind `#[cfg(test)]`. The exact harness should follow whatever the existing `code/view.rs` test code uses.

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /opt/soft/warp
cargo test -p warp_app code::view::tabbed_markdown_tests --features local_fs 2>&1 | tail -20
```

Expected: `open_markdown_tab` not defined.

- [ ] **Step 3: Implement `open_markdown_tab`**

```rust
#[cfg(feature = "local_fs")]
impl CodeView {
    pub fn open_markdown_tab(
        &mut self,
        path: PathBuf,
        ctx: &mut ViewContext<Self>,
    ) {
        if let Some(idx) = self.focus_existing_tab_if_present(&Some(path.clone()), ctx) {
            self.set_active_tab_index(idx, ctx);
            return;
        }
        let session = None; // Markdown notebooks don't need an active session
        let notebook_view = crate::notebooks::file::FileNotebookView::new(
            Some(path.clone()),
            session,
            None,
            ctx.parent(),
        );
        let tab = TabData {
            path: Some(path),
            content: TabContent::Markdown(notebook_view),
            mouse_state_handles: TabDataMouseStateHandles::default(),
            preview: false,
        };
        self.tab_group.push(tab);
        self.active_tab_index = self.tab_group.len() - 1;
        self.update_tab_bar_state(ctx);
        self.update_markdown_mode_segmented_control(ctx);
        ctx.notify();
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cd /opt/soft/warp
cargo test -p warp_app code::view::tabbed_markdown_tests --features local_fs 2>&1 | tail -20
```

Expected: 1 test passes.

- [ ] **Step 5: Commit**

```bash
git add app/src/code/view.rs
git commit -m "feat(code-view): support TabContent::Markdown via open_markdown_tab"
```

---

### Task 9: Render Markdown body when active tab is Markdown

**Files:**
- Modify: `app/src/code/view.rs` (rendering paths)

- [ ] **Step 1: Locate the body-rendering function**

Run:
```bash
grep -n "render_active_tab\|render_body\|fn render(" /opt/soft/warp/app/src/code/view.rs | head
```

Find the function that renders the active tab's content. (Likely called from the `View` trait impl `render(&mut self, ctx: &mut ViewContext<Self>) -> Element`.)

- [ ] **Step 2: Branch on TabContent**

In that render path, replace:
```rust
let active = &self.tab_group[self.active_tab_index];
// existing: render active.editor_view as the body
ChildView::new(active.editor_view()).finish()
```

With:
```rust
let active = &self.tab_group[self.active_tab_index];
match active.content() {
    TabContent::Code(view) => ChildView::new(view).finish(),
    #[cfg(feature = "local_fs")]
    TabContent::Markdown(view) => ChildView::new(view).finish(),
}
```

- [ ] **Step 3: Lift `MarkdownToggleView` from `FileNotebookView` header into `CodeView` header**

In `app/src/notebooks/file/mod.rs::render_header_content`:

Replace the entire `if self.is_markdown_file() { ... } else { ... }` block with the simple `else` branch (Standard header). The Markdown toggle no longer renders here.

In `app/src/code/view.rs::render_header_content` (line 2245):

Add a step that, if `self.tab_group[self.active_tab_index].content().is_markdown()`, emits the segmented control `ChildView::new(&self.markdown_mode_segmented_control.as_ref().unwrap()).finish()` as part of the header's right-side controls. The segmented control is already a field on `CodeView` (see Task 7 baseline grep — confirm it exists; if not, port `display_mode_segmented_control` from `FileNotebookView` to `CodeView`).

- [ ] **Step 4: Wire the toggle event back to the active Markdown tab**

`MarkdownToggleView` emits `MarkdownToggleEvent::ModeSelected(MarkdownDisplayMode)`. In `CodeView`'s view-event subscriptions (search `subscribe_to_view` patterns in `view.rs`), forward this to:

```rust
if let TabContent::Markdown(notebook) = self.tab_group[self.active_tab_index].content() {
    notebook.update(ctx, |nb, ctx| nb.set_markdown_display_mode(new_mode, ctx));
}
```

Add `set_markdown_display_mode(...)` to `FileNotebookView` if absent (it currently handles this via `FileNotebookAction::ToggleMarkdownDisplayMode` — expose a public setter that the action handler calls internally and we can call externally).

- [ ] **Step 5: Build and run app to verify Markdown renders inside CodeView**

```bash
cd /opt/soft/warp
cargo check -p warp_app
```

Expected: clean build.

Manual verification (since UI):
- Run `cargo run -p warp_app`
- Open file tree
- Click a `.md` file → opens as a tab in a CodeView container
- Header shows "Rendered/Raw" toggle; toggling switches the body
- Click a `.rs` file → new tab; toggle disappears (active tab is Code)
- Click `.md` again → toggle reappears, last-selected mode preserved

If a step fails, debug before committing.

- [ ] **Step 6: Commit**

```bash
git add app/src/code/view.rs app/src/notebooks/file/mod.rs
git commit -m "feat(code-view): render Markdown tab body and lift markdown toggle to container header"
```

---

## Phase 4: Workspace integration — replace flag branches with router dispatch

### Task 10: Wire `FilePaneRouter` into `open_file_with_target` / `open_code` / Markdown opens

**Files:**
- Modify: `app/src/workspace/view.rs`

- [ ] **Step 1: Extract a single dispatch helper**

In `view.rs`, add a private method on the workspace view:

```rust
#[cfg(feature = "local_fs")]
fn dispatch_open_file(
    &mut self,
    path: PathBuf,
    kind: crate::workspace::file_pane_router::FileKind,
    line_col: Option<LineAndColumnArg>,
    code_source: CodeSource,
    target_layout: EditorLayout,
    ctx: &mut ViewContext<Self>,
) {
    use crate::workspace::file_pane_router::{route, RouteAction, WorkspaceRouteContext};

    let grouping = *TabSettings::as_ref(ctx).group_opened_files_into_tabs.value();
    let pane_group = self.active_tab_pane_group();
    let opened_files = OpenedFilesModel::handle(ctx);

    let route_ctx = WorkspaceRouteContext {
        grouping,
        active_pane_group: pane_group.as_ref(ctx),
        opened_files: opened_files.as_ref(ctx),
        app_ctx: ctx,
    };
    let action = route(&route_ctx, &path, kind.clone());

    match action {
        RouteAction::FocusExistingTab { container_pane_id, tab_index } => {
            self.focus_pane_and_tab_by_u64(container_pane_id, tab_index, line_col, ctx);
        }
        RouteAction::FocusExistingLone { pane_id } => {
            self.focus_pane_by_u64(pane_id, ctx);
        }
        RouteAction::AppendToContainer { container_pane_id, kind } => {
            self.append_into_container_by_u64(container_pane_id, path, kind, line_col, code_source, ctx);
        }
        RouteAction::PromoteLoneToContainer { lone_pane_id, kind } => {
            self.promote_lone_to_container_by_u64(lone_pane_id, path, kind, line_col, code_source, ctx);
        }
        RouteAction::CreateNewContainer { kind } => {
            self.create_new_container_with_first_tab(path, kind, line_col, code_source, target_layout, ctx);
        }
        RouteAction::CreateNewSplitPane { kind } => {
            self.legacy_create_new_split_pane(path, kind, line_col, code_source, target_layout, ctx);
        }
    }
}
```

- [ ] **Step 2: Implement each helper used above**

The names are descriptive; implementation reuses what already exists:

- `focus_pane_and_tab_by_u64`: convert u64 → `PaneId` (scan PaneGroup for matching `creation_order_id`), call `pane_group.focus_pane(...)`, then `code_view.set_active_tab_index(tab_index, ctx)`, then jump if `line_col` is Some.
- `focus_pane_by_u64`: same conversion + `pane_group.focus_pane(...)`.
- `append_into_container_by_u64`: locate `CodeView` for the pane, call `code_view.open_or_focus_existing(Some(path), line_col, ctx)` for `FileKind::Code`, or new `code_view.open_markdown_tab(path, ctx)` for `FileKind::Markdown`.
- `promote_lone_to_container_by_u64`:
   - If lone pane is a `FilePane` (Markdown): build a fresh `CodeView` with the original markdown as tab 0, append the new file as tab 1, then replace the lone leaf in `PaneGroup` (use `PaneGroup::replace_pane(old_id, new_pane, ctx)` — add this method if missing).
   - If lone pane is a `CodePane` (single-tab): just call `open_or_focus_existing` / `open_markdown_tab` on its `CodeView`.
- `create_new_container_with_first_tab`: build `CodePane::new(...)` with the file already added; add to PaneGroup at `Direction::Right` (current behavior).
- `legacy_create_new_split_pane`: identical to today's path when grouping is OFF — add new pane via `CodePane::new` or `FilePane::new` and split right.

- [ ] **Step 3: Replace the four scattered flag branches with calls to `dispatch_open_file`**

In `open_code` (line ~7306 area), `NewCodeFile` action handler (line ~11049), and the two drop handlers (lines ~13577, ~13720), remove the `if grouping_on { ... } else { ... }` blocks and call `dispatch_open_file` (or for non-open-file paths like drop, call appropriate router-aware helpers — see Step 4).

For Markdown opens, find the `FileTarget::MarkdownViewer` arm in `open_file_with_target`:
```rust
FileTarget::MarkdownViewer(layout) => {
    let session = self.get_active_session(ctx);
    self.open_file_notebook(path.clone(), session, layout, ctx);
}
```
Replace with:
```rust
FileTarget::MarkdownViewer(layout) => {
    self.dispatch_open_file(
        path,
        FileKind::Markdown,
        None,
        code_source,
        layout,
        ctx,
    );
}
```

- [ ] **Step 4: Drop handlers — keep specialized merge/move logic, but route via router for the "open new" path**

Drop handlers do tab moves, not new-file opens — keep that logic. Only the "this drop opens a new file" sub-branches should go through `dispatch_open_file`. Existing `merge_tabs` / `remove_tab_for_move` calls stay.

- [ ] **Step 5: Build and run integration tests**

```bash
cd /opt/soft/warp
cargo check -p warp_app
cargo test -p warp_app workspace::file_pane_router
```

Expected: clean build, router unit tests still pass.

- [ ] **Step 6: Manual smoke test**

Run `cargo run -p warp_app`. Verify:
- Open 1 code file → 1 tab
- Open 2nd code file → 2 tabs in same container
- Open .md file → 3rd tab (Markdown), toggle appears
- Repeat-open same file → focuses existing tab
- Toggle setting OFF in Settings → next file opens as separate split

- [ ] **Step 7: Commit**

```bash
git add app/src/workspace/view.rs
git commit -m "refactor(workspace): route file opens through FilePaneRouter"
```

---

### Task 11: Extend `OpenedFilesModel` with `(PaneId, Option<TabIndex>)` index

**Files:**
- Modify: `app/src/code/opened_files.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_tab_index_on_open() {
        let mut model = OpenedFilesModel::new();
        model.record_pane_for_path(
            PathBuf::from("/repo/a.rs"),
            FakePaneId(1),
            Some(0),
        );
        assert_eq!(
            model.find_pane_and_tab_for(&PathBuf::from("/repo/a.rs")),
            Some((FakePaneId(1), Some(0))),
        );
    }

    #[test]
    fn updates_on_promote() {
        let mut model = OpenedFilesModel::new();
        model.record_pane_for_path(PathBuf::from("/repo/a.md"), FakePaneId(1), None);
        // After promote: same path now lives in container 7 at tab 0
        model.record_pane_for_path(PathBuf::from("/repo/a.md"), FakePaneId(7), Some(0));
        assert_eq!(
            model.find_pane_and_tab_for(&PathBuf::from("/repo/a.md")),
            Some((FakePaneId(7), Some(0))),
        );
    }

    #[test]
    fn forgets_on_close() {
        let mut model = OpenedFilesModel::new();
        model.record_pane_for_path(PathBuf::from("/repo/a.rs"), FakePaneId(1), Some(0));
        model.forget_path(&PathBuf::from("/repo/a.rs"));
        assert_eq!(model.find_pane_and_tab_for(&PathBuf::from("/repo/a.rs")), None);
    }
}
```

(Use whatever fake `PaneId` constructor the existing tests use; if none, define one in a `#[cfg(test)]` block.)

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /opt/soft/warp
cargo test -p warp_app code::opened_files 2>&1 | tail -20
```

Expected: `record_pane_for_path` / `find_pane_and_tab_for` not defined.

- [ ] **Step 3: Implement**

Add to `OpenedFilesModel`:

```rust
#[derive(Default)]
pub struct OpenedFilesModel {
    opened_files: HashMap<PathBuf, OpenedFilesInRepo>,
    pane_index: HashMap<PathBuf, (PaneId, Option<usize>)>,
}

impl OpenedFilesModel {
    pub fn record_pane_for_path(
        &mut self,
        path: PathBuf,
        pane_id: PaneId,
        tab_index: Option<usize>,
    ) {
        self.pane_index.insert(path, (pane_id, tab_index));
    }

    pub fn find_pane_and_tab_for(&self, path: &Path) -> Option<(PaneId, Option<usize>)> {
        self.pane_index.get(path).copied()
    }

    pub fn forget_path(&mut self, path: &Path) {
        self.pane_index.remove(path);
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd /opt/soft/warp
cargo test -p warp_app code::opened_files
```

Expected: 3 tests pass.

- [ ] **Step 5: Wire `record_pane_for_path` / `forget_path` calls into `dispatch_open_file` paths**

In each `RouteAction` handler in `view.rs` (Task 10), add the appropriate `OpenedFilesModel` mutation:
- `CreateNewContainer`/`CreateNewSplitPane`: record `(new_pane_id, Some(0))`
- `AppendToContainer`: record `(container_id, Some(new_tab_idx))`
- `PromoteLoneToContainer`: update old path entry, record new path
- Tab close (in CodeView::close_tab): forget that path
- Pane close: forget all paths whose pane_id matches

- [ ] **Step 6: Commit**

```bash
git add app/src/code/opened_files.rs app/src/workspace/view.rs
git commit -m "feat(opened-files): track (PaneId, TabIndex) for open-file routing"
```

---

## Phase 5: Integration tests

### Task 12: End-to-end Builder/TestStep coverage

**Files:**
- Create: `crates/integration/src/test/file_preview_tabs.rs`
- Modify: `crates/integration/src/test/mod.rs` (register new module)

- [ ] **Step 1: Author the test fixtures**

Use `crates/integration/src/test/file_tree.rs` as the template. Create `file_preview_tabs.rs`:

```rust
use crate::*;
use regex::Regex;

/// Fixture: write three files (one rs, one md, one txt) to test_dir.
fn setup_three_files(utils: &TestUtils) {
    let dir = utils.test_dir();
    let dir_str = dir.to_str().unwrap();
    write_all_rc_files_for_test(&dir, format!("cd {dir_str}"));
    std::fs::write(dir.join("a.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.join("b.md"), "# Hello\n\nworld").unwrap();
    std::fs::write(dir.join("c.rs"), "fn other() {}\n").unwrap();
}

pub fn test_opens_first_file_as_single_tab_container() -> Builder {
    new_builder()
        .with_setup(|u| setup_three_files(u))
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open file tree")
                .with_action(|app, _, _| open_file_tree_panel(app)),
        )
        .with_step(
            new_step_with_default_assertions("Click a.rs")
                .with_click_on_saved_position("file_tree_item:a.rs")
                .add_assertion(|app, win| {
                    let pg = pane_group_view(app, win, 0);
                    pg.read(app, |pg, _| async_assert_eq!(pg.pane_count(), 2, "expected terminal + container"))
                }),
        )
        .with_step(
            new_step_with_default_assertions("Verify a.rs is the active tab")
                .add_assertion(assert_pane_title(0, 1, Regex::new(r"a\.rs$").unwrap())),
        )
}

pub fn test_second_file_creates_second_tab_same_container() -> Builder {
    new_builder()
        .with_setup(|u| setup_three_files(u))
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(open_file_tree_step())
        .with_step(click_file_tree("a.rs"))
        .with_step(click_file_tree("c.rs"))
        .with_step(
            new_step_with_default_assertions("Pane count still 2 (container reused)")
                .add_assertion(|app, win| {
                    let pg = pane_group_view(app, win, 0);
                    pg.read(app, |pg, _| async_assert_eq!(pg.pane_count(), 2, "expected single container"))
                }),
        )
        .with_step(
            new_step_with_default_assertions("Active tab is c.rs (last opened)")
                .add_assertion(assert_pane_title(0, 1, Regex::new(r"c\.rs$").unwrap())),
        )
}

pub fn test_markdown_file_joins_same_container() -> Builder {
    new_builder()
        .with_setup(|u| setup_three_files(u))
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(open_file_tree_step())
        .with_step(click_file_tree("a.rs"))
        .with_step(click_file_tree("b.md"))
        .with_step(
            new_step_with_default_assertions("Same container, b.md active, markdown toggle visible")
                .add_assertion(|app, win| {
                    let pg = pane_group_view(app, win, 0);
                    pg.read(app, |pg, _| async_assert_eq!(pg.pane_count(), 2, "container reused"))
                })
                .add_assertion(assert_pane_title(0, 1, Regex::new(r"b\.md$").unwrap()))
                .add_assertion(assert_markdown_toggle_visible(0, 1)),
        )
}

pub fn test_repeat_open_focuses_existing_tab() -> Builder {
    new_builder()
        .with_setup(|u| setup_three_files(u))
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(open_file_tree_step())
        .with_step(click_file_tree("a.rs"))
        .with_step(click_file_tree("b.md"))
        .with_step(click_file_tree("a.rs"))
        .with_step(
            new_step_with_default_assertions("a.rs is active again, no extra tab")
                .add_assertion(assert_pane_title(0, 1, Regex::new(r"a\.rs$").unwrap()))
                .add_assertion(assert_tab_count(0, 1, 2)),
        )
}

pub fn test_close_last_tab_destroys_container() -> Builder {
    new_builder()
        .with_setup(|u| setup_three_files(u))
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(open_file_tree_step())
        .with_step(click_file_tree("a.rs"))
        .with_step(close_tab_step("a.rs"))
        .with_step(
            new_step_with_default_assertions("Container destroyed")
                .add_assertion(|app, win| {
                    let pg = pane_group_view(app, win, 0);
                    pg.read(app, |pg, _| async_assert_eq!(pg.pane_count(), 1, "only terminal remains"))
                }),
        )
}

pub fn test_setting_off_falls_back_to_split() -> Builder {
    new_builder()
        .with_setup(|u| {
            setup_three_files(u);
            u.set_user_setting("appearance.tabs.group_opened_files_into_tabs", "false");
        })
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(open_file_tree_step())
        .with_step(click_file_tree("a.rs"))
        .with_step(click_file_tree("c.rs"))
        .with_step(
            new_step_with_default_assertions("Two separate panes (split mode)")
                .add_assertion(|app, win| {
                    let pg = pane_group_view(app, win, 0);
                    pg.read(app, |pg, _| async_assert_eq!(pg.pane_count(), 3, "terminal + 2 split file panes"))
                }),
        )
}
```

Helpers used above (`open_file_tree_step`, `click_file_tree`, `close_tab_step`, `assert_tab_count`, `assert_markdown_toggle_visible`) — implement in the same file by reading existing helpers in `crates/integration/src/test/` and following their patterns.

- [ ] **Step 2: Register the module**

In `crates/integration/src/test/mod.rs`, add:

```rust
pub mod file_preview_tabs;
```

And in whichever runner registers manual/CI tests (search for how `file_tree` tests are listed), register each `test_*` function from the new module.

- [ ] **Step 3: Run the new integration tests**

```bash
cd /opt/soft/warp
cargo nextest run -p integration --test ... file_preview_tabs 2>&1 | tail -40
```

Adjust the exact nextest filter to match the project's runner conventions.

Expected: all 6 tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/integration/src/test/file_preview_tabs.rs crates/integration/src/test/mod.rs
git commit -m "test(integration): file preview tabs end-to-end coverage"
```

---

## Phase 6: Cleanup & feature-flag retirement

### Task 13: Remove legacy `prefer_tabbed_editor_view` field and FeatureFlag

> Do this only after at least one release cycle confirms migration ran cleanly. The plan documents both for completeness; in practice the engineer may choose to defer this task and ship Tasks 1–12 first.

**Files:**
- Modify: `app/src/util/file/external_editor/settings.rs` (remove field)
- Modify: `app/src/lib.rs` (remove FeatureFlag definition)
- Modify: `app/src/workspace/tab_settings/migration.rs` (drop legacy reader)

- [ ] **Step 1: Confirm telemetry shows migration completed**

Check telemetry dashboard for `prefer_tabbed_editor_view_migrated` event volume — confirm steady-state low volume (mostly returning users on old clients have already migrated).

- [ ] **Step 2: Delete the legacy field**

In `external_editor/settings.rs`, remove the `prefer_tabbed_editor_view` field block (lines 109–117).

- [ ] **Step 3: Delete the FeatureFlag**

In `lib.rs:2640-2641`, remove the `FeatureFlag::TabbedEditorView` declaration. Search the codebase to ensure nothing else references it:

```bash
grep -rn "TabbedEditorView" /opt/soft/warp/app/src/
```

Expected: 0 matches after this task.

- [ ] **Step 4: Simplify migration.rs**

Once the legacy field is gone, the migration is a no-op. Reduce `run_migration` to a stub that always sets the marker (so we can keep the marker logic for a future rename), or delete the migration module entirely if no future need.

- [ ] **Step 5: Build**

```bash
cd /opt/soft/warp
cargo check -p warp_app && cargo test -p warp_app
```

Expected: clean build, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add app/src/lib.rs app/src/util/file/external_editor/settings.rs app/src/workspace/tab_settings/migration.rs
git commit -m "chore: retire prefer_tabbed_editor_view legacy field and feature flag"
```

---

### Task 14: Add telemetry events

**Files:**
- Modify: telemetry definitions (locate via skill)

- [ ] **Step 1: Use the `add-telemetry` skill** to add these events:

- `file_tab_container_created` — emitted when a `CodeView` container is first created (in `CreateNewContainer` handler)
- `file_tab_added` — emitted on each new tab append; properties: `kind` (code/markdown), `source` (file_tree/ai_link/command_output/other)
- `file_tab_closed` — emitted on tab close; property: `caused_container_destroy` (bool)
- `file_tab_grouping_setting_toggled` — emitted when the new setting changes; property: `enabled` (bool)
- `prefer_tabbed_editor_view_migrated` — emitted once during migration; property: `legacy_value` (bool)

- [ ] **Step 2: Build and verify events compile**

```bash
cd /opt/soft/warp
cargo check -p warp_app
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(telemetry): file preview tabs lifecycle events"
```

---

## Final Verification

- [ ] **All unit tests pass**: `cargo test -p warp_app`
- [ ] **All integration tests pass**: `cargo nextest run -p integration file_preview_tabs`
- [ ] **Manual smoke matches behavior contract** (Phase 4 / Task 10 / Step 6)
- [ ] **Settings UI shows new toggle in Appearance → Tabs**
- [ ] **Settings UI no longer shows the toggle in External Editor**
- [ ] **`grep -rn "TabbedEditorView" app/src/`** returns 0 (after Task 13) or only the FeatureFlag def (if Task 13 deferred)
- [ ] **Memory check**: open 10 files, close them; verify no `OpenedFilesModel.pane_index` leak by adding a debug log

---

## Risk register

| Risk | Mitigation |
|---|---|
| `TabContent` refactor breaks every `editor_view` access in `view.rs` | Task 7 keeps backward-compat panicking accessor; refactor is staged before Markdown variant is even reachable |
| Lifting `MarkdownToggleView` introduces double-render (one in CodeView, one orphaned in FileNotebookView) | Task 9 Step 3 explicitly removes the FileNotebookView render branch |
| Drop handlers' merge logic conflicts with router | Task 10 Step 4 keeps drop-merge logic intact, only routes "new file open" sub-paths |
| Integration tests' Markdown toggle assertion needs new harness | Task 12 implements `assert_markdown_toggle_visible` by reading CodeView's segmented control state |
| Setting migration race on first launch | `run_migration` is idempotent (marker check) and called before first read in Task 2 Step 5 |
| Legacy field removal in Task 13 breaks downgraded clients reading new setting | One-release deferral; new setting was added in Task 1 with a TOML path that can be read by both versions |

