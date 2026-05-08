//! Pure routing decisions for "open file" requests.

use std::path::Path;

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

// ----------------------------------------------------------------------------
// Workspace adapter
// ----------------------------------------------------------------------------
//
// Bridges the pure `RouteContext` trait above to Warp's real workspace state
// (PaneGroup + OpenedFilesModel + AppContext). Task 10 will construct this in
// the open-file flow; for now it is unused (compiler may warn dead_code).
//
// NOTE: Several pane-id <-> u64 hops use `EntityId`'s Display impl (which
// formats the underlying `usize`) parsed back into `u64`. This is stable for
// the lifetime of the process and keeps the pure router agnostic of warp's
// EntityId type without leaking it through the trait. If a richer "focus
// history" API becomes available we should swap the recency proxy below.

#[cfg(feature = "local_fs")]
use crate::pane_group::{pane::PaneId, PaneGroup};

#[cfg(feature = "local_fs")]
#[allow(dead_code)]
pub struct WorkspaceRouteContext<'a> {
    pub grouping: bool,
    pub active_pane_group: &'a PaneGroup,
    pub opened_files: &'a crate::code::opened_files::OpenedFilesModel,
    pub app_ctx: &'a warpui::AppContext,
}

#[cfg(feature = "local_fs")]
#[allow(dead_code)]
fn pane_id_to_u64(p: PaneId) -> u64 {
    // EntityId is internally a `usize` and Displays as the bare number.
    // Round-trip via Display keeps us decoupled from the `pub(crate)`
    // accessors in warpui_core without exposing them publicly.
    format!("{}", p.creation_order_id()).parse::<u64>().unwrap_or(0)
}

#[cfg(feature = "local_fs")]
impl<'a> RouteContext for WorkspaceRouteContext<'a> {
    fn grouping_enabled(&self) -> bool {
        self.grouping
    }

    fn find_tab_for(&self, path: &Path) -> Option<(u64, usize)> {
        for (pane_id, code_view) in self.active_pane_group.code_panes(self.app_ctx) {
            if self.active_pane_group.is_pane_hidden_for_close(pane_id) {
                continue;
            }
            let view = code_view.as_ref(self.app_ctx);
            let count = view.tab_count();
            for idx in 0..count {
                if let Some(tab) = view.tab_at(idx) {
                    if tab.path().as_deref() == Some(path) {
                        return Some((pane_id_to_u64(pane_id), idx));
                    }
                }
            }
        }
        None
    }

    fn find_lone_pane_for(&self, path: &Path) -> Option<u64> {
        for (pane_id, file_view) in self.active_pane_group.file_panes(self.app_ctx) {
            if self.active_pane_group.is_pane_hidden_for_close(pane_id) {
                continue;
            }
            if file_view.as_ref(self.app_ctx).local_path().as_deref() == Some(path) {
                return Some(pane_id_to_u64(pane_id));
            }
        }
        None
    }

    fn find_focused_or_first_container(&self) -> Option<u64> {
        let focused = self.active_pane_group.focused_pane_id(self.app_ctx);
        if focused.is_code_pane()
            && !self.active_pane_group.is_pane_hidden_for_close(focused)
        {
            return Some(pane_id_to_u64(focused));
        }
        // Fall back to the first visible code pane.
        self.active_pane_group
            .code_panes(self.app_ctx)
            .find(|(pid, _)| !self.active_pane_group.is_pane_hidden_for_close(*pid))
            .map(|(pid, _)| pane_id_to_u64(pid))
    }

    fn find_most_recently_focused_lone_file_pane(&self) -> Option<u64> {
        // APPROXIMATION: there is no per-pane focus-history API on PaneGroup
        // today. We use creation order (highest EntityId among visible file
        // panes) as a proxy for "most recently created" lone file pane, which
        // matches the common case where the just-opened markdown preview is
        // the candidate for promotion. Replace with a real focus-history
        // lookup if/when one becomes available.
        self.active_pane_group
            .file_panes(self.app_ctx)
            .filter(|(pid, _)| !self.active_pane_group.is_pane_hidden_for_close(*pid))
            .map(|(pid, _)| pid)
            .max_by_key(|pid| pid.creation_order_id())
            .map(pane_id_to_u64)
    }
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
