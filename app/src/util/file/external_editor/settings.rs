pub use crate::util::openable_file_type::EditorLayout;
use serde::{Deserialize, Deserializer, Serialize};
use settings::{
    macros::define_settings_group, RespectUserSyncSetting, SupportedPlatforms, SyncToCloud,
};

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(
    description = "Which editor to use when opening files.",
    rename_all = "snake_case"
)]
pub enum EditorChoice {
    SystemDefault,
    Warp,
    EnvEditor,
    #[schemars(description = "A specific external code editor.")]
    ExternalEditor(super::Editor),
}

// Custom Deserialize implementation to handle backward compatibility
// with the old `Option<Editor>` format
impl<'de> Deserialize<'de> for EditorChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum EditorChoiceCompat {
            // Try new format first
            New(EditorChoiceInner),
            // Fall back to old Option<Editor> format
            Old(Option<super::Editor>),
        }

        #[derive(Deserialize)]
        enum EditorChoiceInner {
            SystemDefault,
            Warp,
            EnvEditor,
            ExternalEditor(super::Editor),
        }

        match EditorChoiceCompat::deserialize(deserializer)? {
            EditorChoiceCompat::New(inner) => match inner {
                EditorChoiceInner::SystemDefault => Ok(EditorChoice::SystemDefault),
                EditorChoiceInner::Warp => Ok(EditorChoice::Warp),
                EditorChoiceInner::EnvEditor => Ok(EditorChoice::EnvEditor),
                EditorChoiceInner::ExternalEditor(editor) => {
                    Ok(EditorChoice::ExternalEditor(editor))
                }
            },
            EditorChoiceCompat::Old(old_value) => match old_value {
                None => Ok(EditorChoice::SystemDefault),
                Some(editor) => Ok(EditorChoice::ExternalEditor(editor)),
            },
        }
    }
}

define_settings_group!(EditorSettings, settings: [
    open_file_editor: OpenFileEditor {
        type: EditorChoice,
        default: EditorChoice::SystemDefault,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "code.editor.open_file_editor",
        max_table_depth: 0,
        description: "用于打开文件的编辑器。",
    },
    open_code_panels_file_editor: OpenCodePanelsFileEditor {
        type: EditorChoice,
        default: EditorChoice::Warp,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Never,
        private: false,
        toml_path: "code.editor.open_code_panels_file_editor",
        max_table_depth: 0,
        description: "用于从代码面板打开文件的编辑器。",
    },
    open_file_layout: OpenFileLayout {
        type: EditorLayout,
        default: EditorLayout::SplitPane,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.editor.open_file_layout",
        description: "在编辑器中打开文件时使用的布局。",
    },
    prefer_markdown_viewer: PreferMarkdownViewer {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.editor.prefer_markdown_viewer",
        description: "打开 Markdown 文件时是否使用 Markdown 查看器。",
    },
    prefer_tabbed_editor_view: PreferTabbedEditorView {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "code.editor.prefer_tabbed_editor_view",
        description: "是否优先在标签式编辑器视图中打开文件。",
    },
    open_conversation_layout_preference: OpenConversationLayoutPreference {
        type: OpenConversationPreference,
        default: OpenConversationPreference::NewTab,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "agents.warp_agent.other.open_conversation_layout_preference",
        description: "智能体对话是在新标签页还是拆分面板中打开。",
    },
]);

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    settings_value::SettingsValue,
)]
#[schemars(description = "打开智能体对话的方式。", rename_all = "snake_case")]
pub enum OpenConversationPreference {
    NewTab,
    SplitPane,
}

impl OpenConversationPreference {
    pub fn is_new_tab(&self) -> bool {
        matches!(self, Self::NewTab)
    }
}
