//! Immutable, versioned contracts between semantic `AgentIDE` intents and concrete drivers.
#![allow(clippy::missing_errors_doc)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// The released v1 intent catalogue.
pub const INTENT_PROFILE_YAML: &str = include_str!("../../../contracts/intent-profile.yaml");
/// Standalone bindings shipped with the binary.
pub const DEFAULT_BINDINGS_YAML: &str = include_str!("../../../contracts/default-bindings.yaml");

/// A model-visible semantic operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentDefinition {
    /// Stable snake-case name used on transports.
    pub name: String,
    /// Qualified ESS command providing its semantics.
    pub command: String,
    /// Whether this operation is model, operator, or conditionally visible.
    pub exposure: Exposure,
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

/// Intent visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
        let profile: Self = serde_yaml_ng::from_str(yaml)?;
        if profile.format != "agentide.intent-profile/1" {
            return Err(ContractError::Invalid(format!(
                "unsupported profile `{}`",
                profile.format
            )));
        }
        let mut names = BTreeSet::new();
        let mut commands = BTreeSet::new();
        for intent in &profile.intents {
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
            if matches!(intent.exposure, Exposure::Operator) {
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

#[cfg(test)]
mod tests {
    use super::{BindingConfig, IntentProfile};

    #[test]
    fn released_profile_and_bindings_agree() {
        let profile = IntentProfile::embedded().expect("profile");
        let bindings = BindingConfig::embedded().expect("bindings");
        bindings.validate_against(&profile).expect("agreement");
    }
}
