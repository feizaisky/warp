//! Module containing the definition of [`OpenedFilesModel`],
//! which tracks files that have been opened, organized by repository.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use instant::Instant;
use warpui::{Entity, ModelContext, SingletonEntity};

use crate::pane_group::pane::PaneId;

#[derive(Default, Clone)]
pub struct OpenedFilesInRepo(HashMap<PathBuf, Instant>);

impl OpenedFilesInRepo {
    pub fn get(&self, file_path: &PathBuf) -> Option<&Instant> {
        self.0.get(file_path)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &Instant)> {
        self.0.iter()
    }
}

/// Model that tracks files that have been opened, organized by repository.
/// Maps repository paths to files and when they were last opened.
#[derive(Default)]
pub struct OpenedFilesModel {
    opened_files: HashMap<PathBuf, OpenedFilesInRepo>,
    /// Tracks the (PaneId, TabIndex) for each currently-open file path,
    /// enabling fast routing without a live PaneGroup scan.
    pane_index: HashMap<PathBuf, (PaneId, Option<usize>)>,
}

impl Entity for OpenedFilesModel {
    type Event = ();
}

impl SingletonEntity for OpenedFilesModel {}

impl OpenedFilesModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all opened files for a specific repository.
    pub fn opened_files_for_repo(&self, repo_path: &PathBuf) -> Option<&OpenedFilesInRepo> {
        self.opened_files.get(repo_path)
    }

    /// Record that a file has been opened in a repository. If the `file_path` is not within the `repo_path`,
    /// then the file is not recorded.
    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    pub fn file_opened(
        &mut self,
        repo_path: PathBuf,
        file_path: PathBuf,
        ctx: &mut ModelContext<Self>,
    ) {
        let opened_at = Instant::now();

        // Convert absolute file path to relative path from repo root
        let Ok(relative_file_path) = file_path.strip_prefix(&repo_path) else {
            return;
        };

        self.opened_files
            .entry(repo_path.clone())
            .or_default()
            .0
            .insert(relative_file_path.into(), opened_at);

        ctx.notify();
    }

    /// Record (or update) which pane and tab index a file currently lives in.
    pub fn record_pane_for_path(
        &mut self,
        path: PathBuf,
        pane_id: PaneId,
        tab_index: Option<usize>,
    ) {
        self.pane_index.insert(path, (pane_id, tab_index));
    }

    /// Return the (PaneId, tab index) for a file, if it has been recorded.
    pub fn find_pane_and_tab_for(&self, path: &Path) -> Option<(PaneId, Option<usize>)> {
        self.pane_index.get(path).copied()
    }

    /// Remove the pane/tab tracking entry for a file (e.g. when the tab is closed).
    pub fn forget_path(&mut self, path: &Path) {
        self.pane_index.remove(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_tab_index_on_open() {
        let mut model = OpenedFilesModel::new();
        let pane = PaneId::dummy_pane_id();
        model.record_pane_for_path(PathBuf::from("/repo/a.rs"), pane, Some(0));
        assert_eq!(
            model.find_pane_and_tab_for(&PathBuf::from("/repo/a.rs")),
            Some((pane, Some(0))),
        );
    }

    #[test]
    fn updates_on_promote() {
        let mut model = OpenedFilesModel::new();
        let pane1 = PaneId::dummy_pane_id();
        let pane7 = PaneId::dummy_pane_id();
        model.record_pane_for_path(PathBuf::from("/repo/a.md"), pane1, None);
        model.record_pane_for_path(PathBuf::from("/repo/a.md"), pane7, Some(0));
        assert_eq!(
            model.find_pane_and_tab_for(&PathBuf::from("/repo/a.md")),
            Some((pane7, Some(0))),
        );
    }

    #[test]
    fn forgets_on_close() {
        let mut model = OpenedFilesModel::new();
        let pane = PaneId::dummy_pane_id();
        model.record_pane_for_path(PathBuf::from("/repo/a.rs"), pane, Some(0));
        model.forget_path(&PathBuf::from("/repo/a.rs"));
        assert_eq!(model.find_pane_and_tab_for(&PathBuf::from("/repo/a.rs")), None);
    }
}
