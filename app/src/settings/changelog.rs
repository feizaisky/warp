use settings::{
    macros::define_settings_group, RespectUserSyncSetting, SupportedPlatforms, SyncToCloud,
};

define_settings_group!(ChangelogSettings, settings: [
   show_changelog_after_update: ShowChangelogAfterUpdate {
       type: bool,
       default: true,
       supported_platforms: SupportedPlatforms::ALL,
       sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
       private: false,
       toml_path: "general.show_changelog_after_update",
       description: "更新后是否显示更新日志。",
   },
]);
