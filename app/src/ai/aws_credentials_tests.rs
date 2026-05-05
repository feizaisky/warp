use std::time::Duration;

use aws_credential_types::provider::error::CredentialsError;

use super::user_facing_aws_credentials_error_message;

#[test]
fn maps_credentials_not_loaded_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::not_loaded_no_source(),
        "sandbox",
    );

    assert_eq!(
        message,
        "未找到 AWS profile `sandbox` 的 AWS 凭据。请使用 AWS CLI 登录，或更新你的 AWS 凭据配置，然后刷新。"
    );
}

#[test]
fn maps_invalid_configuration_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::invalid_configuration(std::io::Error::other("bad config")),
        "readonly",
    );

    assert_eq!(
        message,
        "AWS profile `readonly` 在本地 AWS 配置中无效或不完整。请更新 AWS profile 设置和凭据，然后刷新。"
    );
}

#[test]
fn maps_provider_timeout_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::provider_timed_out(Duration::from_secs(5)),
        "sandbox",
    );

    assert_eq!(message, "加载 AWS 凭据超时。请刷新后重试。");
}

#[test]
fn maps_provider_error_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::provider_error(std::io::Error::other("provider error")),
        "sandbox",
    );

    assert_eq!(
        message,
        "无法从配置的提供方加载 AWS 凭据。请刷新 AWS 登录后重试。"
    );
}

#[test]
fn maps_unhandled_error_to_user_message() {
    let message = user_facing_aws_credentials_error_message(
        &CredentialsError::unhandled(std::io::Error::other("unexpected")),
        "sandbox",
    );

    assert_eq!(
        message,
        "加载 AWS 凭据时发生意外错误。请刷新 AWS 登录后重试。"
    );
}
