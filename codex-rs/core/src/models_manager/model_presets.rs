use crate::auth::AuthMode;
use codex_protocol::openai_models::ModelPreset;
use codex_protocol::openai_models::ModelUpgrade;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::openai_models::ReasoningEffortPreset;
use indoc::indoc;
use once_cell::sync::Lazy;

pub const HIDE_GPT5_1_MIGRATION_PROMPT_CONFIG: &str = "hide_gpt5_1_migration_prompt";
pub const HIDE_GPT_5_1_CODEX_MAX_MIGRATION_PROMPT_CONFIG: &str =
    "hide_gpt-5.1-codex-max_migration_prompt";

static PRESETS: Lazy<Vec<ModelPreset>> = Lazy::new(|| {
    vec![
        ModelPreset {
            id: "gpt-5.2-codex".to_string(),
            model: "gpt-5.2-codex".to_string(),
            display_name: "gpt-5.2-codex".to_string(),
            description: "最新前沿的智能体编程模型。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "更轻量推理的快速响应".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "兼顾速度与推理深度，适合日常任务".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "复杂问题的更深推理".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            supports_personality: true,
            is_default: true,
            upgrade: None,
            show_in_picker: true,
            supported_in_api: true,
        },
        ModelPreset {
            id: "gpt-5.1-codex-max".to_string(),
            model: "gpt-5.1-codex-max".to_string(),
            display_name: "gpt-5.1-codex-max".to_string(),
            description: "为 Codex 优化的旗舰模型，兼具深度与速度推理。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "更轻量推理的快速响应".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "兼顾速度与推理深度，适合日常任务".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "复杂问题的更深推理".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: true,
            supported_in_api: true,
        },
        ModelPreset {
            id: "gpt-5.1-codex-mini".to_string(),
            model: "gpt-5.1-codex-mini".to_string(),
            display_name: "gpt-5.1-codex-mini".to_string(),
            description: "为 Codex 优化，更便宜更快，但能力较弱。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "根据任务动态调整推理".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: true,
            supported_in_api: true,
        },
        ModelPreset {
            id: "gpt-5.2".to_string(),
            model: "gpt-5.2".to_string(),
            display_name: "gpt-5.2".to_string(),
            description: "最新前沿模型，在知识、推理与编码上都有提升".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "兼顾速度与一定推理；适合直接问题和短解释".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "在通用任务上平衡推理深度与延迟".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: true,
            supported_in_api: true,
        },
        ModelPreset {
            id: "bengalfox".to_string(),
            model: "bengalfox".to_string(),
            display_name: "bengalfox".to_string(),
            description: "bengalfox".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "更轻量推理的快速响应".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "兼顾速度与推理深度，适合日常任务".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "复杂问题的更深推理".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            supports_personality: true,
            is_default: false,
            upgrade: None,
            show_in_picker: false,
            supported_in_api: true,
        },
        ModelPreset {
            id: "boomslang".to_string(),
            model: "boomslang".to_string(),
            display_name: "boomslang".to_string(),
            description: "boomslang".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "兼顾速度与一定推理；适合直接问题和短解释".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "在通用任务上平衡推理深度与延迟".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::XHigh,
                    description: "复杂问题的超高推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: None,
            show_in_picker: false,
            supported_in_api: true,
        },
        // Deprecated models.
        ModelPreset {
            id: "gpt-5-codex".to_string(),
            model: "gpt-5-codex".to_string(),
            display_name: "gpt-5-codex".to_string(),
            description: "为 Codex 优化。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "推理受限但速度最快".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "根据任务动态调整推理".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: false,
            supported_in_api: true,
        },
        ModelPreset {
            id: "gpt-5-codex-mini".to_string(),
            model: "gpt-5-codex-mini".to_string(),
            display_name: "gpt-5-codex-mini".to_string(),
            description: "为 Codex 优化，更便宜更快，但能力较弱。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "根据任务动态调整推理".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: false,
            supported_in_api: true,
        },
        ModelPreset {
            id: "gpt-5.1-codex".to_string(),
            model: "gpt-5.1-codex".to_string(),
            display_name: "gpt-5.1-codex".to_string(),
            description: "为 Codex 优化。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "推理受限但速度最快".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "根据任务动态调整推理".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: false,
            supported_in_api: true,
        },
        ModelPreset {
            id: "gpt-5".to_string(),
            model: "gpt-5".to_string(),
            display_name: "gpt-5".to_string(),
            description: "广泛的世界知识与强通用推理。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Minimal,
                    description: "推理较少但速度最快".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "兼顾速度与一定推理；适合直接问题和短解释".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "在通用任务上平衡推理深度与延迟".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: false,
            supported_in_api: true,
        },
        ModelPreset {
            id: "gpt-5.1".to_string(),
            model: "gpt-5.1".to_string(),
            display_name: "gpt-5.1".to_string(),
            description: "广泛的世界知识与强通用推理。".to_string(),
            default_reasoning_effort: ReasoningEffort::Medium,
            supported_reasoning_efforts: vec![
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Low,
                    description: "兼顾速度与一定推理；适合直接问题和短解释".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::Medium,
                    description: "在通用任务上平衡推理深度与延迟".to_string(),
                },
                ReasoningEffortPreset {
                    effort: ReasoningEffort::High,
                    description: "为复杂或模糊问题最大化推理深度".to_string(),
                },
            ],
            supports_personality: false,
            is_default: false,
            upgrade: Some(gpt_52_codex_upgrade()),
            show_in_picker: false,
            supported_in_api: true,
        },
    ]
});

fn gpt_52_codex_upgrade() -> ModelUpgrade {
    ModelUpgrade {
        id: "gpt-5.2-codex".to_string(),
        reasoning_effort_mapping: None,
        migration_config_key: "gpt-5.2-codex".to_string(),
        model_link: Some("https://openai.com/index/introducing-gpt-5-2-codex".to_string()),
        upgrade_copy: Some(
            "Codex 现已由 gpt-5.2-codex 驱动，这是我们最新的前沿智能体编程模型。它比前代更聪明、更快速，能够胜任长时间的项目级工作。"
                .to_string(),
        ),
        migration_markdown: Some(
            indoc! {r#"
                **Codex 刚刚升级，引入 {model_to}。**

                Codex 现已由 gpt-5.2-codex 驱动，这是我们最新的前沿智能体编程模型。它比前代更聪明、更快速，能够胜任长时间的项目级工作。了解更多： https://openai.com/index/introducing-gpt-5-2-codex

                如有需要，你仍可继续使用 {model_from}。
            "#}
            .to_string(),
        ),
    }
}

pub(super) fn builtin_model_presets(_auth_mode: Option<AuthMode>) -> Vec<ModelPreset> {
    PRESETS.iter().cloned().collect()
}

#[cfg(any(test, feature = "test-support"))]
pub fn all_model_presets() -> &'static Vec<ModelPreset> {
    &PRESETS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_default_model_is_configured() {
        let default_models = PRESETS.iter().filter(|preset| preset.is_default).count();
        assert!(default_models == 1);
    }
}
