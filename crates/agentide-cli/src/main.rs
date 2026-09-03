//! AgentIDE standalone command-line, HTTP, and terminal surfaces.
#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::too_many_lines
)]

mod harness_tui;
mod surface_render;
mod surface_ui;
mod tui;
mod web;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agentide_contracts::{BindingConfig, IntentProfile};
use agentide_core::{Engine, Refusal, StateStore};
use agentide_harness::{CredentialSource, ModelConnection, ModelWire};
use agentide_substrate::SubstratePort;
use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{Value, json};

#[derive(Debug, Parser)]
#[command(
    name = "agentide",
    version,
    about = "Semantic coding-session intents over guarded implementations"
)]
struct Cli {
    /// Override AgentIDE's state directory.
    #[arg(long, global = true, env = "AGENTIDE_STATE_ROOT")]
    state_root: Option<PathBuf>,
    /// Operator-owned implementation bindings.
    #[arg(long, global = true, env = "AGENTIDE_BINDINGS")]
    bindings: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a coding session and open the interactive workbench.
    Run(RunArgs),
    /// Create, list, and close coding sessions.
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    /// Render the compact event-derived session projection.
    Snapshot(SessionArg),
    /// Inspect the released semantic intent catalogue.
    Intents {
        #[command(subcommand)]
        command: IntentsCommand,
    },
    /// Preview, invoke, or resume a semantic intent.
    Intent {
        #[command(subcommand)]
        command: IntentCommand,
    },
    /// Grant an exact pending plan.
    Approval {
        #[command(subcommand)]
        command: ApprovalCommand,
    },
    /// Read the durable event stream.
    Events(EventsArgs),
    /// Inspect implementation bindings and Substrate facts.
    Bindings {
        #[command(subcommand)]
        command: BindingsCommand,
    },
    /// Serve the embedded browser workbench.
    Serve(ServeArgs),
    /// Run the interactive console workbench, optionally with a Harness model loop.
    Tui(TuiArgs),
}

#[derive(Debug, Subcommand)]
enum SessionCommand {
    /// Start a session for an existing workspace.
    Start {
        /// Existing target workspace.
        #[arg(long, default_value = ".")]
        workspace: PathBuf,
        /// Goal visible on every surface.
        #[arg(long, default_value = "Coding session")]
        objective: String,
    },
    /// List local sessions.
    List,
}

#[derive(Debug, Clone, Args)]
struct SessionArg {
    /// Session id.
    #[arg(long)]
    session: String,
}

#[derive(Debug, Clone, Args)]
struct RunArgs {
    /// Existing target workspace.
    #[arg(long, default_value = ".")]
    workspace: PathBuf,
    /// Goal visible on every surface.
    #[arg(long, default_value = "Coding session")]
    objective: String,
    #[command(flatten)]
    workbench: WorkbenchArgs,
}

#[derive(Debug, Clone, Args)]
struct TuiArgs {
    /// Session id.
    #[arg(long)]
    session: String,
    #[command(flatten)]
    workbench: WorkbenchArgs,
}

#[derive(Debug, Clone, Args)]
struct WorkbenchArgs {
    /// Harness model API origin plus prefix. Omit together with --model for projection-only mode.
    #[arg(long, env = "AGENTIDE_BASE_URL", requires = "model")]
    base_url: Option<String>,
    /// Exact model identifier. Omit together with --base-url for projection-only mode.
    #[arg(long, env = "AGENTIDE_MODEL", requires = "base_url")]
    model: Option<String>,
    /// Harness provider-wire projection.
    #[arg(long, value_enum, default_value_t = TuiWire::OpenaiResponses)]
    wire: TuiWire,
    /// Declared model context window.
    #[arg(long, default_value_t = 200_000)]
    context_window: u64,
    /// Messages API per-turn output ceiling.
    #[arg(long, default_value_t = 8_192)]
    max_output_tokens: u64,
    /// Maximum Harness model turns for one submitted prompt.
    #[arg(long, default_value_t = 40)]
    max_turns: u64,
    /// Read an API key from this environment variable at request time.
    #[arg(long, value_name = "NAME", conflicts_with_all = ["oauth_token_env", "oauth_token_file"])]
    api_key_env: Option<String>,
    /// Read an OAuth token from this environment variable at request time.
    #[arg(long, value_name = "NAME", conflicts_with_all = ["api_key_env", "oauth_token_file"])]
    oauth_token_env: Option<String>,
    /// Read an OAuth token from this file at request time.
    #[arg(long, value_name = "FILE", conflicts_with_all = ["api_key_env", "oauth_token_env"])]
    oauth_token_file: Option<PathBuf>,
    /// JSON pointer inside --oauth-token-file.
    #[arg(long, value_name = "POINTER", requires = "oauth_token_file")]
    oauth_token_pointer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum TuiWire {
    OpenaiResponses,
    AnthropicMessages,
}

impl From<TuiWire> for ModelWire {
    fn from(wire: TuiWire) -> Self {
        match wire {
            TuiWire::OpenaiResponses => Self::OpenaiResponses,
            TuiWire::AnthropicMessages => Self::AnthropicMessages,
        }
    }
}

#[derive(Debug, Subcommand)]
enum IntentsCommand {
    /// List released intents.
    List,
    /// Describe one intent and its consequence envelope.
    Describe { name: String },
}

#[derive(Debug, Subcommand)]
enum IntentCommand {
    /// Build and journal the exact implementation plan without dispatching it.
    Preview(IntentArgs),
    /// Preview and dispatch an authority-free intent.
    Call(IntentArgs),
    /// Dispatch a previously previewed (and, where required, approved) plan.
    Resume {
        #[arg(long)]
        session: String,
        #[arg(long)]
        plan: String,
        /// Exact JSON input used for preview, or `@path`.
        #[arg(long, default_value = "{}")]
        input: String,
    },
}

#[derive(Debug, Args)]
struct IntentArgs {
    #[arg(long)]
    session: String,
    /// Stable semantic intent name.
    name: String,
    /// JSON object, or `@path` to a JSON document.
    #[arg(long, default_value = "{}")]
    input: String,
}

#[derive(Debug, Subcommand)]
enum ApprovalCommand {
    /// Approve exactly one pending plan digest.
    Grant {
        #[arg(long)]
        session: String,
        #[arg(long)]
        plan: String,
    },
}

#[derive(Debug, Args)]
struct EventsArgs {
    #[arg(long)]
    session: String,
    #[arg(long, default_value_t = 0)]
    after: u64,
    #[arg(long, default_value_t = 100)]
    limit: usize,
    /// Emit one event JSON object per line.
    #[arg(long)]
    jsonl: bool,
}

#[derive(Debug, Subcommand)]
enum BindingsCommand {
    /// Show selected operations and non-secret capability facts.
    Inspect(SessionArg),
}

#[derive(Debug, Args)]
struct ServeArgs {
    #[arg(long)]
    session: String,
    #[arg(long, default_value = "127.0.0.1:7788")]
    listen: String,
}

type StandaloneEngine = Engine<SubstratePort>;

fn main() {
    match run(Cli::parse()) {
        Ok(Some(value)) => match serde_json::to_string_pretty(&value) {
            Ok(rendered) => println!("{rendered}"),
            Err(error) => {
                eprintln!("{{\"error\":\"output.json\",\"message\":{error:?}}}");
                std::process::exit(2);
            }
        },
        Ok(None) => {}
        Err(error) => {
            let refusal = error
                .downcast_ref::<Refusal>()
                .map_or_else(
                    || json!({"format": "agentide.refusal/1", "code": "agentide.failed", "message": error.to_string(), "retryable": false}),
                    |refusal| json!({"format": "agentide.refusal/1", "code": refusal.code, "message": refusal.message, "retryable": refusal.retryable}),
                );
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&refusal)
                    .unwrap_or_else(|_| "{\"code\":\"output.failed\"}".into())
            );
            std::process::exit(2);
        }
    }
}

fn run(cli: Cli) -> Result<Option<Value>> {
    let store = cli
        .state_root
        .map_or_else(StateStore::discover, |root| Ok(StateStore::at(root)))?;
    match cli.command {
        Command::Run(arguments) => {
            // Opening the port before recording the session prevents an unusable optimistic record.
            SubstratePort::adopt(&arguments.workspace)?;
            let session = store.create(&arguments.workspace, arguments.objective)?;
            println!("Started AgentIDE session {}", session.id);
            println!("Resume later with: agentide tui --session {}", session.id);
            let result = run_workbench(
                store,
                cli.bindings.as_deref(),
                &session.id,
                arguments.workbench,
            );
            println!("AgentIDE session {} is preserved.", session.id);
            println!("Resume with: agentide tui --session {}", session.id);
            result?;
            Ok(None)
        }
        Command::Session {
            command:
                SessionCommand::Start {
                    workspace,
                    objective,
                },
        } => {
            // Opening the port before recording the session prevents an unusable optimistic record.
            SubstratePort::adopt(&workspace)?;
            let session = store.create(&workspace, objective)?;
            Ok(Some(json!({
                "format": "agentide.session-started/1",
                "session_id": session.id,
                "workspace": session.workspace_root,
                "next": format!("agentide snapshot --session {}", session.id),
            })))
        }
        Command::Session {
            command: SessionCommand::List,
        } => {
            let sessions = store.list()?;
            Ok(Some(serde_json::to_value(sessions)?))
        }
        Command::Intents {
            command: IntentsCommand::List,
        } => {
            let profile = IntentProfile::embedded()?;
            Ok(Some(serde_json::to_value(profile.intents)?))
        }
        Command::Intents {
            command: IntentsCommand::Describe { name },
        } => {
            let profile = IntentProfile::embedded()?;
            let intent = profile
                .find(&name)
                .ok_or_else(|| anyhow!("unknown intent `{name}`"))?;
            Ok(Some(serde_json::to_value(intent)?))
        }
        Command::Snapshot(argument) => {
            let engine = engine(&store, &argument.session, cli.bindings.as_deref())?;
            Ok(Some(serde_json::to_value(
                engine.snapshot(&argument.session)?,
            )?))
        }
        Command::Intent {
            command: IntentCommand::Preview(argument),
        } => {
            let engine = engine(&store, &argument.session, cli.bindings.as_deref())?;
            Ok(Some(serde_json::to_value(engine.preview(
                &argument.session,
                &argument.name,
                read_input(&argument.input)?,
            )?)?))
        }
        Command::Intent {
            command: IntentCommand::Call(argument),
        } => {
            let engine = engine(&store, &argument.session, cli.bindings.as_deref())?;
            Ok(Some(engine.call(
                &argument.session,
                &argument.name,
                read_input(&argument.input)?,
            )?))
        }
        Command::Intent {
            command:
                IntentCommand::Resume {
                    session,
                    plan,
                    input,
                },
        } => {
            let engine = engine(&store, &session, cli.bindings.as_deref())?;
            Ok(Some(engine.resume(&session, &plan, read_input(&input)?)?))
        }
        Command::Approval {
            command: ApprovalCommand::Grant { session, plan },
        } => {
            let engine = engine(&store, &session, cli.bindings.as_deref())?;
            engine.grant(&session, &plan)?;
            Ok(Some(
                json!({"format": "agentide.approval/1", "session_id": session, "plan_digest": plan, "status": "granted"}),
            ))
        }
        Command::Events(arguments) => {
            let engine = engine(&store, &arguments.session, cli.bindings.as_deref())?;
            let events = engine.events(&arguments.session, arguments.after, arguments.limit)?;
            if arguments.jsonl {
                for event in events {
                    println!("{}", serde_json::to_string(&event)?);
                }
                Ok(None)
            } else {
                Ok(Some(serde_json::to_value(events)?))
            }
        }
        Command::Bindings {
            command: BindingsCommand::Inspect(argument),
        } => {
            let engine = engine(&store, &argument.session, cli.bindings.as_deref())?;
            Ok(Some(engine.inspect_bindings()))
        }
        Command::Serve(arguments) => {
            let engine = Arc::new(engine(&store, &arguments.session, cli.bindings.as_deref())?);
            // Keep the Substrate port alive outside Tokio: its embedded guarded driver owns a
            // current-thread runtime and must neither execute nor be dropped inside another one.
            let runtime = tokio::runtime::Runtime::new()?;
            let result = runtime.block_on(web::serve(
                Arc::clone(&engine),
                arguments.session,
                &arguments.listen,
            ));
            drop(runtime);
            drop(engine);
            result?;
            Ok(None)
        }
        Command::Tui(argument) => {
            run_workbench(
                store,
                cli.bindings.as_deref(),
                &argument.session,
                argument.workbench,
            )?;
            Ok(None)
        }
    }
}

fn run_workbench(
    store: StateStore,
    binding_path: Option<&Path>,
    session_id: &str,
    arguments: WorkbenchArgs,
) -> Result<()> {
    if let (Some(base_url), Some(model)) = (arguments.base_url, arguments.model) {
        let credential = if let Some(variable) = arguments.api_key_env {
            CredentialSource::ApiKeyEnvironment(variable)
        } else if let Some(variable) = arguments.oauth_token_env {
            CredentialSource::OauthEnvironment(variable)
        } else if let Some(path) = arguments.oauth_token_file {
            CredentialSource::OauthFile {
                path,
                pointer: arguments.oauth_token_pointer,
            }
        } else {
            CredentialSource::None
        };
        harness_tui::run(
            store,
            binding_path,
            session_id,
            ModelConnection {
                wire: arguments.wire.into(),
                base_url,
                model,
                context_window: arguments.context_window,
                max_output_tokens: arguments.max_output_tokens,
                credential,
            },
            arguments.max_turns,
        )?;
    } else {
        let engine = engine(&store, session_id, binding_path)?;
        tui::run(&engine, session_id)?;
    }
    Ok(())
}

fn engine(
    store: &StateStore,
    session_id: &str,
    binding_path: Option<&Path>,
) -> Result<StandaloneEngine> {
    let session = store.load(session_id)?;
    let port = SubstratePort::adopt(&session.workspace_root)?;
    let profile = IntentProfile::embedded()?;
    let bindings = binding_path.map_or_else(BindingConfig::embedded, BindingConfig::from_path)?;
    Engine::new(profile, bindings, store.clone(), port).map_err(Into::into)
}

fn read_input(argument: &str) -> Result<Value> {
    let text = if let Some(path) = argument.strip_prefix('@') {
        std::fs::read_to_string(path).with_context(|| format!("reading input `{path}`"))?
    } else {
        argument.into()
    };
    let input: Value = serde_json::from_str(&text).context("intent input must be JSON")?;
    if !input.is_object() {
        return Err(anyhow!("intent input must be a JSON object"));
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::{Cli, Command};

    #[test]
    fn run_defaults_to_the_current_workspace_and_a_plain_objective() {
        let cli = Cli::try_parse_from(["agentide", "run"]).expect("parse run defaults");
        let Command::Run(arguments) = cli.command else {
            panic!("expected run command");
        };
        assert_eq!(arguments.workspace, Path::new("."));
        assert_eq!(arguments.objective, "Coding session");
        assert!(arguments.workbench.base_url.is_none());
        assert!(arguments.workbench.model.is_none());
    }

    #[test]
    fn run_accepts_an_explicit_model_connection() {
        let cli = Cli::try_parse_from([
            "agentide",
            "run",
            "--base-url",
            "https://model.example/v1",
            "--model",
            "model-id",
            "--api-key-env",
            "MODEL_API_KEY",
        ])
        .expect("parse model-backed run");
        let Command::Run(arguments) = cli.command else {
            panic!("expected run command");
        };
        assert_eq!(
            arguments.workbench.base_url.as_deref(),
            Some("https://model.example/v1")
        );
        assert_eq!(arguments.workbench.model.as_deref(), Some("model-id"));
        assert_eq!(
            arguments.workbench.api_key_env.as_deref(),
            Some("MODEL_API_KEY")
        );
    }

    #[test]
    fn run_refuses_a_partial_model_connection() {
        let error =
            Cli::try_parse_from(["agentide", "run", "--base-url", "https://model.example/v1"])
                .expect_err("a model id is required with a base URL");
        assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn command_help_includes_the_one_step_entrypoint() {
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("run"));
        assert!(help.contains("Create a coding session and open"));
    }
}
