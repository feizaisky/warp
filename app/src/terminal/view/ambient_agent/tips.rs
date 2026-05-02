//! Tips for cloud mode loading screen.

use crate::ai::agent_tips::AITip;
use warpui::keymap::Keystroke;
use warpui::AppContext;

/// A cloud mode tip with text and optional link.
#[derive(Clone, Debug)]
pub struct CloudModeTip {
    text: String,
    link: Option<String>,
}

impl CloudModeTip {
    pub fn new(text: impl Into<String>, link: Option<impl Into<String>>) -> Self {
        Self {
            text: text.into(),
            link: link.map(|l| l.into()),
        }
    }
}

impl AITip for CloudModeTip {
    fn keystroke(&self, _app: &AppContext) -> Option<Keystroke> {
        None
    }

    fn link(&self) -> Option<String> {
        self.link.clone()
    }

    fn description(&self) -> &str {
        &self.text
    }

    // Uses the default implementation which adds "Tip: " prefix and parses backticks as inline code
}

/// Returns a collection of tips for the cloud mode loading screen.
pub fn get_cloud_mode_tips() -> Vec<CloudModeTip> {
    vec![
        CloudModeTip::new(
            "安装 Oz Slack 集成，可从任意频道或私信触发智能体。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/slack"),
        ),
        CloudModeTip::new(
            "以编程方式使用 Oz 的 TypeScript 和 Python SDK 构建智能体。",
            Some("https://docs.warp.dev/reference/api-and-sdk"),
        ),
        CloudModeTip::new(
            "使用 `oz secret` 命令为智能体设置团队或个人密钥。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/secrets"),
        ),
        CloudModeTip::new(
            "在 Oz Web 应用中查看所有智能体运行及其状态。",
            Some("https://oz.warp.dev"),
        ),
        CloudModeTip::new(
            "使用智能体会话共享实时加入任意 Oz 云端智能体运行。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/viewing-cloud-agent-runs"),
        ),
        CloudModeTip::new(
            "设置按 cron 定时运行的周期性智能体，用于自动维护。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents"),
        ),
        CloudModeTip::new(
            "创建智能体，在 Linear 中提交 issue 时自动修复缺陷。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/linear"),
        ),
        CloudModeTip::new(
            "构建可响应 CI 失败并尝试自动修复的智能体。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions"),
        ),
        CloudModeTip::new(
            "使用 `oz-agent-action` 从 GitHub Actions 运行智能体。",
            Some("https://github.com/warpdotdev/oz-agent-action"),
        ),
        CloudModeTip::new(
            "调用 Oz REST API，从任意后端服务或内部工具触发智能体。",
            Some("https://docs.warp.dev/reference/api-and-sdk"),
        ),
        CloudModeTip::new(
            "使用 Docker 镜像创建可复用环境，确保智能体稳定执行。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/environments"),
        ),
        CloudModeTip::new(
            "与团队共享智能体会话链接，协作调试。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/viewing-cloud-agent-runs"),
        ),
        CloudModeTip::new(
            "使用 Oz CLI 的 `--share` 标志从任意位置启用会话共享。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/platform"),
        ),
        CloudModeTip::new(
            "将已完成的 Oz 云端智能体会话分叉到 Warp 中，以便在本地继续工作。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/viewing-cloud-agent-runs"),
        ),
        CloudModeTip::new(
            "构建使用智能体从数据库回答问题的内部工具。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations"),
        ),
        CloudModeTip::new(
            "创建定时智能体，每周自动清理过期功能标志。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents"),
        ),
        CloudModeTip::new(
            "在 Linear issue 中标记 @Oz，可自动调查并提出修复方案。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/linear"),
        ),
        CloudModeTip::new(
            "使用 Oz CLI 在远程开发机或 CI Runner 上运行智能体。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/platform"),
        ),
        CloudModeTip::new(
            "配置 MCP 服务器，让 Oz 云端智能体访问 GitHub、Linear 和 Sentry。",
            Some("https://docs.warp.dev/agent-platform/capabilities/mcp"),
        ),
        CloudModeTip::new(
            "使用 `oz agent run` 启动任务，无需打开 Warp 终端。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/platform"),
        ),
        CloudModeTip::new(
            "在 Oz Web 应用中查看队友的智能体运行，方便共享可见性。",
            Some("https://oz.warp.dev"),
        ),
        CloudModeTip::new(
            "构建可自动分诊并标记传入 GitHub issue 的智能体。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions"),
        ),
        CloudModeTip::new(
            "设置智能体，每天生成新开 issue 摘要。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions"),
        ),
        CloudModeTip::new(
            "创建智能体，自动审查 PR 并提出改进建议。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/github-actions"),
        ),
        CloudModeTip::new(
            "使用 `oz environment create` 定义可复现的执行上下文。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/environments"),
        ),
        CloudModeTip::new(
            "通过 webhook 触发智能体以响应生产事故。",
            Some("https://docs.warp.dev/reference/api-and-sdk"),
        ),
        CloudModeTip::new(
            "构建智能体，在告警触发时重启服务或扩缩容部署。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/triggers"),
        ),
        CloudModeTip::new(
            "使用个人密钥保存仅供您的智能体使用的凭据。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/secrets"),
        ),
        CloudModeTip::new(
            "使用团队密钥在所有智能体之间共享基础设施凭据。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/secrets"),
        ),
        CloudModeTip::new(
            "创建每日执行的智能体，自动检查依赖更新。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents"),
        ),
        CloudModeTip::new(
            "构建按计划自动格式化和 lint 代码的智能体。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents"),
        ),
        CloudModeTip::new(
            "使用 `oz schedule create` 设置 cron 触发的智能体。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents"),
        ),
        CloudModeTip::new(
            "使用 `oz schedule pause` 暂停和恢复定时智能体，而无需删除。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/triggers/scheduled-agents"),
        ),
        CloudModeTip::new(
            "使用 `oz mcp list` 查看您的智能体可用的 MCP 服务器。",
            Some("https://docs.warp.dev/agent-platform/capabilities/mcp"),
        ),
        CloudModeTip::new(
            "构建内部 Slack 机器人，将编码任务委托给 Oz 智能体。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/slack"),
        ),
        CloudModeTip::new(
            "创建智能体，在 Slack 话题中响应 @提及并提供完整上下文。",
            Some("https://docs.warp.dev/agent-platform/cloud-agents/integrations/slack"),
        ),
        CloudModeTip::new(
            "使用 Oz TypeScript SDK 构建自定义自动化流水线。",
            Some("https://docs.warp.dev/reference/api-and-sdk"),
        ),
        CloudModeTip::new(
            "使用 Oz Python SDK 将智能体集成到您的数据流水线。",
            Some("https://docs.warp.dev/reference/api-and-sdk"),
        ),
        CloudModeTip::new(
            "使用 Oz API 监控智能体成功率和运行时长。",
            Some("https://docs.warp.dev/reference/api-and-sdk"),
        ),
        CloudModeTip::new(
            "构建仪表板，跟踪团队中的所有智能体活动。",
            Some("https://docs.warp.dev/reference/api-and-sdk"),
        ),
    ]
}
