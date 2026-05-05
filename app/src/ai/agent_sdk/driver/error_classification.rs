use crate::ai::blocklist::task_status_sync_model::classify_renderable_error;
use crate::server::server_api::ai::TaskStatusUpdate;
use warp_graphql::ai::{AgentTaskState, PlatformErrorCode};

use super::terminal::ShareSessionError;
use super::AgentDriverError;

/// Classify an `AgentDriverError` into a task state and a `TaskStatusUpdate`
/// suitable for reporting via `update_agent_task`.
pub fn classify_driver_error(error: &AgentDriverError) -> (AgentTaskState, TaskStatusUpdate) {
    match error {
        // --- Warp-side errors (task → ERROR) ---
        AgentDriverError::TerminalUnavailable | AgentDriverError::InvalidRuntimeState => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                "发生内部错误。请重新运行任务。如果问题仍然存在，请联系支持团队。",
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::BootstrapFailed => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                "终端会话启动失败。请重新运行任务。",
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::ShareSessionFailed { error: share_err } => {
            let message = match share_err {
                ShareSessionError::Internal(_) => {
                    "由于内部错误，共享智能体会话失败。请重新运行任务。".to_string()
                }
                ShareSessionError::Failed(reason) => {
                    // The reason string comes from the session-sharing layer and is aimed at
                    // interactive users (e.g. "try sharing again"). Provide a cloud-agent-
                    // appropriate message instead of wrapping it, which would produce
                    // repetitive "try again" text.
                    format!("共享智能体会话失败：{reason}")
                }
                ShareSessionError::Disabled => {
                    "你的账户未启用会话共享。这可能是因为管理员为你的团队停用了会话共享。请确认团队设置中已启用会话共享，或尝试不带 --share 标志运行。"
                    .to_string()
                }
                ShareSessionError::Timeout => {
                    "共享智能体会话失败：等待会话共享服务器响应超时。请检查网络连接后重试。"
                    .to_string()
                }
                ShareSessionError::Interrupted => {
                    "会话共享尚未完成就被中断。请重新运行任务。".to_string()
                }
            };
            (
                AgentTaskState::Error,
                TaskStatusUpdate::with_error_code(
                    message,
                    match share_err {
                        ShareSessionError::Disabled => PlatformErrorCode::FeatureNotAvailable,
                        _ => PlatformErrorCode::InternalError,
                    },
                ),
            )
        }
        AgentDriverError::WarpDriveSyncFailed => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                "Warp 云盘同步失败。请检查网络连接后重试。",
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::NotLoggedIn => {
            let bin = warp_cli::binary_name().unwrap_or_else(|| "warp".to_string());
            (
                AgentTaskState::Error,
                TaskStatusUpdate::with_error_code(
                    format!(
                        "Authentication required. Log in via '{bin} login', provide an API key via '--api-key', or set the WARP_API_KEY environment variable."
                    ),
                    PlatformErrorCode::AuthenticationRequired,
                ),
            )
        }
        AgentDriverError::CloudProviderSetupFailed(err) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                format!("Error configuring cloud access: {err:#}"),
                PlatformErrorCode::InternalError,
            ),
        ),

        // --- User-side errors (task → FAILED) ---
        AgentDriverError::MCPServerNotFound(uuid) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "MCP server {uuid} was not found. Verify the server exists in your Warp Drive and the UUID is correct."
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::MCPStartupFailed => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                "One or more MCP servers failed to start. Check that your MCP server configuration is valid and the server process is runnable.",
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::MCPJsonParseError(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Failed to parse MCP server JSON configuration: {msg}"),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::MCPMissingVariables => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                "MCP server configuration is missing required variables. Provide all required environment variables or template values.",
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::ProfileError(name) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Agent profile \"{name}\" not found. Check the profile ID and ensure it exists in your team's Warp Drive."
                ),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::AIWorkflowNotFound(id) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Saved prompt not found for ID {id}. Verify the prompt exists in your Warp Drive."
                ),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::EnvironmentNotFound(id) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Environment '{id}' not found. Verify the environment ID and ensure it exists in your team settings."
                ),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::EnvironmentSetupFailed(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Environment setup failed: {msg}. Check your repository URLs and setup commands."
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::InvalidWorkingDirectory { path, .. } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Working directory '{}' does not exist or is not a directory. Verify the path in your environment configuration.",
                    path.display()
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),

        // --- Conversation errors ---
        // Delegate to classify_renderable_error for proper ERROR vs FAILED
        // distinction and PlatformErrorCode. This is a belt-and-suspenders
        // fallback — TaskStatusSyncModel handles most conversation errors,
        // but the driver catches them too if the conversation ends with an error.
        AgentDriverError::ConversationError { error } => {
            let (state, update) = classify_renderable_error(error);
            (
                state,
                update.unwrap_or_else(|| {
                    TaskStatusUpdate::with_error_code(
                        error.to_string(),
                        PlatformErrorCode::InternalError,
                    )
                }),
            )
        }

        // --- Cancellation / Blocked (no error code) ---
        AgentDriverError::ConversationCancelled { .. } => (
            AgentTaskState::Cancelled,
            TaskStatusUpdate::message("Task cancelled."),
        ),
        AgentDriverError::ConversationBlocked { blocked_action } => (
            AgentTaskState::Blocked,
            TaskStatusUpdate::message(format!(
                "The agent got stuck waiting for user confirmation on the action: {blocked_action}"
            )),
        ),

        // --- Setup errors ---
        AgentDriverError::TeamMetadataRefreshTimeout => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                "刷新团队元数据超时。请检查网络连接后重试。",
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::SkillResolutionFailed(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Skill resolution failed: {msg}"),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::ConfigBuildFailed(err) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Failed to build agent configuration: {err}"),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::PromptResolutionFailed(err) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                format!("Failed to resolve prompt for the run: {err}"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::SecretsFetchFailed(err) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                format!("Failed to fetch task secrets: {err}"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::AwsBedrockCredentialsFailed(msg) => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Failed to initialize AWS Bedrock credentials: {msg}"),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::ConversationLoadFailed(msg) => (
            AgentTaskState::Error,
            TaskStatusUpdate::with_error_code(
                format!("Failed to load conversation: {msg}"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::ConversationHarnessMismatch { conversation_id, expected, got } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Conversation {conversation_id} was produced by the {expected} harness, but --harness {got} was requested. \
                     Re-run with --harness {expected} (or omit --harness) to continue this conversation."
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::TaskHarnessMismatch { task_id, expected, got } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Task {task_id} was created with the {expected} harness, but --harness {got} was requested. \
                     Re-run with --harness {expected} (or omit --harness) to continue this task."
                ),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::ConversationResumeStateMissing { harness, conversation_id } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!(
                    "Conversation {conversation_id} has no stored transcript for the {harness} harness. \
                     The prior run may have crashed before saving any state."
                ),
                PlatformErrorCode::ResourceNotFound,
            ),
        ),
        AgentDriverError::HarnessCommandFailed { exit_code } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Harness command exited with code {exit_code}"),
                PlatformErrorCode::InternalError,
            ),
        ),
        AgentDriverError::HarnessSetupFailed { harness, reason } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Harness '{harness}' validation failed: {reason}"),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
        AgentDriverError::HarnessConfigSetupFailed { harness, error } => (
            AgentTaskState::Failed,
            TaskStatusUpdate::with_error_code(
                format!("Harness '{harness}' config setup failed: {error}"),
                PlatformErrorCode::EnvironmentSetupFailed,
            ),
        ),
    }
}

#[cfg(test)]
#[path = "error_classification_tests.rs"]
mod tests;
