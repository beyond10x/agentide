//! AgentIDE standalone command-line, HTTP, and terminal surfaces.
#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::too_many_lines
)]

mod tui;
mod web;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agentide_contracts::{BindingConfig, IntentProfile};
use agentide_core::{Engine, Refusal, StateStore};
use agentide_substrate::SubstratePort;
use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};
use serde_json::{Value, json};

#[derive(Debug, Parser)]
#[command(
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
    /// Run the interactive console workbench.
    Tui(SessionArg),
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
            let engine = engine(&store, &argument.session, cli.bindings.as_deref())?;
            tui::run(&engine, &argument.session)?;
            Ok(None)
        }
    }
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
