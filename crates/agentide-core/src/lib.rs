//! Durable intent planning, exact approval, journaling, and projection.
#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use agentide_contracts::{Approval, Binding, BindingConfig, IntentDefinition, IntentProfile};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use thiserror::Error;
use ulid::Ulid;

/// A stable refusal presented to both models and operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[error("{code}: {message}")]
pub struct Refusal {
    /// Machine-readable code.
    pub code: String,
    /// Actionable explanation.
    pub message: String,
    /// Whether repeating the exact request may be safe and useful.
    pub retryable: bool,
}

impl Refusal {
    /// Creates a non-retryable refusal.
    #[must_use]
    pub fn named(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable: false,
        }
    }
}

/// One implementation boundary supplied by a standalone or embedded host.
pub trait IntentPort: Send + Sync {
    /// Invokes an already planned and authorized binding.
    fn invoke(&self, binding: &Binding, intent: &str, input: &Value) -> Result<Value, Refusal>;

    /// Reports non-secret capability facts.
    fn capabilities(&self) -> Value;
}

/// Exact, immutable effect plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    /// Plan contract version.
    pub format: String,
    /// Session receiving the effect.
    pub session_id: String,
    /// Semantic intent.
    pub intent: String,
    /// Concrete driver selected outside model input.
    pub driver: String,
    /// Concrete operation selected outside model input.
    pub operation: String,
    /// Digest of the complete non-secret binding, including operator options.
    pub binding_sha256: String,
    /// Digest of the exact model-provided input.
    pub input_sha256: String,
    /// Whether dispatch requires an exact approval.
    pub approval_required: bool,
    /// SHA-256 of all fields above.
    pub digest: String,
}

/// Durable event envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Event contract version.
    pub format: String,
    /// Globally unique event id.
    pub id: String,
    /// Monotonic cursor within a session.
    pub sequence: u64,
    /// Observation time.
    pub at: DateTime<Utc>,
    /// Session identity.
    pub session_id: String,
    /// Stable event kind.
    pub kind: String,
    /// Related intent, if any.
    pub intent: Option<String>,
    /// Related exact plan digest, if any.
    pub plan_digest: Option<String>,
    /// Public, secret-free payload.
    pub payload: Value,
}

/// One virtual workbench pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pane {
    /// Stable pane identity.
    pub id: String,
    /// Semantic pane kind.
    pub kind: String,
    /// Short display title.
    pub title: String,
    /// Workspace-relative file, if applicable.
    pub path: Option<String>,
    /// One-based cursor line.
    pub line: Option<u64>,
    /// One-based cursor column.
    pub column: Option<u64>,
}

/// Renderer-neutral virtual workspace state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workbench {
    /// Ordered panes.
    pub panes: Vec<Pane>,
    /// Focused pane.
    pub focused_pane: Option<String>,
    /// Ordered open files.
    pub open_files: Vec<String>,
}

/// Compact shared projection rendered by CLI, web, and TUI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot contract version.
    pub format: String,
    /// Session identity.
    pub session_id: String,
    /// Operator objective.
    pub objective: String,
    /// Active or closed.
    pub status: String,
    /// Latest event sequence.
    pub cursor: u64,
    /// Virtual workbench projection.
    pub workbench: Workbench,
    /// Exact plan digests waiting for human authority.
    pub pending_approvals: Vec<Plan>,
    /// Process observations present in the journal.
    pub processes: Vec<Value>,
    /// Agent observations present in the journal.
    pub agents: Vec<Value>,
    /// Evidence records present in the journal.
    pub evidence: Vec<Value>,
    /// Most recent completed result.
    pub last_result: Option<Value>,
}

/// Persisted session record. Target source contents are never stored here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Storage format.
    pub format: String,
    /// Session identity.
    pub id: String,
    /// Canonical target workspace path.
    pub workspace_root: PathBuf,
    /// Substrate workspace id.
    pub workspace_id: String,
    /// Operator objective.
    pub objective: String,
    /// Active or closed.
    pub status: String,
    /// Event journal.
    pub events: Vec<Event>,
    /// Plans awaiting dispatch.
    pub pending: BTreeMap<String, PendingIntent>,
    /// Approved exact plan digests.
    pub approvals: BTreeSet<String>,
}

/// Intent input held only until it is dispatched or superseded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingIntent {
    /// Exact plan.
    pub plan: Plan,
}

/// Session persistence errors.
#[derive(Debug, Error)]
pub enum StoreError {
    /// File operation failed.
    #[error("session store I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// Stored JSON was invalid.
    #[error("session store JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    /// Session is absent.
    #[error("session `{0}` does not exist")]
    Missing(String),
    /// Workspace cannot be represented.
    #[error("workspace is invalid: {0}")]
    Workspace(String),
}

/// Durable atomic JSON session store outside target workspaces.
#[derive(Debug, Clone)]
pub struct StateStore {
    root: PathBuf,
}

impl StateStore {
    /// Uses `$XDG_STATE_HOME/agentide`, or `$HOME/.local/state/agentide`.
    pub fn discover() -> Result<Self, StoreError> {
        let root = if let Some(path) = std::env::var_os("XDG_STATE_HOME") {
            PathBuf::from(path).join("agentide")
        } else if let Some(path) = std::env::var_os("HOME") {
            PathBuf::from(path).join(".local/state/agentide")
        } else {
            return Err(StoreError::Workspace(
                "neither XDG_STATE_HOME nor HOME is set".into(),
            ));
        };
        Ok(Self { root })
    }

    /// Uses an explicit root, primarily for embedding and tests.
    #[must_use]
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Creates a new session record.
    pub fn create(&self, workspace: &Path, objective: String) -> Result<Session, StoreError> {
        let workspace_root = workspace.canonicalize()?;
        let workspace_id = workspace_root
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|name| {
                !name.starts_with('-')
                    && name
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            })
            .ok_or_else(|| {
                StoreError::Workspace(
                    "directory name must contain only ASCII letters, digits, `_`, or `-`".into(),
                )
            })?
            .to_owned();
        let id = Ulid::generate().to_string().to_lowercase();
        let mut session = Session {
            format: "agentide.session/1".into(),
            id,
            workspace_root,
            workspace_id,
            objective,
            status: "active".into(),
            events: Vec::new(),
            pending: BTreeMap::new(),
            approvals: BTreeSet::new(),
        };
        let event_objective = session.objective.clone();
        append_event(
            &mut session,
            "session.started",
            None,
            None,
            json!({"objective": event_objective}),
        );
        self.save(&session)?;
        Ok(session)
    }

    /// Loads one record.
    pub fn load(&self, id: &str) -> Result<Session, StoreError> {
        let path = self.session_path(id);
        if !path.is_file() {
            return Err(StoreError::Missing(id.into()));
        }
        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }

    /// Atomically replaces one record.
    pub fn save(&self, session: &Session) -> Result<(), StoreError> {
        std::fs::create_dir_all(self.root.join("sessions"))?;
        let path = self.session_path(&session.id);
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&temporary, serde_json::to_vec_pretty(session)?)?;
        std::fs::rename(temporary, path)?;
        Ok(())
    }

    /// Lists sessions newest first by id.
    pub fn list(&self) -> Result<Vec<Session>, StoreError> {
        let directory = self.root.join("sessions");
        if !directory.is_dir() {
            return Ok(Vec::new());
        }
        let mut sessions: Vec<Session> = Vec::new();
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) == Some("json") {
                sessions.push(serde_json::from_slice(&std::fs::read(path)?)?);
            }
        }
        sessions.sort_by(|left, right| right.id.cmp(&left.id));
        Ok(sessions)
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.root.join("sessions").join(format!("{id}.json"))
    }
}

/// Planning and dispatch engine independent of any concrete implementation.
pub struct Engine<P> {
    profile: IntentProfile,
    bindings: BindingConfig,
    store: StateStore,
    port: P,
}

impl<P: IntentPort> Engine<P> {
    /// Constructs an engine after validating profile/binding coverage.
    pub fn new(
        profile: IntentProfile,
        bindings: BindingConfig,
        store: StateStore,
        port: P,
    ) -> Result<Self, Refusal> {
        bindings
            .validate_against(&profile)
            .map_err(|error| Refusal::named("bindings.invalid", error.to_string()))?;
        Ok(Self {
            profile,
            bindings,
            store,
            port,
        })
    }

    /// Access to the released catalogue.
    #[must_use]
    pub const fn profile(&self) -> &IntentProfile {
        &self.profile
    }

    /// Reports bindings and implementation capabilities without secret values.
    #[must_use]
    pub fn inspect_bindings(&self) -> Value {
        let bindings: BTreeMap<_, _> = self
            .bindings
            .bindings
            .iter()
            .map(|(name, binding)| {
                let digest = binding_digest(binding).unwrap_or_else(|_| "unavailable".into());
                (
                    name,
                    json!({
                        "driver": binding.driver,
                        "operation": binding.operation,
                        "binding_sha256": digest,
                    }),
                )
            })
            .collect();
        json!({
            "format": self.bindings.format,
            "bindings": bindings,
            "unbound": self.bindings.unbound,
            "capabilities": self.port.capabilities(),
        })
    }

    /// Builds and durably records an exact plan.
    pub fn preview(&self, session_id: &str, intent: &str, input: Value) -> Result<Plan, Refusal> {
        let definition = self.definition(intent)?;
        let binding = self.binding(intent, definition)?;
        let mut session = self.store.load(session_id).map_err(store_refusal)?;
        if session.status != "active" {
            return Err(Refusal::named(
                "session.closed",
                "the coding session is closed",
            ));
        }
        let plan = make_plan(session_id, definition, binding, &input)?;
        session
            .pending
            .insert(plan.digest.clone(), PendingIntent { plan: plan.clone() });
        append_event(
            &mut session,
            "intent.planned",
            Some(intent),
            Some(&plan.digest),
            serde_json::to_value(&plan).map_err(json_refusal)?,
        );
        self.store.save(&session).map_err(store_refusal)?;
        Ok(plan)
    }

    /// Grants authority to exactly one plan digest.
    pub fn grant(&self, session_id: &str, digest: &str) -> Result<(), Refusal> {
        let mut session = self.store.load(session_id).map_err(store_refusal)?;
        if !session.pending.contains_key(digest) {
            return Err(Refusal::named(
                "approval.unknown_plan",
                "the plan digest is not pending in this session",
            ));
        }
        session.approvals.insert(digest.into());
        append_event(
            &mut session,
            "approval.granted",
            None,
            Some(digest),
            json!({"plan_digest": digest}),
        );
        self.store.save(&session).map_err(store_refusal)
    }

    /// Dispatches a previously previewed plan if its authority is satisfied.
    pub fn resume(&self, session_id: &str, digest: &str, input: Value) -> Result<Value, Refusal> {
        let mut session = self.store.load(session_id).map_err(store_refusal)?;
        let pending = session.pending.get(digest).cloned().ok_or_else(|| {
            Refusal::named("intent.unknown_plan", "the plan digest is not pending")
        })?;
        if input_digest(&input)? != pending.plan.input_sha256 {
            return Err(Refusal::named(
                "intent.input_changed",
                "the resumed input does not match the exact previewed plan",
            ));
        }
        let binding = self
            .bindings
            .bindings
            .get(&pending.plan.intent)
            .ok_or_else(|| Refusal::named("binding.missing", "binding disappeared"))?;
        if binding_digest(binding)? != pending.plan.binding_sha256 {
            return Err(Refusal::named(
                "binding.changed",
                "the operator binding changed after preview; build a new plan",
            ));
        }
        if pending.plan.approval_required && !session.approvals.remove(digest) {
            return Err(Refusal::named(
                "approval.required",
                format!("approve exact plan `{digest}` before dispatch"),
            ));
        }
        let result = if pending.plan.driver == "core" {
            apply_core(&session, &pending.plan.intent, &input)
        } else {
            self.port.invoke(binding, &pending.plan.intent, &input)
        };
        match &result {
            Ok(value) => append_event(
                &mut session,
                "intent.completed",
                Some(&pending.plan.intent),
                Some(digest),
                journal_result(&pending.plan.intent, value),
            ),
            Err(error) => append_event(
                &mut session,
                "intent.refused",
                Some(&pending.plan.intent),
                Some(digest),
                serde_json::to_value(error).map_err(json_refusal)?,
            ),
        }
        session.pending.remove(digest);
        self.store.save(&session).map_err(store_refusal)?;
        result
    }

    /// Convenience for previewing and immediately dispatching authority-free intents.
    pub fn call(&self, session_id: &str, intent: &str, input: Value) -> Result<Value, Refusal> {
        let plan = self.preview(session_id, intent, input.clone())?;
        self.resume(session_id, &plan.digest, input)
    }

    /// Projects current state from the durable event stream.
    pub fn snapshot(&self, session_id: &str) -> Result<Snapshot, Refusal> {
        let session = self.store.load(session_id).map_err(store_refusal)?;
        Ok(project(&session))
    }

    /// Reads an event window.
    pub fn events(
        &self,
        session_id: &str,
        after: u64,
        limit: usize,
    ) -> Result<Vec<Event>, Refusal> {
        let session = self.store.load(session_id).map_err(store_refusal)?;
        Ok(session
            .events
            .into_iter()
            .filter(|event| event.sequence > after)
            .take(limit.min(1_000))
            .collect())
    }

    fn definition(&self, intent: &str) -> Result<&IntentDefinition, Refusal> {
        self.profile.find(intent).ok_or_else(|| {
            Refusal::named(
                "intent.unknown",
                format!("`{intent}` is not a released intent"),
            )
        })
    }

    fn binding(&self, intent: &str, definition: &IntentDefinition) -> Result<&Binding, Refusal> {
        if self.bindings.unbound.contains(intent) {
            return Err(Refusal::named(
                "binding.unavailable",
                format!("`{intent}` has no implementation in this deployment"),
            ));
        }
        self.bindings.bindings.get(intent).ok_or_else(|| {
            Refusal::named(
                "binding.unavailable",
                format!("port `{}` is not bound for `{intent}`", definition.port),
            )
        })
    }
}

fn make_plan(
    session_id: &str,
    definition: &IntentDefinition,
    binding: &Binding,
    input: &Value,
) -> Result<Plan, Refusal> {
    let input_sha256 = input_digest(input)?;
    let binding_sha256 = binding_digest(binding)?;
    let approval_required = matches!(definition.approval, Approval::Required);
    let seed = json!({
        "format": "agentide.plan/1",
        "session_id": session_id,
        "intent": definition.name,
        "driver": binding.driver,
        "operation": binding.operation,
        "binding_sha256": binding_sha256,
        "input_sha256": input_sha256,
        "approval_required": approval_required,
    });
    let digest = hex(&Sha256::digest(
        serde_json::to_vec(&seed).map_err(json_refusal)?,
    ));
    Ok(Plan {
        format: "agentide.plan/1".into(),
        session_id: session_id.into(),
        intent: definition.name.clone(),
        driver: binding.driver.clone(),
        operation: binding.operation.clone(),
        binding_sha256,
        input_sha256,
        approval_required,
        digest,
    })
}

fn input_digest(input: &Value) -> Result<String, Refusal> {
    Ok(hex(&Sha256::digest(
        serde_json::to_vec(input).map_err(json_refusal)?,
    )))
}

fn binding_digest(binding: &Binding) -> Result<String, Refusal> {
    Ok(hex(&Sha256::digest(
        serde_json::to_vec(binding).map_err(json_refusal)?,
    )))
}

fn journal_result(intent: &str, value: &Value) -> Value {
    let mut observed = redact(value);
    if let Some(object) = observed.as_object_mut() {
        for sensitive in ["content", "stdout", "stderr", "data"] {
            object.remove(sensitive);
        }
        object.insert("intent".into(), Value::String(intent.into()));
    }
    observed
}

fn redact(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(name, _)| {
                    !matches!(name.as_str(), "content" | "stdout" | "stderr" | "data")
                })
                .map(|(name, value)| (name.clone(), redact(value)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact).collect()),
        _ => value.clone(),
    }
}

fn apply_core(session: &Session, intent: &str, input: &Value) -> Result<Value, Refusal> {
    let pane_id = input.get("pane_id").and_then(Value::as_str).map_or_else(
        || format!("pane-{}", session.events.len() + 1),
        ToOwned::to_owned,
    );
    let payload = match intent {
        "file_open" => {
            let path = required_text(input, "path")?;
            json!({"surface_event": "file_opened", "pane_id": pane_id, "kind": "editor", "title": path, "path": path, "line": input.get("line")})
        }
        "file_close" => {
            json!({"surface_event": "file_closed", "path": required_text(input, "path")?})
        }
        "pane_open" => {
            json!({"surface_event": "pane_opened", "pane_id": pane_id, "kind": required_text(input, "kind")?, "title": input.get("title").and_then(Value::as_str).unwrap_or("Pane")})
        }
        "pane_close" => {
            json!({"surface_event": "pane_closed", "pane_id": required_text(input, "pane_id")?})
        }
        "pane_focus" => {
            json!({"surface_event": "pane_focused", "pane_id": required_text(input, "pane_id")?})
        }
        "cursor_move" => {
            json!({"surface_event": "cursor_moved", "pane_id": required_text(input, "pane_id")?, "path": required_text(input, "path")?, "line": required_u64(input, "line")?, "column": required_u64(input, "column")?})
        }
        "diff_show" => {
            json!({"surface_event": "diff_shown", "pane_id": pane_id, "kind": "diff", "title": "Changes", "path": input.get("path")})
        }
        "surface_snapshot" => json!({"surface_event": "observed"}),
        "session_snapshot" => serde_json::to_value(project(session)).map_err(json_refusal)?,
        "event_read" => {
            let after = input.get("after").and_then(Value::as_u64).unwrap_or(0);
            let limit = input
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(100)
                .min(1_000) as usize;
            let events: Vec<_> = session
                .events
                .iter()
                .filter(|event| event.sequence > after)
                .take(limit)
                .collect();
            json!({"after": after, "events": events})
        }
        "evidence_record" => json!({
            "evidence_recorded": true,
            "subject": required_text(input, "subject")?,
            "kind": required_text(input, "kind")?,
            "source": required_text(input, "source")?,
            "reference": input.get("reference"),
        }),
        _ => {
            return Err(Refusal::named(
                "binding.operation_unknown",
                "unknown core surface operation",
            ));
        }
    };
    Ok(payload)
}

fn required_text<'a>(input: &'a Value, name: &str) -> Result<&'a str, Refusal> {
    input
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| Refusal::named("intent.input_invalid", format!("`{name}` must be a string")))
}

fn required_u64(input: &Value, name: &str) -> Result<u64, Refusal> {
    input.get(name).and_then(Value::as_u64).ok_or_else(|| {
        Refusal::named(
            "intent.input_invalid",
            format!("`{name}` must be a positive integer"),
        )
    })
}

/// Deterministically folds events into a renderer-neutral snapshot.
#[must_use]
pub fn project(session: &Session) -> Snapshot {
    let mut workbench = Workbench::default();
    let mut last_result = None;
    let mut processes = Vec::new();
    let mut agents = Vec::new();
    let mut evidence = Vec::new();
    for event in &session.events {
        if event.kind == "intent.completed" {
            last_result = Some(event.payload.clone());
            match event.intent.as_deref() {
                Some(intent) if intent.starts_with("process_") || intent == "code_verify" => {
                    processes.push(event.payload.clone());
                }
                Some(intent) if intent.starts_with("agent_") => {
                    agents.push(event.payload.clone());
                }
                Some("evidence_record") => evidence.push(event.payload.clone()),
                _ => {}
            }
        }
        match event.payload.get("surface_event").and_then(Value::as_str) {
            Some("file_opened" | "pane_opened" | "diff_shown") => {
                let id = event
                    .payload
                    .get("pane_id")
                    .and_then(Value::as_str)
                    .unwrap_or("pane");
                workbench.panes.retain(|pane| pane.id != id);
                let path = event
                    .payload
                    .get("path")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
                if let Some(path) = &path
                    && !workbench.open_files.contains(path)
                {
                    workbench.open_files.push(path.clone());
                }
                workbench.panes.push(Pane {
                    id: id.into(),
                    kind: event
                        .payload
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("editor")
                        .into(),
                    title: event
                        .payload
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or("Pane")
                        .into(),
                    path,
                    line: event.payload.get("line").and_then(Value::as_u64),
                    column: event.payload.get("column").and_then(Value::as_u64),
                });
                workbench.focused_pane = Some(id.into());
            }
            Some("file_closed") => {
                if let Some(path) = event.payload.get("path").and_then(Value::as_str) {
                    workbench.open_files.retain(|open| open != path);
                    workbench
                        .panes
                        .retain(|pane| pane.path.as_deref() != Some(path));
                }
            }
            Some("pane_closed") => {
                if let Some(id) = event.payload.get("pane_id").and_then(Value::as_str) {
                    workbench.panes.retain(|pane| pane.id != id);
                    if workbench.focused_pane.as_deref() == Some(id) {
                        workbench.focused_pane = workbench.panes.last().map(|pane| pane.id.clone());
                    }
                }
            }
            Some("pane_focused") => {
                workbench.focused_pane = event
                    .payload
                    .get("pane_id")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned);
            }
            Some("cursor_moved") => {
                if let Some(pane) = event
                    .payload
                    .get("pane_id")
                    .and_then(Value::as_str)
                    .and_then(|id| workbench.panes.iter_mut().find(|pane| pane.id == id))
                {
                    pane.line = event.payload.get("line").and_then(Value::as_u64);
                    pane.column = event.payload.get("column").and_then(Value::as_u64);
                }
            }
            _ => {}
        }
    }
    Snapshot {
        format: "agentide.snapshot/1".into(),
        session_id: session.id.clone(),
        objective: session.objective.clone(),
        status: session.status.clone(),
        cursor: session.events.last().map_or(0, |event| event.sequence),
        workbench,
        pending_approvals: session
            .pending
            .values()
            .map(|pending| pending.plan.clone())
            .collect(),
        processes,
        agents,
        evidence,
        last_result,
    }
}

fn append_event(
    session: &mut Session,
    kind: &str,
    intent: Option<&str>,
    plan_digest: Option<&str>,
    payload: Value,
) {
    session.events.push(Event {
        format: "agentide.event/1".into(),
        id: Ulid::generate().to_string().to_lowercase(),
        sequence: session.events.len() as u64 + 1,
        at: Utc::now(),
        session_id: session.id.clone(),
        kind: kind.into(),
        intent: intent.map(Into::into),
        plan_digest: plan_digest.map(Into::into),
        payload,
    });
}

fn store_refusal(error: StoreError) -> Refusal {
    Refusal::named("session.store", error.to_string())
}

fn json_refusal(error: serde_json::Error) -> Refusal {
    Refusal::named("contract.json", error.to_string())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Noop;

    impl IntentPort for Noop {
        fn invoke(&self, _: &Binding, intent: &str, _: &Value) -> Result<Value, Refusal> {
            if intent == "code_read" {
                Ok(json!({"path": "a", "content": "sensitive source", "sha256": "digest"}))
            } else {
                Ok(json!({"ok": true}))
            }
        }

        fn capabilities(&self) -> Value {
            json!({})
        }
    }

    #[test]
    fn approval_is_bound_to_exact_plan_and_surface_replays() {
        let temporary = tempfile::tempdir().expect("temporary");
        let workspace = temporary.path().join("workspace");
        std::fs::create_dir(&workspace).expect("workspace");
        let store = StateStore::at(temporary.path().join("state"));
        let session = store.create(&workspace, "test".into()).expect("session");
        let engine = Engine::new(
            IntentProfile::embedded().expect("profile"),
            BindingConfig::embedded().expect("bindings"),
            store.clone(),
            Noop,
        )
        .expect("engine");

        let plan = engine
            .preview(
                &session.id,
                "code_edit",
                json!({"path": "a", "content": "b"}),
            )
            .expect("plan");
        let stored =
            serde_json::to_string(&store.load(&session.id).expect("stored")).expect("json");
        assert!(!stored.contains("\"content\":\"b\""));
        assert_eq!(
            engine
                .resume(
                    &session.id,
                    &plan.digest,
                    json!({"path": "a", "content": "b"}),
                )
                .expect_err("approval")
                .code,
            "approval.required"
        );
        engine.grant(&session.id, &plan.digest).expect("grant");
        assert_eq!(
            engine
                .resume(
                    &session.id,
                    &plan.digest,
                    json!({"path": "a", "content": "changed"}),
                )
                .expect_err("changed input")
                .code,
            "intent.input_changed"
        );
        engine
            .resume(
                &session.id,
                &plan.digest,
                json!({"path": "a", "content": "b"}),
            )
            .expect("resume");

        let read = engine
            .call(&session.id, "code_read", json!({"path": "a"}))
            .expect("read");
        assert_eq!(read["content"], "sensitive source");
        let stored =
            serde_json::to_string(&store.load(&session.id).expect("stored")).expect("json");
        assert!(!stored.contains("sensitive source"));

        engine
            .call(
                &session.id,
                "file_open",
                json!({"path": "src/lib.rs", "line": 7}),
            )
            .expect("open");
        let snapshot = engine.snapshot(&session.id).expect("snapshot");
        assert_eq!(snapshot.workbench.open_files, ["src/lib.rs"]);
        assert_eq!(snapshot.workbench.panes[0].line, Some(7));
    }

    #[test]
    fn operator_binding_options_participate_in_plan_identity() {
        let profile = IntentProfile::embedded().expect("profile");
        let definition = profile.find("code_verify").expect("verify intent");
        let mut bindings = BindingConfig::embedded().expect("bindings");
        let binding = bindings.bindings.get("code_verify").expect("binding");
        let before = make_plan("session", definition, binding, &json!({"level": "focused"}))
            .expect("before");
        bindings
            .bindings
            .get_mut("code_verify")
            .expect("binding")
            .options
            .insert("operator_change".into(), json!(true));
        let binding = bindings.bindings.get("code_verify").expect("binding");
        let after =
            make_plan("session", definition, binding, &json!({"level": "focused"})).expect("after");
        assert_ne!(before.binding_sha256, after.binding_sha256);
        assert_ne!(before.digest, after.digest);
    }
}
