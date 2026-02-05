use clap::Parser;
use clap::ValueHint;
use codex_common::ApprovalModeCliArg;
use codex_common::CliConfigOverrides;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    /// 可选的用户提示词，用于启动会话。
    #[arg(value_name = "提示词", value_hint = clap::ValueHint::Other)]
    pub prompt: Option<String>,

    /// 可选：为初始提示词附加图片。
    #[arg(long = "image", short = 'i', value_name = "文件", value_delimiter = ',', num_args = 1..)]
    pub images: Vec<PathBuf>,

    // Internal controls set by the top-level `codex resume` subcommand.
    // These are not exposed as user flags on the base `codex` command.
    #[clap(skip)]
    pub resume_picker: bool,

    #[clap(skip)]
    pub resume_last: bool,

    /// Internal: resume a specific recorded session by id (UUID). Set by the
    /// top-level `codex resume <SESSION_ID>` wrapper; not exposed as a public flag.
    #[clap(skip)]
    pub resume_session_id: Option<String>,

    /// Internal: show all sessions (disables cwd filtering and shows CWD column).
    #[clap(skip)]
    pub resume_show_all: bool,

    // Internal controls set by the top-level `codex fork` subcommand.
    // These are not exposed as user flags on the base `codex` command.
    #[clap(skip)]
    pub fork_picker: bool,

    #[clap(skip)]
    pub fork_last: bool,

    /// Internal: fork a specific recorded session by id (UUID). Set by the
    /// top-level `codex fork <SESSION_ID>` wrapper; not exposed as a public flag.
    #[clap(skip)]
    pub fork_session_id: Option<String>,

    /// Internal: show all sessions (disables cwd filtering and shows CWD column).
    #[clap(skip)]
    pub fork_show_all: bool,

    /// 代理应使用的模型。
    #[arg(long, short = 'm')]
    pub model: Option<String>,

    /// 快捷开关：选择本地开源模型提供方。等价于 -c
    /// model_provider=oss；并会校验本地 LM Studio 或 Ollama 服务是否在运行。
    #[arg(long = "oss", default_value_t = false)]
    pub oss: bool,

    /// 指定本地提供方（lmstudio 或 ollama）。
    /// 若未与 --oss 一起指定，则使用配置默认值或弹出选择。
    #[arg(long = "local-provider")]
    pub oss_provider: Option<String>,

    /// 从 config.toml 选择配置 profile 作为默认选项。
    #[arg(long = "profile", short = 'p')]
    pub config_profile: Option<String>,

    /// 选择执行模型生成的 shell
    /// 命令时使用的沙箱策略。
    #[arg(long = "sandbox", short = 's')]
    pub sandbox_mode: Option<codex_common::SandboxModeCliArg>,

    /// 配置在执行命令前何时需要人工审批。
    #[arg(long = "ask-for-approval", short = 'a')]
    pub approval_policy: Option<ApprovalModeCliArg>,

    /// 低摩擦的沙箱自动执行快捷别名（-a on-request，--sandbox workspace-write）。
    #[arg(long = "full-auto", default_value_t = false)]
    pub full_auto: bool,

    /// 跳过所有确认提示，并在无沙箱情况下执行命令。
    /// 极其危险。仅用于外部已经提供沙箱隔离的环境。
    #[arg(
        long = "dangerously-bypass-approvals-and-sandbox",
        alias = "yolo",
        default_value_t = false,
        conflicts_with_all = ["approval_policy", "full_auto"]
    )]
    pub dangerously_bypass_approvals_and_sandbox: bool,

    /// 指定代理的工作根目录。
    #[clap(long = "cd", short = 'C', value_name = "目录")]
    pub cwd: Option<PathBuf>,

    /// 启用实时联网搜索。启用后，模型可使用 Responses 原生的 `web_search` 工具（无需逐次审批）。
    #[arg(long = "search", default_value_t = false)]
    pub web_search: bool,

    /// 除主工作区外，额外允许写入的目录。
    #[arg(long = "add-dir", value_name = "目录", value_hint = ValueHint::DirPath)]
    pub add_dir: Vec<PathBuf>,

    /// 禁用备用屏幕模式
    ///
    /// 以行内模式运行 TUI，保留终端滚动回溯历史。这在
    /// 类似 Zellij 这类严格遵循 xterm 规范、并在备用屏幕缓冲区禁用
    /// 滚动回溯的终端复用器中很有用。
    #[arg(long = "no-alt-screen", default_value_t = false)]
    pub no_alt_screen: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,
}
