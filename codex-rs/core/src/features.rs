//! Centralized feature flags and metadata.
//!
//! This module defines a small set of toggles that gate experimental and
//! optional behavior across the codebase. Instead of wiring individual
//! booleans through multiple types, call sites consult a single `Features`
//! container attached to `Config`.

use crate::config::ConfigToml;
use crate::config::profile::ConfigProfile;
use codex_otel::OtelManager;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

mod legacy;
pub(crate) use legacy::LegacyFeatureToggles;
pub(crate) use legacy::legacy_feature_keys;

/// High-level lifecycle stage for a feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Closed beta features to be used while developing or within the company.
    Beta,
    /// Experimental features made available to users through the `/experimental` menu
    Experimental {
        name: &'static str,
        menu_description: &'static str,
        announcement: &'static str,
    },
    /// Stable features. The feature flag is kept for ad-hoc enabling/disabling
    Stable,
    /// Deprecated feature that should not be used anymore.
    Deprecated,
    /// The feature flag is useless but kept for backward compatibility reason.
    Removed,
}

impl Stage {
    pub fn beta_menu_name(self) -> Option<&'static str> {
        match self {
            Stage::Experimental { name, .. } => Some(name),
            _ => None,
        }
    }

    pub fn beta_menu_description(self) -> Option<&'static str> {
        match self {
            Stage::Experimental {
                menu_description, ..
            } => Some(menu_description),
            _ => None,
        }
    }

    pub fn beta_announcement(self) -> Option<&'static str> {
        match self {
            Stage::Experimental { announcement, .. } => Some(announcement),
            _ => None,
        }
    }
}

/// Unique features toggled via configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Feature {
    // Stable.
    /// Create a ghost commit at each turn.
    GhostCommit,
    /// Enable the default shell tool.
    ShellTool,

    // Experimental
    /// Use the single unified PTY-backed exec tool.
    UnifiedExec,
    /// Include the freeform apply_patch tool.
    ApplyPatchFreeform,
    /// Allow the model to request web searches that fetch live content.
    WebSearchRequest,
    /// Allow the model to request web searches that fetch cached content.
    /// Takes precedence over `WebSearchRequest`.
    WebSearchCached,
    /// Gate the execpolicy enforcement for shell/unified exec.
    ExecPolicy,
    /// Enable Windows sandbox (restricted token) on Windows.
    WindowsSandbox,
    /// Use the elevated Windows sandbox pipeline (setup + runner).
    WindowsSandboxElevated,
    /// Remote compaction enabled (only for ChatGPT auth)
    RemoteCompaction,
    /// Refresh remote models and emit AppReady once the list is available.
    RemoteModels,
    /// Experimental shell snapshotting.
    ShellSnapshot,
    /// Append additional AGENTS.md guidance to user instructions.
    ChildAgentsMd,
    /// Enforce UTF8 output in Powershell.
    PowershellUtf8,
    /// Compress request bodies (zstd) when sending streaming requests to codex-backend.
    EnableRequestCompression,
    /// Enable collab tools.
    Collab,
    /// Steer feature flag - when enabled, Enter submits immediately instead of queuing.
    Steer,
    /// Enable collaboration modes (Plan, Pair Programming, Execute).
    CollaborationModes,
    /// Use the Responses API WebSocket transport for OpenAI by default.
    ResponsesWebsockets,
}

impl Feature {
    pub fn key(self) -> &'static str {
        self.info().key
    }

    pub fn stage(self) -> Stage {
        self.info().stage
    }

    pub fn default_enabled(self) -> bool {
        self.info().default_enabled
    }

    fn info(self) -> &'static FeatureSpec {
        FEATURES
            .iter()
            .find(|spec| spec.id == self)
            .unwrap_or_else(|| unreachable!("missing FeatureSpec for {:?}", self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LegacyFeatureUsage {
    pub alias: String,
    pub feature: Feature,
}

/// Holds the effective set of enabled features.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Features {
    enabled: BTreeSet<Feature>,
    legacy_usages: BTreeSet<LegacyFeatureUsage>,
}

#[derive(Debug, Clone, Default)]
pub struct FeatureOverrides {
    pub include_apply_patch_tool: Option<bool>,
    pub web_search_request: Option<bool>,
}

impl FeatureOverrides {
    fn apply(self, features: &mut Features) {
        LegacyFeatureToggles {
            include_apply_patch_tool: self.include_apply_patch_tool,
            tools_web_search: self.web_search_request,
            ..Default::default()
        }
        .apply(features);
    }
}

impl Features {
    /// Starts with built-in defaults.
    pub fn with_defaults() -> Self {
        let mut set = BTreeSet::new();
        for spec in FEATURES {
            if spec.default_enabled {
                set.insert(spec.id);
            }
        }
        Self {
            enabled: set,
            legacy_usages: BTreeSet::new(),
        }
    }

    pub fn enabled(&self, f: Feature) -> bool {
        self.enabled.contains(&f)
    }

    pub fn enable(&mut self, f: Feature) -> &mut Self {
        self.enabled.insert(f);
        self
    }

    pub fn disable(&mut self, f: Feature) -> &mut Self {
        self.enabled.remove(&f);
        self
    }

    pub fn record_legacy_usage_force(&mut self, alias: &str, feature: Feature) {
        self.legacy_usages.insert(LegacyFeatureUsage {
            alias: alias.to_string(),
            feature,
        });
    }

    pub fn record_legacy_usage(&mut self, alias: &str, feature: Feature) {
        if alias == feature.key() {
            return;
        }
        self.record_legacy_usage_force(alias, feature);
    }

    pub fn legacy_feature_usages(&self) -> impl Iterator<Item = (&str, Feature)> + '_ {
        self.legacy_usages
            .iter()
            .map(|usage| (usage.alias.as_str(), usage.feature))
    }

    pub fn emit_metrics(&self, otel: &OtelManager) {
        for feature in FEATURES {
            if self.enabled(feature.id) != feature.default_enabled {
                otel.counter(
                    "codex.feature.state",
                    1,
                    &[
                        ("feature", feature.key),
                        ("value", &self.enabled(feature.id).to_string()),
                    ],
                );
            }
        }
    }

    /// Apply a table of key -> bool toggles (e.g. from TOML).
    pub fn apply_map(&mut self, m: &BTreeMap<String, bool>) {
        for (k, v) in m {
            match feature_for_key(k) {
                Some(feat) => {
                    if k != feat.key() {
                        self.record_legacy_usage(k.as_str(), feat);
                    }
                    if *v {
                        self.enable(feat);
                    } else {
                        self.disable(feat);
                    }
                }
                None => {
                    tracing::warn!("unknown feature key in config: {k}");
                }
            }
        }
    }

    pub fn from_config(
        cfg: &ConfigToml,
        config_profile: &ConfigProfile,
        overrides: FeatureOverrides,
    ) -> Self {
        let mut features = Features::with_defaults();

        let base_legacy = LegacyFeatureToggles {
            experimental_use_freeform_apply_patch: cfg.experimental_use_freeform_apply_patch,
            experimental_use_unified_exec_tool: cfg.experimental_use_unified_exec_tool,
            tools_web_search: cfg.tools.as_ref().and_then(|t| t.web_search),
            ..Default::default()
        };
        base_legacy.apply(&mut features);

        if let Some(base_features) = cfg.features.as_ref() {
            features.apply_map(&base_features.entries);
        }

        let profile_legacy = LegacyFeatureToggles {
            include_apply_patch_tool: config_profile.include_apply_patch_tool,
            experimental_use_freeform_apply_patch: config_profile
                .experimental_use_freeform_apply_patch,

            experimental_use_unified_exec_tool: config_profile.experimental_use_unified_exec_tool,
            tools_web_search: config_profile.tools_web_search,
        };
        profile_legacy.apply(&mut features);
        if let Some(profile_features) = config_profile.features.as_ref() {
            features.apply_map(&profile_features.entries);
        }

        overrides.apply(&mut features);

        features
    }

    pub fn enabled_features(&self) -> Vec<Feature> {
        self.enabled.iter().copied().collect()
    }
}

/// Keys accepted in `[features]` tables.
fn feature_for_key(key: &str) -> Option<Feature> {
    for spec in FEATURES {
        if spec.key == key {
            return Some(spec.id);
        }
    }
    legacy::feature_for_key(key)
}

/// Returns `true` if the provided string matches a known feature toggle key.
pub fn is_known_feature_key(key: &str) -> bool {
    feature_for_key(key).is_some()
}

/// Deserializable features table for TOML.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
pub struct FeaturesToml {
    #[serde(flatten)]
    pub entries: BTreeMap<String, bool>,
}

/// Single, easy-to-read registry of all feature definitions.
#[derive(Debug, Clone, Copy)]
pub struct FeatureSpec {
    pub id: Feature,
    pub key: &'static str,
    pub stage: Stage,
    pub default_enabled: bool,
}

pub const FEATURES: &[FeatureSpec] = &[
    // Stable features.
    FeatureSpec {
        id: Feature::GhostCommit,
        key: "undo",
        stage: Stage::Stable,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::ShellTool,
        key: "shell_tool",
        stage: Stage::Stable,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::WebSearchRequest,
        key: "web_search_request",
        stage: Stage::Stable,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::WebSearchCached,
        key: "web_search_cached",
        stage: Stage::Beta,
        default_enabled: false,
    },
    // Beta program. Rendered in the `/experimental` menu for users.
    FeatureSpec {
        id: Feature::UnifiedExec,
        key: "unified_exec",
        stage: Stage::Experimental {
            name: "后台终端",
            menu_description: "在后台运行耗时的终端命令。",
            announcement: "新功能！可在后台运行耗时命令。到 /experimental 启用。",
        },
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::ShellSnapshot,
        key: "shell_snapshot",
        stage: Stage::Experimental {
            name: "Shell 快照",
            menu_description: "保存 shell 环境快照，避免每次命令都重跑登录脚本。",
            announcement: "新功能！试试 Shell 快照，让 Codex 更快。到 /experimental 启用。",
        },
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::ChildAgentsMd,
        key: "child_agents_md",
        stage: Stage::Beta,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::ApplyPatchFreeform,
        key: "apply_patch_freeform",
        stage: Stage::Beta,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::ExecPolicy,
        key: "exec_policy",
        stage: Stage::Beta,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::WindowsSandbox,
        key: "experimental_windows_sandbox",
        stage: Stage::Beta,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::WindowsSandboxElevated,
        key: "elevated_windows_sandbox",
        stage: Stage::Beta,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::RemoteCompaction,
        key: "remote_compaction",
        stage: Stage::Beta,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::RemoteModels,
        key: "remote_models",
        stage: Stage::Beta,
        default_enabled: true,
    },
    FeatureSpec {
        id: Feature::PowershellUtf8,
        key: "powershell_utf8",
        #[cfg(windows)]
        stage: Stage::Experimental {
            name: "Powershell UTF-8 支持",
            menu_description: "在 Powershell 中启用 UTF-8 输出。",
            announcement: "Codex 现已支持 Powershell UTF-8 输出。如遇问题，可在 /experimental 中关闭。",
        },
        #[cfg(windows)]
        default_enabled: true,
        #[cfg(not(windows))]
        stage: Stage::Beta,
        #[cfg(not(windows))]
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::EnableRequestCompression,
        key: "enable_request_compression",
        stage: Stage::Beta,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::Collab,
        key: "collab",
        stage: Stage::Experimental {
            name: "多代理",
            menu_description: "允许 Codex 按需生成并与其他代理协作（原名 `collab`）。",
            announcement: "新功能！Codex 现在可生成其他代理并协同解决问题。到 /experimental 启用。",
        },
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::Steer,
        key: "steer",
        stage: Stage::Experimental {
            name: "引导会话",
            menu_description: "Enter 立即提交；任务运行时用 Tab 将消息加入队列。",
            announcement: "新功能！试试引导模式：Enter 立即提交，Tab 入队。到 /experimental 启用。",
        },
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::CollaborationModes,
        key: "collaboration_modes",
        stage: Stage::Beta,
        default_enabled: false,
    },
    FeatureSpec {
        id: Feature::ResponsesWebsockets,
        key: "responses_websockets",
        stage: Stage::Beta,
        default_enabled: false,
    },
];
