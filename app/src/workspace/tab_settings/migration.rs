//! One-shot migration: prefer_tabbed_editor_view -> group_opened_files_into_tabs.

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
    fn set_legacy_prefer_tabbed_editor_view(&mut self, v: Option<bool>) {
        self.legacy = v;
    }
    fn set_new_group_opened_files_into_tabs(&mut self, v: Option<bool>) {
        self.new_value = v;
    }
    fn get_new_group_opened_files_into_tabs(&self) -> Option<bool> {
        self.new_value
    }
    fn set_migration_marker(&mut self) {
        self.marker = true;
    }
    fn migration_marker_set(&self) -> bool {
        self.marker
    }
}

#[cfg(test)]
impl SettingsStore for MockSettingsStore {
    fn legacy_prefer_tabbed_editor_view(&self) -> Option<bool> {
        self.legacy
    }
    fn new_group_opened_files_into_tabs(&self) -> Option<bool> {
        self.new_value
    }
    fn set_new_group_opened_files_into_tabs(&mut self, value: bool) {
        self.new_value = Some(value);
    }
    fn migration_marker_set(&self) -> bool {
        self.marker
    }
    fn set_migration_marker(&mut self) {
        self.marker = true;
    }
}

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
