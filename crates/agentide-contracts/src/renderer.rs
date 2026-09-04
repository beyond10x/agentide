//! Transport-neutral contracts shared by browser renderer targets and their hosts.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Current renderer target lifecycle contract.
pub const RENDERER_TARGET_FORMAT: &str = "agentide.renderer-target/2";
/// Current immutable frame contract.
pub const RENDERER_FRAME_FORMAT: &str = "agentide.renderer-frame/2";
/// Current host-to-renderer transient event contract.
pub const RENDERER_EVENT_FORMAT: &str = "agentide.renderer-event/2";
/// Current renderer-to-controller semantic action contract.
pub const RENDERER_ACTION_FORMAT: &str = "agentide.renderer-action/2";

/// One renderer implementation discoverable by a browser host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererTargetManifest {
    /// Contract discriminator.
    pub format: String,
    /// Stable target name.
    pub id: String,
    /// Informational implementation family.
    pub framework: String,
    /// Accepted frame format.
    pub frame_format: String,
    /// Accepted event format.
    pub event_format: String,
    /// Emitted action format.
    pub action_format: String,
}

impl RendererTargetManifest {
    /// Verifies that a target implements the complete current protocol.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != RENDERER_TARGET_FORMAT
            || self.frame_format != RENDERER_FRAME_FORMAT
            || self.event_format != RENDERER_EVENT_FORMAT
            || self.action_format != RENDERER_ACTION_FORMAT
            || self.id.trim().is_empty()
            || self.framework.trim().is_empty()
        {
            return Err("renderer.target_invalid: renderer target contract is incomplete".into());
        }
        Ok(())
    }
}

/// Lifecycle states a renderer may display for a coding session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererSessionStatus {
    /// The backing workspace is still being prepared.
    Preparing,
    /// The session accepts interactions.
    Active,
    /// The durable `AgentIDE` session is closed.
    Closed,
    /// The session finished successfully.
    Completed,
    /// The session failed.
    Failed,
    /// A replacement session superseded this one.
    Superseded,
}

/// Display-only session metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererSession {
    /// Session identity, never a bearer credential.
    pub id: String,
    /// Operator-visible objective.
    pub objective: String,
    /// Current lifecycle state.
    pub status: RendererSessionStatus,
    /// Latest event cursor represented by the frame.
    pub cursor: u64,
}

/// Renderer-neutral pane kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererPaneKind {
    /// Workspace source editor.
    Editor,
    /// Current source changes.
    Diff,
    /// Interactive terminal.
    Terminal,
    /// Agent conversation.
    Chat,
    /// Durable event timeline.
    Timeline,
    /// Collaborating agents.
    Agents,
    /// Exact-plan approvals.
    Approvals,
    /// Evidence and outcomes.
    Evidence,
}

/// Renderer-neutral pane metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererPane {
    /// Stable pane identity.
    pub id: String,
    /// Semantic pane kind.
    pub kind: RendererPaneKind,
    /// Human-readable title.
    pub title: String,
    /// Optional workspace-relative path.
    pub path: Option<String>,
    /// Optional one-based line.
    pub line: Option<u64>,
    /// Optional one-based column.
    pub column: Option<u64>,
}

/// Workspace tree entry kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererTreeEntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symbolic link.
    Symlink,
}

/// One entry in a lazily observed workspace tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererTreeEntry {
    /// Normalized workspace-relative path.
    pub path: String,
    /// Display name.
    pub name: String,
    /// Entry kind.
    pub kind: RendererTreeEntryKind,
}

/// Discriminator for a workspace tree page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererTreeProjectionKind {
    /// Workspace tree page.
    Tree,
}

/// One bounded page of the workspace tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererTreeProjection {
    /// The projection discriminator.
    pub kind: RendererTreeProjectionKind,
    /// Directory represented by this page.
    pub root: String,
    /// Bounded entries in stable order.
    pub entries: Vec<RendererTreeEntry>,
    /// Opaque continuation cursor supplied by the host.
    pub next_cursor: Option<String>,
}

/// One observed editor document. Draft content remains browser-local.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererEditorDocument {
    /// Normalized workspace-relative path.
    pub path: String,
    /// Renderer language identifier.
    pub language: String,
    /// Current browser projection bytes.
    pub content: String,
    /// Host-issued optimistic-concurrency version.
    pub version: String,
    /// Whether writes are unavailable.
    pub read_only: bool,
    /// Whether content differs from the last host observation.
    pub dirty: bool,
}

/// One source editor projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererEditorProjection {
    /// Observed document or browser-local draft.
    pub document: RendererEditorDocument,
}

/// Workspace change kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererChangeStatus {
    /// Added path.
    Added,
    /// Modified path.
    Modified,
    /// Deleted path.
    Deleted,
    /// Renamed path.
    Renamed,
    /// Untracked path.
    Untracked,
}

/// One observed workspace change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererChange {
    /// Normalized workspace-relative path.
    pub path: String,
    /// Change kind.
    pub status: RendererChangeStatus,
    /// Optional bounded unified patch.
    pub patch: Option<String>,
}

/// Current workspace change projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererDiffProjection {
    /// Exact source commit used as the baseline.
    pub baseline_commit: String,
    /// Bounded changes.
    pub changes: Vec<RendererChange>,
    /// Whether more changes or patch bytes exist.
    pub truncated: bool,
}

/// Roles represented in an agent conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererChatRole {
    /// Operator message.
    User,
    /// Agent response.
    Assistant,
    /// Host-produced system message.
    System,
}

/// Lifecycle of a projected chat message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererChatMessageState {
    /// More Markdown deltas may follow.
    Streaming,
    /// The message is complete.
    Complete,
    /// The turn failed after the message began.
    Failed,
}

/// One safely rendered agent conversation message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererChatMessage {
    /// Stable message identity.
    pub id: String,
    /// Message role.
    pub role: RendererChatRole,
    /// Markdown source, potentially incomplete while streaming.
    pub markdown: String,
    /// Message lifecycle.
    pub state: RendererChatMessageState,
    /// RFC 3339 creation time.
    pub created_at: String,
}

/// Current agent conversation projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererChatProjection {
    /// Ordered messages.
    pub messages: Vec<RendererChatMessage>,
}

/// Lifecycle of an interactive terminal projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererTerminalState {
    /// The terminal is being created.
    Opening,
    /// The terminal accepts input.
    Open,
    /// The terminal has closed.
    Closed,
    /// Terminal creation or transport failed.
    Failed,
}

/// Current interactive terminal projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererTerminalProjection {
    /// Host-owned terminal identity.
    pub terminal_id: String,
    /// Terminal lifecycle.
    pub state: RendererTerminalState,
    /// Current columns.
    pub columns: u16,
    /// Current rows.
    pub rows: u16,
}

/// Named refusal displayed in a pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererRefusalProjection {
    /// Stable refusal code.
    pub code: String,
    /// Operator-safe explanation.
    pub message: String,
    /// Whether retrying may succeed without changing the request.
    pub retryable: bool,
}

/// Empty pane state with an operator-facing explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererEmptyProjection {
    /// Operator-facing explanation.
    pub message: String,
}

/// Typed projection selected for one pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RendererPaneProjection {
    /// Source editor state.
    Editor(RendererEditorProjection),
    /// Source changes.
    Diff(RendererDiffProjection),
    /// Agent conversation.
    Chat(RendererChatProjection),
    /// Interactive terminal state.
    Terminal(RendererTerminalProjection),
    /// Named refusal.
    Refusal(RendererRefusalProjection),
    /// Empty state.
    Empty(RendererEmptyProjection),
}

impl RendererPaneProjection {
    fn pane_kind(&self) -> Option<RendererPaneKind> {
        match self {
            Self::Editor(_) => Some(RendererPaneKind::Editor),
            Self::Diff(_) => Some(RendererPaneKind::Diff),
            Self::Chat(_) => Some(RendererPaneKind::Chat),
            Self::Terminal(_) => Some(RendererPaneKind::Terminal),
            Self::Refusal(_) | Self::Empty(_) => None,
        }
    }
}

/// Display-only workbench projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererWorkbench {
    /// Ordered panes.
    pub panes: Vec<RendererPane>,
    /// Focused pane identity.
    pub focused_pane: Option<String>,
    /// Ordered open workspace paths.
    pub open_files: Vec<String>,
    /// Pane projections keyed by pane identity.
    pub projections: BTreeMap<String, RendererPaneProjection>,
    /// Optional lazily observed tree page.
    pub tree: Option<RendererTreeProjection>,
}

/// Exact plan awaiting a human decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererApproval {
    /// Exact plan digest.
    pub digest: String,
    /// Semantic intent being authorized.
    pub intent: String,
    /// Optional consequence label.
    pub risk: Option<String>,
    /// Whether the exact plan requires a decision.
    pub approval_required: bool,
}

/// Actor-private context pin summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererContextPin {
    /// Stable pin identity.
    pub id: String,
    /// Operator-facing label.
    pub label: String,
    /// Typed host-owned source reference.
    pub source: String,
}

/// Current state of a projected capability grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RendererGrantState {
    /// The grant may currently be exercised.
    Active,
    /// The grant passed its deadline.
    Expired,
    /// The grant was explicitly revoked.
    Revoked,
}

/// Display-only grant summary without credential material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererGrantSummary {
    /// Stable grant identity.
    pub id: String,
    /// Semantic capability.
    pub capability: String,
    /// Current grant state.
    pub state: RendererGrantState,
}

/// One displayable activity entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererActivity {
    /// Event sequence.
    pub sequence: u64,
    /// RFC 3339 observation time.
    pub at: String,
    /// Event kind.
    pub kind: String,
    /// Optional semantic intent.
    pub intent: Option<String>,
}

/// Current materialization or session-preparation stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererPreparation {
    /// Stable preparation stage.
    pub stage: String,
    /// Operator-safe status.
    pub message: String,
    /// Whether the preparation may be retried.
    pub retryable: bool,
}

/// Complete immutable state delivered to a renderer target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererFrame {
    /// Contract discriminator.
    pub format: String,
    /// Display-only session metadata.
    pub session: RendererSession,
    /// Optional preparation progress.
    pub preparation: Option<RendererPreparation>,
    /// Current renderer-neutral workbench projection.
    pub workbench: RendererWorkbench,
    /// Exact plans waiting for a human decision.
    pub pending_approvals: Vec<RendererApproval>,
    /// Actor-private context pins.
    pub context_pins: Vec<RendererContextPin>,
    /// Credential-free capability summaries.
    pub grants: Vec<RendererGrantSummary>,
    /// Recent activity projection.
    pub activity: Vec<RendererActivity>,
    /// Optional host-produced status or refusal message.
    pub notice: Option<String>,
}

impl RendererFrame {
    /// Verifies the discriminator, identities, and typed pane/projection relationship.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != RENDERER_FRAME_FORMAT || self.session.id.trim().is_empty() {
            return Err("renderer.frame_invalid: renderer frame identity is incomplete".into());
        }
        let mut pane_ids = BTreeSet::new();
        for pane in &self.workbench.panes {
            if pane.id.trim().is_empty()
                || pane.title.trim().is_empty()
                || !pane_ids.insert(pane.id.as_str())
            {
                return Err("renderer.frame_invalid: pane identities must be unique".into());
            }
        }
        if self
            .workbench
            .focused_pane
            .as_ref()
            .is_some_and(|focused| !pane_ids.contains(focused.as_str()))
        {
            return Err("renderer.frame_invalid: focused pane is not declared".into());
        }
        for (pane_id, projection) in &self.workbench.projections {
            let pane = self
                .workbench
                .panes
                .iter()
                .find(|candidate| candidate.id == *pane_id)
                .ok_or_else(|| {
                    "renderer.frame_invalid: projection pane is not declared".to_string()
                })?;
            if projection
                .pane_kind()
                .is_some_and(|projection_kind| projection_kind != pane.kind)
            {
                return Err("renderer.frame_invalid: projection does not match pane kind".into());
            }
        }
        Ok(())
    }
}

/// Transient host input that does not replace the immutable frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(missing_docs)]
pub enum RendererEvent {
    AssistantDelta {
        format: String,
        message_id: String,
        sequence: u64,
        markdown_delta: String,
    },
    TerminalOutput {
        format: String,
        terminal_id: String,
        sequence: u64,
        bytes: Vec<u8>,
    },
    Notice {
        format: String,
        message: String,
    },
}

/// Semantic user action emitted by a renderer target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(missing_docs)]
pub enum RendererAction {
    Refresh {
        format: String,
    },
    LoadTree {
        format: String,
        path: String,
        cursor: Option<String>,
    },
    OpenFile {
        format: String,
        path: String,
    },
    EditFile {
        format: String,
        path: String,
        content: String,
        version: String,
    },
    SaveFile {
        format: String,
        path: String,
        content: String,
        version: String,
    },
    FocusPane {
        format: String,
        pane_id: String,
    },
    ClosePane {
        format: String,
        pane_id: String,
    },
    ShowDiff {
        format: String,
    },
    Approve {
        format: String,
        plan_digest: String,
    },
    Deny {
        format: String,
        plan_digest: String,
    },
    SubmitPrompt {
        format: String,
        content: String,
    },
    PinContext {
        format: String,
        source: String,
    },
    RemoveContextPin {
        format: String,
        pin_id: String,
    },
    OpenTerminal {
        format: String,
        columns: u16,
        rows: u16,
    },
    TerminalInput {
        format: String,
        terminal_id: String,
        data: String,
    },
    TerminalResize {
        format: String,
        terminal_id: String,
        columns: u16,
        rows: u16,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_requires_exact_protocol_formats() {
        let target = RendererTargetManifest {
            format: RENDERER_TARGET_FORMAT.into(),
            id: "vue".into(),
            framework: "vue-3".into(),
            frame_format: RENDERER_FRAME_FORMAT.into(),
            event_format: RENDERER_EVENT_FORMAT.into(),
            action_format: RENDERER_ACTION_FORMAT.into(),
        };
        assert_eq!(target.validate(), Ok(()));
    }

    #[test]
    fn frame_rejects_projection_for_wrong_pane_kind() {
        let frame = RendererFrame {
            format: RENDERER_FRAME_FORMAT.into(),
            session: RendererSession {
                id: "session-1".into(),
                objective: "test".into(),
                status: RendererSessionStatus::Active,
                cursor: 1,
            },
            preparation: None,
            workbench: RendererWorkbench {
                panes: vec![RendererPane {
                    id: "chat".into(),
                    kind: RendererPaneKind::Chat,
                    title: "Agent".into(),
                    path: None,
                    line: None,
                    column: None,
                }],
                focused_pane: Some("chat".into()),
                open_files: Vec::new(),
                projections: BTreeMap::from([(
                    "chat".into(),
                    RendererPaneProjection::Editor(RendererEditorProjection {
                        document: RendererEditorDocument {
                            path: "README.md".into(),
                            language: "markdown".into(),
                            content: String::new(),
                            version: "v1".into(),
                            read_only: false,
                            dirty: false,
                        },
                    }),
                )]),
                tree: None,
            },
            pending_approvals: Vec::new(),
            context_pins: Vec::new(),
            grants: Vec::new(),
            activity: Vec::new(),
            notice: None,
        };
        assert_eq!(
            frame.validate(),
            Err("renderer.frame_invalid: projection does not match pane kind".into())
        );
    }
}
