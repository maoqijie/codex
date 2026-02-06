use crate::codex::TurnContext;
use crate::protocol::EventMsg;
use crate::protocol::WarningEvent;
use crate::state::TaskKind;
use crate::tasks::SessionTask;
use crate::tasks::SessionTaskContext;
use async_trait::async_trait;
use codex_git::CreateGhostCommitOptions;
use codex_git::GhostSnapshotReport;
use codex_git::GitToolingError;
use codex_git::create_ghost_commit_with_report;
use codex_protocol::models::ResponseItem;
use codex_protocol::user_input::UserInput;
use codex_utils_readiness::Readiness;
use codex_utils_readiness::Token;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tracing::info;
use tracing::warn;

pub(crate) struct GhostSnapshotTask {
    token: Token,
}

const SNAPSHOT_WARNING_THRESHOLD: Duration = Duration::from_secs(240);

#[async_trait]
impl SessionTask for GhostSnapshotTask {
    fn kind(&self) -> TaskKind {
        TaskKind::Regular
    }

    async fn run(
        self: Arc<Self>,
        session: Arc<SessionTaskContext>,
        ctx: Arc<TurnContext>,
        _input: Vec<UserInput>,
        cancellation_token: CancellationToken,
    ) -> Option<String> {
        tokio::task::spawn(async move {
            let token = self.token;
            let warnings_enabled = !ctx.ghost_snapshot.disable_warnings;
            // Channel used to signal when the snapshot work has finished so the
            // timeout warning task can exit early without sending a warning.
            let (snapshot_done_tx, snapshot_done_rx) = oneshot::channel::<()>();
            if warnings_enabled {
                let ctx_for_warning = ctx.clone();
                let cancellation_token_for_warning = cancellation_token.clone();
                let session_for_warning = session.clone();
                // Fire a generic warning if the snapshot is still running after
                // three minutes; this helps users discover large untracked files
                // that might need to be added to .gitignore.
                tokio::task::spawn(async move {
                    tokio::select! {
                        _ = tokio::time::sleep(SNAPSHOT_WARNING_THRESHOLD) => {
                            session_for_warning.session
                                .send_event(
                                    &ctx_for_warning,
                                    EventMsg::Warning(WarningEvent {
                                        message: "仓库快照耗时超出预期。大量未跟踪或被忽略的文件会拖慢快照；可以考虑将大文件或目录加入 `.gitignore`，或在配置中禁用 `undo`。".to_string()
                                    }),
                                )
                                .await;
                        }
                        _ = snapshot_done_rx => {}
                        _ = cancellation_token_for_warning.cancelled() => {}
                    }
                });
            } else {
                drop(snapshot_done_rx);
            }

            let ctx_for_task = ctx.clone();
            let cancelled = tokio::select! {
                _ = cancellation_token.cancelled() => true,
                _ = async {
                    let repo_path = ctx_for_task.cwd.clone();
                    let ghost_snapshot = ctx_for_task.ghost_snapshot.clone();
                    let ghost_snapshot_for_commit = ghost_snapshot.clone();
                    // Required to run in a dedicated blocking pool.
                    match tokio::task::spawn_blocking(move || {
                        let options =
                            CreateGhostCommitOptions::new(&repo_path).ghost_snapshot(ghost_snapshot_for_commit);
                        create_ghost_commit_with_report(&options)
                    })
                    .await
                    {
                        Ok(Ok((ghost_commit, report))) => {
                            info!("ghost snapshot blocking task finished");
                            if warnings_enabled {
                                for message in format_snapshot_warnings(
                                    ghost_snapshot.ignore_large_untracked_files,
                                    ghost_snapshot.ignore_large_untracked_dirs,
                                    &report,
                                ) {
                                    session
                                        .session
                                        .send_event(
                                            &ctx_for_task,
                                            EventMsg::Warning(WarningEvent { message }),
                                        )
                                        .await;
                                }
                            }
                            session
                                .session
                                .record_conversation_items(&ctx, &[ResponseItem::GhostSnapshot {
                                    ghost_commit: ghost_commit.clone(),
                                }])
                                .await;
                            info!("ghost commit captured: {}", ghost_commit.id());
                        }
                        Ok(Err(err)) => match err {
                            GitToolingError::NotAGitRepository { .. } => info!(
                                sub_id = ctx_for_task.sub_id.as_str(),
                                "skipping ghost snapshot because current directory is not a Git repository"
                            ),
                            _ => {
                                warn!(
                                    sub_id = ctx_for_task.sub_id.as_str(),
                                    "failed to capture ghost snapshot: {err}"
                                );
                            }
                        },
                        Err(err) => {
                            warn!(
                                sub_id = ctx_for_task.sub_id.as_str(),
                                "ghost snapshot task panicked: {err}"
                            );
                            let message = format!("ghost snapshot 发生 panic，已禁用快照：{err}。");
                            session
                                .session
                                .notify_background_event(&ctx_for_task, message)
                                .await;
                        }
                    }
                } => false,
            };

            let _ = snapshot_done_tx.send(());

            if cancelled {
                info!("ghost snapshot task cancelled");
            }

            match ctx.tool_call_gate.mark_ready(token).await {
                Ok(true) => info!("ghost snapshot gate marked ready"),
                Ok(false) => warn!("ghost snapshot gate already ready"),
                Err(err) => warn!("failed to mark ghost snapshot ready: {err}"),
            }
        });
        None
    }
}

impl GhostSnapshotTask {
    pub(crate) fn new(token: Token) -> Self {
        Self { token }
    }
}

fn format_snapshot_warnings(
    ignore_large_untracked_files: Option<i64>,
    ignore_large_untracked_dirs: Option<i64>,
    report: &GhostSnapshotReport,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if let Some(message) = format_large_untracked_warning(ignore_large_untracked_dirs, report) {
        warnings.push(message);
    }
    if let Some(message) =
        format_ignored_untracked_files_warning(ignore_large_untracked_files, report)
    {
        warnings.push(message);
    }
    warnings
}

fn format_large_untracked_warning(
    ignore_large_untracked_dirs: Option<i64>,
    report: &GhostSnapshotReport,
) -> Option<String> {
    if report.large_untracked_dirs.is_empty() {
        return None;
    }
    let threshold = ignore_large_untracked_dirs?;
    const MAX_DIRS: usize = 3;
    let mut parts: Vec<String> = Vec::new();
    for dir in report.large_untracked_dirs.iter().take(MAX_DIRS) {
        parts.push(format!(
            "{}（{} 个文件）",
            dir.path.display(),
            dir.file_count
        ));
    }
    if report.large_untracked_dirs.len() > MAX_DIRS {
        let remaining = report.large_untracked_dirs.len() - MAX_DIRS;
        parts.push(format!("另有 {remaining} 个"));
    }
    Some(format!(
        "仓库快照已忽略较大的未跟踪目录（>= {threshold} 个文件）：{}。这些目录不会被纳入快照和撤销清理。可调整 `ghost_snapshot.ignore_large_untracked_dirs` 来改变该行为。",
        parts.join(", ")
    ))
}

fn format_ignored_untracked_files_warning(
    ignore_large_untracked_files: Option<i64>,
    report: &GhostSnapshotReport,
) -> Option<String> {
    let threshold = ignore_large_untracked_files?;
    if report.ignored_untracked_files.is_empty() {
        return None;
    }

    const MAX_FILES: usize = 3;
    let mut parts: Vec<String> = Vec::new();
    for file in report.ignored_untracked_files.iter().take(MAX_FILES) {
        parts.push(format!(
            "{} ({})",
            file.path.display(),
            format_bytes(file.byte_size)
        ));
    }
    if report.ignored_untracked_files.len() > MAX_FILES {
        let remaining = report.ignored_untracked_files.len() - MAX_FILES;
        parts.push(format!("{remaining} more"));
    }

    Some(format!(
        "仓库快照已忽略大于 {} 的未跟踪文件：{}。这些文件在撤销清理时会被保留，但其内容不会被写入快照。可调整 `ghost_snapshot.ignore_large_untracked_files` 来改变该行为。若想避免再次提示，请更新你的 `.gitignore`。",
        format_bytes(threshold),
        parts.join(", ")
    ))
}

fn format_bytes(bytes: i64) -> String {
    const KIB: i64 = 1024;
    const MIB: i64 = 1024 * 1024;

    if bytes >= MIB {
        return format!("{} MiB", bytes / MIB);
    }
    if bytes >= KIB {
        return format!("{} KiB", bytes / KIB);
    }
    format!("{bytes} B")
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_git::LargeUntrackedDir;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn large_untracked_warning_includes_threshold() {
        let report = GhostSnapshotReport {
            large_untracked_dirs: vec![LargeUntrackedDir {
                path: PathBuf::from("models"),
                file_count: 250,
            }],
            ignored_untracked_files: Vec::new(),
        };

        let message = format_large_untracked_warning(Some(200), &report).unwrap();
        assert!(message.contains(">= 200 个文件"));
    }

    #[test]
    fn large_untracked_warning_disabled_when_threshold_disabled() {
        let report = GhostSnapshotReport {
            large_untracked_dirs: vec![LargeUntrackedDir {
                path: PathBuf::from("models"),
                file_count: 250,
            }],
            ignored_untracked_files: Vec::new(),
        };

        assert_eq!(format_large_untracked_warning(None, &report), None);
    }
}
