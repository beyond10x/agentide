//! Harness-native model, tool, approval, and event seams for AgentIDE.
#![allow(
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::too_many_lines
)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use agentide_contracts::{
    Approval as IntentApproval, Effect as IntentEffect, Exposure, IntentDefinition,
    Risk as IntentRisk,
};
use agentide_core::{Engine, IntentPort, Plan, Refusal};
use b10x_harness_credential::{NamedSource, SubscriptionToken};
use b10x_harness_loop::{ApprovalDecision, ApprovalPort, LoopCancel};
use b10x_harness_messages::{Endpoint as MessagesEndpoint, MessagesClient};
use b10x_harness_responses::{Endpoint as ResponsesEndpoint, ResponsesClient};
use b10x_harness_wire::{
    AccessKind, Bearer, BearerSource, CredentialKind, Effect, Envelope, Idempotency, ModelPort,
    Risk, Subject, ToolCall, ToolName, ToolOutcome, ToolPort, ToolSpec, WireError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// One exact AgentIDE plan presented through Harness's approval gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Harness call identity.
    pub call_id: String,
    /// Stable AgentIDE intent name.
    pub intent: String,
    /// ESS command that defines the intent.
    pub command: String,
    /// Model-provided semantic arguments, without host-bound session fields.
    pub arguments: Value,
    /// Exact implementation plan awaiting the decision.
    pub plan: Plan,
}

/// UI-owned decision surface for an exact AgentIDE plan.
pub trait PlanApprover {
    /// Blocks until a person approves or denies the request.
    fn decide(&mut self, request: &ApprovalRequest) -> ApprovalDecision;
}

/// Harness model API projection used by the first standalone TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelWire {
    /// OpenAI-compatible Responses API.
    OpenaiResponses,
    /// Anthropic-compatible Messages API.
    AnthropicMessages,
}

/// A credential source named by configuration; no variant stores a credential value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    /// Send no credential header.
    None,
    /// Read an API key from this environment variable for every request.
    ApiKeyEnvironment(String),
    /// Read an OAuth token from this environment variable for every request.
    OauthEnvironment(String),
    /// Read an OAuth token from a named file and optional JSON pointer for every request.
    OauthFile {
        /// Credential document.
        path: PathBuf,
        /// JSON pointer inside the document, when it is structured.
        pointer: Option<String>,
    },
}

/// Complete non-secret model connection selected by the embedding host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelConnection {
    /// Model API projection.
    pub wire: ModelWire,
    /// API origin plus prefix.
    pub base_url: String,
    /// Exact model identifier.
    pub model: String,
    /// Declared model context window.
    pub context_window: u64,
    /// Per-turn output ceiling used by the Messages route.
    pub max_output_tokens: u64,
    /// Named credential source.
    pub credential: CredentialSource,
}

impl ModelConnection {
    /// Builds the selected Harness wire over one cancellation token.
    pub fn connect(&self, cancel: LoopCancel) -> Result<Box<dyn ModelPort>, WireError> {
        let bearer = credential(&self.credential);
        match self.wire {
            ModelWire::OpenaiResponses => {
                let endpoint = ResponsesEndpoint::new(
                    self.base_url.clone(),
                    self.model.clone(),
                    self.context_window,
                )?;
                match bearer {
                    None => ResponsesClient::unauthenticated(endpoint)
                        .map(|client| Box::new(client.with_cancel(cancel)) as Box<dyn ModelPort>),
                    Some(bearer) => ResponsesClient::new(endpoint, bearer)
                        .map(|client| Box::new(client.with_cancel(cancel)) as Box<dyn ModelPort>),
                }
            }
            ModelWire::AnthropicMessages => {
                let endpoint = MessagesEndpoint::new(
                    self.base_url.clone(),
                    self.model.clone(),
                    self.context_window,
                )?
                .with_max_output_tokens(self.max_output_tokens)?;
                match bearer {
                    None => MessagesClient::unauthenticated(endpoint)
                        .map(|client| Box::new(client.with_cancel(cancel)) as Box<dyn ModelPort>),
                    Some(bearer) => MessagesClient::new(endpoint, bearer)
                        .map(|client| Box::new(client.with_cancel(cancel)) as Box<dyn ModelPort>),
                }
            }
        }
    }
}

#[derive(Debug)]
struct EnvironmentCredential {
    variable: String,
    kind: CredentialKind,
}

impl BearerSource for EnvironmentCredential {
    fn bearer(&self) -> Result<Bearer, WireError> {
        let value = std::env::var(&self.variable).map_err(|_| {
            WireError::unauthorized(format!(
                "credential environment variable `{}` is unavailable",
                self.variable
            ))
        })?;
        if value.is_empty() {
            return Err(WireError::unauthorized(format!(
                "credential environment variable `{}` is empty",
                self.variable
            )));
        }
        Ok(Bearer::new(value))
    }

    fn kind(&self) -> CredentialKind {
        self.kind
    }
}

fn credential(source: &CredentialSource) -> Option<Arc<dyn BearerSource>> {
    match source {
        CredentialSource::None => None,
        CredentialSource::ApiKeyEnvironment(variable) => Some(Arc::new(EnvironmentCredential {
            variable: variable.clone(),
            kind: CredentialKind::ApiKey,
        })),
        CredentialSource::OauthEnvironment(variable) => Some(Arc::new(EnvironmentCredential {
            variable: variable.clone(),
            kind: CredentialKind::Oauth,
        })),
        CredentialSource::OauthFile { path, pointer } => {
            let mut token = SubscriptionToken::new(NamedSource::file(path));
            if let Some(pointer) = pointer {
                token = token.at_pointer(pointer);
            }
            Some(Arc::new(token))
        }
    }
}

/// Adapter construction failures happen before any model request is sent.
#[derive(Debug, Error)]
pub enum AdapterError {
    /// A generated ESS command schema is absent from this released adapter.
    #[error("no generated ESS schema is embedded for `{0}`")]
    SchemaMissing(String),
    /// A generated ESS command schema is invalid JSON.
    #[error("generated ESS schema for `{command}` is invalid: {source}")]
    SchemaInvalid {
        /// Qualified ESS command.
        command: String,
        /// JSON failure.
        source: serde_json::Error,
    },
    /// An intent name cannot be published by Harness.
    #[error("intent `{intent}` cannot be a Harness tool name: {message}")]
    ToolName {
        /// Intent name.
        intent: String,
        /// Identifier refusal.
        message: String,
    },
}

#[derive(Debug, Clone)]
struct Prepared {
    digest: String,
    input: Value,
}

struct Shared<P> {
    engine: Engine<P>,
    session_id: String,
    pending: BTreeMap<String, Prepared>,
}

/// Harness tool surface over the bound subset of the AgentIDE intent profile.
pub struct IntentTools<P> {
    shared: Arc<Mutex<Shared<P>>>,
    definitions: BTreeMap<String, IntentDefinition>,
    specs: Vec<ToolSpec>,
}

/// Harness approval port that binds a decision to AgentIDE's exact plan digest.
pub struct IntentApprovals<P, A> {
    shared: Arc<Mutex<Shared<P>>>,
    definitions: BTreeMap<String, IntentDefinition>,
    approver: A,
}

/// Constructs the paired Harness ports. Required intents cannot execute through the tool half
/// unless the approval half first prepared and granted their exact AgentIDE plan.
pub fn ports<P: IntentPort, A: PlanApprover>(
    engine: Engine<P>,
    session_id: impl Into<String>,
    approver: A,
) -> Result<(IntentTools<P>, IntentApprovals<P, A>), AdapterError> {
    let bound = engine
        .inspect_bindings()
        .get("bindings")
        .and_then(Value::as_object)
        .map(|bindings| bindings.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut definitions = BTreeMap::new();
    let mut specs = Vec::new();
    for definition in &engine.profile().intents {
        if matches!(definition.exposure, Exposure::Operator) || !bound.contains(&definition.name) {
            continue;
        }
        let name = ToolName::new(&definition.name).map_err(|error| AdapterError::ToolName {
            intent: definition.name.clone(),
            message: error.to_string(),
        })?;
        let schema_text = schema_for(&definition.command)
            .ok_or_else(|| AdapterError::SchemaMissing(definition.command.clone()))?;
        let schema = model_schema(&definition.command, schema_text)?;
        specs.push(ToolSpec {
            name,
            description: description(definition, &schema),
            input_schema: schema,
            approval: match definition.approval {
                IntentApproval::Never => b10x_harness_wire::Approval::NotRequired,
                IntentApproval::Required => b10x_harness_wire::Approval::Required,
            },
            envelope: envelope(definition),
        });
        definitions.insert(definition.name.clone(), definition.clone());
    }
    let shared = Arc::new(Mutex::new(Shared {
        engine,
        session_id: session_id.into(),
        pending: BTreeMap::new(),
    }));
    Ok((
        IntentTools {
            shared: Arc::clone(&shared),
            definitions: definitions.clone(),
            specs,
        },
        IntentApprovals {
            shared,
            definitions,
            approver,
        },
    ))
}

impl<P: IntentPort> ToolPort for IntentTools<P> {
    fn specs(&self) -> &[ToolSpec] {
        &self.specs
    }

    fn subjects(&self, call: &ToolCall) -> Vec<Subject> {
        self.definitions
            .get(call.name.as_str())
            .map_or_else(Vec::new, |definition| subjects(definition, &call.arguments))
    }

    fn operation(&self, call: &ToolCall) -> Option<String> {
        self.definitions
            .get(call.name.as_str())
            .map(|definition| definition.command.clone())
    }

    fn operations(&self) -> Vec<&'static str> {
        // Qualified ESS command strings are owned contract values rather than static neutral
        // operation constants. `operation` records the exact value per call.
        Vec::new()
    }

    fn call(&mut self, call: &ToolCall) -> ToolOutcome {
        let Some(definition) = self.definitions.get(call.name.as_str()) else {
            return ToolOutcome::failed(format!("unknown AgentIDE intent `{}`", call.name));
        };
        let mut shared = self
            .shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let result = match definition.approval {
            IntentApproval::Never => {
                let input = host_input(&shared.session_id, call);
                shared
                    .engine
                    .call(&shared.session_id, call.name.as_str(), input)
            }
            IntentApproval::Required => {
                let Some(prepared) = shared.pending.remove(call.call_id.as_str()) else {
                    return refusal_outcome(Refusal::named(
                        "harness.approval_missing",
                        "the required intent reached the port without its exact Harness approval",
                    ));
                };
                shared
                    .engine
                    .resume(&shared.session_id, &prepared.digest, prepared.input)
            }
        };
        match result {
            Ok(value) => ToolOutcome::ok(value),
            Err(refusal) => refusal_outcome(refusal),
        }
    }
}

impl<P: IntentPort, A: PlanApprover> ApprovalPort for IntentApprovals<P, A> {
    fn decide(&mut self, call: &ToolCall, _: &ToolSpec) -> ApprovalDecision {
        let Some(definition) = self.definitions.get(call.name.as_str()) else {
            return ApprovalDecision::denied("the AgentIDE intent is not part of this adapter");
        };
        let request = {
            let mut shared = self
                .shared
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let input = host_input(&shared.session_id, call);
            let plan =
                match shared
                    .engine
                    .preview(&shared.session_id, call.name.as_str(), input.clone())
                {
                    Ok(plan) => plan,
                    Err(error) => return ApprovalDecision::denied(error.to_string()),
                };
            shared.pending.insert(
                call.call_id.as_str().to_owned(),
                Prepared {
                    digest: plan.digest.clone(),
                    input,
                },
            );
            ApprovalRequest {
                call_id: call.call_id.as_str().to_owned(),
                intent: call.name.as_str().to_owned(),
                command: definition.command.clone(),
                arguments: call.arguments.clone(),
                plan,
            }
        };
        let decision = self.approver.decide(&request);
        let mut shared = self
            .shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &decision {
            ApprovalDecision::Approved => {
                if let Err(error) = shared
                    .engine
                    .grant(&shared.session_id, &request.plan.digest)
                {
                    shared.pending.remove(&request.call_id);
                    let _ = shared.engine.deny(
                        &shared.session_id,
                        &request.plan.digest,
                        "the Harness approval could not be recorded",
                    );
                    return ApprovalDecision::denied(error.to_string());
                }
            }
            ApprovalDecision::Denied { reason } => {
                shared.pending.remove(&request.call_id);
                if let Err(error) =
                    shared
                        .engine
                        .deny(&shared.session_id, &request.plan.digest, reason)
                {
                    return ApprovalDecision::denied(error.to_string());
                }
            }
        }
        decision
    }
}

fn host_input(session_id: &str, call: &ToolCall) -> Value {
    let mut input = call.arguments.as_object().cloned().unwrap_or_default();
    input.insert("session_id".into(), Value::String(session_id.into()));
    input.insert(
        "request_id".into(),
        Value::String(call.call_id.as_str().into()),
    );
    Value::Object(input)
}

fn refusal_outcome(refusal: Refusal) -> ToolOutcome {
    ToolOutcome {
        output: serde_json::to_value(refusal).unwrap_or_else(|error| {
            Value::String(format!("refusal serialization failed: {error}"))
        }),
        failed: true,
        refusal: None,
    }
}

fn subjects(definition: &IntentDefinition, arguments: &Value) -> Vec<Subject> {
    let mut subjects = Vec::new();
    if let Some(path) = arguments.get("path").and_then(Value::as_str) {
        subjects.push(Subject::file(path));
    }
    if let Some(paths) = arguments.get("paths").and_then(Value::as_array) {
        subjects.extend(paths.iter().filter_map(Value::as_str).map(Subject::file));
    }
    if let Some(profile) = arguments.get("profile").and_then(Value::as_str) {
        subjects.push(Subject::process(profile));
    }
    if let Some(process) = arguments.get("process_id").and_then(Value::as_str) {
        subjects.push(Subject::process(process));
    }
    if let Some(environment) = arguments.get("environment").and_then(Value::as_str) {
        subjects.push(Subject::host(environment));
    }
    if subjects.is_empty()
        && definition
            .subjects
            .iter()
            .any(|subject| matches!(subject.as_str(), "workspace" | "repository"))
    {
        subjects.push(Subject::file("."));
    }
    subjects
}

fn envelope(definition: &IntentDefinition) -> Envelope {
    let (effects, access) = match definition.effect {
        IntentEffect::Observe => (vec![Effect::Read], Vec::new()),
        IntentEffect::State => (vec![Effect::Write], Vec::new()),
        IntentEffect::Mutate => (
            vec![Effect::Write, Effect::Filesystem],
            vec![AccessKind::Filesystem],
        ),
        IntentEffect::Execute => (
            vec![Effect::Process, Effect::Filesystem],
            vec![AccessKind::Process, AccessKind::Filesystem],
        ),
        IntentEffect::Communicate | IntentEffect::External => {
            (vec![Effect::Network], vec![AccessKind::Network])
        }
    };
    Envelope {
        effects,
        risk: match definition.risk {
            IntentRisk::None | IntentRisk::Low => Risk::Low,
            IntentRisk::Medium => Risk::Medium,
            IntentRisk::High => Risk::High,
            IntentRisk::Critical => Risk::Destructive,
        },
        idempotency: match definition.effect {
            IntentEffect::Observe | IntentEffect::State => Idempotency::Idempotent,
            _ => Idempotency::Conditional,
        },
        access,
    }
}

fn description(definition: &IntentDefinition, schema: &Value) -> String {
    let title = schema
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&definition.name);
    format!(
        "{title}. AgentIDE semantic command `{}` through the `{}` implementation port; session and request identity are supplied by the host.",
        definition.command, definition.port
    )
}

fn model_schema(command: &str, text: &str) -> Result<Value, AdapterError> {
    let mut schema: Value =
        serde_json::from_str(text).map_err(|source| AdapterError::SchemaInvalid {
            command: command.into(),
            source,
        })?;
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        properties.remove("session_id");
        properties.remove("request_id");
    }
    if let Some(required) = schema.get_mut("required").and_then(Value::as_array_mut) {
        required.retain(|name| !matches!(name.as_str(), Some("session_id" | "request_id")));
    }
    Ok(schema)
}

macro_rules! command_schemas {
    ($(($command:literal, $path:literal)),+ $(,)?) => {
        fn schema_for(command: &str) -> Option<&'static str> {
            match command {
                $($command => Some(include_str!($path)),)+
                _ => None,
            }
        }
    };
}

command_schemas!(
    (
        "agentide.session.StartSession",
        "../../../generated/ess/schema/commands/agentide.session.StartSession.schema.json"
    ),
    (
        "agentide.session.CloseSession",
        "../../../generated/ess/schema/commands/agentide.session.CloseSession.schema.json"
    ),
    (
        "agentide.session.SnapshotSession",
        "../../../generated/ess/schema/commands/agentide.session.SnapshotSession.schema.json"
    ),
    (
        "agentide.session.ReadEvents",
        "../../../generated/ess/schema/commands/agentide.session.ReadEvents.schema.json"
    ),
    (
        "agentide.coding.SearchCode",
        "../../../generated/ess/schema/commands/agentide.coding.SearchCode.schema.json"
    ),
    (
        "agentide.coding.ReadCode",
        "../../../generated/ess/schema/commands/agentide.coding.ReadCode.schema.json"
    ),
    (
        "agentide.coding.ObserveChanges",
        "../../../generated/ess/schema/commands/agentide.coding.ObserveChanges.schema.json"
    ),
    (
        "agentide.coding.EditCode",
        "../../../generated/ess/schema/commands/agentide.coding.EditCode.schema.json"
    ),
    (
        "agentide.coding.VerifyCode",
        "../../../generated/ess/schema/commands/agentide.coding.VerifyCode.schema.json"
    ),
    (
        "agentide.coding.ObserveWorktree",
        "../../../generated/ess/schema/commands/agentide.coding.ObserveWorktree.schema.json"
    ),
    (
        "agentide.coding.CreateWorktree",
        "../../../generated/ess/schema/commands/agentide.coding.CreateWorktree.schema.json"
    ),
    (
        "agentide.coding.FinishWorktree",
        "../../../generated/ess/schema/commands/agentide.coding.FinishWorktree.schema.json"
    ),
    (
        "agentide.coding.StartProcess",
        "../../../generated/ess/schema/commands/agentide.coding.StartProcess.schema.json"
    ),
    (
        "agentide.coding.ObserveProcess",
        "../../../generated/ess/schema/commands/agentide.coding.ObserveProcess.schema.json"
    ),
    (
        "agentide.coding.InputProcess",
        "../../../generated/ess/schema/commands/agentide.coding.InputProcess.schema.json"
    ),
    (
        "agentide.coding.WaitProcess",
        "../../../generated/ess/schema/commands/agentide.coding.WaitProcess.schema.json"
    ),
    (
        "agentide.coding.CancelProcess",
        "../../../generated/ess/schema/commands/agentide.coding.CancelProcess.schema.json"
    ),
    (
        "agentide.coding.ObserveAgents",
        "../../../generated/ess/schema/commands/agentide.coding.ObserveAgents.schema.json"
    ),
    (
        "agentide.coding.DelegateAgent",
        "../../../generated/ess/schema/commands/agentide.coding.DelegateAgent.schema.json"
    ),
    (
        "agentide.coding.MessageAgent",
        "../../../generated/ess/schema/commands/agentide.coding.MessageAgent.schema.json"
    ),
    (
        "agentide.coding.WaitAgent",
        "../../../generated/ess/schema/commands/agentide.coding.WaitAgent.schema.json"
    ),
    (
        "agentide.coding.RecordEvidence",
        "../../../generated/ess/schema/commands/agentide.coding.RecordEvidence.schema.json"
    ),
    (
        "agentide.coding.PublishCode",
        "../../../generated/ess/schema/commands/agentide.coding.PublishCode.schema.json"
    ),
    (
        "agentide.coding.CutRelease",
        "../../../generated/ess/schema/commands/agentide.coding.CutRelease.schema.json"
    ),
    (
        "agentide.coding.ObserveDeployment",
        "../../../generated/ess/schema/commands/agentide.coding.ObserveDeployment.schema.json"
    ),
    (
        "agentide.coding.ApplyDeployment",
        "../../../generated/ess/schema/commands/agentide.coding.ApplyDeployment.schema.json"
    ),
    (
        "agentide.surface.OpenFile",
        "../../../generated/ess/schema/commands/agentide.surface.OpenFile.schema.json"
    ),
    (
        "agentide.surface.CloseFile",
        "../../../generated/ess/schema/commands/agentide.surface.CloseFile.schema.json"
    ),
    (
        "agentide.surface.OpenPane",
        "../../../generated/ess/schema/commands/agentide.surface.OpenPane.schema.json"
    ),
    (
        "agentide.surface.ClosePane",
        "../../../generated/ess/schema/commands/agentide.surface.ClosePane.schema.json"
    ),
    (
        "agentide.surface.FocusPane",
        "../../../generated/ess/schema/commands/agentide.surface.FocusPane.schema.json"
    ),
    (
        "agentide.surface.MoveCursor",
        "../../../generated/ess/schema/commands/agentide.surface.MoveCursor.schema.json"
    ),
    (
        "agentide.surface.ShowDiff",
        "../../../generated/ess/schema/commands/agentide.surface.ShowDiff.schema.json"
    ),
    (
        "agentide.surface.SnapshotSurface",
        "../../../generated/ess/schema/commands/agentide.surface.SnapshotSurface.schema.json"
    ),
);

#[cfg(test)]
mod tests {
    use super::*;
    use agentide_contracts::{Binding, BindingConfig, IntentProfile};
    use agentide_core::StateStore;
    use b10x_harness_loop::{AgentLoop, LoopConfig, VecLoopSink};
    use b10x_harness_wire::{
        CallId, Item, ModelPort, StopReason, StreamEvent, StreamSink, ToolName, TurnOutcome,
        TurnRequest, WireId,
    };
    use serde_json::json;

    struct MemoryPort;

    impl IntentPort for MemoryPort {
        fn invoke(&self, _: &Binding, intent: &str, input: &Value) -> Result<Value, Refusal> {
            Ok(json!({"intent": intent, "input": input}))
        }

        fn capabilities(&self) -> Value {
            json!({"driver": "memory"})
        }
    }

    struct Approves;

    impl PlanApprover for Approves {
        fn decide(&mut self, _: &ApprovalRequest) -> ApprovalDecision {
            ApprovalDecision::Approved
        }
    }

    struct Denies;

    impl PlanApprover for Denies {
        fn decide(&mut self, _: &ApprovalRequest) -> ApprovalDecision {
            ApprovalDecision::denied("not this change")
        }
    }

    struct ScriptedModel {
        wire: WireId,
        turn: u8,
    }

    impl ScriptedModel {
        fn new() -> Self {
            Self {
                wire: WireId::new("test-wire").expect("wire"),
                turn: 0,
            }
        }
    }

    impl ModelPort for ScriptedModel {
        fn wire(&self) -> &WireId {
            &self.wire
        }

        fn turn(
            &mut self,
            request: &TurnRequest,
            sink: &mut dyn StreamSink,
        ) -> Result<TurnOutcome, WireError> {
            self.turn += 1;
            if self.turn == 1 {
                assert!(
                    request
                        .tools
                        .iter()
                        .any(|tool| tool.name.as_str() == "code_edit")
                );
                return Ok(TurnOutcome {
                    stop_reason: StopReason::ToolCalls,
                    items: vec![Item::ToolCall(ToolCall {
                        call_id: CallId::new("call-loop-edit").expect("call"),
                        name: ToolName::new("code_edit").expect("name"),
                        arguments: json!({
                            "operation_id": "edit-loop-1",
                            "path": "src/lib.rs",
                            "content": "changed"
                        }),
                    })],
                    usage: None,
                });
            }
            assert!(
                request
                    .items
                    .iter()
                    .any(|item| { matches!(item, Item::ToolResult { failed: false, .. }) })
            );
            sink.emit(StreamEvent::TextDelta {
                text: "done".into(),
            });
            Ok(TurnOutcome {
                stop_reason: StopReason::EndTurn,
                items: vec![Item::assistant("done")],
                usage: None,
            })
        }
    }

    fn fixture_with<A: PlanApprover>(
        approver: A,
    ) -> (
        IntentTools<MemoryPort>,
        IntentApprovals<MemoryPort, A>,
        StateStore,
        String,
        tempfile::TempDir,
    ) {
        let temporary = tempfile::tempdir().expect("temporary");
        let workspace = temporary.path().join("workspace");
        std::fs::create_dir(&workspace).expect("workspace");
        let store = StateStore::at(temporary.path().join("state"));
        let session = store.create(&workspace, "adapter".into()).expect("session");
        let engine = Engine::new(
            IntentProfile::embedded().expect("profile"),
            BindingConfig::embedded().expect("bindings"),
            store.clone(),
            MemoryPort,
        )
        .expect("engine");
        let (tools, approvals) = ports(engine, &session.id, approver).expect("ports");
        (tools, approvals, store, session.id, temporary)
    }

    fn fixture() -> (
        IntentTools<MemoryPort>,
        IntentApprovals<MemoryPort, Approves>,
        StateStore,
        String,
        tempfile::TempDir,
    ) {
        fixture_with(Approves)
    }

    #[test]
    fn published_tools_are_the_bound_model_subset_with_host_fields_removed() {
        let (tools, _, _, _, _temporary) = fixture();
        let read = tools
            .specs()
            .iter()
            .find(|spec| spec.name.as_str() == "code_read")
            .expect("read");
        assert!(read.input_schema["properties"].get("path").is_some());
        assert!(read.input_schema["properties"].get("session_id").is_none());
        assert!(read.input_schema["properties"].get("request_id").is_none());
        let call = ToolCall {
            call_id: CallId::new("call-read").expect("call"),
            name: ToolName::new("code_read").expect("name"),
            arguments: json!({"path": "src/lib.rs", "offset": 0, "limit_bytes": 1024}),
        };
        assert_eq!(tools.subjects(&call)[0].as_str(), "file:src/lib.rs");
        assert!(
            tools
                .specs()
                .iter()
                .all(|spec| spec.name.as_str() != "session_start")
        );
        assert!(
            tools
                .specs()
                .iter()
                .all(|spec| spec.name.as_str() != "code_publish")
        );
    }

    #[test]
    fn harness_approval_grants_and_dispatches_the_same_exact_plan() {
        let (mut tools, mut approvals, store, session_id, _temporary) = fixture();
        let call = ToolCall {
            call_id: CallId::new("call-edit").expect("call id"),
            name: ToolName::new("code_edit").expect("tool name"),
            arguments: json!({"operation_id": "edit-1", "path": "src/lib.rs", "content": "new"}),
        };
        let spec = tools.invoked(&call).expect("spec");
        assert!(approvals.decide(&call, &spec).is_approved());
        let pending = store.load(&session_id).expect("session");
        assert_eq!(pending.pending.len(), 1);
        assert_eq!(pending.approvals.len(), 1);

        let outcome = tools.call(&call);
        assert!(!outcome.failed, "{outcome:?}");
        let completed = store.load(&session_id).expect("session");
        assert!(completed.pending.is_empty());
        assert!(completed.approvals.is_empty());
        assert!(
            completed
                .events
                .iter()
                .any(|event| event.kind == "approval.granted")
        );
    }

    #[test]
    fn a_required_intent_cannot_bypass_the_paired_approval_port() {
        let (mut tools, _, _, _, _temporary) = fixture();
        let call = ToolCall {
            call_id: CallId::new("call-bypass").expect("call id"),
            name: ToolName::new("code_edit").expect("tool name"),
            arguments: json!({"operation_id": "edit-1", "path": "src/lib.rs", "content": "new"}),
        };
        let outcome = tools.call(&call);
        assert!(outcome.failed);
        assert_eq!(outcome.output["code"], "harness.approval_missing");
    }

    #[test]
    fn a_denial_is_durable_and_leaves_no_dispatchable_plan() {
        let (mut tools, mut approvals, store, session_id, _temporary) = fixture_with(Denies);
        let call = ToolCall {
            call_id: CallId::new("call-denied").expect("call id"),
            name: ToolName::new("code_edit").expect("tool name"),
            arguments: json!({"operation_id": "edit-1", "path": "src/lib.rs", "content": "new"}),
        };
        let spec = tools.invoked(&call).expect("spec");
        assert!(!approvals.decide(&call, &spec).is_approved());
        let session = store.load(&session_id).expect("session");
        assert!(session.pending.is_empty());
        assert!(session.approvals.is_empty());
        assert!(
            session
                .events
                .iter()
                .any(|event| event.kind == "approval.denied")
        );
        assert!(tools.call(&call).failed);
    }

    #[test]
    fn a_native_harness_loop_calls_an_ess_intent_through_exact_plan_approval() {
        let (mut tools, mut approvals, store, session_id, _temporary) = fixture();
        let mut model = ScriptedModel::new();
        let mut sink = VecLoopSink::new();
        let outcome = AgentLoop::new(
            &mut model,
            &mut tools,
            &mut approvals,
            LoopConfig::new("scripted", "test"),
        )
        .run("make the edit", &mut sink)
        .expect("loop");
        assert_eq!(outcome.text, "done");
        assert!(outcome.stop.is_completed());
        let session = store.load(&session_id).expect("session");
        assert!(
            session
                .events
                .iter()
                .any(|event| event.kind == "approval.granted")
        );
        assert!(session.events.iter().any(|event| {
            event.kind == "intent.completed" && event.intent.as_deref() == Some("code_edit")
        }));
    }
}
