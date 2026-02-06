//! Standard type to use with the `--approval-mode` CLI option.
//! Available when the `cli` feature is enabled for the crate.

use clap::ValueEnum;

use codex_core::protocol::AskForApproval;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ApprovalModeCliArg {
    /// 不询问即可运行“可信”命令（例如：ls、cat、sed）。
    /// 若模型提出的命令不在“可信”集合中，将升级为向用户请求批准。
    Untrusted,

    /// 不询问即可运行所有命令。
    /// 仅当命令执行失败时才会请求批准；此时会升级为请求在未沙箱限制下执行。
    OnFailure,

    /// 由模型决定何时向用户请求批准。
    OnRequest,

    /// 从不向用户请求批准。
    /// 任何执行失败都会立即返回给模型。
    Never,
}

impl From<ApprovalModeCliArg> for AskForApproval {
    fn from(value: ApprovalModeCliArg) -> Self {
        match value {
            ApprovalModeCliArg::Untrusted => AskForApproval::UnlessTrusted,
            ApprovalModeCliArg::OnFailure => AskForApproval::OnFailure,
            ApprovalModeCliArg::OnRequest => AskForApproval::OnRequest,
            ApprovalModeCliArg::Never => AskForApproval::Never,
        }
    }
}
