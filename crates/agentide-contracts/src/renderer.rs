//! Transport-neutral contracts shared by browser renderer targets and their hosts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current renderer target lifecycle contract.
pub const RENDERER_TARGET_FORMAT: &str = "agentide.renderer-target/1";
/// Current immutable frame contract.
pub const RENDERER_FRAME_FORMAT: &str = "agentide.renderer-frame/1";
/// Current host-to-renderer transient event contract.
pub const RENDERER_EVENT_FORMAT: &str = "agentide.renderer-event/1";
/// Current renderer-to-host semantic action contract.
pub const RENDERER_ACTION_FORMAT: &str = "agentide.renderer-action/1";

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

/// Display-only session metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererSession {
    /// Session identity, never a bearer credential.
    pub id: String,
    /// Operator-visible objective.
    pub objective: String,
    /// Current lifecycle label.
    pub status: String,
    /// Latest event cursor represented by the frame.
    pub cursor: u64,
}

/// Renderer-neutral pane metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererPane {
    /// Stable pane identity.
    pub id: String,
    /// Semantic pane kind.
    pub kind: String,
    /// Human-readable title.
    pub title: String,
    /// Optional workspace-relative path.
    pub path: Option<String>,
    /// Optional one-based line.
    pub line: Option<u64>,
    /// Optional one-based column.
    pub column: Option<u64>,
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

/// Complete immutable state delivered to a renderer target.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RendererFrame {
    /// Contract discriminator.
    pub format: String,
    /// Display-only session metadata.
    pub session: RendererSession,
    /// Current renderer-neutral workbench projection.
    pub workbench: RendererWorkbench,
    /// Exact plans waiting for a human decision.
    pub pending_approvals: Vec<RendererApproval>,
    /// Recent activity projection.
    pub activity: Vec<RendererActivity>,
    /// Optional bounded observation selected by the host.
    pub observation: Option<Value>,
    /// Optional host-produced status or refusal message.
    pub notice: Option<String>,
}

impl RendererFrame {
    /// Verifies the frame discriminator and minimal identity requirements.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != RENDERER_FRAME_FORMAT
            || self.session.id.trim().is_empty()
            || self.session.status.trim().is_empty()
        {
            return Err("renderer.frame_invalid: renderer frame identity is incomplete".into());
        }
        Ok(())
    }
}

/// Transient host input that does not replace the immutable frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(missing_docs)]
pub enum RendererEvent {
    /// Incremental assistant text.
    TextDelta { format: String, text: String },
    /// Ordered terminal output.
    TerminalOutput {
        format: String,
        terminal_id: String,
        sequence: u64,
        bytes: Vec<u8>,
    },
    /// Host-produced status or refusal message.
    Notice { format: String, message: String },
}

/// Semantic user action emitted by a renderer target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(missing_docs)]
pub enum RendererAction {
    Refresh {
        format: String,
    },
    OpenFile {
        format: String,
        path: String,
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
}
