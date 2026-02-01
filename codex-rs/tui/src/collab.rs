use crate::history_cell::PlainHistoryCell;
use crate::render::line_utils::prefix_lines;
use crate::text_formatting::truncate_text;
use codex_core::protocol::AgentStatus;
use codex_core::protocol::CollabAgentInteractionEndEvent;
use codex_core::protocol::CollabAgentSpawnEndEvent;
use codex_core::protocol::CollabCloseEndEvent;
use codex_core::protocol::CollabWaitingBeginEvent;
use codex_core::protocol::CollabWaitingEndEvent;
use codex_protocol::ThreadId;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use std::collections::HashMap;

const COLLAB_PROMPT_PREVIEW_GRAPHEMES: usize = 160;
const COLLAB_AGENT_ERROR_PREVIEW_GRAPHEMES: usize = 160;
const COLLAB_AGENT_RESPONSE_PREVIEW_GRAPHEMES: usize = 240;

pub(crate) fn spawn_end(ev: CollabAgentSpawnEndEvent) -> PlainHistoryCell {
    let CollabAgentSpawnEndEvent {
        call_id,
        sender_thread_id: _,
        new_thread_id,
        prompt,
        status,
    } = ev;
    let new_agent = new_thread_id
        .map(|id| Span::from(id.to_string()))
        .unwrap_or_else(|| Span::from("未创建").dim());
    let mut details = vec![
        detail_line("调用", call_id),
        detail_line("代理", new_agent),
        status_line(&status),
    ];
    if let Some(line) = prompt_line(&prompt) {
        details.push(line);
    }
    collab_event("已创建代理", details)
}

pub(crate) fn interaction_end(ev: CollabAgentInteractionEndEvent) -> PlainHistoryCell {
    let CollabAgentInteractionEndEvent {
        call_id,
        sender_thread_id: _,
        receiver_thread_id,
        prompt,
        status,
    } = ev;
    let mut details = vec![
        detail_line("调用", call_id),
        detail_line("接收方", receiver_thread_id.to_string()),
        status_line(&status),
    ];
    if let Some(line) = prompt_line(&prompt) {
        details.push(line);
    }
    collab_event("已发送输入", details)
}

pub(crate) fn waiting_begin(ev: CollabWaitingBeginEvent) -> PlainHistoryCell {
    let CollabWaitingBeginEvent {
        call_id,
        sender_thread_id: _,
        receiver_thread_ids,
    } = ev;
    let details = vec![
        detail_line("调用", call_id),
        detail_line("接收方", format_thread_ids(&receiver_thread_ids)),
    ];
    collab_event("等待代理响应", details)
}

pub(crate) fn waiting_end(ev: CollabWaitingEndEvent) -> PlainHistoryCell {
    let CollabWaitingEndEvent {
        call_id,
        sender_thread_id: _,
        statuses,
    } = ev;
    let mut details = vec![detail_line("调用", call_id)];
    details.extend(wait_complete_lines(&statuses));
    collab_event("等待完成", details)
}

pub(crate) fn close_end(ev: CollabCloseEndEvent) -> PlainHistoryCell {
    let CollabCloseEndEvent {
        call_id,
        sender_thread_id: _,
        receiver_thread_id,
        status,
    } = ev;
    let details = vec![
        detail_line("调用", call_id),
        detail_line("接收方", receiver_thread_id.to_string()),
        status_line(&status),
    ];
    collab_event("已关闭代理", details)
}

fn collab_event(title: impl Into<String>, details: Vec<Line<'static>>) -> PlainHistoryCell {
    let title = title.into();
    let mut lines: Vec<Line<'static>> =
        vec![vec![Span::from("• ").dim(), Span::from(title).bold()].into()];
    if !details.is_empty() {
        lines.extend(prefix_lines(details, "  └ ".dim(), "    ".into()));
    }
    PlainHistoryCell::new(lines)
}

fn detail_line(label: &str, value: impl Into<Span<'static>>) -> Line<'static> {
    vec![Span::from(format!("{label}: ")).dim(), value.into()].into()
}

fn status_line(status: &AgentStatus) -> Line<'static> {
    detail_line("状态", status_span(status))
}

fn status_span(status: &AgentStatus) -> Span<'static> {
    match status {
        AgentStatus::PendingInit => Span::from("初始化中").dim(),
        AgentStatus::Running => Span::from("运行中").cyan().bold(),
        AgentStatus::Completed(_) => Span::from("已完成").green(),
        AgentStatus::Errored(_) => Span::from("出错").red(),
        AgentStatus::Shutdown => Span::from("已关闭").dim(),
        AgentStatus::NotFound => Span::from("未找到").red(),
    }
}

fn prompt_line(prompt: &str) -> Option<Line<'static>> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(detail_line(
            "提示词",
            Span::from(truncate_text(trimmed, COLLAB_PROMPT_PREVIEW_GRAPHEMES)).dim(),
        ))
    }
}

fn format_thread_ids(ids: &[ThreadId]) -> Span<'static> {
    if ids.is_empty() {
        return Span::from("无").dim();
    }
    let joined = ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    Span::from(joined)
}

fn wait_complete_lines(statuses: &HashMap<ThreadId, AgentStatus>) -> Vec<Line<'static>> {
    if statuses.is_empty() {
        return vec![detail_line("代理", Span::from("无").dim())];
    }

    let mut pending_init = 0usize;
    let mut running = 0usize;
    let mut completed = 0usize;
    let mut errored = 0usize;
    let mut shutdown = 0usize;
    let mut not_found = 0usize;
    for status in statuses.values() {
        match status {
            AgentStatus::PendingInit => pending_init += 1,
            AgentStatus::Running => running += 1,
            AgentStatus::Completed(_) => completed += 1,
            AgentStatus::Errored(_) => errored += 1,
            AgentStatus::Shutdown => shutdown += 1,
            AgentStatus::NotFound => not_found += 1,
        }
    }

    let mut summary = vec![Span::from(format!("共 {} 个", statuses.len())).dim()];
    push_status_count(
        &mut summary,
        pending_init,
        "初始化中",
        ratatui::prelude::Stylize::dim,
    );
    push_status_count(&mut summary, running, "运行中", |span| {
        span.cyan().bold()
    });
    push_status_count(
        &mut summary,
        completed,
        "已完成",
        ratatui::prelude::Stylize::green,
    );
    push_status_count(
        &mut summary,
        errored,
        "出错",
        ratatui::prelude::Stylize::red,
    );
    push_status_count(
        &mut summary,
        shutdown,
        "已关闭",
        ratatui::prelude::Stylize::dim,
    );
    push_status_count(
        &mut summary,
        not_found,
        "未找到",
        ratatui::prelude::Stylize::red,
    );

    let mut entries: Vec<(String, &AgentStatus)> = statuses
        .iter()
        .map(|(thread_id, status)| (thread_id.to_string(), status))
        .collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut lines = Vec::with_capacity(entries.len() + 1);
    lines.push(detail_line_spans("代理", summary));
    lines.extend(entries.into_iter().map(|(thread_id, status)| {
        let mut spans = vec![
            Span::from(thread_id).dim(),
            Span::from(" ").dim(),
            status_span(status),
        ];
        match status {
            AgentStatus::Completed(Some(message)) => {
                let message_preview = truncate_text(
                    &message.split_whitespace().collect::<Vec<_>>().join(" "),
                    COLLAB_AGENT_RESPONSE_PREVIEW_GRAPHEMES,
                );
                spans.push(Span::from(": ").dim());
                spans.push(Span::from(message_preview));
            }
            AgentStatus::Errored(error) => {
                let error_preview = truncate_text(
                    &error.split_whitespace().collect::<Vec<_>>().join(" "),
                    COLLAB_AGENT_ERROR_PREVIEW_GRAPHEMES,
                );
                spans.push(Span::from(": ").dim());
                spans.push(Span::from(error_preview).dim());
            }
            _ => {}
        }
        spans.into()
    }));
    lines
}

fn push_status_count(
    spans: &mut Vec<Span<'static>>,
    count: usize,
    label: &'static str,
    style: impl FnOnce(Span<'static>) -> Span<'static>,
) {
    if count == 0 {
        return;
    }

    spans.push(Span::from(" · ").dim());
    spans.push(style(Span::from(format!("{count} {label}"))));
}

fn detail_line_spans(label: &str, mut value: Vec<Span<'static>>) -> Line<'static> {
    let mut spans = Vec::with_capacity(value.len() + 1);
    spans.push(Span::from(format!("{label}: ")).dim());
    spans.append(&mut value);
    spans.into()
}
