//! Immutable, versioned contracts between semantic `AgentIDE` intents and concrete drivers.
#![allow(clippy::missing_errors_doc)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

mod hosted;
mod renderer;

pub use hosted::*;
pub use renderer::*;

/// The immutable v1 intent catalogue retained for compatibility loading.
pub const INTENT_PROFILE_V1_YAML: &str = include_str!("../../../contracts/intent-profile.yaml");
/// The current actor-aware intent catalogue.
pub const INTENT_PROFILE_YAML: &str = include_str!("../../../contracts/intent-profile-v2.yaml");
/// Standalone bindings shipped with the binary.
pub const DEFAULT_BINDINGS_YAML: &str = include_str!("../../../contracts/default-bindings.yaml");
/// Renderer-neutral interaction and presentation profile shipped with every surface.
pub const SURFACE_PROFILE_YAML: &str = include_str!("../../../contracts/surface-profile.yaml");

/// A model-visible semantic operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntentDefinition {
    /// Stable snake-case name used on transports.
    pub name: String,
    /// Qualified ESS command providing its semantics.
    pub command: String,
    /// Actor classes for which this operation may become available.
    #[serde(default)]
    pub audiences: Vec<Audience>,
    /// Legacy v1 exposure, normalized away by the compatibility loader.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure: Option<Exposure>,
    /// Abstract implementation port.
    pub port: String,
    /// Kind of consequence.
    pub effect: Effect,
    /// Human-oriented consequence tier.
    pub risk: Risk,
    /// Authority rule.
    pub approval: Approval,
    /// Semantic resources affected or observed, never concrete credentials or destinations.
    pub subjects: Vec<String>,
}

/// Actor class eligible to receive an intent, subject to current bindings and authority.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Audience {
    /// An authenticated person using an interactive surface.
    Human,
    /// A model-backed coding agent.
    Agent,
    /// A non-interactive automation principal.
    Automation,
}

/// Intent visibility.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Exposure {
    /// Published to a model.
    Model,
    /// Available only to the operator surface.
    Operator,
    /// Published only when a binding and policy enable it.
    Conditional,
}

/// Observable consequence class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// No mutation.
    Observe,
    /// AgentIDE-local state only.
    State,
    /// Workspace mutation.
    Mutate,
    /// Confined execution.
    Execute,
    /// Communication with another subject.
    Communicate,
    /// Effect outside the local session.
    External,
}

/// Consequence tier.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    /// Observational.
    None,
    /// Reversible session state.
    Low,
    /// Local mutation or execution.
    Medium,
    /// Source-control publication.
    High,
    /// Release or deployment.
    Critical,
}

/// Approval requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Approval {
    /// No human approval is required.
    Never,
    /// Approval of the exact plan digest is required.
    Required,
}

/// Released intent profile document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentProfile {
    /// Version marker.
    pub format: String,
    /// Cross-intent rules retained for inspection.
    pub rules: BTreeMap<String, Value>,
    /// Stable catalogue.
    pub intents: Vec<IntentDefinition>,
}

/// A concrete driver operation selected by the operator or embedding application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    /// Driver implementation name.
    pub driver: String,
    /// Driver-owned operation name.
    pub operation: String,
    /// Operator-owned options, never copied from model input.
    #[serde(default)]
    pub options: BTreeMap<String, Value>,
}

/// Externally supplied binding table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BindingConfig {
    /// Version marker.
    pub format: String,
    /// Intent name to implementation binding.
    pub bindings: BTreeMap<String, Binding>,
    /// Intentionally unavailable operations.
    #[serde(default)]
    pub unbound: BTreeSet<String>,
}

/// A semantic surface region, independent of terminal or browser geometry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceRegion {
    /// Stable region identifier.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Whether keyboard focus may enter this region.
    pub focusable: bool,
    /// Action that exposes this region when a viewport hides it.
    pub overlay_action: Option<String>,
}

/// One adaptive viewport class. The most specific matching class wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewportProfile {
    /// Stable class identifier.
    pub id: String,
    /// Minimum terminal columns.
    pub min_columns: u16,
    /// Minimum terminal rows.
    pub min_rows: u16,
    /// Regions rendered directly by this class.
    pub visible_regions: Vec<String>,
    /// Focus used when the previous region becomes unavailable.
    pub default_focus: String,
}

/// How a surface action is implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceActionKind {
    /// State transition contained entirely within the renderer.
    Local,
    /// Dispatch through a named `AgentIDE` semantic intent.
    Intent,
}

/// Coarse, deterministic action availability rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAvailability {
    /// Available regardless of model or workbench state.
    Always,
    /// Available only while no model or intent operation is active.
    Idle,
    /// Available only with a focused durable pane and an idle worker.
    Pane,
}

/// One command exposed by shortcuts or the command palette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceAction {
    /// Stable action identifier.
    pub id: String,
    /// Human-readable command label.
    pub label: String,
    /// Local transition or semantic intent dispatch.
    pub kind: SurfaceActionKind,
    /// Intent name when `kind` is `intent`.
    pub intent: Option<String>,
    /// Whether the action appears in the command palette.
    #[serde(default)]
    pub palette: bool,
    /// Availability rule and disabled-reason source.
    pub availability: ActionAvailability,
    /// Region affected by the action, when applicable.
    pub target_region: Option<String>,
}

/// A normalized keyboard chord bound within one mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyBinding {
    /// Normalized key chord, such as `ctrl+k` or `shift+tab`.
    pub key: String,
    /// Surface action identifier.
    pub action: String,
}

/// A mutually exclusive interaction mode and its keymap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceMode {
    /// Stable mode identifier.
    pub id: String,
    /// Key bindings active in the mode.
    pub bindings: Vec<KeyBinding>,
}

/// Unicode glyph with a mandatory ASCII equivalent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Glyph {
    /// Preferred Unicode rendering.
    pub unicode: String,
    /// Portable fallback which must contain only ASCII.
    pub ascii: String,
}

/// Theme roles and terminal fallbacks. Role names are stable public vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceTheme {
    /// Six-digit RGB values used in truecolor terminals and the browser.
    pub truecolor: BTreeMap<String, String>,
    /// ANSI 256-color palette indices.
    pub color_256: BTreeMap<String, u8>,
    /// Portable named ANSI colors.
    pub color_16: BTreeMap<String, String>,
    /// Semantic glyph catalogue.
    pub glyphs: BTreeMap<String, Glyph>,
}

/// Released presentation and interaction contract shared by `AgentIDE` surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceProfile {
    /// Contract version marker.
    pub format: String,
    /// Known semantic regions.
    pub regions: Vec<SurfaceRegion>,
    /// Adaptive viewport classes.
    pub viewports: Vec<ViewportProfile>,
    /// Commands and their semantic implementation boundary.
    pub actions: Vec<SurfaceAction>,
    /// Mutually exclusive interaction modes.
    pub modes: Vec<SurfaceMode>,
    /// Shared visual vocabulary.
    pub theme: SurfaceTheme,
}

/// Contract load or consistency failure.
#[derive(Debug, Error)]
pub enum ContractError {
    /// YAML could not be decoded.
    #[error("contract YAML is invalid: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    /// File could not be read.
    #[error("contract could not be read: {0}")]
    Io(#[from] std::io::Error),
    /// A semantic invariant failed.
    #[error("contract is inconsistent: {0}")]
    Invalid(String),
}

impl IntentProfile {
    /// Loads and validates the embedded released profile.
    pub fn embedded() -> Result<Self, ContractError> {
        Self::from_yaml(INTENT_PROFILE_YAML)
    }

    /// Parses and validates one profile.
    pub fn from_yaml(yaml: &str) -> Result<Self, ContractError> {
        let mut profile: Self = serde_yaml_ng::from_str(yaml)?;
        if !matches!(
            profile.format.as_str(),
            "agentide.intent-profile/1" | "agentide.intent-profile/2"
        ) {
            return Err(ContractError::Invalid(format!(
                "unsupported profile `{}`",
                profile.format
            )));
        }
        let legacy = profile.format == "agentide.intent-profile/1";
        let mut names = BTreeSet::new();
        let mut commands = BTreeSet::new();
        for intent in &mut profile.intents {
            if legacy {
                intent.audiences = match intent.exposure {
                    Some(Exposure::Model) => vec![Audience::Agent, Audience::Automation],
                    Some(Exposure::Operator) => vec![Audience::Human],
                    Some(Exposure::Conditional) => {
                        vec![Audience::Human, Audience::Agent, Audience::Automation]
                    }
                    None => {
                        return Err(ContractError::Invalid(format!(
                            "legacy intent `{}` has no exposure",
                            intent.name
                        )));
                    }
                };
                intent.exposure = None;
            } else if intent.audiences.is_empty() {
                return Err(ContractError::Invalid(format!(
                    "intent `{}` has no audience",
                    intent.name
                )));
            } else if intent.exposure.is_some() {
                return Err(ContractError::Invalid(format!(
                    "v2 intent `{}` uses legacy exposure",
                    intent.name
                )));
            }
            if !names.insert(intent.name.as_str()) {
                return Err(ContractError::Invalid(format!(
                    "duplicate intent `{}`",
                    intent.name
                )));
            }
            if !commands.insert(intent.command.as_str()) {
                return Err(ContractError::Invalid(format!(
                    "duplicate ESS command `{}`",
                    intent.command
                )));
            }
        }
        profile.format = "agentide.intent-profile/2".into();
        Ok(profile)
    }

    /// Resolves a stable name.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&IntentDefinition> {
        self.intents.iter().find(|intent| intent.name == name)
    }
}

impl BindingConfig {
    /// Loads the standalone defaults embedded in the binary.
    pub fn embedded() -> Result<Self, ContractError> {
        Self::from_yaml(DEFAULT_BINDINGS_YAML)
    }

    /// Loads an operator-owned binding file.
    pub fn from_path(path: &Path) -> Result<Self, ContractError> {
        Self::from_yaml(&std::fs::read_to_string(path)?)
    }

    /// Parses and checks one binding document.
    pub fn from_yaml(yaml: &str) -> Result<Self, ContractError> {
        let config: Self = serde_yaml_ng::from_str(yaml)?;
        if config.format != "agentide.bindings/1" {
            return Err(ContractError::Invalid(format!(
                "unsupported bindings `{}`",
                config.format
            )));
        }
        for name in config.bindings.keys() {
            if config.unbound.contains(name) {
                return Err(ContractError::Invalid(format!(
                    "`{name}` is both bound and explicitly unbound"
                )));
            }
        }
        Ok(config)
    }

    /// Confirms every released intent is bound or explicitly withheld.
    pub fn validate_against(&self, profile: &IntentProfile) -> Result<(), ContractError> {
        for intent in &profile.intents {
            if intent.audiences == [Audience::Human] {
                continue;
            }
            if !self.bindings.contains_key(&intent.name) && !self.unbound.contains(&intent.name) {
                return Err(ContractError::Invalid(format!(
                    "intent `{}` is neither bound nor explicitly unbound",
                    intent.name
                )));
            }
        }
        for name in self.bindings.keys().chain(&self.unbound) {
            if profile.find(name).is_none() {
                return Err(ContractError::Invalid(format!(
                    "binding names unknown intent `{name}`"
                )));
            }
        }
        Ok(())
    }
}

impl SurfaceProfile {
    /// Loads and validates the released surface profile against the released intents.
    pub fn embedded() -> Result<Self, ContractError> {
        let intents = IntentProfile::embedded()?;
        Self::from_yaml(SURFACE_PROFILE_YAML, &intents)
    }

    /// Parses and validates a surface profile against an intent catalogue.
    pub fn from_yaml(yaml: &str, intents: &IntentProfile) -> Result<Self, ContractError> {
        let profile: Self = serde_yaml_ng::from_str(yaml)?;
        profile.validate_against(intents)?;
        Ok(profile)
    }

    /// Enforces cross-reference, reachability, modal-safety, and fallback invariants.
    #[allow(clippy::too_many_lines)]
    pub fn validate_against(&self, intents: &IntentProfile) -> Result<(), ContractError> {
        if self.format != "agentide.surface-profile/1" {
            return Err(invalid(format!(
                "unsupported surface profile `{}`",
                self.format
            )));
        }

        let region_ids = unique_ids("region", self.regions.iter().map(|region| &region.id))?;
        let action_ids = unique_ids("action", self.actions.iter().map(|action| &action.id))?;
        unique_ids("mode", self.modes.iter().map(|mode| &mode.id))?;
        unique_ids(
            "viewport",
            self.viewports.iter().map(|viewport| &viewport.id),
        )?;

        if self.viewports.is_empty() {
            return Err(invalid("at least one viewport is required"));
        }
        if !self.modes.iter().any(|mode| mode.id == "normal") {
            return Err(invalid("the `normal` interaction mode is required"));
        }
        if !self.modes.iter().any(|mode| mode.id == "approval") {
            return Err(invalid("the `approval` interaction mode is required"));
        }

        for action in &self.actions {
            match (action.kind, action.intent.as_deref()) {
                (SurfaceActionKind::Local, None) => {}
                (SurfaceActionKind::Intent, Some(intent)) if intents.find(intent).is_some() => {}
                (SurfaceActionKind::Intent, Some(intent)) => {
                    return Err(invalid(format!(
                        "action `{}` references unknown intent `{intent}`",
                        action.id
                    )));
                }
                (SurfaceActionKind::Local, Some(_)) => {
                    return Err(invalid(format!(
                        "local action `{}` must not name an intent",
                        action.id
                    )));
                }
                (SurfaceActionKind::Intent, None) => {
                    return Err(invalid(format!(
                        "intent action `{}` must name an intent",
                        action.id
                    )));
                }
            }
            if let Some(region) = &action.target_region
                && !region_ids.contains(region.as_str())
            {
                return Err(invalid(format!(
                    "action `{}` targets unknown region `{region}`",
                    action.id
                )));
            }
        }

        for mode in &self.modes {
            let mut keys = BTreeSet::new();
            for binding in &mode.bindings {
                if binding.key.trim().is_empty() || binding.key != binding.key.to_ascii_lowercase()
                {
                    return Err(invalid(format!(
                        "mode `{}` contains non-normalized key `{}`",
                        mode.id, binding.key
                    )));
                }
                if !keys.insert(binding.key.as_str()) {
                    return Err(invalid(format!(
                        "mode `{}` binds key `{}` more than once",
                        mode.id, binding.key
                    )));
                }
                if !action_ids.contains(binding.action.as_str()) {
                    return Err(invalid(format!(
                        "mode `{}` references unknown action `{}`",
                        mode.id, binding.action
                    )));
                }
            }
        }

        let Some(approval) = self.mode("approval") else {
            return Err(invalid("the `approval` interaction mode is required"));
        };
        let approval_actions: BTreeSet<_> = approval
            .bindings
            .iter()
            .map(|binding| binding.action.as_str())
            .collect();
        let permitted = [
            "approve",
            "deny",
            "scroll_up",
            "scroll_down",
            "scroll_left",
            "scroll_right",
        ];
        if !approval_actions.contains("deny")
            || approval_actions
                .iter()
                .any(|action| !permitted.contains(action))
        {
            return Err(invalid(
                "approval mode must always deny and may only inspect, approve, or deny",
            ));
        }

        let mut last_columns = None;
        let mut last_rows = None;
        for viewport in &self.viewports {
            if last_columns.is_some_and(|columns| viewport.min_columns < columns)
                || last_rows.is_some_and(|rows| viewport.min_rows < rows)
            {
                return Err(invalid(
                    "viewports must be ordered from least to most specific",
                ));
            }
            last_columns = Some(viewport.min_columns);
            last_rows = Some(viewport.min_rows);
            let visible: BTreeSet<_> = viewport
                .visible_regions
                .iter()
                .map(String::as_str)
                .collect();
            if visible.len() != viewport.visible_regions.len() {
                return Err(invalid(format!(
                    "viewport `{}` contains a duplicate region",
                    viewport.id
                )));
            }
            for region in &visible {
                if !region_ids.contains(region) {
                    return Err(invalid(format!(
                        "viewport `{}` references unknown region `{region}`",
                        viewport.id
                    )));
                }
            }
            for required in ["canvas", "composer", "status"] {
                if !visible.contains(required) {
                    return Err(invalid(format!(
                        "viewport `{}` must retain `{required}`",
                        viewport.id
                    )));
                }
            }
            if !visible.contains(viewport.default_focus.as_str())
                || !self
                    .region(&viewport.default_focus)
                    .is_some_and(|region| region.focusable)
            {
                return Err(invalid(format!(
                    "viewport `{}` has an invisible or non-focusable default",
                    viewport.id
                )));
            }
            for region in self
                .regions
                .iter()
                .filter(|region| region.focusable && !visible.contains(region.id.as_str()))
            {
                let Some(action) = region.overlay_action.as_deref() else {
                    return Err(invalid(format!(
                        "hidden focusable region `{}` has no overlay action",
                        region.id
                    )));
                };
                if !action_ids.contains(action) {
                    return Err(invalid(format!(
                        "region `{}` references unknown overlay action `{action}`",
                        region.id
                    )));
                }
            }
        }

        for role in [
            "background",
            "panel",
            "raised",
            "line",
            "muted",
            "text",
            "accent",
            "warning",
            "danger",
            "success",
        ] {
            let Some(rgb) = self.theme.truecolor.get(role) else {
                return Err(invalid(format!("truecolor theme is missing `{role}`")));
            };
            if !is_rgb(rgb) {
                return Err(invalid(format!("truecolor role `{role}` is not #rrggbb")));
            }
            if !self.theme.color_256.contains_key(role) || !self.theme.color_16.contains_key(role) {
                return Err(invalid(format!(
                    "theme role `{role}` is missing a reduced-color fallback"
                )));
            }
        }
        for (name, glyph) in &self.theme.glyphs {
            if glyph.ascii.is_empty() || !glyph.ascii.is_ascii() {
                return Err(invalid(format!(
                    "glyph `{name}` must define a non-empty ASCII fallback"
                )));
            }
        }
        Ok(())
    }

    /// Finds an action by stable id.
    #[must_use]
    pub fn action(&self, id: &str) -> Option<&SurfaceAction> {
        self.actions.iter().find(|action| action.id == id)
    }

    /// Finds an interaction mode by stable id.
    #[must_use]
    pub fn mode(&self, id: &str) -> Option<&SurfaceMode> {
        self.modes.iter().find(|mode| mode.id == id)
    }

    /// Finds a semantic region by stable id.
    #[must_use]
    pub fn region(&self, id: &str) -> Option<&SurfaceRegion> {
        self.regions.iter().find(|region| region.id == id)
    }

    /// Selects the most specific viewport supported by the terminal dimensions.
    #[must_use]
    pub fn viewport(&self, columns: u16, rows: u16) -> &ViewportProfile {
        self.viewports
            .iter()
            .rev()
            .find(|viewport| columns >= viewport.min_columns && rows >= viewport.min_rows)
            .unwrap_or(&self.viewports[0])
    }
}

fn unique_ids<'a>(
    kind: &str,
    values: impl Iterator<Item = &'a String>,
) -> Result<BTreeSet<&'a str>, ContractError> {
    let mut ids = BTreeSet::new();
    for value in values {
        if value.is_empty()
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return Err(invalid(format!(
                "{kind} id `{value}` must be lower snake case"
            )));
        }
        if !ids.insert(value.as_str()) {
            return Err(invalid(format!("duplicate {kind} `{value}`")));
        }
    }
    Ok(ids)
}

fn is_rgb(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn invalid(message: impl Into<String>) -> ContractError {
    ContractError::Invalid(message.into())
}

#[cfg(test)]
mod tests {
    use super::{
        Audience, BindingConfig, INTENT_PROFILE_V1_YAML, IntentProfile, SURFACE_PROFILE_YAML,
        SurfaceProfile,
    };

    #[test]
    fn released_profile_and_bindings_agree() {
        let profile = IntentProfile::embedded().expect("profile");
        let bindings = BindingConfig::embedded().expect("bindings");
        bindings.validate_against(&profile).expect("agreement");
    }

    #[test]
    fn v1_exposure_is_normalized_to_v2_audiences() {
        let profile = IntentProfile::from_yaml(INTENT_PROFILE_V1_YAML).expect("v1 profile");
        assert_eq!(profile.format, "agentide.intent-profile/2");
        assert_eq!(
            profile
                .find("session_start")
                .expect("session start")
                .audiences,
            [Audience::Human]
        );
        assert_eq!(
            profile.find("code_read").expect("code read").audiences,
            [Audience::Agent, Audience::Automation]
        );
        assert!(
            profile
                .intents
                .iter()
                .all(|intent| intent.exposure.is_none())
        );
    }

    #[test]
    fn released_surface_profile_is_reachable_and_semantic() {
        let profile = SurfaceProfile::embedded().expect("surface profile");
        assert_eq!(profile.viewport(80, 24).id, "compact");
        assert_eq!(profile.viewport(120, 32).id, "standard");
        assert_eq!(profile.viewport(180, 50).id, "wide");
        assert_eq!(
            profile
                .action("show_diff")
                .and_then(|action| action.intent.as_deref()),
            Some("diff_show")
        );
    }

    #[test]
    fn surface_profile_rejects_key_collisions_and_unsafe_approval_modes() {
        let intents = IntentProfile::embedded().expect("intents");
        let collision = SURFACE_PROFILE_YAML.replace(
            "- {key: ctrl+p, action: quick_open}",
            "- {key: ctrl+k, action: quick_open}",
        );
        let error = SurfaceProfile::from_yaml(&collision, &intents).expect_err("collision");
        assert!(
            error
                .to_string()
                .contains("binds key `ctrl+k` more than once")
        );

        let unsafe_mode =
            SURFACE_PROFILE_YAML.replace("- {key: y, action: approve}", "- {key: y, action: quit}");
        let error = SurfaceProfile::from_yaml(&unsafe_mode, &intents).expect_err("unsafe mode");
        assert!(error.to_string().contains("approval mode"));
    }

    #[test]
    fn surface_profile_rejects_unknown_fields_and_non_ascii_fallbacks() {
        let intents = IntentProfile::embedded().expect("intents");
        let unknown = SURFACE_PROFILE_YAML.replacen(
            "format: agentide.surface-profile/1",
            "format: agentide.surface-profile/1\nsurprise: true",
            1,
        );
        assert!(SurfaceProfile::from_yaml(&unknown, &intents).is_err());

        let non_ascii = SURFACE_PROFILE_YAML.replace(
            "active: {unicode: \"●\", ascii: \"*\"}",
            "active: {unicode: \"●\", ascii: \"●\"}",
        );
        let error = SurfaceProfile::from_yaml(&non_ascii, &intents).expect_err("ASCII guard");
        assert!(error.to_string().contains("ASCII fallback"));
    }
}
