use settings::{
    macros::define_settings_group, RespectUserSyncSetting, SupportedPlatforms, SyncToCloud,
};

// Settings for visibility of non-user command blocks like the bootstrap block
// and in-band command blocks.
define_settings_group!(BlockVisibilitySettings, settings: [
   should_show_bootstrap_block: ShouldShowBootstrapBlock {
       type: bool,
       default: false,
       supported_platforms: SupportedPlatforms::ALL,
       sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
       private: false,
       toml_path: "appearance.blocks.should_show_bootstrap_block",
       description: "终端中是否显示 bootstrap 块。",
   },
   should_show_in_band_command_blocks: ShouldShowInBandCommandBlocks {
       type: bool,
       default: false,
       supported_platforms: SupportedPlatforms::ALL,
       sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
       private: false,
       toml_path: "appearance.blocks.should_show_in_band_command_blocks",
       description: "终端中是否显示带内命令块。",
   },
   should_show_ssh_block: ShouldShowSSHBlock {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "appearance.blocks.should_show_ssh_block",
        description: "终端中是否显示 SSH 连接块。",
   }
]);
