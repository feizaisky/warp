use serde::Serialize;
use warpui::{elements::MouseStateHandle, notification::RequestPermissionsOutcome, Element};

use crate::{
    appearance::Appearance,
    terminal::{
        session_settings::NotificationsMode,
        view::{InlineBannerId, NotificationsTrigger, TerminalAction},
    },
};

use super::{
    render_inline_block_list_banner, InlineBannerButtonState, InlineBannerCloseButton,
    InlineBannerContent, InlineBannerStyle, InlineBannerTextButton, InlineBannerTextButtonVariant,
};

#[derive(Clone, Copy, Debug, Serialize)]
pub enum NotificationsDiscoveryBannerAction {
    LearnMore,
    Troubleshoot,
    TurnOn(NotificationsTrigger),
    Configure,
    Close,
}

#[derive(Default)]
pub struct NotificationsDiscoveryBannerMouseStates {
    pub learn_more: MouseStateHandle,
    pub troubleshoot: MouseStateHandle,
    pub turn_on: MouseStateHandle,
    pub configure: MouseStateHandle,
    pub close: MouseStateHandle,
}

/// State necessary to render the (singleton) notifications discovery banner.
pub struct NotificationsDiscoveryBannerState {
    pub banner_id: InlineBannerId,
    pub mouse_states: NotificationsDiscoveryBannerMouseStates,
}

pub fn render_inline_notifications_discovery_banner(
    trigger: NotificationsTrigger,
    request_outcome: Option<RequestPermissionsOutcome>,
    state: &NotificationsDiscoveryBannerState,
    notifications_mode: NotificationsMode,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let active_ui_text_color = appearance.theme().active_ui_text_color().into_solid();

    let learn_more_button = InlineBannerTextButton {
        text: "了解更多".to_string(),
        text_color: active_ui_text_color,
        button_state: InlineBannerButtonState {
            on_click_event: TerminalAction::NotificationsDiscoveryBanner(
                NotificationsDiscoveryBannerAction::LearnMore,
            ),
            mouse_state_handle: state.mouse_states.learn_more.clone(),
        },
        font: Default::default(),
        position_id: None,
        variant: InlineBannerTextButtonVariant::Secondary,
    };
    let troubleshoot_button = InlineBannerTextButton {
        text: "排查问题".to_string(),
        text_color: active_ui_text_color,
        button_state: InlineBannerButtonState {
            on_click_event: TerminalAction::NotificationsDiscoveryBanner(
                NotificationsDiscoveryBannerAction::Troubleshoot,
            ),
            mouse_state_handle: state.mouse_states.troubleshoot.clone(),
        },
        font: Default::default(),
        position_id: None,
        variant: InlineBannerTextButtonVariant::Secondary,
    };

    let (title, buttons) = match notifications_mode {
        NotificationsMode::Dismissed => (
            "我们不会再显示此横幅，但您随时可以前往设置启用通知。",
            vec![],
        ),
        NotificationsMode::Disabled => (
            "通知已关闭，但您随时可以前往设置启用通知。",
            vec![],
        ),
        NotificationsMode::Unset => (
            trigger.discovery_banner_copy(),
            vec![
                learn_more_button,
                InlineBannerTextButton {
                    text: "启用".to_string(),
                    text_color: active_ui_text_color,
                    button_state: InlineBannerButtonState {
                        on_click_event: TerminalAction::NotificationsDiscoveryBanner(
                            NotificationsDiscoveryBannerAction::TurnOn(trigger),
                        ),
                        mouse_state_handle: state.mouse_states.turn_on.clone(),
                    },
                    font: Default::default(),
                    position_id: None,
                    variant: InlineBannerTextButtonVariant::Primary,
                },
            ],
        ),
        NotificationsMode::Enabled => {
            // Determine the messaging based on what the user's response was to the
            // permissions request (if any)
            let (title, docs_button) = match request_outcome {
                Some(request_outcome) => match request_outcome {
                    RequestPermissionsOutcome::Accepted => (
                        "成功！您现在可以接收桌面通知了。",
                        learn_more_button,
                    ),
                    RequestPermissionsOutcome::PermissionsDenied => (
                        "Warp 被拒绝发送通知的权限。",
                        troubleshoot_button,
                    ),
                    RequestPermissionsOutcome::OtherError { .. } => (
                        "申请权限时发生错误。",
                        troubleshoot_button,
                    ),
                },
                None => (
                    "请记得在权限请求弹窗中点击「允许」以完成通知设置。",
                    learn_more_button,
                ),
            };

            (
                title,
                vec![
                    docs_button,
                    InlineBannerTextButton {
                        text: "配置通知".to_string(),
                        text_color: active_ui_text_color,
                        button_state: InlineBannerButtonState {
                            on_click_event: TerminalAction::NotificationsDiscoveryBanner(
                                NotificationsDiscoveryBannerAction::Configure,
                            ),
                            mouse_state_handle: state.mouse_states.configure.clone(),
                        },
                        font: Default::default(),
                        position_id: None,
                        variant: InlineBannerTextButtonVariant::Secondary,
                    },
                ],
            )
        }
    };

    let close_button = InlineBannerCloseButton(InlineBannerButtonState {
        on_click_event: TerminalAction::NotificationsDiscoveryBanner(
            NotificationsDiscoveryBannerAction::Close,
        ),
        mouse_state_handle: state.mouse_states.close.clone(),
    });

    render_inline_block_list_banner(
        InlineBannerStyle::CallToAction,
        appearance,
        InlineBannerContent {
            title: title.to_owned(),
            buttons,
            close_button: Some(close_button),
            ..Default::default()
        },
    )
}
