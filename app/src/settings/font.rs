use warp_core::ui::builder::MIN_FONT_SIZE;
use warpui::{fonts::Weight, rendering::ThinStrokes, AppContext, SingletonEntity};

use settings::{
    macros::define_settings_group, RespectUserSyncSetting, Setting, SupportedPlatforms, SyncToCloud,
};
use warpui::elements::DEFAULT_UI_LINE_HEIGHT_RATIO;

use super::EnforceMinimumContrast as EnforceMinimumContrastEnum;

pub const DEFAULT_MONOSPACE_FONT_NAME: &str = "Hack";
pub const DEFAULT_MONOSPACE_FONT_SIZE: f32 = 13.0;
pub const DEFAULT_MONOSPACE_FONT_WEIGHT: Weight = Weight::Normal;

define_settings_group!(FontSettings,
    settings: [
        monospace_font_name: MonospaceFontName {
            type: String,
            default: DEFAULT_MONOSPACE_FONT_NAME.to_string(),
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            storage_key: "FontName",
            toml_path: "appearance.text.font_name",
            description: "终端中使用的等宽字体。",
        },
        monospace_font_size: MonospaceFontSize {
            type: f32,
            default: DEFAULT_MONOSPACE_FONT_SIZE,
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            storage_key: "FontSize",
            toml_path: "appearance.text.font_size",
            description: "终端等宽字体大小。",
        },
        monospace_font_weight: MonospaceFontWeight {
            type: Weight,
            default: DEFAULT_MONOSPACE_FONT_WEIGHT,
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            storage_key: "FontWeight",
            toml_path: "appearance.text.font_weight",
            description: "终端等宽字体字重。",
        },
        line_height_ratio: LineHeightRatio {
            type: f32,
            default: DEFAULT_UI_LINE_HEIGHT_RATIO,
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            toml_path: "appearance.text.line_height_ratio",
            description: "终端文本行高比例。",
        },
        ai_font_name: AIFontName {
            type: String,
            default: DEFAULT_MONOSPACE_FONT_NAME.to_string(),
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            storage_key: "AIFontName",
            toml_path: "appearance.text.ai_font_name",
            description: "AI 生成内容使用的字体。",
        },
        match_ai_font_to_terminal_font: MatchAIFontToTerminalFont {
            type: bool,
            default: false,
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            storage_key: "MatchAIFont",
            toml_path: "appearance.text.match_ai_font",
            description: "AI 字体是否自动匹配终端字体。",
        },
        notebook_font_size: NotebookFontSize {
            type: f32,
            default: 14.0,
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            toml_path: "appearance.text.notebook_font_size",
            description: "笔记本中使用的字体大小。",
        },
        match_notebook_to_monospace_font_size: MatchNotebookToMonospaceFontSize {
            type: bool,
            default: true,
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
            private: false,
            toml_path: "appearance.text.match_notebook_to_monospace_font_size",
            description: "笔记本字体大小是否匹配终端字体大小。",
        },
        enforce_minimum_contrast: EnforceMinimumContrast {
            type: EnforceMinimumContrastEnum,
            default: EnforceMinimumContrastEnum::default(),
            supported_platforms: SupportedPlatforms::ALL,
            sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
            private: false,
            toml_path: "appearance.text.enforce_minimum_contrast",
            description: "是否强制最低对比度以确保文本可读性。",
        },
        use_thin_strokes: UseThinStrokes {
            type: ThinStrokes,
            default: ThinStrokes::default(),
            supported_platforms: SupportedPlatforms::MAC,
            sync_to_cloud: SyncToCloud::Never,
            private: false,
            toml_path: "appearance.text.use_thin_strokes",
            description: "是否在 macOS 上使用细字体笔画。",
        },
    ]
);

const MAX_NOTEBOOK_FONT_SIZE: f32 = 25.0;
const NOTEBOOK_FONT_SIZE_INCREMENT: f32 = 1.0;

pub fn increase_notebook_font_size(ctx: &mut AppContext) -> anyhow::Result<()> {
    adjust_notebook_font_size(NOTEBOOK_FONT_SIZE_INCREMENT, ctx)
}

pub fn decrease_notebook_font_size(ctx: &mut AppContext) -> anyhow::Result<()> {
    adjust_notebook_font_size(-NOTEBOOK_FONT_SIZE_INCREMENT, ctx)
}

fn adjust_notebook_font_size(delta: f32, ctx: &mut AppContext) -> anyhow::Result<()> {
    let current_size = derived_notebook_font_size(FontSettings::as_ref(ctx));
    let new_font_size = (current_size + delta).clamp(MIN_FONT_SIZE, MAX_NOTEBOOK_FONT_SIZE);
    FontSettings::handle(ctx).update(ctx, |font_settings, ctx| {
        font_settings
            .notebook_font_size
            .set_value(new_font_size, ctx)
    })
}

pub fn derived_notebook_font_size(font_settings: &FontSettings) -> f32 {
    if *font_settings.match_notebook_to_monospace_font_size {
        *font_settings.monospace_font_size
    } else {
        *font_settings.notebook_font_size
    }
}
