//! Renderer-neutral contracts for hosted coding sessions.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{Audience, Effect, IntentDefinition, IntentProfile, Risk};

/// Authenticated class of one session actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            format: "agentide.actor-context/1".into(),
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
        if self.format != "agentide.actor-context/1" || self.subject.trim().is_empty() {
            return Err("actor context has an invalid format or subject".into());
        }
        if [&self.agent, &self.attempt, &self.delegation]
            .into_iter()
            .flatten()
            .any(|value| value.trim().is_empty())
        {
            return Err("actor references must not be empty".into());
        }
        if self.kind != ActorKind::Agent && (self.agent.is_some() || self.attempt.is_some()) {
            return Err("only an agent actor may carry agent or attempt references".into());
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CursorPosition {
    /// One-based line.
    pub line: u64,
    /// One-based column.
    pub column: u64,
}

/// Actor-private workbench state. Unsaved source bytes are deliberately absent.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorWorkbench {
    /// Ordered renderer-neutral pane descriptions.
    pub panes: Vec<Value>,
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

/// Source of one deliberately attached context selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// One complete, bounded selection deliberately shared with the session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextSelection {
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
    /// Incomplete selections are visible to humans but refused for model injection.
    pub truncated: bool,
}

/// Metadata for an open file whose bytes were not injected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Shared session context assembled immediately before a model turn.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
    pub processes: Vec<Value>,
    /// Agent lane observations.
    pub agent_lanes: Vec<Value>,
    /// Pending exact decisions and checkpoints.
    pub approvals: Vec<Value>,
    /// Durable evidence records.
    pub evidence: Vec<Value>,
    /// Recent secret-free activity.
    pub recent_activity: Vec<Value>,
    /// Monotonic context revision.
    pub revision: u64,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AvailableIntent {
    /// Released semantic definition.
    pub intent: IntentDefinition,
    /// Current authorization path.
    pub authorization: AuthorizationPath,
}

/// Why a known intent is not currently available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        let digest = sha256(&serde_json::to_vec(&intents)?);
        Ok(Self {
            format: "agentide.intent-inventory/1".into(),
            revision,
            intents,
            digest,
        })
    }
}

/// Actor-derived hosted workbench projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActorView {
    /// Contract discriminator.
    pub format: String,
    /// Server-derived actor coordinates.
    pub actor: ActorContext,
    /// Actor-private workbench state.
    pub workbench: ActorWorkbench,
    /// Deliberately shared context.
    pub context: ContextPack,
    /// Exact current tool inventory.
    pub inventory: IntentInventory,
    /// Known intents currently withheld and why.
    pub withheld: Vec<WithheldIntent>,
}

/// A revocable bounded authorization for routine operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        if self.format != "agentide.authority-grant/1"
            || self.id.trim().is_empty()
            || self.session_id.trim().is_empty()
            || self.grantee.trim().is_empty()
            || self.allowed_intents.is_empty()
            || self.revision == 0
        {
            return Err("authority grant is missing required identity or scope".into());
        }
        if self.path_prefixes.is_empty()
            || self
                .path_prefixes
                .iter()
                .any(|prefix| normalize_workspace_path(prefix).as_deref() != Some(prefix.as_str()))
        {
            return Err("authority grant path prefixes must be normalized workspace paths".into());
        }
        if let Some(expires_at) = &self.expires_at {
            DateTime::parse_from_rfc3339(expires_at)
                .map_err(|_| "authority grant expiry must be RFC 3339".to_owned())?;
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
        actor: &ActorContext,
        intent: &IntentDefinition,
        path: Option<&str>,
    ) -> bool {
        !self.revoked
            && self.validate().is_ok()
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
            format: "agentide.authority-grant/1".into(),
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
pub fn resolve_intent_inventory(
    profile: &IntentProfile,
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
pub fn authorize_intent(
    intent: &IntentDefinition,
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
        .any(|grant| grant.is_current_at(now) && grant.admits(actor, intent, path))
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Requested canonical diff detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffRange {
    /// One-based start line.
    pub start: u64,
    /// Number of represented lines.
    pub lines: u64,
}

/// One structured canonical diff line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffLine {
    /// Context, addition, deletion, or no-newline marker.
    pub kind: String,
    /// Old-side line number.
    pub old_line: Option<u64>,
    /// New-side line number.
    pub new_line: Option<u64>,
    /// UTF-8 line content.
    pub content: String,
}

/// One structured canonical diff hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// One changed file in a canonical projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffFile {
    /// Workspace-relative old path.
    pub old_path: Option<String>,
    /// Workspace-relative new path.
    pub new_path: Option<String>,
    /// Added, modified, deleted, renamed, mode-changed, or binary.
    pub status: String,
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

/// One server-resolved authoritative diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffProjection {
    /// Contract discriminator.
    pub format: String,
    /// Exact resolved selector.
    pub selector: ChangeSelector,
    /// Returned detail mode.
    pub mode: DiffMode,
    /// SHA-256 of the complete canonical projection.
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

/// Complete-file revision metadata returned with every bounded read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub state: String,
    /// Whether the returned content is complete.
    pub complete: bool,
}

/// Bounded editor read with explicit binary and completeness state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Exact human or granted replacement of one saved file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplaceFile {
    /// Workspace-relative path.
    pub path: String,
    /// Complete new UTF-8 content.
    pub content: String,
    /// Digest loaded before editing; required to prevent blind overwrite.
    pub expected_sha256: String,
}

/// Exact creation of one absent file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateFile {
    /// Workspace-relative path.
    pub path: String,
    /// Complete new UTF-8 content.
    pub content: String,
    /// Must be true; makes absent-state authority explicit.
    pub expected_absent: bool,
}

/// Exact deletion of one loaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteFile {
    /// Workspace-relative path.
    pub path: String,
    /// Digest observed before deletion.
    pub expected_sha256: String,
}

/// Exact rename of one loaded file to an absent destination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Deployment-declared confined interactive terminal profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminalProfile {
    /// Stable profile identity.
    pub id: String,
    /// Immutable Substrate runtime/toolchain reference.
    pub runtime: String,
    /// Fixed shell argv.
    pub shell: Vec<String>,
    /// Initial workspace-relative directory.
    pub working_directory: String,
    /// Read-only or read-write workspace posture.
    pub workspace_access: String,
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

/// Durable terminal lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Durable terminal control event. Substrate's existing JSON WSS carries PTY data and replay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// One bounded project-tree entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeEntry {
    /// Workspace-relative path.
    pub path: String,
    /// File, directory, or unsupported.
    pub kind: String,
    /// Complete file size when applicable.
    pub size: Option<u64>,
    /// Complete file digest when applicable.
    pub sha256: Option<String>,
}

/// Bounded searchable tree result with explicit omission state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            format: "agentide.authority-grant/1".into(),
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
        assert!(grant.admits(&actor, &intent(), Some("src/lib.rs")));
        assert!(!grant.admits(&actor, &intent(), Some("tests/check.rs")));
    }

    #[test]
    fn model_context_counts_pins_and_focused_selections_together() {
        let selection = ContextSelection {
            id: "selection".into(),
            kind: SelectionKind::Editor,
            reference: "src/lib.rs".into(),
            start_line: Some(1),
            end_line: Some(1),
            content: "x".repeat(60),
            sha256: sha256(b"placeholder"),
            truncated: false,
        };
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
            resolve_intent_inventory(&profile, &human, &implemented, &[], true, now, 3)
                .expect("human inventory");
        assert!(human_inventory.intents.iter().any(|available| {
            available.intent.name == "code_edit"
                && available.authorization == AuthorizationPath::ExplicitHumanAction
        }));
        assert!(human_withheld.iter().any(|withheld| {
            withheld.name == "interactive_terminal" && withheld.code == "authority.grant_required"
        }));

        let grant = AuthorityGrant {
            format: "agentide.authority-grant/1".into(),
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
    fn delegated_grant_is_an_intersection() {
        let parent = AuthorityGrant {
            format: "agentide.authority-grant/1".into(),
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
}
