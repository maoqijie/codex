#![cfg(not(target_os = "windows"))]

use anyhow::Ok;
use codex_app_server_protocol::ConfigLayerSource;
use codex_core::config_loader::ConfigLayerEntry;
use codex_core::config_loader::ConfigLayerStack;
use codex_core::config_loader::ConfigRequirements;
use codex_core::config_loader::ConfigRequirementsToml;
use codex_core::features::Feature;
use codex_core::protocol::DeprecationNoticeEvent;
use codex_core::protocol::EventMsg;
use core_test_support::responses::start_mock_server;
use core_test_support::skip_if_no_network;
use core_test_support::test_absolute_path;
use core_test_support::test_codex::TestCodex;
use core_test_support::test_codex::test_codex;
use core_test_support::wait_for_event_match;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;
use toml::Value as TomlValue;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emits_deprecation_notice_for_legacy_feature_flag() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;

    let mut builder = test_codex().with_config(|config| {
        config.features.enable(Feature::UnifiedExec);
        config
            .features
            .record_legacy_usage_force("use_experimental_unified_exec_tool", Feature::UnifiedExec);
        config.use_experimental_unified_exec_tool = true;
    });

    let TestCodex { codex, .. } = builder.build(&server).await?;

    let notice = wait_for_event_match(&codex, |event| match event {
        EventMsg::DeprecationNotice(ev) => Some(ev.clone()),
        _ => None,
    })
    .await;

    let DeprecationNoticeEvent { summary, details } = notice;
    assert_eq!(
        summary,
        "`use_experimental_unified_exec_tool` 已弃用，请改用 `[features].unified_exec`。"
            .to_string(),
    );
    assert_eq!(
        details.as_deref(),
        Some(
            "可通过 `--enable unified_exec` 或在 config.toml 的 `[features].unified_exec` 中启用。详情见：https://github.com/openai/codex/blob/main/docs/config.md#feature-flags"
        ),
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emits_deprecation_notice_for_experimental_instructions_file() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;

    let mut builder = test_codex().with_config(|config| {
        let mut table = toml::map::Map::new();
        table.insert(
            "experimental_instructions_file".to_string(),
            TomlValue::String("legacy.md".to_string()),
        );
        let config_layer = ConfigLayerEntry::new(
            ConfigLayerSource::User {
                file: test_absolute_path("/tmp/config.toml"),
            },
            TomlValue::Table(table),
        );
        let config_layer_stack = ConfigLayerStack::new(
            vec![config_layer],
            ConfigRequirements::default(),
            ConfigRequirementsToml::default(),
        )
        .expect("build config layer stack");
        config.config_layer_stack = config_layer_stack;
    });

    let TestCodex { codex, .. } = builder.build(&server).await?;

    let notice = wait_for_event_match(&codex, |event| match event {
        EventMsg::DeprecationNotice(ev)
            if ev.summary.contains("experimental_instructions_file") =>
        {
            Some(ev.clone())
        }
        _ => None,
    })
    .await;

    let DeprecationNoticeEvent { summary, details } = notice;
    assert_eq!(
        summary,
        "`experimental_instructions_file` 已弃用且会被忽略，请改用 `model_instructions_file`。"
            .to_string(),
    );
    assert_eq!(
        details.as_deref(),
        Some(
            "请将该设置迁移到 config.toml（或 profile）中的 `model_instructions_file`，以从文件加载指引。"
        ),
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emits_deprecation_notice_for_web_search_feature_flags() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;

    let mut builder = test_codex().with_config(|config| {
        let mut entries = BTreeMap::new();
        entries.insert("web_search_request".to_string(), true);
        config.features.apply_map(&entries);
    });

    let TestCodex { codex, .. } = builder.build(&server).await?;

    let notice = wait_for_event_match(&codex, |event| match event {
        EventMsg::DeprecationNotice(ev) if ev.summary.contains("[features].web_search_request") => {
            Some(ev.clone())
        }
        _ => None,
    })
    .await;

    let DeprecationNoticeEvent { summary, details } = notice;
    assert_eq!(
        summary,
        "`[features].web_search_request` 已弃用，请改用 `web_search`。".to_string(),
    );
    assert_eq!(
        details.as_deref(),
        Some(
            "请在 config.toml 顶层（或 profile 下）将 `web_search` 设置为 `\"live\"`、`\"cached\"` 或 `\"disabled\"`。"
        ),
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn emits_deprecation_notice_for_disabled_web_search_feature_flag() -> anyhow::Result<()> {
    skip_if_no_network!(Ok(()));

    let server = start_mock_server().await;

    let mut builder = test_codex().with_config(|config| {
        let mut entries = BTreeMap::new();
        entries.insert("web_search_request".to_string(), false);
        config.features.apply_map(&entries);
    });

    let TestCodex { codex, .. } = builder.build(&server).await?;

    let notice = wait_for_event_match(&codex, |event| match event {
        EventMsg::DeprecationNotice(ev) if ev.summary.contains("[features].web_search_request") => {
            Some(ev.clone())
        }
        _ => None,
    })
    .await;

    let DeprecationNoticeEvent { summary, details } = notice;
    assert_eq!(
        summary,
        "`[features].web_search_request` 已弃用，请改用 `web_search`。".to_string(),
    );
    assert_eq!(
        details.as_deref(),
        Some(
            "请在 config.toml 顶层（或 profile 下）将 `web_search` 设置为 `\"live\"`、`\"cached\"` 或 `\"disabled\"`。"
        ),
    );

    Ok(())
}
