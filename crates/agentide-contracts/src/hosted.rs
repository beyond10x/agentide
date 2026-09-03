//! Renderer-neutral contracts for hosted coding sessions.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{Audience, Effect, IntentDefinition, IntentProfile, Risk};

/// Binary terminal output prefix width: one network-order sequence number.
pub const TERMINAL_SEQUENCE_PREFIX_BYTES: usize = 8;
/// Maximum replay retained by a conforming hosted terminal transport.
pub const TERMINAL_REPLAY_BYTES: usize = 4 * 1024 * 1024;

/// SHA-256 of a compact JSON value with recursively lexicographically sorted object keys.
///
/// The format discriminator is intentionally part of every sealed value. Consumers in other
/// languages can reproduce the digest without depending on Rust struct field order.
pub fn canonical_json_sha256<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    let mut bytes = Vec::new();
    write_canonical_json(&value, &mut bytes)?;
    Ok(sha256(&bytes))
}

fn write_canonical_json(value: &Value, bytes: &mut Vec<u8>) -> Result<(), serde_json::Error> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_writer(bytes, value)?;
        }
        Value::Array(values) => {
            bytes.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    bytes.push(b',');
                }
                write_canonical_json(value, bytes)?;
            }
            bytes.push(b']');
        }
        Value::Object(values) => {
            bytes.push(b'{');
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_by_key(|(left, _)| *left);
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    bytes.push(b',');
                }
                serde_json::to_writer(&mut *bytes, key)?;
                bytes.push(b':');
                write_canonical_json(value, bytes)?;
            }
            bytes.push(b'}');
        }
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_reference(value: &str) -> bool {
    !value.trim().is_empty() && !value.contains(['\0', '\r', '\n'])
}

fn parse_timestamp(value: &str) -> bool {
    DateTime::parse_from_rfc3339(value).is_ok()
}

/// Authenticated class of one session actor.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    /// A person using an interactive client.
    Human,
    /// A model-backed coding agent.
    Agent,
    /// A non-interactive service principal.
    Automation,
}

impl ActorKind {
    /// Matching actor-aware intent audience.
    #[must_use]
    pub const fn audience(self) -> Audience {
        match self {
            Self::Human => Audience::Human,
            Self::Agent => Audience::Agent,
            Self::Automation => Audience::Automation,
        }
    }
}

/// Server-derived identity and delegation coordinates for one request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActorContext {
    /// Contract discriminator.
    pub format: String,
    /// Authenticated actor class.
    pub kind: ActorKind,
    /// Stable authenticated subject.
    pub subject: String,
    /// Verified Agent Platform agent reference.
    pub agent: Option<String>,
    /// Verified Agent Platform attempt reference.
    pub attempt: Option<String>,
    /// Verified delegation reference.
    pub delegation: Option<String>,
}

impl ActorContext {
    /// Builds a minimal server-derived actor and validates its invariants.
    pub fn new(kind: ActorKind, subject: impl Into<String>) -> Result<Self, String> {
        let actor = Self {
            format: "agentide.actor-context/2".into(),
            kind,
            subject: subject.into(),
            agent: None,
            attempt: None,
            delegation: None,
        };
        actor.validate()?;
        Ok(actor)
    }

    /// Refuses malformed or caller-shaped actor coordinates.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.actor-context/2" || !valid_reference(&self.subject) {
            return Err("actor.invalid: actor context has an invalid format or subject".into());
        }
        if [&self.agent, &self.attempt, &self.delegation]
            .into_iter()
            .flatten()
            .any(|value| value.trim().is_empty())
        {
            return Err("actor.reference_invalid: actor references must not be empty".into());
        }
        if self.kind != ActorKind::Agent && (self.agent.is_some() || self.attempt.is_some()) {
            return Err(
                "actor.kind_mismatch: only an agent actor may carry agent or attempt references"
                    .into(),
            );
        }
        Ok(())
    }

    /// Stable key for actor-private state; never an authorization credential.
    #[must_use]
    pub fn view_key(&self) -> String {
        format!("{:?}:{}", self.kind, self.subject).to_ascii_lowercase()
    }
}

/// One-based line and column in an actor-private editor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CursorPosition {
    /// One-based line.
    pub line: u64,
    /// One-based column.
    pub column: u64,
}

impl CursorPosition {
    fn validate(&self) -> Result<(), String> {
        if self.line == 0 || self.column == 0 {
            return Err("cursor.invalid: line and column are one-based".into());
        }
        Ok(())
    }
}

/// Stable renderer-neutral pane metadata. Editor buffer bytes are deliberately absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkbenchPane {
    /// Stable actor-private pane identity.
    pub id: String,
    /// Editor, diff, transcript, evidence, terminal, or another declared renderer kind.
    pub kind: String,
    /// Optional durable resource reference opened by the pane.
    pub reference: Option<String>,
}

/// Actor-private workbench state. Unsaved source bytes are deliberately absent.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActorWorkbench {
    /// Ordered renderer-neutral pane descriptions.
    pub panes: Vec<WorkbenchPane>,
    /// Ordered open editor tabs.
    pub tabs: Vec<String>,
    /// Focused pane reference.
    pub focused_pane: Option<String>,
    /// Focused editor path.
    pub focused_file: Option<String>,
    /// Last saved cursor for each open path.
    pub cursors: BTreeMap<String, CursorPosition>,
    /// Selected terminal reference.
    pub selected_terminal: Option<String>,
    /// Paths with client-local unsaved buffers; bytes are never included.
    pub dirty_paths: Vec<String>,
}

impl ActorWorkbench {
    /// Validates actor-private metadata without accepting editor buffer bytes.
    pub fn validate(&self) -> Result<(), String> {
        let mut pane_ids = BTreeSet::new();
        for pane in &self.panes {
            if !valid_reference(&pane.id)
                || !valid_reference(&pane.kind)
                || !pane_ids.insert(&pane.id)
                || pane
                    .reference
                    .as_ref()
                    .is_some_and(|value| !valid_reference(value))
            {
                return Err(
                    "workbench.pane_invalid: pane ids and kinds must be unique and non-empty"
                        .into(),
                );
            }
        }
        for cursor in self.cursors.values() {
            cursor.validate()?;
        }
        if self
            .dirty_paths
            .iter()
            .any(|path| normalize_workspace_path(path).as_deref() != Some(path))
        {
            return Err(
                "workbench.dirty_path_invalid: dirty paths must be normalized workspace paths"
                    .into(),
            );
        }
        Ok(())
    }
}

/// Source of one deliberately attached context selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SelectionKind {
    /// Saved editor range.
    Editor,
    /// Canonical server-resolved diff hunk.
    DiffHunk,
    /// Explicit terminal selection.
    Terminal,
    /// Structured process result.
    Process,
    /// Durable evidence record.
    Evidence,
}

/// Server-derived provenance for deliberately shared context bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentProvenance {
    /// Contract discriminator.
    pub format: String,
    /// Authenticated actor who attached the bytes.
    pub actor: ActorContext,
    /// Workspace, diff, terminal, process, or evidence authority that supplied them.
    pub source: String,
    /// Immutable source revision, event id, or output sequence bound.
    pub source_revision: String,
    /// RFC 3339 server observation time.
    pub observed_at: String,
}

impl AttachmentProvenance {
    /// Validates that attachment provenance is attributable and immutable.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.attachment-provenance/1"
            || !valid_reference(&self.source)
            || !valid_reference(&self.source_revision)
            || !parse_timestamp(&self.observed_at)
        {
            return Err("context.provenance_invalid: provenance has an invalid format, source, revision, or timestamp".into());
        }
        self.actor.validate()
    }
}

/// One complete, bounded selection deliberately shared with the session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextSelection {
    /// Contract discriminator.
    pub format: String,
    /// Stable selection reference.
    pub id: String,
    /// Selection source.
    pub kind: SelectionKind,
    /// Workspace path or durable record reference.
    pub reference: String,
    /// Optional one-based start line.
    pub start_line: Option<u64>,
    /// Optional one-based end line.
    pub end_line: Option<u64>,
    /// Complete selected UTF-8 content.
    pub content: String,
    /// SHA-256 of the complete content.
    pub sha256: String,
    /// Server-derived source and actor provenance.
    pub provenance: AttachmentProvenance,
    /// Incomplete selections are visible to humans but refused for model injection.
    pub truncated: bool,
}

impl ContextSelection {
    /// Seals a complete selection with the digest of the exact UTF-8 bytes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        kind: SelectionKind,
        reference: impl Into<String>,
        start_line: Option<u64>,
        end_line: Option<u64>,
        content: impl Into<String>,
        provenance: AttachmentProvenance,
    ) -> Result<Self, String> {
        let content = content.into();
        let selection = Self {
            format: "agentide.context-selection/1".into(),
            id: id.into(),
            kind,
            reference: reference.into(),
            start_line,
            end_line,
            sha256: sha256(content.as_bytes()),
            content,
            provenance,
            truncated: false,
        };
        selection.validate()?;
        Ok(selection)
    }

    /// Validates source attribution, exact bytes, and one-based range semantics.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.context-selection/1"
            || !valid_reference(&self.id)
            || !valid_reference(&self.reference)
        {
            return Err(
                "context.selection_invalid: selection identity or reference is invalid".into(),
            );
        }
        match (self.start_line, self.end_line) {
            (None, None) => {}
            (Some(start), Some(end)) if start > 0 && end >= start => {}
            _ => {
                return Err(
                    "context.range_invalid: line bounds must be paired, one-based, and ordered"
                        .into(),
                );
            }
        }
        if !valid_sha256(&self.sha256) || sha256(self.content.as_bytes()) != self.sha256 {
            return Err("context.digest_mismatch: selection bytes do not match sha256".into());
        }
        self.provenance.validate()
    }
}

/// Metadata for an open file whose bytes were not injected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpenFileReference {
    /// Workspace-relative path.
    pub path: String,
    /// Complete current file digest.
    pub sha256: String,
    /// Actor-private cursor.
    pub cursor: Option<CursorPosition>,
    /// Whether the actor has a client-local unsaved buffer.
    pub dirty: bool,
}

impl OpenFileReference {
    /// Validates digest-bearing metadata for a file whose bytes were withheld.
    pub fn validate(&self) -> Result<(), String> {
        if normalize_workspace_path(&self.path).as_deref() != Some(self.path.as_str())
            || !valid_sha256(&self.sha256)
        {
            return Err("context.open_file_invalid: path or digest is invalid".into());
        }
        if let Some(cursor) = &self.cursor {
            cursor.validate()?;
        }
        Ok(())
    }
}

/// Bounded reference to a durable process, agent, approval, evidence, or activity record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextRecord {
    /// Durable record identity.
    pub id: String,
    /// Stable record kind.
    pub kind: String,
    /// Optional lifecycle state.
    pub state: Option<String>,
    /// Secret-free human summary.
    pub summary: String,
    /// Digest of the complete referred payload when available.
    pub sha256: Option<String>,
    /// RFC 3339 event or observation time when available.
    pub observed_at: Option<String>,
}

impl ContextRecord {
    /// Validates bounded durable metadata without importing its full payload.
    pub fn validate(&self) -> Result<(), String> {
        if !valid_reference(&self.id)
            || !valid_reference(&self.kind)
            || self.summary.contains(['\0', '\r'])
            || self
                .sha256
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
            || self
                .observed_at
                .as_ref()
                .is_some_and(|value| !parse_timestamp(value))
        {
            return Err("context.record_invalid: durable record metadata is invalid".into());
        }
        Ok(())
    }
}

/// Shared session context assembled immediately before a model turn.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPack {
    /// Contract discriminator.
    pub format: String,
    /// Session objective.
    pub objective: String,
    /// Exact pinned source revision.
    pub source_revision: String,
    /// Canonical current working-diff digest.
    pub working_changes: Option<String>,
    /// Deliberately shared pinned selections.
    pub pins: Vec<ContextSelection>,
    /// Deliberately attached selections for the next turn.
    pub focused_selections: Vec<ContextSelection>,
    /// Other open files represented without source bytes.
    pub open_files: Vec<OpenFileReference>,
    /// Active canonical diff selector.
    pub active_diff: Option<ChangeSelector>,
    /// Durable terminal metadata without raw scrollback.
    pub terminals: Vec<TerminalSession>,
    /// Structured process observations.
    pub processes: Vec<ContextRecord>,
    /// Agent lane observations.
    pub agent_lanes: Vec<ContextRecord>,
    /// Pending exact decisions and checkpoints.
    pub approvals: Vec<ContextRecord>,
    /// Durable evidence records.
    pub evidence: Vec<ContextRecord>,
    /// Recent secret-free activity.
    pub recent_activity: Vec<ContextRecord>,
    /// Monotonic context revision.
    pub revision: u64,
    /// SHA-256 of this complete pack with this field omitted.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub digest: String,
}

/// Fixed attachment limits further constrained by model context size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextBounds {
    /// Maximum combined pinned and focused selections.
    pub selections: usize,
    /// Maximum UTF-8 bytes in one selection.
    pub per_selection_bytes: usize,
    /// Maximum UTF-8 bytes across all selections.
    pub total_bytes: usize,
}

impl ContextBounds {
    /// Computes the eight/32-KiB/64-KiB limits and ten-percent context cap.
    #[must_use]
    pub fn for_context_window(context_window: usize) -> Self {
        Self {
            selections: 8,
            per_selection_bytes: 32 * 1024,
            total_bytes: (64 * 1024).min(context_window / 10),
        }
    }
}

impl ContextPack {
    /// Seals the current pack after validating all referenced records.
    pub fn seal(&mut self) -> Result<(), String> {
        self.validate_unsealed()?;
        let mut payload = self.clone();
        payload.digest.clear();
        self.digest = canonical_json_sha256(&payload)
            .map_err(|error| format!("context.canonicalization_failed: {error}"))?;
        Ok(())
    }

    /// Verifies the complete context and its canonical digest.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_unsealed()?;
        if !valid_sha256(&self.digest) {
            return Err("context.digest_invalid: context digest must be lowercase SHA-256".into());
        }
        let mut payload = self.clone();
        payload.digest.clear();
        let expected = canonical_json_sha256(&payload)
            .map_err(|error| format!("context.canonicalization_failed: {error}"))?;
        if self.digest != expected {
            return Err("context.digest_mismatch: context fields changed after sealing".into());
        }
        Ok(())
    }

    fn validate_unsealed(&self) -> Result<(), String> {
        if self.format != "agentide.context-pack/2"
            || self.objective.trim().is_empty()
            || !valid_reference(&self.source_revision)
            || self.revision == 0
            || self
                .working_changes
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
        {
            return Err("context.invalid: format, objective, source revision, working digest, or revision is invalid".into());
        }
        for selection in self.pins.iter().chain(&self.focused_selections) {
            selection.validate()?;
        }
        for file in &self.open_files {
            file.validate()?;
        }
        if let Some(selector) = &self.active_diff {
            selector.validate()?;
        }
        for terminal in &self.terminals {
            terminal.validate()?;
        }
        for record in self
            .processes
            .iter()
            .chain(&self.agent_lanes)
            .chain(&self.approvals)
            .chain(&self.evidence)
            .chain(&self.recent_activity)
        {
            record.validate()?;
        }
        Ok(())
    }

    /// Validates complete automatic attachments against every declared bound.
    pub fn validate_model_attachments(&self, context_window: usize) -> Result<(), String> {
        let bounds = ContextBounds::for_context_window(context_window);
        let selections = self.pins.iter().chain(&self.focused_selections);
        if selections.clone().count() > bounds.selections {
            return Err(format!(
                "context.selection_count_exceeded: maximum is {}",
                bounds.selections
            ));
        }
        let mut total = 0usize;
        for selection in selections {
            selection.validate()?;
            if selection.truncated {
                return Err(format!(
                    "context.selection_incomplete: `{}` is truncated",
                    selection.id
                ));
            }
            let size = selection.content.len();
            if size > bounds.per_selection_bytes {
                return Err(format!(
                    "context.selection_too_large: `{}` is {size} bytes; maximum is {}",
                    selection.id, bounds.per_selection_bytes
                ));
            }
            total = total.saturating_add(size);
        }
        if total > bounds.total_bytes {
            return Err(format!(
                "context.total_too_large: {total} bytes; maximum is {}",
                bounds.total_bytes
            ));
        }
        Ok(())
    }
}

/// Authorization route applicable to one current intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationPath {
    /// The authenticated human action authorizes exactly one reversible effect.
    ExplicitHumanAction,
    /// A current bounded grant covers the effect.
    BoundedGrant,
    /// The exact plan digest requires a separate human decision.
    ExactPlanApproval,
    /// Observation needs no additional grant.
    None,
}

/// One intent currently available to an actor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AvailableIntent {
    /// Released semantic definition.
    pub intent: IntentDefinition,
    /// Current authorization path.
    pub authorization: AuthorizationPath,
}

/// Why a known intent is not currently available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WithheldIntent {
    /// Semantic intent name.
    pub name: String,
    /// Stable refusal code.
    pub code: String,
    /// Secret-free explanation.
    pub message: String,
}

/// Exact actor-specific tool catalogue for one revision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntentInventory {
    /// Contract discriminator.
    pub format: String,
    /// Monotonic session inventory revision.
    pub revision: u64,
    /// Available operations.
    pub intents: Vec<AvailableIntent>,
    /// SHA-256 of the canonical available catalogue.
    pub digest: String,
}

impl IntentInventory {
    /// Canonically seals one current inventory.
    pub fn new(revision: u64, intents: Vec<AvailableIntent>) -> Result<Self, serde_json::Error> {
        let digest = canonical_json_sha256(&intents)?;
        Ok(Self {
            format: "agentide.intent-inventory/2".into(),
            revision,
            intents,
            digest,
        })
    }

    /// Verifies the exact current tool catalogue and deterministic digest.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.intent-inventory/2" || self.revision == 0 {
            return Err("inventory.invalid: format or revision is invalid".into());
        }
        let mut names = BTreeSet::new();
        if self
            .intents
            .iter()
            .any(|available| !names.insert(&available.intent.name))
        {
            return Err("inventory.duplicate_intent: intent names must be unique".into());
        }
        let expected = canonical_json_sha256(&self.intents)
            .map_err(|error| format!("inventory.canonicalization_failed: {error}"))?;
        if !valid_sha256(&self.digest) || self.digest != expected {
            return Err("inventory.digest_mismatch: catalogue does not match digest".into());
        }
        Ok(())
    }
}

/// One independently changing durable coordination identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoordinationRevision {
    /// Monotonic Service SDK/Eventlog projection revision.
    pub revision: u64,
    /// SHA-256 of the durable coordination snapshot or event cursor projection.
    pub digest: String,
}

impl CoordinationRevision {
    /// Validates that coordination state can be compared independently of context and tools.
    pub fn validate(&self) -> Result<(), String> {
        if self.revision == 0 || !valid_sha256(&self.digest) {
            return Err("coordination.revision_invalid: revision and digest are required".into());
        }
        Ok(())
    }
}

/// Actor-derived hosted workbench projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActorView {
    /// Contract discriminator.
    pub format: String,
    /// Server-derived actor coordinates.
    pub actor: ActorContext,
    /// Actor-private workbench state.
    pub workbench: ActorWorkbench,
    /// Service SDK/Eventlog coordination identity, independent of prompt context.
    pub coordination: CoordinationRevision,
    /// Deliberately shared context.
    pub context: ContextPack,
    /// Exact current tool inventory.
    pub inventory: IntentInventory,
    /// Known intents currently withheld and why.
    pub withheld: Vec<WithheldIntent>,
}

impl ActorView {
    /// Verifies an actor-derived view before it enters a renderer or model turn.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.actor-view/2" {
            return Err("actor_view.format_invalid: unsupported actor view format".into());
        }
        self.actor.validate()?;
        self.workbench.validate()?;
        self.coordination.validate()?;
        self.context.validate()?;
        self.inventory.validate()?;
        let available = self
            .inventory
            .intents
            .iter()
            .map(|intent| intent.intent.name.as_str())
            .collect::<BTreeSet<_>>();
        if self.withheld.iter().any(|withheld| {
            !valid_reference(&withheld.name)
                || !valid_reference(&withheld.code)
                || withheld.message.trim().is_empty()
                || available.contains(withheld.name.as_str())
        }) {
            return Err(
                "actor_view.inventory_overlap: withheld intents are invalid or available".into(),
            );
        }
        Ok(())
    }
}

/// A revocable bounded authorization for routine operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorityGrant {
    /// Contract discriminator.
    pub format: String,
    /// Stable grant identity.
    pub id: String,
    /// Parent session.
    pub session_id: String,
    /// Exact actor subject receiving authority.
    pub grantee: String,
    /// Closed semantic intent set.
    pub allowed_intents: Vec<String>,
    /// Normalized workspace-relative path prefixes.
    pub path_prefixes: Vec<String>,
    /// Highest admitted consequence tier.
    pub maximum_risk: Risk,
    /// RFC 3339 expiry; absent means session close.
    pub expires_at: Option<String>,
    /// Monotonic revision used by dispatch and revocation.
    pub revision: u64,
    /// Whether the grant is revoked.
    pub revoked: bool,
}

impl AuthorityGrant {
    /// Validates identity, time, intents, and normalized confined path prefixes.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.authority-grant/2"
            || self.id.trim().is_empty()
            || self.session_id.trim().is_empty()
            || self.grantee.trim().is_empty()
            || self.allowed_intents.is_empty()
            || self.revision == 0
        {
            return Err(
                "authority.grant_invalid: grant is missing required identity or scope".into(),
            );
        }
        let mut intents = BTreeSet::new();
        if self
            .allowed_intents
            .iter()
            .any(|intent| !valid_reference(intent) || !intents.insert(intent))
        {
            return Err(
                "authority.intent_scope_invalid: intent scope must be unique and non-empty".into(),
            );
        }
        if self.path_prefixes.is_empty()
            || self
                .path_prefixes
                .iter()
                .any(|prefix| normalize_workspace_path(prefix).as_deref() != Some(prefix.as_str()))
        {
            return Err(
                "authority.path_scope_invalid: path prefixes must be normalized workspace paths"
                    .into(),
            );
        }
        if let Some(expires_at) = &self.expires_at {
            DateTime::parse_from_rfc3339(expires_at)
                .map_err(|_| "authority.expiry_invalid: expiry must be RFC 3339".to_owned())?;
        }
        Ok(())
    }

    /// Reports whether this grant is valid and unexpired at the supplied server time.
    #[must_use]
    pub fn is_current_at(&self, now: DateTime<Utc>) -> bool {
        !self.revoked
            && self.validate().is_ok()
            && self.expires_at.as_ref().is_none_or(|expires_at| {
                DateTime::parse_from_rfc3339(expires_at).is_ok_and(|expiry| expiry > now)
            })
    }

    /// Tests actor, intent, path, and risk without interpreting expiry.
    #[must_use]
    pub fn admits(
        &self,
        session_id: &str,
        actor: &ActorContext,
        intent: &IntentDefinition,
        path: Option<&str>,
    ) -> bool {
        !self.revoked
            && self.validate().is_ok()
            && self.session_id == session_id
            && self.grantee == actor.subject
            && self.allowed_intents.contains(&intent.name)
            && intent.risk <= self.maximum_risk
            && path.is_none_or(|path| {
                let Some(path) = normalize_workspace_path(path) else {
                    return false;
                };
                self.path_prefixes.iter().any(|prefix| {
                    prefix.is_empty()
                        || path == *prefix
                        || path
                            .strip_prefix(prefix)
                            .is_some_and(|suffix| suffix.starts_with('/'))
                })
            })
    }

    /// Creates a child grant that is the strict intersection of this grant and a request.
    #[allow(clippy::too_many_arguments)]
    pub fn intersect_for_child(
        &self,
        id: impl Into<String>,
        child_subject: impl Into<String>,
        requested_intents: &[String],
        requested_prefixes: &[String],
        requested_maximum_risk: Risk,
        requested_expiry: Option<&str>,
        revision: u64,
    ) -> Result<Self, String> {
        self.validate()?;
        let requested_expiry_time = requested_expiry
            .map(DateTime::parse_from_rfc3339)
            .transpose()
            .map_err(|_| "child grant expiry must be RFC 3339".to_owned())?;
        let parent_expiry_time = self
            .expires_at
            .as_deref()
            .map(DateTime::parse_from_rfc3339)
            .transpose()
            .map_err(|_| "parent grant expiry must be RFC 3339".to_owned())?;
        let expires_at = match (parent_expiry_time, requested_expiry_time) {
            (Some(parent), Some(requested)) if parent <= requested => self.expires_at.clone(),
            (Some(_) | None, Some(_)) => requested_expiry.map(str::to_owned),
            (Some(_), None) => self.expires_at.clone(),
            (None, None) => None,
        };
        let allowed_intents = requested_intents
            .iter()
            .filter(|intent| self.allowed_intents.contains(intent))
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let mut path_prefixes = BTreeSet::new();
        for requested in requested_prefixes {
            let requested = normalize_workspace_path(requested)
                .filter(|normalized| normalized == requested)
                .ok_or_else(|| "child grant path prefixes must be normalized".to_owned())?;
            for parent in &self.path_prefixes {
                if path_contains(parent, &requested) {
                    path_prefixes.insert(requested.clone());
                } else if path_contains(&requested, parent) {
                    path_prefixes.insert(parent.clone());
                }
            }
        }
        if allowed_intents.is_empty() || path_prefixes.is_empty() {
            return Err("child grant has no authority after intersection".into());
        }
        let child = Self {
            format: "agentide.authority-grant/2".into(),
            id: id.into(),
            session_id: self.session_id.clone(),
            grantee: child_subject.into(),
            allowed_intents,
            path_prefixes: path_prefixes.into_iter().collect(),
            maximum_risk: self.maximum_risk.min(requested_maximum_risk),
            expires_at,
            revision,
            revoked: false,
        };
        child.validate()?;
        Ok(child)
    }
}

/// Resolves one actor-specific inventory from currently wired implementations and grants.
///
/// `implemented` is supplied by the composing service after Connector, Workspace, Substrate, and
/// Agent Platform bindings are resolved. It is not caller-provided request data.
#[allow(clippy::too_many_arguments)]
pub fn resolve_intent_inventory(
    profile: &IntentProfile,
    session_id: &str,
    actor: &ActorContext,
    implemented: &BTreeSet<String>,
    grants: &[AuthorityGrant],
    session_active: bool,
    now: DateTime<Utc>,
    revision: u64,
) -> Result<(IntentInventory, Vec<WithheldIntent>), String> {
    actor.validate()?;
    let mut available = Vec::new();
    let mut withheld = Vec::new();
    for intent in &profile.intents {
        match authorize_intent(
            intent,
            session_id,
            actor,
            implemented,
            grants,
            session_active,
            now,
            None,
        ) {
            Ok(authorization) => available.push(AvailableIntent {
                intent: intent.clone(),
                authorization,
            }),
            Err(reason) => withheld.push(reason),
        }
    }
    let inventory = IntentInventory::new(revision, available).map_err(|error| error.to_string())?;
    Ok((inventory, withheld))
}

/// Rechecks one current operation, including its normalized workspace path, before dispatch.
#[allow(clippy::too_many_arguments)]
pub fn authorize_intent(
    intent: &IntentDefinition,
    session_id: &str,
    actor: &ActorContext,
    implemented: &BTreeSet<String>,
    grants: &[AuthorityGrant],
    session_active: bool,
    now: DateTime<Utc>,
    path: Option<&str>,
) -> Result<AuthorizationPath, WithheldIntent> {
    let refuse = |code: &str, message: &str| WithheldIntent {
        name: intent.name.clone(),
        code: code.into(),
        message: message.into(),
    };
    if actor.validate().is_err() || !valid_reference(session_id) {
        return Err(refuse(
            "actor.invalid",
            "the server-derived actor or session identity is invalid",
        ));
    }
    if !intent.audiences.contains(&actor.kind.audience()) {
        return Err(refuse(
            "intent.actor_withheld",
            "the authenticated actor kind is not an audience for this intent",
        ));
    }
    if !implemented.contains(&intent.name) {
        return Err(refuse(
            "intent.unavailable",
            "no current deployment binding implements this intent",
        ));
    }
    if intent.name == "session_start" && session_active {
        return Err(refuse(
            "session.already_active",
            "session start is unavailable inside an active session",
        ));
    }
    if intent.name != "session_start"
        && !session_active
        && !matches!(intent.effect, Effect::Observe)
    {
        return Err(refuse(
            "session.inactive",
            "the session is not active for effects",
        ));
    }
    if matches!(intent.effect, Effect::Observe | Effect::State) && intent.risk <= Risk::Low {
        return Ok(AuthorizationPath::None);
    }
    if matches!(intent.effect, Effect::External) || intent.risk >= Risk::High {
        return Ok(AuthorizationPath::ExactPlanApproval);
    }
    if actor.kind == ActorKind::Human && intent.name != "interactive_terminal" {
        return Ok(AuthorizationPath::ExplicitHumanAction);
    }
    if grants
        .iter()
        .any(|grant| grant.is_current_at(now) && grant.admits(session_id, actor, intent, path))
    {
        return Ok(AuthorizationPath::BoundedGrant);
    }
    Err(refuse(
        "authority.grant_required",
        "a current bounded grant does not cover this actor, intent, risk, and path",
    ))
}

fn normalize_workspace_path(path: &str) -> Option<String> {
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return None;
    }
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" if parts.is_empty() => {}
            "" | "." | ".." => return None,
            value => parts.push(value),
        }
    }
    Some(parts.join("/"))
}

fn path_contains(prefix: &str, path: &str) -> bool {
    prefix.is_empty()
        || path == prefix
        || path
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

/// Authoritative source selector for one canonical diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChangeSelector {
    /// Writable materialization against its immutable base.
    Workspace,
    /// Exact pending or approved plan.
    Plan {
        /// Exact plan digest.
        digest: String,
    },
    /// One Agent Platform attempt.
    AgentAttempt {
        /// Agent Platform attempt reference.
        attempt_id: String,
    },
    /// One publication projection.
    Publication {
        /// Publication record reference.
        publication_id: String,
    },
    /// Explicit immutable revision pair.
    RevisionPair {
        /// Old immutable revision.
        old: String,
        /// New immutable revision.
        new: String,
    },
}

impl ChangeSelector {
    /// Validates immutable selector coordinates before canonical resolution.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Workspace => Ok(()),
            Self::Plan { digest } if valid_sha256(digest) => Ok(()),
            Self::AgentAttempt { attempt_id } if valid_reference(attempt_id) => Ok(()),
            Self::Publication { publication_id } if valid_reference(publication_id) => Ok(()),
            Self::RevisionPair { old, new }
                if valid_reference(old) && valid_reference(new) && old != new =>
            {
                Ok(())
            }
            _ => Err(
                "diff.selector_invalid: selector coordinates are incomplete or malformed".into(),
            ),
        }
    }
}

/// Requested canonical diff detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiffMode {
    /// Structured hunks and lines.
    Patch,
    /// Per-file statistics.
    Stat,
    /// File names and statuses only.
    FilesOnly,
}

/// One old or new hunk range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffRange {
    /// One-based start line.
    pub start: u64,
    /// Number of represented lines.
    pub lines: u64,
}

impl DiffRange {
    fn validate(&self) -> Result<(), String> {
        if (self.lines == 0 && self.start != 0) || (self.lines > 0 && self.start == 0) {
            return Err(
                "diff.range_invalid: empty ranges start at zero and non-empty ranges are one-based"
                    .into(),
            );
        }
        Ok(())
    }
}

/// Semantic kind of one canonical diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    /// Unchanged context.
    Context,
    /// New-side addition.
    Addition,
    /// Old-side deletion.
    Deletion,
    /// Canonical no-newline marker.
    NoNewline,
}

/// One structured canonical diff line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffLine {
    /// Context, addition, deletion, or no-newline marker.
    pub kind: DiffLineKind,
    /// Old-side line number.
    pub old_line: Option<u64>,
    /// New-side line number.
    pub new_line: Option<u64>,
    /// UTF-8 line content.
    pub content: String,
}

impl DiffLine {
    fn validate(&self) -> Result<(), String> {
        let valid = match self.kind {
            DiffLineKind::Context => self.old_line.is_some() && self.new_line.is_some(),
            DiffLineKind::Addition => self.old_line.is_none() && self.new_line.is_some(),
            DiffLineKind::Deletion => self.old_line.is_some() && self.new_line.is_none(),
            DiffLineKind::NoNewline => self.old_line.is_none() && self.new_line.is_none(),
        };
        if !valid || self.old_line == Some(0) || self.new_line == Some(0) {
            return Err(
                "diff.line_invalid: line numbers do not match the semantic line kind".into(),
            );
        }
        Ok(())
    }
}

/// One structured canonical diff hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffHunk {
    /// Stable hunk digest.
    pub id: String,
    /// Old-side range.
    pub old: DiffRange,
    /// New-side range.
    pub new: DiffRange,
    /// Optional function or section heading.
    pub heading: Option<String>,
    /// Ordered structured lines.
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    fn validate(&self) -> Result<(), String> {
        if !valid_sha256(&self.id) {
            return Err("diff.hunk_digest_invalid: hunk id must be lowercase SHA-256".into());
        }
        self.old.validate()?;
        self.new.validate()?;
        for line in &self.lines {
            line.validate()?;
        }
        Ok(())
    }
}

/// Canonical status of one changed file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DiffFileStatus {
    /// File exists only on the new side.
    Added,
    /// File content changed in place.
    Modified,
    /// File exists only on the old side.
    Deleted,
    /// File path changed.
    Renamed,
    /// Executable or another supported mode changed.
    ModeChanged,
    /// Binary content changed without text statistics.
    Binary,
}

/// One changed file in a canonical projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffFile {
    /// Workspace-relative old path.
    pub old_path: Option<String>,
    /// Workspace-relative new path.
    pub new_path: Option<String>,
    /// Added, modified, deleted, renamed, mode-changed, or binary.
    pub status: DiffFileStatus,
    /// Text additions when representable.
    pub additions: Option<u64>,
    /// Text deletions when representable.
    pub deletions: Option<u64>,
    /// Complete old file digest.
    pub old_sha256: Option<String>,
    /// Complete new file digest.
    pub new_sha256: Option<String>,
    /// Hunks in patch mode.
    pub hunks: Vec<DiffHunk>,
    /// Actors or operations attributed by the server.
    pub attribution: Vec<String>,
}

impl DiffFile {
    fn validate(&self) -> Result<(), String> {
        for path in [&self.old_path, &self.new_path].into_iter().flatten() {
            if normalize_workspace_path(path).as_deref() != Some(path.as_str()) {
                return Err(
                    "diff.path_invalid: diff paths must be normalized workspace paths".into(),
                );
            }
        }
        let path_shape = match self.status {
            DiffFileStatus::Added => self.old_path.is_none() && self.new_path.is_some(),
            DiffFileStatus::Deleted => self.old_path.is_some() && self.new_path.is_none(),
            DiffFileStatus::Renamed => {
                self.old_path.is_some() && self.new_path.is_some() && self.old_path != self.new_path
            }
            _ => self.old_path.is_some() && self.new_path.is_some(),
        };
        if !path_shape
            || self
                .old_sha256
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
            || self
                .new_sha256
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
            || self.attribution.iter().any(|value| !valid_reference(value))
        {
            return Err(
                "diff.file_invalid: file status, paths, digests, or attribution are inconsistent"
                    .into(),
            );
        }
        if self.status == DiffFileStatus::Binary
            && (self.additions.is_some() || self.deletions.is_some() || !self.hunks.is_empty())
        {
            return Err(
                "diff.binary_has_patch: binary files cannot carry text statistics or hunks".into(),
            );
        }
        for hunk in &self.hunks {
            hunk.validate()?;
        }
        Ok(())
    }
}

/// One server-resolved authoritative diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffProjection {
    /// Contract discriminator.
    pub format: String,
    /// Exact resolved selector.
    pub selector: ChangeSelector,
    /// Returned detail mode.
    pub mode: DiffMode,
    /// SHA-256 of the complete canonical projection.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub digest: String,
    /// Total changed files before client filtering.
    pub file_count: u64,
    /// Total text additions.
    pub additions: u64,
    /// Total text deletions.
    pub deletions: u64,
    /// Ordered changed files.
    pub files: Vec<DiffFile>,
}

impl DiffProjection {
    /// Seals one canonical server-side projection.
    pub fn seal(&mut self) -> Result<(), String> {
        self.validate_unsealed()?;
        let mut payload = self.clone();
        payload.digest.clear();
        self.digest = canonical_json_sha256(&payload)
            .map_err(|error| format!("diff.canonicalization_failed: {error}"))?;
        Ok(())
    }

    /// Verifies summary counts, structured files, selector, and canonical digest.
    pub fn validate(&self) -> Result<(), String> {
        self.validate_unsealed()?;
        let mut payload = self.clone();
        payload.digest.clear();
        let expected = canonical_json_sha256(&payload)
            .map_err(|error| format!("diff.canonicalization_failed: {error}"))?;
        if !valid_sha256(&self.digest) || self.digest != expected {
            return Err("diff.digest_mismatch: projection fields changed after resolution".into());
        }
        Ok(())
    }

    fn validate_unsealed(&self) -> Result<(), String> {
        if self.format != "agentide.diff-projection/2" {
            return Err("diff.format_invalid: unsupported diff projection format".into());
        }
        self.selector.validate()?;
        for file in &self.files {
            file.validate()?;
        }
        let additions = self
            .files
            .iter()
            .filter_map(|file| file.additions)
            .sum::<u64>();
        let deletions = self
            .files
            .iter()
            .filter_map(|file| file.deletions)
            .sum::<u64>();
        if self.file_count != self.files.len() as u64
            || self.additions != additions
            || self.deletions != deletions
        {
            return Err(
                "diff.summary_mismatch: file or line totals do not match the projection".into(),
            );
        }
        Ok(())
    }
}

/// Workspace modification state for one complete file revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileModificationState {
    /// Matches the immutable base.
    Unchanged,
    /// Exists only in the writable materialization.
    Added,
    /// Differs from the immutable base.
    Modified,
    /// Deleted from the writable materialization.
    Deleted,
}

/// Complete-file revision metadata returned with every bounded read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileRevision {
    /// Contract discriminator.
    pub format: String,
    /// Workspace-relative path.
    pub path: String,
    /// SHA-256 of the complete file.
    pub sha256: String,
    /// Complete file size.
    pub size: u64,
    /// Language id inferred from path.
    pub language: String,
    /// Unchanged, added, modified, or deleted.
    pub state: FileModificationState,
    /// Whether the returned content is complete.
    pub complete: bool,
}

impl FileRevision {
    /// Validates complete-file identity independently of bounded response content.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.file-revision/2"
            || normalize_workspace_path(&self.path).as_deref() != Some(self.path.as_str())
            || !valid_sha256(&self.sha256)
            || !valid_reference(&self.language)
        {
            return Err(
                "file.revision_invalid: format, path, digest, or language is invalid".into(),
            );
        }
        Ok(())
    }
}

/// Bounded editor read with explicit binary and completeness state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileProjection {
    /// Complete-file metadata.
    pub revision: FileRevision,
    /// UTF-8 content when editable.
    pub content: Option<String>,
    /// Whether the file is binary or unsupported for editing.
    pub read_only: bool,
    /// Stable refusal or partial reason when content is absent or incomplete.
    pub reason: Option<String>,
}

impl FileProjection {
    /// Validates explicit complete, partial, refused, and read-only file states.
    pub fn validate(&self) -> Result<(), String> {
        self.revision.validate()?;
        if let Some(content) = &self.content {
            if !self.revision.complete
                || self.read_only
                || self.reason.is_some()
                || self.revision.size != content.len() as u64
                || sha256(content.as_bytes()) != self.revision.sha256
            {
                return Err(
                    "file.content_mismatch: editable content is not the complete declared revision"
                        .into(),
                );
            }
        } else if !self.read_only && self.reason.is_none() {
            return Err("file.state_ambiguous: absent content must be read-only or carry an explicit reason".into());
        }
        if !self.revision.complete && self.reason.is_none() {
            return Err(
                "file.partial_reason_missing: incomplete content requires an explicit reason"
                    .into(),
            );
        }
        Ok(())
    }
}

/// Exact human or granted replacement of one saved file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaceFile {
    /// Workspace-relative path.
    pub path: String,
    /// Complete new UTF-8 content.
    pub content: String,
    /// Digest loaded before editing; required to prevent blind overwrite.
    pub expected_sha256: String,
}

impl ReplaceFile {
    /// Validates a stale-safe complete replacement request.
    pub fn validate(&self) -> Result<(), String> {
        validate_mutation_path(&self.path)?;
        if !valid_sha256(&self.expected_sha256) {
            return Err(
                "file.expected_digest_invalid: replacement requires a lowercase SHA-256".into(),
            );
        }
        Ok(())
    }
}

/// Exact creation of one absent file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateFile {
    /// Workspace-relative path.
    pub path: String,
    /// Complete new UTF-8 content.
    pub content: String,
    /// Must be true; makes absent-state authority explicit.
    pub expected_absent: bool,
}

impl CreateFile {
    /// Validates an exact absent-state creation request.
    pub fn validate(&self) -> Result<(), String> {
        validate_mutation_path(&self.path)?;
        if !self.expected_absent {
            return Err(
                "file.expected_absent_required: creation must bind the absent state".into(),
            );
        }
        Ok(())
    }
}

/// Exact deletion of one loaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteFile {
    /// Workspace-relative path.
    pub path: String,
    /// Digest observed before deletion.
    pub expected_sha256: String,
}

impl DeleteFile {
    /// Validates an exact digest-bound deletion request.
    pub fn validate(&self) -> Result<(), String> {
        validate_mutation_path(&self.path)?;
        if !valid_sha256(&self.expected_sha256) {
            return Err(
                "file.expected_digest_invalid: deletion requires a lowercase SHA-256".into(),
            );
        }
        Ok(())
    }
}

/// Exact rename of one loaded file to an absent destination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenameFile {
    /// Existing workspace-relative path.
    pub from: String,
    /// New workspace-relative path.
    pub to: String,
    /// Digest observed for the source.
    pub expected_sha256: String,
    /// Must be true; makes destination absent-state authority explicit.
    pub expected_destination_absent: bool,
}

impl RenameFile {
    /// Validates an exact source and absent-destination rename request.
    pub fn validate(&self) -> Result<(), String> {
        validate_mutation_path(&self.from)?;
        validate_mutation_path(&self.to)?;
        if self.from == self.to || !valid_sha256(&self.expected_sha256) {
            return Err(
                "file.rename_invalid: source, destination, or expected digest is invalid".into(),
            );
        }
        if !self.expected_destination_absent {
            return Err(
                "file.expected_absent_required: rename must bind the destination absent state"
                    .into(),
            );
        }
        Ok(())
    }
}

fn validate_mutation_path(path: &str) -> Result<(), String> {
    if path.is_empty() || normalize_workspace_path(path).as_deref() != Some(path) {
        return Err(
            "file.path_invalid: mutation path must be a non-empty normalized workspace path".into(),
        );
    }
    Ok(())
}

/// Declared terminal access to the writable Workspace materialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerminalWorkspaceAccess {
    /// Immutable base and working files are readable only.
    ReadOnly,
    /// Working materialization is writable inside Workspace authority.
    ReadWrite,
}

/// Deployment-declared confined interactive terminal profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminalProfile {
    /// Contract discriminator.
    pub format: String,
    /// Stable profile identity.
    pub id: String,
    /// Immutable Substrate runtime/toolchain reference.
    pub runtime: String,
    /// Fixed shell argv.
    pub shell: Vec<String>,
    /// Initial workspace-relative directory.
    pub working_directory: String,
    /// Read-only or read-write workspace posture.
    pub workspace_access: TerminalWorkspaceAccess,
    /// Explicit sanitized environment names.
    pub environment: Vec<String>,
    /// `none` or a deployment-named confined egress policy.
    pub network: String,
    /// CPU ceiling in milliseconds.
    pub cpu_millis: u64,
    /// Memory ceiling in bytes.
    pub memory_bytes: u64,
    /// Process ceiling.
    pub process_limit: u64,
}

impl TerminalProfile {
    /// Validates one deployment-declared confined terminal profile.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.terminal-profile/2"
            || !valid_reference(&self.id)
            || !valid_reference(&self.runtime)
            || self.shell.is_empty()
            || self.shell.iter().any(|value| !valid_reference(value))
            || normalize_workspace_path(&self.working_directory).as_deref()
                != Some(self.working_directory.as_str())
            || !valid_reference(&self.network)
            || self.cpu_millis == 0
            || self.memory_bytes == 0
            || self.process_limit == 0
        {
            return Err(
                "terminal.profile_invalid: confinement profile is incomplete or malformed".into(),
            );
        }
        let mut names = BTreeSet::new();
        if self.environment.iter().any(|name| {
            name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
                || !names.insert(name)
        }) {
            return Err("terminal.environment_invalid: sanitized environment names must be unique shell identifiers".into());
        }
        Ok(())
    }
}

/// Durable terminal lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerminalState {
    /// PTY is running and attachable.
    Running,
    /// PTY exited naturally.
    Exited,
    /// An authorized explicit termination completed.
    Terminated,
}

/// Durable Substrate PTY identity and confinement metadata; scrollback is absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminalSession {
    /// Contract discriminator.
    pub format: String,
    /// Stable terminal identity.
    pub id: String,
    /// Parent coding session.
    pub session_id: String,
    /// Deployment terminal profile.
    pub profile: String,
    /// Server-derived creator.
    pub actor: ActorContext,
    /// Substrate process/session identity.
    pub process_id: String,
    /// Current workspace-relative directory.
    pub working_directory: String,
    /// Declared network posture.
    pub network: String,
    /// Lifecycle state.
    pub state: TerminalState,
    /// Last observed replay sequence.
    pub output_sequence: u64,
    /// Exit code once known.
    pub exit_code: Option<i32>,
}

impl TerminalSession {
    /// Validates durable PTY identity and its declared confinement metadata.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.terminal-session/2"
            || !valid_reference(&self.id)
            || !valid_reference(&self.session_id)
            || !valid_reference(&self.profile)
            || !valid_reference(&self.process_id)
            || normalize_workspace_path(&self.working_directory).as_deref()
                != Some(self.working_directory.as_str())
            || !valid_reference(&self.network)
            || (self.state == TerminalState::Running && self.exit_code.is_some())
        {
            return Err("terminal.session_invalid: terminal identity, path, network, or lifecycle is inconsistent".into());
        }
        self.actor.validate()
    }
}

/// Signals admitted by the hosted terminal control protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerminalSignal {
    /// Terminal hangup.
    Hangup,
    /// Interactive interrupt.
    Interrupt,
    /// Graceful process termination.
    Terminate,
    /// Immediate termination, always a separate explicit action.
    Kill,
}

/// One browser-to-BFF JSON terminal control. Stdin remains a binary frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TerminalControl {
    /// Request replay strictly after a previously observed output sequence.
    Replay {
        /// Last fully observed sequence.
        after: u64,
    },
    /// Resize the Substrate PTY.
    Resize {
        /// Non-zero terminal columns.
        columns: u16,
        /// Non-zero terminal rows.
        rows: u16,
    },
    /// Detach this browser without terminating the PTY.
    Detach,
    /// Explicitly terminate the PTY.
    Terminate,
    /// Send one admitted process signal.
    Signal {
        /// Requested confined signal.
        signal: TerminalSignal,
    },
}

/// Versioned client terminal control envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminalControlFrame {
    /// Contract discriminator.
    pub format: String,
    /// Stable request identity for lifecycle/refusal correlation.
    pub request_id: String,
    /// Durable terminal identity.
    pub terminal_id: String,
    /// Exact requested control.
    pub control: TerminalControl,
}

impl TerminalControlFrame {
    /// Validates a complete JSON control before forwarding it to Workspace.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.terminal-control/1"
            || !valid_reference(&self.request_id)
            || !valid_reference(&self.terminal_id)
        {
            return Err("terminal.control_invalid: control envelope identity is invalid".into());
        }
        if matches!(
            self.control,
            TerminalControl::Resize { columns: 0, .. } | TerminalControl::Resize { rows: 0, .. }
        ) {
            return Err("terminal.resize_invalid: rows and columns must be non-zero".into());
        }
        Ok(())
    }
}

/// Explicit replay coverage returned during attach or reconnect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminalReplayBounds {
    /// Sequence requested by the client.
    pub requested_after: u64,
    /// Earliest sequence for which the retained window is complete.
    pub available_after: u64,
    /// Latest sequence currently retained or emitted.
    pub latest: u64,
    /// Whether no requested output fell outside the bounded ring.
    pub complete: bool,
}

impl TerminalReplayBounds {
    fn validate(&self) -> Result<(), String> {
        if self.available_after > self.latest
            || (self.complete && self.requested_after < self.available_after)
        {
            return Err("terminal.replay_bounds_invalid: replay coverage is contradictory".into());
        }
        Ok(())
    }
}

/// Durable terminal control event. Substrate's existing JSON WSS carries PTY data and replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TerminalEvent {
    /// PTY created.
    Created {
        /// Created terminal metadata.
        terminal: Box<TerminalSession>,
    },
    /// Client attached or reattached after an observed sequence.
    Attached {
        /// Attached terminal.
        terminal_id: String,
        /// Last sequence already observed by the client.
        after: u64,
    },
    /// Client detached without terminating the PTY.
    Detached {
        /// Detached terminal.
        terminal_id: String,
    },
    /// PTY dimensions changed.
    Resized {
        /// Resized terminal.
        terminal_id: String,
        /// New columns.
        columns: u16,
        /// New rows.
        rows: u16,
    },
    /// Replay could not cover the complete requested window.
    ReplayPartial {
        /// Terminal whose replay was partial.
        terminal_id: String,
        /// Sequence requested by the client.
        requested_after: u64,
        /// Earliest sequence retained by Substrate.
        available_after: u64,
    },
    /// PTY exited.
    Exited {
        /// Exited terminal.
        terminal_id: String,
        /// Process exit code, when available.
        exit_code: Option<i32>,
        /// Terminating signal, when available.
        signal: Option<String>,
    },
    /// Control request refused without changing the PTY.
    Refused {
        /// Target terminal, when the refusal addressed one.
        terminal_id: Option<String>,
        /// Stable refusal code.
        code: String,
    },
}

impl TerminalEvent {
    /// Validates durable lifecycle events independently of raw PTY scrollback.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::Created { terminal } => terminal.validate(),
            Self::Attached { terminal_id, .. } | Self::Detached { terminal_id } => {
                if valid_reference(terminal_id) {
                    Ok(())
                } else {
                    Err("terminal.event_invalid: terminal identity is invalid".into())
                }
            }
            Self::Resized {
                terminal_id,
                columns,
                rows,
            } => {
                if valid_reference(terminal_id) && *columns > 0 && *rows > 0 {
                    Ok(())
                } else {
                    Err("terminal.resize_invalid: terminal, rows, and columns are required".into())
                }
            }
            Self::ReplayPartial {
                terminal_id,
                requested_after,
                available_after,
            } => {
                if valid_reference(terminal_id) && requested_after < available_after {
                    Ok(())
                } else {
                    Err(
                        "terminal.replay_bounds_invalid: partial replay must omit requested output"
                            .into(),
                    )
                }
            }
            Self::Exited {
                terminal_id,
                signal,
                ..
            } => {
                if valid_reference(terminal_id)
                    && signal.as_ref().is_none_or(|value| valid_reference(value))
                {
                    Ok(())
                } else {
                    Err("terminal.exit_invalid: exit identity or signal is invalid".into())
                }
            }
            Self::Refused { terminal_id, code } => {
                if terminal_id
                    .as_ref()
                    .is_none_or(|value| valid_reference(value))
                    && valid_reference(code)
                {
                    Ok(())
                } else {
                    Err("terminal.refusal_invalid: refusal identity or code is invalid".into())
                }
            }
        }
    }
}

/// Versioned server-to-browser terminal lifecycle envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TerminalServerFrame {
    /// Contract discriminator.
    pub format: String,
    /// Correlated client request when applicable.
    pub request_id: Option<String>,
    /// Current replay coverage when attach/replay was requested.
    pub replay: Option<TerminalReplayBounds>,
    /// Durable lifecycle or refusal event.
    pub event: TerminalEvent,
}

impl TerminalServerFrame {
    /// Validates lifecycle correlation and replay metadata.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.terminal-server-frame/1"
            || self
                .request_id
                .as_ref()
                .is_some_and(|value| !valid_reference(value))
        {
            return Err(
                "terminal.server_frame_invalid: frame format or request identity is invalid".into(),
            );
        }
        if let Some(replay) = &self.replay {
            replay.validate()?;
        }
        self.event.validate()
    }
}

/// Decoded server binary frame containing sequence-prefixed PTY output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalOutputFrame {
    /// Monotonic PTY output sequence.
    pub sequence: u64,
    /// Exact PTY output bytes after the prefix.
    pub output: Vec<u8>,
}

impl TerminalOutputFrame {
    /// Encodes an eight-byte big-endian sequence prefix followed by unchanged PTY bytes.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut frame = Vec::with_capacity(TERMINAL_SEQUENCE_PREFIX_BYTES + self.output.len());
        frame.extend_from_slice(&self.sequence.to_be_bytes());
        frame.extend_from_slice(&self.output);
        frame
    }

    /// Decodes one bounded server binary frame.
    pub fn decode(frame: &[u8]) -> Result<Self, String> {
        if frame.len() < TERMINAL_SEQUENCE_PREFIX_BYTES {
            return Err(
                "terminal.binary_prefix_missing: server output lacks the sequence prefix".into(),
            );
        }
        if frame.len() - TERMINAL_SEQUENCE_PREFIX_BYTES > TERMINAL_REPLAY_BYTES {
            return Err(
                "terminal.binary_frame_too_large: output exceeds the bounded replay capacity"
                    .into(),
            );
        }
        let prefix: [u8; TERMINAL_SEQUENCE_PREFIX_BYTES] =
            frame[..TERMINAL_SEQUENCE_PREFIX_BYTES].try_into().map_err(
                |_| "terminal.binary_prefix_missing: server output lacks the sequence prefix",
            )?;
        let sequence = u64::from_be_bytes(prefix);
        Ok(Self {
            sequence,
            output: frame[TERMINAL_SEQUENCE_PREFIX_BYTES..].to_vec(),
        })
    }
}

/// Semantic kind of a bounded project-tree entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TreeEntryKind {
    /// Complete supported file.
    File,
    /// Directory.
    Directory,
    /// Symlink, submodule, or another explicitly unsupported entry.
    Unsupported,
}

/// One bounded project-tree entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TreeEntry {
    /// Workspace-relative path.
    pub path: String,
    /// File, directory, or unsupported.
    pub kind: TreeEntryKind,
    /// Complete file size when applicable.
    pub size: Option<u64>,
    /// Complete file digest when applicable.
    pub sha256: Option<String>,
}

impl TreeEntry {
    fn validate(&self) -> Result<(), String> {
        if normalize_workspace_path(&self.path).as_deref() != Some(self.path.as_str())
            || self
                .sha256
                .as_ref()
                .is_some_and(|value| !valid_sha256(value))
        {
            return Err("tree.entry_invalid: entry path or digest is invalid".into());
        }
        let metadata_matches = match self.kind {
            TreeEntryKind::File => self.size.is_some() && self.sha256.is_some(),
            TreeEntryKind::Directory | TreeEntryKind::Unsupported => self.sha256.is_none(),
        };
        if !metadata_matches {
            return Err(
                "tree.entry_metadata_invalid: entry metadata does not match its kind".into(),
            );
        }
        Ok(())
    }
}

/// Bounded searchable tree result with explicit omission state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TreeProjection {
    /// Contract discriminator.
    pub format: String,
    /// Ordered entries.
    pub entries: Vec<TreeEntry>,
    /// Whether more entries exist.
    pub truncated: bool,
    /// Known omitted count.
    pub omitted: Option<u64>,
    /// Opaque next cursor.
    pub next_cursor: Option<String>,
}

impl TreeProjection {
    /// Validates explicit complete, truncated, omitted, and cursor states.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "agentide.tree-projection/2"
            || (!self.truncated && (self.omitted.is_some() || self.next_cursor.is_some()))
            || self
                .next_cursor
                .as_ref()
                .is_some_and(|value| !valid_reference(value))
        {
            return Err(
                "tree.projection_invalid: format or pagination state is contradictory".into(),
            );
        }
        for entry in &self.entries {
            entry.validate()?;
        }
        Ok(())
    }
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Approval, Effect};

    fn provenance() -> AttachmentProvenance {
        AttachmentProvenance {
            format: "agentide.attachment-provenance/1".into(),
            actor: ActorContext::new(ActorKind::Human, "user:one").expect("actor"),
            source: "workspace".into(),
            source_revision: "commit:0123456789abcdef".into(),
            observed_at: "2026-09-03T10:00:00Z".into(),
        }
    }

    fn sealed_diff() -> DiffProjection {
        let mut projection = DiffProjection {
            format: "agentide.diff-projection/2".into(),
            selector: ChangeSelector::Workspace,
            mode: DiffMode::Patch,
            digest: String::new(),
            file_count: 1,
            additions: 1,
            deletions: 1,
            files: vec![DiffFile {
                old_path: Some("src/lib.rs".into()),
                new_path: Some("src/lib.rs".into()),
                status: DiffFileStatus::Modified,
                additions: Some(1),
                deletions: Some(1),
                old_sha256: Some("a".repeat(64)),
                new_sha256: Some("b".repeat(64)),
                hunks: vec![DiffHunk {
                    id: "c".repeat(64),
                    old: DiffRange { start: 1, lines: 1 },
                    new: DiffRange { start: 1, lines: 1 },
                    heading: None,
                    lines: vec![
                        DiffLine {
                            kind: DiffLineKind::Deletion,
                            old_line: Some(1),
                            new_line: None,
                            content: "old".into(),
                        },
                        DiffLine {
                            kind: DiffLineKind::Addition,
                            old_line: None,
                            new_line: Some(1),
                            content: "new".into(),
                        },
                    ],
                }],
                attribution: vec!["operation:edit-one".into()],
            }],
        };
        projection.seal().expect("seal diff");
        projection
    }

    fn intent() -> IntentDefinition {
        IntentDefinition {
            name: "code_edit".into(),
            command: "agentide.coding.EditCode".into(),
            audiences: vec![Audience::Human, Audience::Agent],
            exposure: None,
            port: "workspace".into(),
            effect: Effect::Mutate,
            risk: Risk::Medium,
            approval: Approval::Required,
            subjects: vec!["path".into()],
        }
    }

    #[test]
    fn grant_is_actor_path_and_risk_bounded() {
        let actor = ActorContext::new(ActorKind::Agent, "agent:one").expect("actor");
        let grant = AuthorityGrant {
            format: "agentide.authority-grant/2".into(),
            id: "grant".into(),
            session_id: "session".into(),
            grantee: actor.subject.clone(),
            allowed_intents: vec!["code_edit".into()],
            path_prefixes: vec!["src".into()],
            maximum_risk: Risk::Medium,
            expires_at: None,
            revision: 1,
            revoked: false,
        };
        assert!(grant.admits("session", &actor, &intent(), Some("src/lib.rs")));
        assert!(!grant.admits("other", &actor, &intent(), Some("src/lib.rs")));
        assert!(!grant.admits("session", &actor, &intent(), Some("tests/check.rs")));
    }

    #[test]
    fn canonical_json_digest_does_not_depend_on_object_key_order() {
        let left = serde_json::json!({"z": 1, "a": {"right": 2, "left": 1}});
        let right = serde_json::json!({"a": {"left": 1, "right": 2}, "z": 1});
        assert_eq!(
            canonical_json_sha256(&left).expect("left digest"),
            canonical_json_sha256(&right).expect("right digest")
        );
    }

    #[test]
    fn attached_selection_refuses_changed_bytes() {
        let mut selection = ContextSelection::new(
            "selection",
            SelectionKind::Editor,
            "src/lib.rs",
            Some(1),
            Some(1),
            "trusted bytes",
            provenance(),
        )
        .expect("selection");
        selection.content.push_str(" changed");
        assert_eq!(
            selection.validate().expect_err("digest mismatch"),
            "context.digest_mismatch: selection bytes do not match sha256"
        );
    }

    #[test]
    fn context_digest_covers_every_field() {
        let mut context = ContextPack {
            format: "agentide.context-pack/2".into(),
            objective: "Review the protocol".into(),
            source_revision: "commit:0123456789abcdef".into(),
            pins: vec![
                ContextSelection::new(
                    "selection",
                    SelectionKind::Editor,
                    "src/lib.rs",
                    Some(1),
                    Some(1),
                    "trusted bytes",
                    provenance(),
                )
                .expect("selection"),
            ],
            revision: 1,
            ..ContextPack::default()
        };
        context.seal().expect("seal context");
        context.validate().expect("valid context");
        context.objective.push_str(" changed");
        assert!(
            context
                .validate()
                .expect_err("digest mismatch")
                .starts_with("context.digest_mismatch")
        );
    }

    #[test]
    fn model_context_counts_pins_and_focused_selections_together() {
        let selection = ContextSelection::new(
            "selection",
            SelectionKind::Editor,
            "src/lib.rs",
            Some(1),
            Some(1),
            "x".repeat(60),
            AttachmentProvenance {
                format: "agentide.attachment-provenance/1".into(),
                actor: ActorContext::new(ActorKind::Human, "user:one").expect("actor"),
                source: "workspace".into(),
                source_revision: "revision:one".into(),
                observed_at: "2026-09-03T10:00:00Z".into(),
            },
        )
        .expect("selection");
        let mut context = ContextPack {
            pins: vec![selection.clone()],
            focused_selections: vec![selection],
            ..ContextPack::default()
        };
        assert!(
            context
                .validate_model_attachments(1_000)
                .expect_err("combined total")
                .starts_with("context.total_too_large")
        );
        context.focused_selections[0].truncated = true;
        assert!(
            context
                .validate_model_attachments(10_000)
                .expect_err("truncation")
                .starts_with("context.selection_incomplete")
        );
    }

    #[test]
    fn inventory_is_actor_and_authority_specific() {
        let profile = IntentProfile::embedded().expect("profile");
        let implemented = ["code_edit", "interactive_terminal", "code_publish"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let now = "2026-09-03T10:00:00Z".parse().expect("time");
        let human = ActorContext::new(ActorKind::Human, "user:one").expect("human");
        let agent = ActorContext::new(ActorKind::Agent, "agent:one").expect("agent");

        let (human_inventory, human_withheld) =
            resolve_intent_inventory(&profile, "session", &human, &implemented, &[], true, now, 3)
                .expect("human inventory");
        assert!(human_inventory.intents.iter().any(|available| {
            available.intent.name == "code_edit"
                && available.authorization == AuthorizationPath::ExplicitHumanAction
        }));
        assert!(human_withheld.iter().any(|withheld| {
            withheld.name == "interactive_terminal" && withheld.code == "authority.grant_required"
        }));

        let grant = AuthorityGrant {
            format: "agentide.authority-grant/2".into(),
            id: "grant-agent".into(),
            session_id: "session".into(),
            grantee: agent.subject.clone(),
            allowed_intents: vec!["code_edit".into()],
            path_prefixes: vec!["src".into()],
            maximum_risk: Risk::Medium,
            expires_at: Some("2026-09-03T11:00:00Z".into()),
            revision: 1,
            revoked: false,
        };
        let (agent_inventory, _) = resolve_intent_inventory(
            &profile,
            "session",
            &agent,
            &implemented,
            std::slice::from_ref(&grant),
            true,
            now,
            4,
        )
        .expect("agent inventory");
        assert!(agent_inventory.intents.iter().any(|available| {
            available.intent.name == "code_edit"
                && available.authorization == AuthorizationPath::BoundedGrant
        }));
        assert_eq!(
            authorize_intent(
                profile.find("code_edit").expect("edit"),
                "session",
                &agent,
                &implemented,
                &[grant],
                true,
                now,
                Some("../outside"),
            )
            .expect_err("traversal refusal")
            .code,
            "authority.grant_required"
        );
    }

    #[test]
    fn inventory_digest_is_deterministic_and_detects_mutation() {
        let available = AvailableIntent {
            intent: intent(),
            authorization: AuthorizationPath::BoundedGrant,
        };
        let left = IntentInventory::new(1, vec![available.clone()]).expect("inventory");
        let mut right = IntentInventory::new(9, vec![available]).expect("inventory");
        assert_eq!(left.digest, right.digest);
        right.intents[0].authorization = AuthorizationPath::ExactPlanApproval;
        assert!(
            right
                .validate()
                .expect_err("digest mismatch")
                .starts_with("inventory.digest_mismatch")
        );
    }

    #[test]
    fn delegated_grant_is_an_intersection() {
        let parent = AuthorityGrant {
            format: "agentide.authority-grant/2".into(),
            id: "parent".into(),
            session_id: "session".into(),
            grantee: "agent:parent".into(),
            allowed_intents: vec!["code_edit".into(), "code_create".into()],
            path_prefixes: vec!["src".into()],
            maximum_risk: Risk::Medium,
            expires_at: Some("2026-09-03T12:00:00Z".into()),
            revision: 1,
            revoked: false,
        };
        let child = parent
            .intersect_for_child(
                "child",
                "agent:child",
                &["code_edit".into(), "code_publish".into()],
                &["src/lib".into(), "tests".into()],
                Risk::High,
                Some("2026-09-03T13:00:00Z"),
                2,
            )
            .expect("intersection");
        assert_eq!(child.allowed_intents, ["code_edit"]);
        assert_eq!(child.path_prefixes, ["src/lib"]);
        assert_eq!(child.maximum_risk, Risk::Medium);
        assert_eq!(child.expires_at.as_deref(), Some("2026-09-03T12:00:00Z"));
    }

    #[test]
    fn canonical_diff_refuses_changed_counts_or_content() {
        let projection = sealed_diff();
        projection.validate().expect("valid diff");

        let mut wrong_count = projection.clone();
        wrong_count.additions = 2;
        assert!(
            wrong_count
                .validate()
                .expect_err("summary mismatch")
                .starts_with("diff.summary_mismatch")
        );

        let mut changed_content = projection;
        changed_content.files[0].hunks[0].lines[1].content = "different".into();
        assert!(
            changed_content
                .validate()
                .expect_err("digest mismatch")
                .starts_with("diff.digest_mismatch")
        );
    }

    #[test]
    fn terminal_binary_frames_are_network_order_and_bounded() {
        let output = TerminalOutputFrame {
            sequence: 0x0102_0304_0506_0708,
            output: vec![0x1b, b'[', b'3', b'1', b'm'],
        };
        let encoded = output.encode();
        assert_eq!(&encoded[..8], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(
            TerminalOutputFrame::decode(&encoded).expect("decode"),
            output
        );
        assert!(
            TerminalOutputFrame::decode(&[0; 7])
                .expect_err("short prefix")
                .starts_with("terminal.binary_prefix_missing")
        );
        let too_large = vec![0; TERMINAL_SEQUENCE_PREFIX_BYTES + TERMINAL_REPLAY_BYTES + 1];
        assert!(
            TerminalOutputFrame::decode(&too_large)
                .expect_err("oversized output")
                .starts_with("terminal.binary_frame_too_large")
        );
    }
}
