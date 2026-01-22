use codex_core::protocol::AskForApproval;
use codex_core::protocol::SandboxPolicy;

/// A simple preset pairing an approval policy with a sandbox policy.
#[derive(Debug, Clone)]
pub struct ApprovalPreset {
    /// Stable identifier for the preset.
    pub id: &'static str,
    /// Display label shown in UIs.
    pub label: &'static str,
    /// Short human description shown next to the label in UIs.
    pub description: &'static str,
    /// Approval policy to apply.
    pub approval: AskForApproval,
    /// Sandbox policy to apply.
    pub sandbox: SandboxPolicy,
}

/// Built-in list of approval presets that pair approval and sandbox policy.
///
/// Keep this UI-agnostic so it can be reused by both TUI and MCP server.
pub fn builtin_approval_presets() -> Vec<ApprovalPreset> {
    vec![
        ApprovalPreset {
            id: "read-only",
            label: "只读",
            description: "Codex 可以读取当前工作区的文件。编辑文件或访问互联网需要批准。",
            approval: AskForApproval::OnRequest,
            sandbox: SandboxPolicy::ReadOnly,
        },
        ApprovalPreset {
            id: "auto",
            label: "默认",
            description: "Codex 可以读取并编辑当前工作区的文件，并运行命令。访问互联网或编辑其他文件需要批准。（与代理模式一致）",
            approval: AskForApproval::OnRequest,
            sandbox: SandboxPolicy::new_workspace_write_policy(),
        },
        ApprovalPreset {
            id: "full-access",
            label: "全权限",
            description: "Codex 可以在无需批准的情况下编辑工作区外的文件并访问互联网。使用时请谨慎。",
            approval: AskForApproval::Never,
            sandbox: SandboxPolicy::DangerFullAccess,
        },
    ]
}
