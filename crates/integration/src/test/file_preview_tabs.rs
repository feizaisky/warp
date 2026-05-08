//! Integration tests for the file preview tabs feature.
//!
//! These tests exercise the end-to-end flow where multiple files opened from
//! the file tree end up grouped into a single CodeView container with one tab
//! per file (the default `appearance.tabs.group_opened_files_into_tabs = true`
//! behavior). The router lives in `app::workspace::file_pane_router` and is
//! reached via `Workspace::dispatch_open_file`.
//!
//! Scenarios covered:
//!  * Opening one file produces a 2-pane layout (terminal + container).
//!  * Opening a second file reuses the existing container (still 2 panes,
//!    container has 2 tabs, second is active).
//!  * Mixing a markdown file into the same container works.
//!  * Re-opening an already-open file focuses the existing tab without
//!    growing the tab count.
//!  * Closing the last tab tears the container pane down.
//!
//! The `setting_off` scenario from the spec is intentionally omitted: the
//! current Builder/TestStep harness has no clean helper for mutating user
//! settings before bootstrap, and writing a one-off settings-file-injection
//! helper for a single scenario is out of scope here.

use super::{new_builder, Builder};
use regex::Regex;

use warp::{
    integration_testing::{
        step::new_step_with_default_assertions,
        tab::assert_pane_title,
        terminal::wait_until_bootstrapped_single_pane_for_tab,
        view_getters::{pane_group_view, workspace_view},
        CodeViewAction,
    },
    workspace::WorkspaceAction,
};
use warpui::{async_assert_eq, integration::TestStep, App, WindowId};

use crate::util::write_all_rc_files_for_test;

fn open_file_tree_panel(app: &mut App) {
    let window_id = app.read(|ctx| {
        ctx.windows()
            .active_window()
            .expect("should have active window")
    });
    let workspace = workspace_view(app, window_id);
    app.update(|ctx| {
        ctx.dispatch_typed_action_for_view(
            window_id,
            workspace.id(),
            &WorkspaceAction::ToggleProjectExplorer,
        );
    });
}

/// Writes the standard test fixtures used by every scenario in this module:
/// - `a.rs` — a Rust source file
/// - `b.md` — a Markdown file
/// - `c.rs` — a second Rust source file
fn setup_three_files(test_dir: &std::path::Path) {
    std::fs::write(test_dir.join("a.rs"), "fn a() {\n    println!(\"a\");\n}\n")
        .expect("Failed to create a.rs");
    std::fs::write(
        test_dir.join("b.md"),
        "# Heading B\n\nSome markdown content.\n",
    )
    .expect("Failed to create b.md");
    std::fs::write(test_dir.join("c.rs"), "fn c() {\n    println!(\"c\");\n}\n")
        .expect("Failed to create c.rs");
}

/// Read the tab count from the (single) `CodeView` container in the active
/// pane group of tab 0. Panics if there is no `CodeView` in the pane group.
fn assert_container_tab_count(
    app: &mut App,
    window_id: WindowId,
    expected: usize,
) -> warpui::integration::AssertionOutcome {
    let pane_group = pane_group_view(app, window_id, 0);
    let count = pane_group.read(app, |pane_group, ctx| {
        let mut iter = pane_group.code_panes(ctx);
        let (_id, code_view) = iter
            .next()
            .expect("Expected at least one CodeView container in the pane group");
        code_view.read(ctx, |view, _| view.tab_count())
    });
    async_assert_eq!(
        count,
        expected,
        "Expected container to have {} tab(s), got {}",
        expected,
        count
    )
}

/// Assert that the active tab inside the (single) CodeView container has a
/// path whose file name matches `expected_name`.
fn assert_container_active_tab_name(
    app: &mut App,
    window_id: WindowId,
    expected_name: &str,
) -> warpui::integration::AssertionOutcome {
    let pane_group = pane_group_view(app, window_id, 0);
    let active_name = pane_group.read(app, |pane_group, ctx| {
        let mut iter = pane_group.code_panes(ctx);
        let (_id, code_view) = iter
            .next()
            .expect("Expected at least one CodeView container in the pane group");
        code_view.read(ctx, |view, _| {
            let idx = view.active_tab_index();
            view.tab_at(idx)
                .and_then(|t| t.path())
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        })
    });
    async_assert_eq!(
        active_name.as_deref(),
        Some(expected_name),
        "Expected active tab to be {}, got {:?}",
        expected_name,
        active_name
    )
}

/// Open one code file via the file tree. Should yield a 2-pane layout
/// (terminal + container) and a single tab in the container.
pub fn test_opens_first_file_as_single_tab_container() -> Builder {
    new_builder()
        .with_setup(|utils| {
            let test_dir = utils.test_dir();
            let dir_string = test_dir.to_str().expect("test dir to str");
            write_all_rc_files_for_test(&test_dir, format!("cd {dir_string}"));
            setup_three_files(&test_dir);
        })
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open file tree panel")
                .with_action(|app, _, _| open_file_tree_panel(app)),
        )
        .with_step(
            new_step_with_default_assertions("Click a.rs")
                .with_click_on_saved_position("file_tree_item:a.rs")
                .add_assertion(|app, window_id| {
                    let pane_group = pane_group_view(app, window_id, 0);
                    pane_group.read(app, |pane_group, _ctx| {
                        async_assert_eq!(
                            pane_group.pane_count(),
                            2,
                            "Expected 2 panes (terminal + container) after opening one file"
                        )
                    })
                })
                .add_assertion(|app, window_id| assert_container_tab_count(app, window_id, 1))
                .add_assertion(assert_pane_title(0, 1, Regex::new(r"a\.rs$").unwrap())),
        )
}

/// Opening a second code file should reuse the existing container — pane
/// count stays at 2, container now has 2 tabs, and the second file is active.
pub fn test_second_file_creates_second_tab_same_container() -> Builder {
    new_builder()
        .with_setup(|utils| {
            let test_dir = utils.test_dir();
            let dir_string = test_dir.to_str().expect("test dir to str");
            write_all_rc_files_for_test(&test_dir, format!("cd {dir_string}"));
            setup_three_files(&test_dir);
        })
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open file tree panel")
                .with_action(|app, _, _| open_file_tree_panel(app)),
        )
        .with_step(
            new_step_with_default_assertions("Click a.rs")
                .with_click_on_saved_position("file_tree_item:a.rs"),
        )
        .with_step(
            new_step_with_default_assertions("Click c.rs")
                .with_click_on_saved_position("file_tree_item:c.rs")
                .add_assertion(|app, window_id| {
                    let pane_group = pane_group_view(app, window_id, 0);
                    pane_group.read(app, |pane_group, _ctx| {
                        async_assert_eq!(
                            pane_group.pane_count(),
                            2,
                            "Expected pane count to stay at 2 — second file should reuse container"
                        )
                    })
                })
                .add_assertion(|app, window_id| assert_container_tab_count(app, window_id, 2))
                .add_assertion(|app, window_id| {
                    assert_container_active_tab_name(app, window_id, "c.rs")
                }),
        )
}

/// Opening a markdown file after a code file should reuse the same container.
pub fn test_markdown_file_joins_same_container() -> Builder {
    new_builder()
        .with_setup(|utils| {
            let test_dir = utils.test_dir();
            let dir_string = test_dir.to_str().expect("test dir to str");
            write_all_rc_files_for_test(&test_dir, format!("cd {dir_string}"));
            setup_three_files(&test_dir);
        })
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open file tree panel")
                .with_action(|app, _, _| open_file_tree_panel(app)),
        )
        .with_step(
            new_step_with_default_assertions("Click a.rs")
                .with_click_on_saved_position("file_tree_item:a.rs"),
        )
        .with_step(
            new_step_with_default_assertions("Click b.md")
                .with_click_on_saved_position("file_tree_item:b.md")
                .add_assertion(|app, window_id| {
                    let pane_group = pane_group_view(app, window_id, 0);
                    pane_group.read(app, |pane_group, _ctx| {
                        async_assert_eq!(
                            pane_group.pane_count(),
                            2,
                            "Expected pane count to stay at 2 — markdown should join same container"
                        )
                    })
                })
                .add_assertion(|app, window_id| assert_container_tab_count(app, window_id, 2))
                .add_assertion(|app, window_id| {
                    assert_container_active_tab_name(app, window_id, "b.md")
                }),
        )
}

/// Opening A, then B, then A again should focus the existing A tab without
/// growing the tab count.
pub fn test_repeat_open_focuses_existing_tab() -> Builder {
    new_builder()
        .with_setup(|utils| {
            let test_dir = utils.test_dir();
            let dir_string = test_dir.to_str().expect("test dir to str");
            write_all_rc_files_for_test(&test_dir, format!("cd {dir_string}"));
            setup_three_files(&test_dir);
        })
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open file tree panel")
                .with_action(|app, _, _| open_file_tree_panel(app)),
        )
        .with_step(
            new_step_with_default_assertions("Click a.rs")
                .with_click_on_saved_position("file_tree_item:a.rs"),
        )
        .with_step(
            new_step_with_default_assertions("Click c.rs")
                .with_click_on_saved_position("file_tree_item:c.rs"),
        )
        .with_step(
            new_step_with_default_assertions("Click a.rs again")
                .with_click_on_saved_position("file_tree_item:a.rs")
                .add_assertion(|app, window_id| {
                    let pane_group = pane_group_view(app, window_id, 0);
                    pane_group.read(app, |pane_group, _ctx| {
                        async_assert_eq!(
                            pane_group.pane_count(),
                            2,
                            "Expected 2 panes after re-opening already-open file"
                        )
                    })
                })
                .add_assertion(|app, window_id| assert_container_tab_count(app, window_id, 2))
                .add_assertion(|app, window_id| {
                    assert_container_active_tab_name(app, window_id, "a.rs")
                }),
        )
}

/// After opening one file and closing it, the container pane should be torn
/// down, leaving just the terminal pane.
pub fn test_close_last_tab_destroys_container() -> Builder {
    new_builder()
        .with_setup(|utils| {
            let test_dir = utils.test_dir();
            let dir_string = test_dir.to_str().expect("test dir to str");
            write_all_rc_files_for_test(&test_dir, format!("cd {dir_string}"));
            setup_three_files(&test_dir);
        })
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(
            new_step_with_default_assertions("Open file tree panel")
                .with_action(|app, _, _| open_file_tree_panel(app)),
        )
        .with_step(
            new_step_with_default_assertions("Click a.rs")
                .with_click_on_saved_position("file_tree_item:a.rs")
                .add_assertion(|app, window_id| {
                    let pane_group = pane_group_view(app, window_id, 0);
                    pane_group.read(app, |pane_group, _ctx| {
                        async_assert_eq!(pane_group.pane_count(), 2, "Expected 2 panes")
                    })
                }),
        )
        .with_step(
            // Dispatch CloseAll on the CodeView, which closes all of its tabs
            // and tears down the surrounding pane.
            TestStep::new("Close the only tab in the container")
                .with_action(|app, _step_data, _utils| {
                    let window_id = app.read(|ctx| {
                        ctx.windows()
                            .active_window()
                            .expect("should have active window")
                    });
                    let pane_group = pane_group_view(app, window_id, 0);
                    let code_view = pane_group.read(app, |pg, ctx| {
                        pg.code_panes(ctx)
                            .next()
                            .map(|(_id, v)| v)
                            .expect("Expected a CodeView container")
                    });
                    app.update(|ctx| {
                        ctx.dispatch_typed_action_for_view(
                            window_id,
                            code_view.id(),
                            &CodeViewAction::CloseAll,
                        );
                    });
                })
                .add_assertion(|app, window_id| {
                    let pane_group = pane_group_view(app, window_id, 0);
                    pane_group.read(app, |pane_group, _ctx| {
                        // close_pane hides (rather than removes) panes when
                        // UndoClosedPanes is enabled, so check visible count.
                        async_assert_eq!(
                            pane_group.visible_pane_count(),
                            1,
                            "Expected visible pane count to drop back to 1 after closing the only file tab"
                        )
                    })
                }),
        )
}
