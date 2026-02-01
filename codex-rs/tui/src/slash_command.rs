use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model,
    Approvals,
    Permissions,
    #[strum(serialize = "setup-elevated-sandbox")]
    ElevateSandbox,
    Experimental,
    Skills,
    Review,
    Rename,
    New,
    Resume,
    Fork,
    Init,
    Compact,
    Plan,
    Collab,
    Agent,
    // Undo,
    Diff,
    Mention,
    Status,
    Mcp,
    Apps,
    Logout,
    Quit,
    Exit,
    Feedback,
    Rollout,
    Ps,
    Personality,
    TestApproval,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Feedback => "发送日志给维护者",
            SlashCommand::New => "在对话中开启新聊天",
            SlashCommand::Init => "创建包含 Codex 指令的 AGENTS.md 文件",
            SlashCommand::Compact => "总结对话以避免触及上下文上限",
            SlashCommand::Review => "审查当前改动并找出问题",
            SlashCommand::Rename => "重命名当前会话",
            SlashCommand::Resume => "恢复已保存的聊天",
            SlashCommand::Fork => "分叉当前聊天",
            // SlashCommand::Undo => "ask Codex to undo a turn",
            SlashCommand::Quit | SlashCommand::Exit => "退出 Codex",
            SlashCommand::Diff => "显示 git diff（包含未跟踪文件）",
            SlashCommand::Mention => "提及文件",
            SlashCommand::Skills => "使用技能提升 Codex 执行特定任务的效果",
            SlashCommand::Status => "显示当前会话配置与 token 用量",
            SlashCommand::Ps => "列出后台终端",
            SlashCommand::Model => "选择模型与推理强度",
            SlashCommand::Personality => "选择 Codex 的交流风格",
            SlashCommand::Plan => "切换到计划模式",
            SlashCommand::Collab => "切换协作模式（实验性）",
            SlashCommand::Agent => "切换当前代理线程",
            SlashCommand::Approvals => "选择 Codex 可在无需批准时执行的操作",
            SlashCommand::Permissions => "选择 Codex 允许执行的操作",
            SlashCommand::ElevateSandbox => "配置提升权限的代理沙箱",
            SlashCommand::Experimental => "切换实验功能",
            SlashCommand::Mcp => "列出已配置的 MCP 工具",
            SlashCommand::Apps => "管理 Apps（连接器）",
            SlashCommand::Logout => "登出 Codex",
            SlashCommand::Rollout => "打印 rollout 文件路径",
            SlashCommand::TestApproval => "测试审批请求",
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::New
            | SlashCommand::Resume
            | SlashCommand::Fork
            | SlashCommand::Init
            | SlashCommand::Compact
            // | SlashCommand::Undo
            | SlashCommand::Model
            | SlashCommand::Personality
            | SlashCommand::Approvals
            | SlashCommand::Permissions
            | SlashCommand::ElevateSandbox
            | SlashCommand::Experimental
            | SlashCommand::Review
            | SlashCommand::Logout => false,
            SlashCommand::Diff
            | SlashCommand::Rename
            | SlashCommand::Mention
            | SlashCommand::Skills
            | SlashCommand::Status
            | SlashCommand::Ps
            | SlashCommand::Mcp
            | SlashCommand::Apps
            | SlashCommand::Feedback
            | SlashCommand::Quit
            | SlashCommand::Exit => true,
            SlashCommand::Rollout => true,
            SlashCommand::TestApproval => true,
            SlashCommand::Plan => true,
            SlashCommand::Collab => true,
            SlashCommand::Agent => true,
        }
    }

    fn is_visible(self) -> bool {
        match self {
            SlashCommand::Rollout | SlashCommand::TestApproval => cfg!(debug_assertions),
            _ => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter()
        .filter(|command| command.is_visible())
        .map(|c| (c.command(), c))
        .collect()
}
