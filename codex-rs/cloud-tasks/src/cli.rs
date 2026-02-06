use clap::Args;
use clap::Parser;
use codex_common::CliConfigOverrides;

#[derive(Parser, Debug, Default)]
#[command(version)]
pub struct Cli {
    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// 无需启动 TUI，提交新的 Codex Cloud 任务。
    Exec(ExecCommand),
    /// 查看 Codex Cloud 任务状态。
    Status(StatusCommand),
    /// 列出 Codex Cloud 任务。
    List(ListCommand),
    /// 将 Codex Cloud 任务的 diff 应用到本地。
    Apply(ApplyCommand),
    /// 显示 Codex Cloud 任务的统一 diff。
    Diff(DiffCommand),
}

#[derive(Debug, Args)]
pub struct ExecCommand {
    /// 要在 Codex Cloud 中运行的任务提示。
    #[arg(value_name = "QUERY")]
    pub query: Option<String>,

    /// 目标环境标识符（运行 `codex2 cloud` 可浏览）。
    #[arg(long = "env", value_name = "ENV_ID")]
    pub environment: String,

    /// 助手尝试次数（best-of-N）。
    #[arg(
        long = "attempts",
        default_value_t = 1usize,
        value_parser = parse_attempts
    )]
    pub attempts: usize,

    /// 在 Codex Cloud 中运行的 Git 分支（默认当前分支）。
    #[arg(long = "branch", value_name = "BRANCH")]
    pub branch: Option<String>,
}

fn parse_attempts(input: &str) -> Result<usize, String> {
    let value: usize = input
        .parse()
        .map_err(|_| "必须是 1 到 4 之间的整数".to_string())?;
    if (1..=4).contains(&value) {
        Ok(value)
    } else {
        Err("必须在 1 到 4 之间".to_string())
    }
}

fn parse_limit(input: &str) -> Result<i64, String> {
    let value: i64 = input
        .parse()
        .map_err(|_| "必须是 1 到 20 之间的整数".to_string())?;
    if (1..=20).contains(&value) {
        Ok(value)
    } else {
        Err("必须在 1 到 20 之间".to_string())
    }
}

#[derive(Debug, Args)]
pub struct StatusCommand {
    /// 要查看的 Codex Cloud 任务 ID。
    #[arg(value_name = "TASK_ID")]
    pub task_id: String,
}

#[derive(Debug, Args)]
pub struct ListCommand {
    /// 按环境标识符过滤任务。
    #[arg(long = "env", value_name = "ENV_ID")]
    pub environment: Option<String>,

    /// 返回的最大任务数（1-20）。
    #[arg(long = "limit", default_value_t = 20, value_parser = parse_limit, value_name = "N")]
    pub limit: i64,

    /// 上一次调用返回的分页游标。
    #[arg(long = "cursor", value_name = "CURSOR")]
    pub cursor: Option<String>,

    /// 输出 JSON（而非纯文本）。
    #[arg(long = "json", default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ApplyCommand {
    /// 要应用的 Codex Cloud 任务 ID。
    #[arg(value_name = "TASK_ID")]
    pub task_id: String,

    /// 要应用的尝试序号（从 1 开始）。
    #[arg(long = "attempt", value_parser = parse_attempts, value_name = "N")]
    pub attempt: Option<usize>,
}

#[derive(Debug, Args)]
pub struct DiffCommand {
    /// 要展示的 Codex Cloud 任务 ID。
    #[arg(value_name = "TASK_ID")]
    pub task_id: String,

    /// 要展示的尝试序号（从 1 开始）。
    #[arg(long = "attempt", value_parser = parse_attempts, value_name = "N")]
    pub attempt: Option<usize>,
}
