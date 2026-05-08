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
