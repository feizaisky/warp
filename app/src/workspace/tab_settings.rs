use std::collections::HashMap;
use std::path::Path;

use settings::{
    macros::define_settings_group, RespectUserSyncSetting, SupportedPlatforms, SyncToCloud,
};
use warp_core::ui::theme::AnsiColorIdentifier;

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "新标签页在标签栏中的放置位置。",
    rename_all = "snake_case"
)]
pub enum NewTabPlacement {
    #[default]
    AfterCurrentTab,
    AfterAllTabs,
}

settings::macros::implement_setting_for_enum!(
    NewTabPlacement,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Never,
    private: false,
    toml_path: "general.new_tab_placement",
    description: "新标签页在标签栏中的放置位置。",
);

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "标签页关闭按钮的位置。", rename_all = "snake_case")]
pub enum TabCloseButtonPosition {
    #[default]
    Right,
    Left,
}

settings::macros::implement_setting_for_enum!(
    TabCloseButtonPosition,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.tabs.tab_close_button_position",
    description: "标签页关闭按钮的位置。",
);

/// Visibility options for workspace decorations like the tab bar.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "标签栏等工作区装饰元素的显示时机。",
    rename_all = "snake_case"
)]
pub enum WorkspaceDecorationVisibility {
    /// Always show workspace decorations.
    AlwaysShow,
    /// Hide workspace decorations if fullscreen.
    #[default]
    HideFullscreen,
    /// Only show workspace decorations on hover.
    OnHover,
}

settings::macros::implement_setting_for_enum!(
    WorkspaceDecorationVisibility,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.tabs.workspace_decoration_visibility",
    description: "标签栏等工作区装饰元素的显示时机。",
);

impl WorkspaceDecorationVisibility {
    /// Choose a visibility setting that's logically opposite from this one.
    pub fn toggled(self) -> Self {
        // If we add other variants, there should still be logical opposites for each. For example,
        // toggling from any form of hidden workspace decorations should re-enable them.
        match self {
            WorkspaceDecorationVisibility::AlwaysShow => WorkspaceDecorationVisibility::OnHover,
            WorkspaceDecorationVisibility::OnHover => WorkspaceDecorationVisibility::HideFullscreen,
            WorkspaceDecorationVisibility::HideFullscreen => WorkspaceDecorationVisibility::OnHover,
        }
    }

    /// True if this is a setting where workspace decorations are hidden by default.
    pub fn hides_decorations_by_default(self) -> bool {
        matches!(self, WorkspaceDecorationVisibility::OnHover,)
    }

    /// True if *window* decorations should be shown.
    pub fn show_window_decorations(self) -> bool {
        !matches!(self, WorkspaceDecorationVisibility::OnHover)
    }
}

/// Represents the color state for a directory entry in the tab-color settings.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Color assignment state for a directory's tab.",
    rename_all = "snake_case"
)]
pub enum DirectoryTabColor {
    /// User explicitly removed this directory. Retained for backwards compatibility with settings files written by older versions.
    #[schemars(description = "The directory was explicitly removed from tab coloring.")]
    Suppressed,
    /// Directory is tracked but has no assigned color.
    #[schemars(description = "The directory is tracked but has no assigned color.")]
    Unassigned,
    /// Directory is tracked with a specific color.
    #[schemars(description = "The directory is assigned a specific color.")]
    Color(AnsiColorIdentifier),
}

impl DirectoryTabColor {
    pub(crate) fn ansi_color(self) -> Option<AnsiColorIdentifier> {
        match self {
            DirectoryTabColor::Color(c) => Some(c),
            DirectoryTabColor::Suppressed | DirectoryTabColor::Unassigned => None,
        }
    }
}

/// User-configured directory→color mappings for tab coloring.
///
/// Keys are directory paths (as strings). Values indicate the color state:
/// - `Suppressed`: directory was explicitly removed by the user via the per-row X button.
///   Retained so `color_for_directory` can shadow broader prefix matches, and for
///   backwards compatibility with settings files written by older versions.
/// - `Unassigned`: directory is tracked but has no specific color.
/// - `Color(c)`: directory is tracked with the given color.
#[derive(
    Default,
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "目录路径到标签页颜色分配的映射。")]
pub struct DirectoryTabColors(pub(crate) HashMap<String, DirectoryTabColor>);

settings::macros::implement_setting_for_enum!(
    DirectoryTabColors,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Never,
    private: false,
    toml_path: "appearance.tabs.directory_tab_colors",
    max_table_depth: 0,
    description: "目录路径到标签页颜色分配的映射。",
    feature_flag: warp_core::features::FeatureFlag::DirectoryTabColors,
);

impl DirectoryTabColors {
    /// Returns the configured tab color for a directory using longest-prefix matching.
    /// Returns `None` if no configured directory is a prefix of `dir`.
    pub fn color_for_directory(&self, dir: &Path) -> Option<DirectoryTabColor> {
        let canonical_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        self.0
            .iter()
            .filter_map(|(configured_path, color)| {
                let configured = Path::new(configured_path);
                match color {
                    DirectoryTabColor::Suppressed => None,
                    _ => canonical_dir
                        .starts_with(configured)
                        .then_some((configured, *color)),
                }
            })
            .max_by_key(|(configured, _)| configured.as_os_str().len())
            .map(|(_, color)| color)
    }

    /// Returns a new value with the given directory's color updated.
    pub fn with_color(&self, path: &Path, color: DirectoryTabColor) -> Self {
        let mut map = self.0.clone();

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        map.insert(canonical.to_string_lossy().to_string(), color);
        Self(map)
    }
}

#[derive(
    Clone,
    Debug,
    Default,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "垂直标签页面板标题栏中工具栏标签的配置。",
    rename_all = "snake_case"
)]
pub enum HeaderToolbarChipSelection {
    #[default]
    Default,
    Custom {
        left: Vec<super::header_toolbar_item::HeaderToolbarItemKind>,
        right: Vec<super::header_toolbar_item::HeaderToolbarItemKind>,
    },
}

impl HeaderToolbarChipSelection {
    pub fn left_items(&self) -> Vec<super::header_toolbar_item::HeaderToolbarItemKind> {
        use super::header_toolbar_item::HeaderToolbarItemKind;
        match self {
            Self::Default => HeaderToolbarItemKind::default_left(),
            Self::Custom { left, .. } => left.clone(),
        }
    }

    pub fn right_items(&self) -> Vec<super::header_toolbar_item::HeaderToolbarItemKind> {
        use super::header_toolbar_item::HeaderToolbarItemKind;
        match self {
            Self::Default => HeaderToolbarItemKind::default_right(),
            Self::Custom { right, .. } => right.clone(),
        }
    }
}

settings::macros::implement_setting_for_enum!(
    HeaderToolbarChipSelection,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.tabs.header_toolbar_chip_selection",
    description: "垂直标签页面板标题栏中工具栏标签的配置。",
);

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "垂直标签栏的显示模式。", rename_all = "snake_case")]
pub enum VerticalTabsViewMode {
    #[default]
    Compact,
    Expanded,
}

settings::macros::implement_setting_for_enum!(
    VerticalTabsViewMode,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.vertical_tabs.view_mode",
    description: "垂直标签栏的显示模式。",
);

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "垂直标签页面板中显示行的粒度。",
    rename_all = "snake_case"
)]
pub enum VerticalTabsDisplayGranularity {
    #[default]
    Panes,
    Tabs,
}

settings::macros::implement_setting_for_enum!(
    VerticalTabsDisplayGranularity,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.vertical_tabs.display_granularity",
    description: "垂直标签页面板中显示行的粒度。",
);

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "垂直标签页中的标签项显示模式。",
    rename_all = "snake_case"
)]
pub enum VerticalTabsTabItemMode {
    #[default]
    FocusedSession,
    Summary,
}

settings::macros::implement_setting_for_enum!(
    VerticalTabsTabItemMode,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.vertical_tabs.tab_item_mode",
    description: "垂直标签页中的标签项显示模式。",
);

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Primary information displayed on vertical tabs.",
    rename_all = "snake_case"
)]
pub enum VerticalTabsPrimaryInfo {
    #[default]
    Command,
    WorkingDirectory,
    Branch,
}

settings::macros::implement_setting_for_enum!(
    VerticalTabsPrimaryInfo,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.vertical_tabs.primary_info",
    description: "垂直标签页上显示的主要信息。",
);

#[derive(
    Default,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Copy,
    Clone,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "紧凑垂直标签页上显示的副标题。",
    rename_all = "snake_case"
)]
pub enum VerticalTabsCompactSubtitle {
    #[default]
    Branch,
    WorkingDirectory,
    Command,
}

settings::macros::implement_setting_for_enum!(
    VerticalTabsCompactSubtitle,
    TabSettings,
    SupportedPlatforms::ALL,
    SyncToCloud::Globally(RespectUserSyncSetting::Yes),
    private: false,
    toml_path: "appearance.vertical_tabs.compact_subtitle",
    description: "紧凑垂直标签页上显示的副标题。",
);

define_settings_group!(TabSettings, settings: [
    show_indicators: ShowIndicatorsButton {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.tabs.show_indicators_button",
        description: "是否在标签页上显示活动指示器。",
    },
    show_code_review_button: ShowCodeReviewButton {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.editor.show_code_review_button",
        description: "是否在标签页上显示代码审查按钮。",
    },
    show_code_review_diff_stats: ShowCodeReviewDiffStats {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.editor.show_code_review_diff_stats",
        description: "是否在代码审查按钮上显示新增/删除行数。",
    },
    preserve_active_tab_color: PreserveActiveTabColor {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.tabs.preserve_active_tab_color",
        description: "切换标签页时是否保留当前活动标签页的颜色。",
    },
    use_vertical_tabs: UseVerticalTabs {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.vertical_tabs.enabled",
        description: "是否以垂直方式而非水平方式显示标签页。",
    },
    show_vertical_tab_panel_in_restored_windows: ShowVerticalTabPanelInRestoredWindows {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.vertical_tabs.show_panel_in_restored_windows",
        description: "恢复窗口时，即使保存会话时垂直标签页面板是关闭的，也打开该面板。",
    },
    use_latest_user_prompt_as_conversation_title_in_tab_names: UseLatestUserPromptAsConversationTitleInTabNames {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.vertical_tabs.use_latest_prompt_as_title",
        description: "智能体对话的垂直标签页名称是否使用最新的用户提示词。",
    },
    vertical_tabs_display_granularity: VerticalTabsDisplayGranularity,
    vertical_tabs_tab_item_mode: VerticalTabsTabItemMode,
    vertical_tabs_view_mode: VerticalTabsViewMode,
    vertical_tabs_primary_info: VerticalTabsPrimaryInfo,
    vertical_tabs_compact_subtitle: VerticalTabsCompactSubtitle,
    vertical_tabs_show_pr_link: VerticalTabsShowPrLink {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.vertical_tabs.show_pr_link",
        description: "是否在垂直标签页上显示 PR 链接。",
    },
    vertical_tabs_show_diff_stats: VerticalTabsShowDiffStats {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.vertical_tabs.show_diff_stats",
        description: "是否在垂直标签页上显示差异统计。",
    },
    vertical_tabs_show_details_on_hover: VerticalTabsShowDetailsOnHover {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.vertical_tabs.show_details_on_hover",
        description: "悬停在垂直标签页上时是否显示详情侧栏。",
    },
    header_toolbar_chip_selection: HeaderToolbarChipSelection,
    new_tab_placement: NewTabPlacement,
    workspace_decoration_visibility: WorkspaceDecorationVisibility,
    close_button_position: TabCloseButtonPosition,
    directory_tab_colors: DirectoryTabColors,
]);

#[cfg(test)]
#[path = "tab_settings_tests.rs"]
mod tests;
