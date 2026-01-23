use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use app_test_support::McpProcess;
use app_test_support::to_response;
use app_test_support::write_models_cache;
use codex_app_server_protocol::JSONRPCError;
use codex_app_server_protocol::JSONRPCResponse;
use codex_app_server_protocol::Model;
use codex_app_server_protocol::ModelListParams;
use codex_app_server_protocol::ModelListResponse;
use codex_app_server_protocol::ReasoningEffortOption;
use codex_app_server_protocol::RequestId;
use codex_protocol::openai_models::ReasoningEffort;
use pretty_assertions::assert_eq;
use tempfile::TempDir;
use tokio::time::timeout;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const INVALID_REQUEST_ERROR_CODE: i64 = -32600;

#[tokio::test]
async fn list_models_returns_all_models_with_large_limit() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_models_cache(codex_home.path())?;
    let mut mcp = McpProcess::new(codex_home.path()).await?;

    timeout(DEFAULT_TIMEOUT, mcp.initialize()).await??;

    let request_id = mcp
        .send_list_models_request(ModelListParams {
            limit: Some(100),
            cursor: None,
        })
        .await?;

    let response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(request_id)),
    )
    .await??;

    let ModelListResponse {
        data: items,
        next_cursor,
    } = to_response::<ModelListResponse>(response)?;

    let expected_models = vec![
        Model {
            id: "gpt-5.2-codex".to_string(),
            model: "gpt-5.2-codex".to_string(),
            display_name: "gpt-5.2-codex".to_string(),
            description: "最新前沿的智能体编程模型。".to_string(),
            supported_reasoning_efforts: vec![
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::Low,
                    description: "更轻量推理的快速响应".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::Medium,
                    description: "兼顾速度与推理深度，适合日常任务".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::High,
                    description: "复杂问题的更深推理".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            default_reasoning_effort: ReasoningEffort::Medium,
            is_default: true,
        },
        Model {
            id: "gpt-5.1-codex-max".to_string(),
            model: "gpt-5.1-codex-max".to_string(),
            display_name: "gpt-5.1-codex-max".to_string(),
            description: "为 Codex 优化的旗舰模型，兼具深度与速度推理。".to_string(),
            supported_reasoning_efforts: vec![
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::Low,
                    description: "更轻量推理的快速响应".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::Medium,
                    description: "兼顾速度与推理深度，适合日常任务".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::High,
                    description: "复杂问题的更深推理".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            default_reasoning_effort: ReasoningEffort::Medium,
            is_default: false,
        },
        Model {
            id: "gpt-5.1-codex-mini".to_string(),
            model: "gpt-5.1-codex-mini".to_string(),
            display_name: "gpt-5.1-codex-mini".to_string(),
            description: "为 Codex 优化，更便宜更快，但能力较弱。".to_string(),
            supported_reasoning_efforts: vec![
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::Medium,
                    description: "根据任务动态调整推理".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
            ],
            default_reasoning_effort: ReasoningEffort::Medium,
            is_default: false,
        },
        Model {
            id: "gpt-5.2".to_string(),
            model: "gpt-5.2".to_string(),
            display_name: "gpt-5.2".to_string(),
            description: "最新前沿模型，在知识、推理与编码上都有提升".to_string(),
            supported_reasoning_efforts: vec![
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::Low,
                    description: "兼顾速度与一定推理；适合直接问题和短解释".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::Medium,
                    description: "在通用任务上平衡推理深度与延迟".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
                ReasoningEffortOption {
                    reasoning_effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            default_reasoning_effort: ReasoningEffort::Medium,
            is_default: false,
        },
    ];

    assert_eq!(items, expected_models);
    assert!(next_cursor.is_none());
    Ok(())
}

#[tokio::test]
async fn list_models_pagination_works() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_models_cache(codex_home.path())?;
    let mut mcp = McpProcess::new(codex_home.path()).await?;

    timeout(DEFAULT_TIMEOUT, mcp.initialize()).await??;

    let first_request = mcp
        .send_list_models_request(ModelListParams {
            limit: Some(1),
            cursor: None,
        })
        .await?;

    let first_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(first_request)),
    )
    .await??;

    let ModelListResponse {
        data: first_items,
        next_cursor: first_cursor,
    } = to_response::<ModelListResponse>(first_response)?;

    assert_eq!(first_items.len(), 1);
    assert_eq!(first_items[0].id, "gpt-5.2-codex");
    let next_cursor = first_cursor.ok_or_else(|| anyhow!("cursor for second page"))?;

    let second_request = mcp
        .send_list_models_request(ModelListParams {
            limit: Some(1),
            cursor: Some(next_cursor.clone()),
        })
        .await?;

    let second_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(second_request)),
    )
    .await??;

    let ModelListResponse {
        data: second_items,
        next_cursor: second_cursor,
    } = to_response::<ModelListResponse>(second_response)?;

    assert_eq!(second_items.len(), 1);
    assert_eq!(second_items[0].id, "gpt-5.1-codex-max");
    let third_cursor = second_cursor.ok_or_else(|| anyhow!("cursor for third page"))?;

    let third_request = mcp
        .send_list_models_request(ModelListParams {
            limit: Some(1),
            cursor: Some(third_cursor.clone()),
        })
        .await?;

    let third_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(third_request)),
    )
    .await??;

    let ModelListResponse {
        data: third_items,
        next_cursor: third_cursor,
    } = to_response::<ModelListResponse>(third_response)?;

    assert_eq!(third_items.len(), 1);
    assert_eq!(third_items[0].id, "gpt-5.1-codex-mini");
    let fourth_cursor = third_cursor.ok_or_else(|| anyhow!("cursor for fourth page"))?;

    let fourth_request = mcp
        .send_list_models_request(ModelListParams {
            limit: Some(1),
            cursor: Some(fourth_cursor.clone()),
        })
        .await?;

    let fourth_response: JSONRPCResponse = timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_response_message(RequestId::Integer(fourth_request)),
    )
    .await??;

    let ModelListResponse {
        data: fourth_items,
        next_cursor: fourth_cursor,
    } = to_response::<ModelListResponse>(fourth_response)?;

    assert_eq!(fourth_items.len(), 1);
    assert_eq!(fourth_items[0].id, "gpt-5.2");
    assert!(fourth_cursor.is_none());
    Ok(())
}

#[tokio::test]
async fn list_models_rejects_invalid_cursor() -> Result<()> {
    let codex_home = TempDir::new()?;
    write_models_cache(codex_home.path())?;
    let mut mcp = McpProcess::new(codex_home.path()).await?;

    timeout(DEFAULT_TIMEOUT, mcp.initialize()).await??;

    let request_id = mcp
        .send_list_models_request(ModelListParams {
            limit: None,
            cursor: Some("invalid".to_string()),
        })
        .await?;

    let error: JSONRPCError = timeout(
        DEFAULT_TIMEOUT,
        mcp.read_stream_until_error_message(RequestId::Integer(request_id)),
    )
    .await??;

    assert_eq!(error.id, RequestId::Integer(request_id));
    assert_eq!(error.error.code, INVALID_REQUEST_ERROR_CODE);
    assert_eq!(error.error.message, "invalid cursor: invalid");
    Ok(())
}
