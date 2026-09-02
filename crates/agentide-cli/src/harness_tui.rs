//! Interactive AgentIDE renderer over a native Harness model loop.

use std::fmt::Write as _;
use std::io;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use agentide_contracts::{BindingConfig, IntentProfile, SurfaceProfile};
use agentide_core::{Engine, Snapshot, StateStore, project};
use agentide_harness::{ApprovalRequest, ModelConnection, PlanApprover, ports};
use agentide_substrate::SubstratePort;
use anyhow::{Context, Result, anyhow};
use b10x_harness_loop::{
    AgentLoop, ApprovalDecision, Budget, LoopCancel, LoopConfig, LoopEvent, LoopSink, LoopStop,
    RunLedger,
};
use b10x_harness_wire::{CallId, Item, Risk, ToolCall, ToolName, ToolOutcome, ToolPort};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use serde_json::{Value, json};

use crate::surface_render::{RenderState, draw};
use crate::surface_ui::{MainView, SurfaceState, UiEffect, UiEvent};

/// Runs the interactive Harness-native TUI for one durable AgentIDE session.
pub fn run(
    store: StateStore,
    binding_path: Option<&Path>,
    session_id: &str,
    connection: ModelConnection,
    max_turns: u64,
) -> Result<()> {
    let session = store.load(session_id)?;
    let bindings = binding_path.map_or_else(BindingConfig::embedded, BindingConfig::from_path)?;
    let surface_profile = SurfaceProfile::embedded()?;
    let ui_store = store.clone();
    let model_name = connection.model.clone();
    let (commands, worker_commands) = mpsc::channel();
    let (messages, worker_messages) = mpsc::channel();
    let cancel = LoopCancel::new();
    let worker_cancel = cancel.clone();
    let worker_session = session_id.to_owned();
    let worker = thread::Builder::new()
        .name("agentide-harness".into())
        .spawn(move || {
            worker_loop(
                worker_commands,
                messages,
                worker_cancel,
                store.clone(),
                worker_session,
                session.workspace_root,
                session.objective,
                bindings,
                connection,
                max_turns,
            );
        })?;

    enable_raw_mode()?;
    let mut output = io::stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    let result = ui_loop(
        &mut terminal,
        &commands,
        &worker_messages,
        &ui_store,
        session_id,
        &model_name,
        &surface_profile,
    );
    cancel.cancel();
    drop(commands);
    let joined = worker.join();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    if let Err(payload) = joined {
        return Err(anyhow!(
            "Harness TUI worker panicked: {}",
            panic_words(&payload)
        ));
    }
    result
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn worker_loop(
    commands: Receiver<WorkerCommand>,
    messages: Sender<UiMessage>,
    cancel: LoopCancel,
    store: StateStore,
    session_id: String,
    workspace: std::path::PathBuf,
    objective: String,
    bindings: BindingConfig,
    connection: ModelConnection,
    max_turns: u64,
) {
    let ready = (|| -> Result<WorkerRuntime> {
        let port = SubstratePort::adopt(&workspace)?;
        let engine = Engine::new(IntentProfile::embedded()?, bindings, store, port)?;
        let approver = ChannelApprover {
            messages: messages.clone(),
        };
        let (tools, approvals) = ports(engine, &session_id, approver)?;
        let model = connection.connect(cancel.clone())?;
        let config = LoopConfig::new(&connection.model, standing_instruction(&objective))
            .with_budget(Budget::default().with_max_turns(max_turns))
            .with_unattended_ceiling(Risk::Low)
            .with_context_window(Some(connection.context_window));
        Ok(WorkerRuntime {
            model,
            tools,
            approvals,
            config,
            cancel,
            items: Vec::new(),
            messages: messages.clone(),
            call_sequence: 0,
        })
    })();
    let mut runtime = match ready {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = messages.send(UiMessage::Fatal(error.to_string()));
            return;
        }
    };
    let _ = messages.send(UiMessage::Ready);
    while let Ok(command) = commands.recv() {
        match command {
            WorkerCommand::Prompt(prompt) => runtime.prompt(prompt),
            WorkerCommand::Intent { name, input } => runtime.intent(name, input),
            WorkerCommand::Shutdown => break,
        }
        if runtime.cancel.is_cancelled() {
            break;
        }
    }
}

struct WorkerRuntime {
    model: Box<dyn b10x_harness_wire::ModelPort>,
    tools: agentide_harness::IntentTools<SubstratePort>,
    approvals: agentide_harness::IntentApprovals<SubstratePort, ChannelApprover>,
    config: LoopConfig,
    cancel: LoopCancel,
    items: Vec<Item>,
    messages: Sender<UiMessage>,
    call_sequence: u64,
}

impl WorkerRuntime {
    fn prompt(&mut self, prompt: String) {
        let mut sink = ChannelSink {
            messages: self.messages.clone(),
        };
        let mut loop_ = AgentLoop::new(
            self.model.as_mut(),
            &mut self.tools,
            &mut self.approvals,
            self.config.clone(),
        )
        .with_cancel(self.cancel.clone());
        let mut spend = RunLedger::default();
        let outcome = loop_.run_in(&mut self.items, &mut spend, prompt, &mut sink);
        let result = outcome
            .map(|outcome| {
                let tokens = outcome.total_tokens();
                PromptResult {
                    stop: outcome.stop,
                    text: outcome.text,
                    turns: outcome.turns,
                    tokens,
                }
            })
            .map_err(|error| error.to_string());
        let _ = self.messages.send(UiMessage::PromptFinished(result));
    }

    fn intent(&mut self, name: String, input: Value) {
        self.call_sequence = self.call_sequence.saturating_add(1);
        let call_id = format!("surface-{}", self.call_sequence);
        let outcome = match (CallId::new(call_id), ToolName::new(&name)) {
            (Ok(call_id), Ok(name)) => self.tools.call(&ToolCall {
                call_id,
                name,
                arguments: input,
            }),
            (Err(error), _) | (_, Err(error)) => ToolOutcome::failed(error.to_string()),
        };
        let _ = self
            .messages
            .send(UiMessage::IntentFinished { name, outcome });
    }
}

fn standing_instruction(objective: &str) -> String {
    format!(
        "You are the coding agent inside an AgentIDE session. The operator's objective is: {objective}\n\nEvery callable operation is an AgentIDE semantic intent implemented through Harness. Use only published tools and treat a failed result as an effect that did not happen. Keep the shared workbench useful: open a file when it becomes the focus of the work and show the diff after making changes. Session identity, implementation selection, credentials, destinations, and authority are supplied by the host; never invent them. Ground every claim in observations and say plainly what remains unverified."
    )
}

#[derive(Debug)]
enum WorkerCommand {
    Prompt(String),
    Intent { name: String, input: Value },
    Shutdown,
}

#[derive(Debug)]
enum UiMessage {
    Ready,
    Loop(LoopEvent),
    Approval {
        request: ApprovalRequest,
        reply: Sender<ApprovalDecision>,
    },
    PromptFinished(Result<PromptResult, String>),
    IntentFinished {
        name: String,
        outcome: ToolOutcome,
    },
    Fatal(String),
}

#[derive(Debug)]
struct PromptResult {
    stop: LoopStop,
    text: String,
    turns: u64,
    tokens: Option<(u64, u64)>,
}

struct ChannelSink {
    messages: Sender<UiMessage>,
}

impl LoopSink for ChannelSink {
    fn emit(&mut self, event: LoopEvent) {
        let _ = self.messages.send(UiMessage::Loop(event));
    }
}

struct ChannelApprover {
    messages: Sender<UiMessage>,
}

impl PlanApprover for ChannelApprover {
    fn decide(&mut self, request: &ApprovalRequest) -> ApprovalDecision {
        let (reply, decision) = mpsc::channel();
        if self
            .messages
            .send(UiMessage::Approval {
                request: request.clone(),
                reply,
            })
            .is_err()
        {
            return ApprovalDecision::denied("the AgentIDE approval surface disconnected");
        }
        decision.recv().unwrap_or_else(|_| {
            ApprovalDecision::denied("the AgentIDE approval surface closed without a decision")
        })
    }
}

struct PendingApproval {
    request: ApprovalRequest,
    reply: Sender<ApprovalDecision>,
}

struct UiState {
    harness: HarnessProjection,
    surface: SurfaceState,
    notice: String,
    observation: String,
    observed_target: Option<String>,
    approval: Option<PendingApproval>,
    model_busy: bool,
    pending_intents: usize,
    fatal: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            harness: HarnessProjection::default(),
            surface: SurfaceState::default(),
            notice: "connecting Harness…".into(),
            observation: String::new(),
            observed_target: None,
            approval: None,
            model_busy: false,
            pending_intents: 0,
            fatal: None,
        }
    }
}

impl UiState {
    fn busy(&self) -> bool {
        self.model_busy || self.pending_intents > 0
    }
}

#[derive(Debug, Default)]
struct HarnessProjection {
    status: String,
    transcript: String,
    reasoning: String,
    activity: Vec<String>,
    turn: u64,
    input_tokens: u64,
    output_tokens: u64,
    retry_start: usize,
    answer_start: usize,
}

impl HarnessProjection {
    fn event(&mut self, event: &LoopEvent) {
        match event {
            LoopEvent::Started {
                model,
                published_tools,
                ..
            } => {
                self.status = format!("ready · {model} · {} intents", published_tools.len());
                self.push(format!(
                    "Harness published {} AgentIDE intents",
                    published_tools.len()
                ));
            }
            LoopEvent::TurnStarted { turn } => {
                self.turn = *turn;
                self.retry_start = self.transcript.len();
                self.status = format!("running turn {turn}");
            }
            LoopEvent::TurnRetried {
                attempt, reason, ..
            } => {
                self.transcript.truncate(self.retry_start);
                self.push(format!("turn retried ({attempt}): {reason}"));
            }
            LoopEvent::TextDelta { text } => self.transcript.push_str(text),
            LoopEvent::ReasoningDelta { text } => {
                self.reasoning.push_str(text);
                if self.reasoning.len() > 2_000 {
                    self.reasoning.drain(..self.reasoning.len() - 2_000);
                }
            }
            LoopEvent::ToolRequested {
                call, operation, ..
            } => {
                let operation = operation.as_deref().unwrap_or(call.name.as_str());
                self.push(format!(
                    "→ {operation} {}",
                    compact_json(&call.arguments, 160)
                ));
            }
            LoopEvent::ApprovalRequired { name, .. } => {
                self.status = format!("approval · {name}");
                self.push(format!("? approval required for {name}"));
            }
            LoopEvent::ApprovalResolved { approved, .. } => {
                self.push(
                    if *approved {
                        "✓ approved"
                    } else {
                        "× denied"
                    }
                    .into(),
                );
            }
            LoopEvent::ToolCompleted { failed, .. } => {
                self.push(
                    if *failed {
                        "× tool failed"
                    } else {
                        "✓ tool completed"
                    }
                    .into(),
                );
            }
            LoopEvent::Usage(usage) => {
                self.input_tokens = self.input_tokens.saturating_add(usage.input_tokens);
                self.output_tokens = self.output_tokens.saturating_add(usage.output_tokens);
            }
            LoopEvent::Warning { code, message } => self.push(format!("! {code}: {message}")),
            LoopEvent::Compacted {
                bytes_before,
                bytes_after,
                ..
            } => {
                self.push(format!(
                    "conversation compacted {bytes_before} → {bytes_after} bytes"
                ));
            }
            LoopEvent::Finished { stop, turns } => {
                self.status = format!("{} · {turns} turn(s)", stop_label(stop));
            }
            _ => {}
        }
    }

    fn push(&mut self, line: String) {
        self.activity.push(line);
        while self.activity.len() > 200 {
            self.activity.remove(0);
        }
    }
}

fn ui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    commands: &Sender<WorkerCommand>,
    messages: &Receiver<UiMessage>,
    store: &StateStore,
    session_id: &str,
    model: &str,
    profile: &SurfaceProfile,
) -> Result<()> {
    let mut state = UiState::default();
    loop {
        drain_messages(messages, &mut state);
        if let Some(error) = state.fatal.take() {
            return Err(anyhow!(error));
        }
        let snapshot = project(&store.load(session_id)?);
        let area = terminal.size()?;
        state.surface.reduce(
            UiEvent::Resize {
                columns: area.width,
                rows: area.height,
            },
            profile,
            &snapshot,
            state.busy(),
            state.approval.is_some(),
        );
        terminal.draw(|frame| {
            draw(
                frame,
                &RenderState {
                    snapshot: &snapshot,
                    profile,
                    surface: &state.surface,
                    model,
                    harness_status: &state.harness.status,
                    transcript: &state.harness.transcript,
                    activity: &state.harness.activity,
                    observation: &state.observation,
                    notice: &state.notice,
                    turn: state.harness.turn,
                    input_tokens: state.harness.input_tokens,
                    output_tokens: state.harness.output_tokens,
                    reasoning: !state.harness.reasoning.is_empty(),
                    busy: state.busy(),
                    approval: state.approval.as_ref().map(|pending| &pending.request),
                },
            );
        })?;
        if !event::poll(Duration::from_millis(60))? {
            maybe_refresh(commands, &snapshot, &mut state);
            continue;
        }
        let key = match event::read()? {
            Event::Key(key) => key,
            Event::Resize(columns, rows) => {
                state.surface.reduce(
                    UiEvent::Resize { columns, rows },
                    profile,
                    &snapshot,
                    state.busy(),
                    state.approval.is_some(),
                );
                continue;
            }
            _ => continue,
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            deny_pending(&mut state, "operator cancelled the TUI");
            let _ = commands.send(WorkerCommand::Shutdown);
            return Ok(());
        }
        let effects = state.surface.reduce(
            UiEvent::Key {
                chord: key_chord(key),
                character: key_character(key),
            },
            profile,
            &snapshot,
            state.busy(),
            state.approval.is_some(),
        );
        if apply_effects(effects, commands, &snapshot, &mut state)? {
            let _ = commands.send(WorkerCommand::Shutdown);
            return Ok(());
        }
    }
}

fn apply_effects(
    effects: Vec<UiEffect>,
    commands: &Sender<WorkerCommand>,
    snapshot: &Snapshot,
    state: &mut UiState,
) -> Result<bool> {
    for effect in effects {
        match effect {
            UiEffect::Prompt(prompt) => {
                let _ = write!(state.harness.transcript, "\n\nYou › {prompt}\n\nAgent › ");
                state.harness.answer_start = state.harness.transcript.len();
                commands
                    .send(WorkerCommand::Prompt(prompt))
                    .context("starting Harness prompt")?;
                state.model_busy = true;
                state.notice = "Harness is running".into();
            }
            UiEffect::Intent { name, input } => {
                if name == "code_changes" {
                    state.observed_target = Some("diff".into());
                } else if name == "code_read" {
                    state.observed_target = input
                        .get("path")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                } else if name == "pane_close" {
                    state.observed_target = None;
                }
                send_intent(commands, state, &name, input);
            }
            UiEffect::Refresh => {
                state.observed_target = None;
                maybe_refresh(commands, snapshot, state);
            }
            UiEffect::Approval(approved) => resolve_approval(state, approved),
            UiEffect::Quit => return Ok(true),
        }
    }
    Ok(false)
}

fn key_chord(key: KeyEvent) -> String {
    let prefix = if key.modifiers.contains(KeyModifiers::CONTROL) {
        "ctrl+"
    } else if key.modifiers.contains(KeyModifiers::ALT) {
        "alt+"
    } else {
        ""
    };
    let key = match key.code {
        KeyCode::BackTab => "shift+tab".into(),
        KeyCode::Tab => "tab".into(),
        KeyCode::Enter => "enter".into(),
        KeyCode::Esc => "escape".into(),
        KeyCode::Backspace => "backspace".into(),
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::Char(character) => character.to_ascii_lowercase().to_string(),
        _ => return String::new(),
    };
    if key.starts_with("shift+") {
        key
    } else {
        format!("{prefix}{key}")
    }
}

fn key_character(key: KeyEvent) -> Option<char> {
    match key.code {
        KeyCode::Char(character)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            Some(character)
        }
        _ => None,
    }
}

fn drain_messages(messages: &Receiver<UiMessage>, state: &mut UiState) {
    loop {
        match messages.try_recv() {
            Ok(UiMessage::Ready) => state.notice = "Harness ready · i to prompt".into(),
            Ok(UiMessage::Loop(event)) => state.harness.event(&event),
            Ok(UiMessage::Approval { request, reply }) => {
                state.notice = format!("approve exact plan {}?", short(&request.plan.digest));
                state.surface.reset_approval_scroll();
                state.approval = Some(PendingApproval { request, reply });
            }
            Ok(UiMessage::PromptFinished(result)) => {
                state.model_busy = false;
                match result {
                    Ok(result) => {
                        state.notice = format!(
                            "{} · {} turn(s){}",
                            stop_label(&result.stop),
                            result.turns,
                            result.tokens.map_or_else(String::new, |(input, output)| {
                                format!(" · {input} in/{output} out")
                            })
                        );
                        if state.harness.transcript.len() == state.harness.answer_start
                            && !result.text.is_empty()
                        {
                            state.harness.transcript.push_str(&result.text);
                        }
                    }
                    Err(error) => {
                        state.notice = format!("Harness failed: {error}");
                        state.harness.push(format!("! Harness failed: {error}"));
                    }
                }
                state.observed_target = None;
            }
            Ok(UiMessage::IntentFinished { name, outcome }) => {
                state.pending_intents = state.pending_intents.saturating_sub(1);
                if outcome.failed {
                    state.notice = format!("{name} failed: {}", compact_json(&outcome.output, 180));
                } else {
                    state.notice = format!("{name} completed");
                    if matches!(name.as_str(), "code_read" | "code_changes") {
                        state.observation = render_observation(&outcome.output);
                    }
                }
            }
            Ok(UiMessage::Fatal(error)) => state.fatal = Some(error),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
        }
    }
}

fn send_intent(commands: &Sender<WorkerCommand>, state: &mut UiState, name: &str, input: Value) {
    if commands
        .send(WorkerCommand::Intent {
            name: name.into(),
            input,
        })
        .is_ok()
    {
        state.pending_intents = state.pending_intents.saturating_add(1);
    }
}

fn maybe_refresh(commands: &Sender<WorkerCommand>, snapshot: &Snapshot, state: &mut UiState) {
    if state.busy() || state.surface.view != MainView::Workbench {
        return;
    }
    let Some(pane) = focused_pane(snapshot) else {
        return;
    };
    let target = pane.path.clone().unwrap_or_else(|| pane.kind.clone());
    if state.observed_target.as_deref() == Some(&target) {
        return;
    }
    match pane.kind.as_str() {
        "editor" => {
            if let Some(path) = &pane.path {
                send_intent(commands, state, "code_read", json!({"path": path}));
                state.observed_target = Some(target);
            }
        }
        "diff" => {
            send_intent(commands, state, "code_changes", json!({}));
            state.observed_target = Some(target);
        }
        _ => {}
    }
}

fn resolve_approval(state: &mut UiState, approved: bool) {
    let Some(pending) = state.approval.take() else {
        return;
    };
    let decision = if approved {
        ApprovalDecision::Approved
    } else {
        ApprovalDecision::denied("operator denied the exact plan in AgentIDE")
    };
    let _ = pending.reply.send(decision);
    state.notice = if approved {
        format!("approved {}", short(&pending.request.plan.digest))
    } else {
        format!("denied {}", short(&pending.request.plan.digest))
    };
}

fn deny_pending(state: &mut UiState, reason: &str) {
    if let Some(pending) = state.approval.take() {
        let _ = pending.reply.send(ApprovalDecision::denied(reason));
    }
}

fn focused_pane(snapshot: &Snapshot) -> Option<&agentide_core::Pane> {
    snapshot
        .workbench
        .focused_pane
        .as_deref()
        .and_then(|id| snapshot.workbench.panes.iter().find(|pane| pane.id == id))
}

fn render_observation(value: &Value) -> String {
    value
        .get("content")
        .and_then(Value::as_str)
        .map_or_else(
            || serde_json::to_string_pretty(value).unwrap_or_else(|_| "unrenderable result".into()),
            ToOwned::to_owned,
        )
        .chars()
        .take(150_000)
        .collect()
}

fn compact_json(value: &Value, limit: usize) -> String {
    let rendered = serde_json::to_string(value).unwrap_or_else(|_| "<invalid-json>".into());
    if rendered.chars().count() <= limit {
        return rendered;
    }
    let mut compact: String = rendered.chars().take(limit.saturating_sub(1)).collect();
    compact.push('…');
    compact
}

fn short(value: &str) -> &str {
    value.get(..12).unwrap_or(value)
}

fn stop_label(stop: &LoopStop) -> &'static str {
    match stop {
        LoopStop::Completed => "completed",
        LoopStop::MaxTurns { .. } => "max turns",
        LoopStop::MaxInputTokens { .. } => "input budget",
        LoopStop::MaxOutputTokens { .. } => "output budget",
        LoopStop::MaxCost { .. } => "cost budget",
        LoopStop::BudgetUnobservable { .. } => "budget unobservable",
        LoopStop::Deadline { .. } => "deadline",
        LoopStop::Cancelled { .. } => "cancelled",
        LoopStop::ProviderIncomplete { .. } => "provider incomplete",
        LoopStop::Unstructured { .. } => "unstructured",
    }
}

fn panic_words(payload: &Box<dyn std::any::Any + Send>) -> String {
    payload.downcast_ref::<&str>().map_or_else(
        || {
            payload
                .downcast_ref::<String>()
                .map_or_else(|| "unknown panic".into(), Clone::clone)
        },
        |message| (*message).into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use b10x_harness_wire::{CallId, ToolName, Usage};

    #[test]
    fn harness_events_reduce_into_agent_text_activity_and_usage() {
        let mut projection = HarnessProjection::default();
        projection.event(&LoopEvent::TurnStarted { turn: 1 });
        projection.event(&LoopEvent::TextDelta {
            text: "hello".into(),
        });
        projection.event(&LoopEvent::ToolRequested {
            call: ToolCall {
                call_id: CallId::new("call-1").expect("call"),
                name: ToolName::new("code_read").expect("name"),
                arguments: json!({"path": "src/lib.rs"}),
            },
            operation: Some("agentide.coding.ReadCode".into()),
            subjects: vec!["file:src/lib.rs".into()],
        });
        projection.event(&LoopEvent::Usage(Usage {
            model: "test".into(),
            input_tokens: 10,
            output_tokens: 3,
            cached_input_tokens: 0,
            cache_creation_input_tokens: None,
        }));
        assert_eq!(projection.transcript, "hello");
        assert!(
            projection
                .activity
                .last()
                .expect("activity")
                .contains("ReadCode")
        );
        assert_eq!((projection.input_tokens, projection.output_tokens), (10, 3));
    }

    #[test]
    fn a_retried_turn_discards_the_streamed_attempt() {
        let mut projection = HarnessProjection {
            transcript: "prior\n".into(),
            ..HarnessProjection::default()
        };
        projection.event(&LoopEvent::TurnStarted { turn: 2 });
        projection.event(&LoopEvent::TextDelta {
            text: "partial".into(),
        });
        projection.event(&LoopEvent::TurnRetried {
            turn: 2,
            attempt: 1,
            reason: "network".into(),
        });
        assert_eq!(projection.transcript, "prior\n");
    }
}
