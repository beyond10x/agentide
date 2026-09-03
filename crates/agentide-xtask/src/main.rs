//! Repository-local reproducibility and conformance gate.
#![allow(clippy::missing_errors_doc, clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use agentide_contracts::{
    ActorContext, ActorKind, ActorView, ActorWorkbench, Approval, AttachmentProvenance, Audience,
    AuthorityGrant, AuthorizationPath, AvailableIntent, ChangeSelector, ContextPack, ContextRecord,
    ContextSelection, ContextSelectionDraft, CoordinationRevision, DiffFile, DiffFileStatus,
    DiffHunk, DiffLine, DiffLineKind, DiffMode, DiffProjection, DiffRange, Effect,
    FileModificationState, FileProjection, FileRevision, IntentDefinition, IntentInventory, Risk,
    SelectionKind, TerminalControl, TerminalControlFrame, TerminalEvent, TerminalProfile,
    TerminalReplayBounds, TerminalServerFrame, TerminalSession, TerminalState,
    TerminalWorkspaceAccess, TreeEntry, TreeEntryKind, TreeProjection, WorkbenchPane,
};
use anyhow::{Context, Result, anyhow, bail};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

type GeneratedDocuments = BTreeMap<PathBuf, Vec<u8>>;
type HostedContractDocuments = (GeneratedDocuments, GeneratedDocuments);

fn main() -> Result<()> {
    let command = std::env::args().nth(1).unwrap_or_else(|| "help".into());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask is two levels below workspace root");

    if command == "generate-surface-profile" {
        let target = root.join("web/src/generated/surface-profile.ts");
        std::fs::create_dir_all(target.parent().expect("generated source parent"))?;
        std::fs::write(&target, render_web_surface_profile()?)?;
        println!("{}", target.display());
        return Ok(());
    }
    if command == "generate-hosted-contracts" {
        write_hosted_contracts(root)?;
        println!("contracts/schemas/hosted contracts/fixtures/hosted");
        return Ok(());
    }
    if command == "generate-service" {
        let package = service_builder::package::ServicePackage::read(&root.join("service.yaml"))
            .context("loading the AgentIDE Service SDK package")?;
        let build = service_builder::build_package(&package)
            .context("building the AgentIDE Service SDK package")?;
        build.artifacts.write(&root.join("generated/service"))?;
        println!("generated/service");
        return Ok(());
    }
    if command == "generate-realizations" {
        let ir = compile_ess(root)?;
        let target = root.join("docs/running-modes.md");
        std::fs::write(&target, render_realizations(root, &ir)?)?;
        println!("{}", target.display());
        return Ok(());
    }
    if command != "gate" {
        bail!(
            "usage: cargo xtask <gate|generate-hosted-contracts|generate-realizations|generate-service|generate-surface-profile>"
        );
    }

    validate_contracts(root)?;
    validate_hosted_contracts(root)?;
    let ir = validate_ess(root)?;
    validate_realizations(root, &ir)?;
    validate_generated_ess(root)?;
    validate_generated_service(root)?;
    validate_fixtures(root)?;
    run(root, "aep", &["artifact", "validate"])?;
    run(root, "cargo", &["fmt", "--all", "--", "--check"])?;
    run(root, "cargo", &["check", "--workspace", "--locked"])?;
    run(root, "cargo", &["test", "--workspace", "--locked"])?;
    run(
        root,
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run(&root.join("web"), "npm", &["ci", "--ignore-scripts"])?;
    run(&root.join("web"), "npm", &["run", "check"])?;
    run(&root.join("web"), "npm", &["run", "build"])?;
    run(
        root,
        "git",
        &["diff", "--exit-code", "--", "web/dist", "generated/ess"],
    )?;
    println!("AgentIDE gate passed");
    Ok(())
}

fn validate_contracts(root: &Path) -> Result<()> {
    let profile = agentide_contracts::IntentProfile::embedded()?;
    let bindings = agentide_contracts::BindingConfig::embedded()?;
    bindings.validate_against(&profile)?;
    let surface = agentide_contracts::SurfaceProfile::embedded()?;
    let expected = render_web_surface_profile()?;
    let generated = root.join("web/src/generated/surface-profile.ts");
    let observed = std::fs::read_to_string(&generated).with_context(|| {
        format!(
            "reading {}; regenerate it with `cargo xtask generate-surface-profile`",
            generated.display()
        )
    })?;
    if observed != expected {
        bail!(
            "web surface profile has drifted; regenerate it with `cargo xtask generate-surface-profile`"
        );
    }
    if surface.viewport(180, 50).id != "wide" {
        bail!("released surface profile has no wide workbench viewport");
    }
    Ok(())
}

fn render_web_surface_profile() -> Result<String> {
    let profile = agentide_contracts::SurfaceProfile::embedded()?;
    let json = serde_json::to_string_pretty(&profile)?;
    Ok(format!(
        "// Generated by `cargo xtask generate-surface-profile`; do not edit.\nexport const surfaceProfile = {json} as const;\n\nexport const surfaceProfileFormat = surfaceProfile.format;\n"
    ))
}

fn write_hosted_contracts(root: &Path) -> Result<()> {
    let (schemas, fixtures) = hosted_contract_documents()?;
    write_documents(&root.join("contracts/schemas/hosted"), &schemas)?;
    write_documents(&root.join("contracts/fixtures/hosted"), &fixtures)?;
    Ok(())
}

fn write_documents(root: &Path, documents: &GeneratedDocuments) -> Result<()> {
    if root.exists() {
        std::fs::remove_dir_all(root)?;
    }
    std::fs::create_dir_all(root)?;
    for (path, bytes) in documents {
        let target = root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(target, bytes)?;
    }
    Ok(())
}

fn validate_hosted_contracts(root: &Path) -> Result<()> {
    let (expected_schemas, expected_fixtures) = hosted_contract_documents()?;
    let observed_schemas = read_tree(&root.join("contracts/schemas/hosted"))?;
    let observed_fixtures = read_tree(&root.join("contracts/fixtures/hosted"))?;
    if expected_schemas != observed_schemas || expected_fixtures != observed_fixtures {
        bail!(
            "hosted contract schemas or golden vectors drifted; regenerate with `cargo xtask generate-hosted-contracts`"
        );
    }

    for (path, schema_bytes) in &observed_schemas {
        let schema: serde_json::Value = serde_json::from_slice(schema_bytes)
            .with_context(|| format!("parsing hosted schema {}", path.display()))?;
        jsonschema::meta::validate(&schema)
            .map_err(|error| anyhow!("invalid hosted schema {}: {error}", path.display()))?;
        let fixture_path = path.with_extension("json");
        let fixture_bytes = observed_fixtures.get(&fixture_path).ok_or_else(|| {
            anyhow!(
                "hosted schema {} has no matching golden vector",
                path.display()
            )
        })?;
        let fixture: serde_json::Value = serde_json::from_slice(fixture_bytes)
            .with_context(|| format!("parsing hosted fixture {}", fixture_path.display()))?;
        let validator = jsonschema::validator_for(&schema)
            .with_context(|| format!("compiling hosted schema {}", path.display()))?;
        if let Err(error) = validator.validate(&fixture) {
            bail!(
                "hosted fixture {} does not conform to {}: {error}",
                fixture_path.display(),
                path.display()
            );
        }
        validate_hosted_fixture(&fixture_path, &fixture)?;
    }
    Ok(())
}

fn hosted_contract_documents() -> Result<HostedContractDocuments> {
    let mut schemas = BTreeMap::new();
    let mut fixtures = BTreeMap::new();
    macro_rules! contract {
        ($name:literal, $type:ty, $fixture:expr) => {{
            let path = PathBuf::from(concat!($name, ".json"));
            schemas.insert(
                path.clone(),
                render_hosted_schema::<$type>(concat!(
                    "https://beyond10x.github.io/agentide/contracts/hosted/",
                    $name,
                    ".json"
                ))?,
            );
            let fixture: $type = $fixture;
            fixtures.insert(path, render_json(&fixture)?);
        }};
    }

    let actor = sample_actor()?;
    let context = sample_context(actor.clone())?;
    let inventory = sample_inventory()?;
    let actor_view = ActorView {
        format: "agentide.actor-view/2".into(),
        actor: actor.clone(),
        workbench: ActorWorkbench {
            panes: vec![WorkbenchPane {
                id: "pane:editor:one".into(),
                kind: "editor".into(),
                reference: Some("src/lib.rs".into()),
            }],
            tabs: vec!["src/lib.rs".into()],
            focused_pane: Some("pane:editor:one".into()),
            focused_file: Some("src/lib.rs".into()),
            cursors: BTreeMap::new(),
            selected_terminal: None,
            dirty_paths: vec![],
        },
        coordination: CoordinationRevision {
            revision: 11,
            digest: "c".repeat(64),
        },
        context: context.clone(),
        inventory: inventory.clone(),
        withheld: vec![],
    };
    actor_view.validate().map_err(anyhow::Error::msg)?;

    contract!("actor-context-v2.schema", ActorContext, actor);
    contract!(
        "context-selection-draft-v1.schema",
        ContextSelectionDraft,
        sample_selection_draft()?
    );
    contract!("context-pack-v2.schema", ContextPack, context);
    contract!("intent-inventory-v2.schema", IntentInventory, inventory);
    contract!("actor-view-v2.schema", ActorView, actor_view);
    contract!("authority-grant-v2.schema", AuthorityGrant, sample_grant()?);
    contract!("diff-projection-v2.schema", DiffProjection, sample_diff()?);
    contract!("file-projection-v2.schema", FileProjection, sample_file()?);
    contract!(
        "terminal-profile-v2.schema",
        TerminalProfile,
        sample_terminal_profile()?
    );
    contract!(
        "terminal-session-v2.schema",
        TerminalSession,
        sample_terminal_session(sample_actor()?)?
    );
    contract!(
        "terminal-control-v1.schema",
        TerminalControlFrame,
        sample_terminal_control()?
    );
    contract!(
        "terminal-event-v1.schema",
        TerminalEvent,
        TerminalEvent::Detached {
            terminal_id: "terminal:one".into()
        }
    );
    contract!(
        "terminal-server-frame-v1.schema",
        TerminalServerFrame,
        sample_terminal_server_frame()?
    );
    contract!("tree-projection-v2.schema", TreeProjection, sample_tree()?);
    Ok((schemas, fixtures))
}

fn render_hosted_schema<T: schemars::JsonSchema>(id: &str) -> Result<Vec<u8>> {
    let mut schema = serde_json::to_value(schemars::schema_for!(T))?;
    schema["$id"] = serde_json::Value::String(id.into());
    harden_schema(&mut schema);
    render_json(&schema)
}

fn harden_schema(schema: &mut serde_json::Value) {
    if let Some(object) = schema.as_object_mut() {
        let discriminator = object
            .get("title")
            .and_then(serde_json::Value::as_str)
            .and_then(format_for_schema_title);
        if let (Some(discriminator), Some(properties)) = (
            discriminator,
            object
                .get_mut("properties")
                .and_then(serde_json::Value::as_object_mut),
        ) {
            properties.insert(
                "format".into(),
                serde_json::json!({"type":"string", "const":discriminator}),
            );
        }
        if let Some(properties) = object
            .get_mut("properties")
            .and_then(serde_json::Value::as_object_mut)
        {
            for (name, property) in properties {
                if name == "digest"
                    || name == "sha256"
                    || name.ends_with("_sha256")
                    || name == "working_changes"
                {
                    constrain_string_pattern(property, "^[0-9a-f]{64}$");
                }
                if matches!(
                    name.as_str(),
                    "revision" | "line" | "column" | "rows" | "columns"
                ) {
                    constrain_integer_minimum(property, 1);
                }
            }
        }
        for value in object.values_mut() {
            harden_schema(value);
        }
    } else if let Some(values) = schema.as_array_mut() {
        for value in values {
            harden_schema(value);
        }
    }
}

fn format_for_schema_title(title: &str) -> Option<&'static str> {
    match title {
        "ActorContext" => Some("agentide.actor-context/2"),
        "AttachmentProvenance" => Some("agentide.attachment-provenance/1"),
        "ContextSelectionDraft" => Some("agentide.context-selection-draft/1"),
        "ContextSelection" => Some("agentide.context-selection/1"),
        "ContextPack" => Some("agentide.context-pack/2"),
        "IntentInventory" => Some("agentide.intent-inventory/2"),
        "ActorView" => Some("agentide.actor-view/2"),
        "AuthorityGrant" => Some("agentide.authority-grant/2"),
        "DiffProjection" => Some("agentide.diff-projection/2"),
        "FileRevision" => Some("agentide.file-revision/2"),
        "TerminalProfile" => Some("agentide.terminal-profile/2"),
        "TerminalSession" => Some("agentide.terminal-session/2"),
        "TerminalControlFrame" => Some("agentide.terminal-control/1"),
        "TerminalServerFrame" => Some("agentide.terminal-server-frame/1"),
        "TreeProjection" => Some("agentide.tree-projection/2"),
        _ => None,
    }
}

fn constrain_string_pattern(schema: &mut serde_json::Value, pattern: &str) {
    if let Some(object) = schema.as_object_mut() {
        if object.get("type").and_then(serde_json::Value::as_str) == Some("string") {
            object.insert("pattern".into(), serde_json::Value::String(pattern.into()));
        }
        for value in object.values_mut() {
            constrain_string_pattern(value, pattern);
        }
    } else if let Some(values) = schema.as_array_mut() {
        for value in values {
            constrain_string_pattern(value, pattern);
        }
    }
}

fn constrain_integer_minimum(schema: &mut serde_json::Value, minimum: u64) {
    if let Some(object) = schema.as_object_mut() {
        if object.get("type").and_then(serde_json::Value::as_str) == Some("integer") {
            object.insert("minimum".into(), serde_json::Value::from(minimum));
        }
        for value in object.values_mut() {
            constrain_integer_minimum(value, minimum);
        }
    } else if let Some(values) = schema.as_array_mut() {
        for value in values {
            constrain_integer_minimum(value, minimum);
        }
    }
}

fn render_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn validate_hosted_fixture(path: &Path, value: &serde_json::Value) -> Result<()> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("hosted fixture has no UTF-8 name"))?;
    match name {
        "actor-context-v2.schema.json" => serde_json::from_value::<ActorContext>(value.clone())?
            .validate()
            .map_err(anyhow::Error::msg),
        "context-selection-draft-v1.schema.json" => {
            serde_json::from_value::<ContextSelectionDraft>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "context-pack-v2.schema.json" => serde_json::from_value::<ContextPack>(value.clone())?
            .validate()
            .map_err(anyhow::Error::msg),
        "intent-inventory-v2.schema.json" => {
            serde_json::from_value::<IntentInventory>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "actor-view-v2.schema.json" => serde_json::from_value::<ActorView>(value.clone())?
            .validate()
            .map_err(anyhow::Error::msg),
        "authority-grant-v2.schema.json" => {
            serde_json::from_value::<AuthorityGrant>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "diff-projection-v2.schema.json" => {
            serde_json::from_value::<DiffProjection>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "file-projection-v2.schema.json" => {
            serde_json::from_value::<FileProjection>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "terminal-profile-v2.schema.json" => {
            serde_json::from_value::<TerminalProfile>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "terminal-session-v2.schema.json" => {
            serde_json::from_value::<TerminalSession>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "terminal-control-v1.schema.json" => {
            serde_json::from_value::<TerminalControlFrame>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "terminal-event-v1.schema.json" => serde_json::from_value::<TerminalEvent>(value.clone())?
            .validate()
            .map_err(anyhow::Error::msg),
        "terminal-server-frame-v1.schema.json" => {
            serde_json::from_value::<TerminalServerFrame>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        "tree-projection-v2.schema.json" => {
            serde_json::from_value::<TreeProjection>(value.clone())?
                .validate()
                .map_err(anyhow::Error::msg)
        }
        _ => bail!("unknown hosted fixture {}", path.display()),
    }
}

fn sample_actor() -> Result<ActorContext> {
    ActorContext::new(ActorKind::Human, "user:example").map_err(anyhow::Error::msg)
}

fn sample_selection_draft() -> Result<ContextSelectionDraft> {
    ContextSelectionDraft::new(
        "selection:one",
        SelectionKind::Editor,
        "src/lib.rs",
        Some(1),
        Some(1),
        "pub fn example() {}\n",
    )
    .map_err(anyhow::Error::msg)
}

fn sample_selection(actor: ActorContext) -> Result<ContextSelection> {
    ContextSelection::new(
        "selection:one",
        SelectionKind::Editor,
        "src/lib.rs",
        Some(1),
        Some(1),
        "pub fn example() {}\n",
        AttachmentProvenance {
            format: "agentide.attachment-provenance/1".into(),
            actor,
            source: "workspace".into(),
            source_revision: "commit:0123456789abcdef".into(),
            observed_at: "2026-09-03T12:00:00Z".into(),
        },
    )
    .map_err(anyhow::Error::msg)
}

fn sample_context(actor: ActorContext) -> Result<ContextPack> {
    let mut context = ContextPack {
        format: "agentide.context-pack/2".into(),
        objective: "Review the hosted protocol".into(),
        source_revision: "commit:0123456789abcdef".into(),
        working_changes: Some("b".repeat(64)),
        pins: vec![sample_selection(actor)?],
        focused_selections: vec![],
        open_files: vec![],
        active_diff: Some(ChangeSelector::Workspace),
        terminals: vec![],
        processes: vec![],
        agent_lanes: vec![],
        approvals: vec![],
        evidence: vec![ContextRecord {
            id: "evidence:one".into(),
            kind: "test_result".into(),
            state: Some("passed".into()),
            summary: "Hosted protocol conformance passed".into(),
            sha256: Some("d".repeat(64)),
            observed_at: Some("2026-09-03T12:00:00Z".into()),
        }],
        recent_activity: vec![],
        revision: 7,
        digest: String::new(),
    };
    context.seal().map_err(anyhow::Error::msg)?;
    Ok(context)
}

fn sample_inventory() -> Result<IntentInventory> {
    IntentInventory::new(
        5,
        vec![AvailableIntent {
            intent: IntentDefinition {
                name: "code_edit".into(),
                command: "agentide.coding.EditCode".into(),
                audiences: vec![Audience::Human, Audience::Agent],
                exposure: None,
                port: "workspace".into(),
                effect: Effect::Mutate,
                risk: Risk::Medium,
                approval: Approval::Required,
                subjects: vec!["path".into()],
            },
            authorization: AuthorizationPath::ExplicitHumanAction,
        }],
    )
    .map_err(anyhow::Error::msg)
}

fn sample_grant() -> Result<AuthorityGrant> {
    let grant = AuthorityGrant {
        format: "agentide.authority-grant/2".into(),
        id: "grant:one".into(),
        session_id: "session:one".into(),
        grantee: "agent:one".into(),
        allowed_intents: vec!["code_edit".into()],
        path_prefixes: vec!["src".into()],
        maximum_risk: Risk::Medium,
        expires_at: Some("2026-09-03T13:00:00Z".into()),
        revision: 3,
        revoked: false,
    };
    grant.validate().map_err(anyhow::Error::msg)?;
    Ok(grant)
}

fn sample_diff() -> Result<DiffProjection> {
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
                heading: Some("example".into()),
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
    projection.seal().map_err(anyhow::Error::msg)?;
    Ok(projection)
}

fn sample_file() -> Result<FileProjection> {
    let file = FileProjection {
        revision: FileRevision {
            format: "agentide.file-revision/2".into(),
            path: "src/lib.rs".into(),
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            size: 0,
            language: "rust".into(),
            state: FileModificationState::Unchanged,
            complete: true,
        },
        content: Some(String::new()),
        read_only: false,
        reason: None,
    };
    file.validate().map_err(anyhow::Error::msg)?;
    Ok(file)
}

fn sample_terminal_profile() -> Result<TerminalProfile> {
    let profile = TerminalProfile {
        format: "agentide.terminal-profile/2".into(),
        id: "review".into(),
        runtime: "substrate-runtime:review@sha256:example".into(),
        shell: vec!["/bin/sh".into()],
        working_directory: String::new(),
        workspace_access: TerminalWorkspaceAccess::ReadWrite,
        environment: vec!["TERM".into()],
        network: "none".into(),
        cpu_millis: 1000,
        memory_bytes: 268_435_456,
        process_limit: 128,
    };
    profile.validate().map_err(anyhow::Error::msg)?;
    Ok(profile)
}

fn sample_terminal_session(actor: ActorContext) -> Result<TerminalSession> {
    let session = TerminalSession {
        format: "agentide.terminal-session/2".into(),
        id: "terminal:one".into(),
        session_id: "session:one".into(),
        profile: "review".into(),
        actor,
        process_id: "process:one".into(),
        working_directory: String::new(),
        network: "none".into(),
        state: TerminalState::Running,
        output_sequence: 2,
        exit_code: None,
    };
    session.validate().map_err(anyhow::Error::msg)?;
    Ok(session)
}

fn sample_terminal_control() -> Result<TerminalControlFrame> {
    let frame = TerminalControlFrame {
        format: "agentide.terminal-control/1".into(),
        request_id: "request:one".into(),
        terminal_id: "terminal:one".into(),
        control: TerminalControl::Resize {
            columns: 120,
            rows: 40,
        },
    };
    frame.validate().map_err(anyhow::Error::msg)?;
    Ok(frame)
}

fn sample_terminal_server_frame() -> Result<TerminalServerFrame> {
    let frame = TerminalServerFrame {
        format: "agentide.terminal-server-frame/1".into(),
        request_id: Some("request:one".into()),
        replay: Some(TerminalReplayBounds {
            requested_after: 0,
            available_after: 0,
            latest: 2,
            complete: true,
        }),
        event: TerminalEvent::Attached {
            terminal_id: "terminal:one".into(),
            after: 0,
        },
    };
    frame.validate().map_err(anyhow::Error::msg)?;
    Ok(frame)
}

fn sample_tree() -> Result<TreeProjection> {
    let tree = TreeProjection {
        format: "agentide.tree-projection/2".into(),
        entries: vec![TreeEntry {
            path: "src/lib.rs".into(),
            kind: TreeEntryKind::File,
            size: Some(0),
            sha256: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into()),
        }],
        truncated: false,
        omitted: None,
        next_cursor: None,
    };
    tree.validate().map_err(anyhow::Error::msg)?;
    Ok(tree)
}

fn compile_ess(root: &Path) -> Result<ess_compiler::EssIr> {
    let directory = root.join("spec/agentide");
    let mut pending = vec![directory.clone()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path)
            .with_context(|| format!("reading ESS directory {}", path.display()))?
        {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut parsed = Vec::new();
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    for path in files {
        let label = path.strip_prefix(&directory)?.display().to_string();
        let text = std::fs::read_to_string(&path)?;
        let raw = RawSpecFile::parse(&text)
            .map_err(|error| anyhow!("ESS {label} did not parse: {error}"))?;
        sources.insert(label.clone(), text);
        labels.push(label.clone());
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .map_err(|errors| anyhow!("ESS did not validate:\n{errors}"))?;
    let ir = ess_compiler::resolve::compile_locating(&specification, &sources, &labels)
        .map_err(|errors| anyhow!("ESS did not resolve:\n{errors}"))?;
    Ok(ir)
}

fn validate_ess(root: &Path) -> Result<ess_compiler::EssIr> {
    let ir = compile_ess(root)?;
    let profile = agentide_contracts::IntentProfile::embedded()?;
    let semantic_commands: BTreeSet<_> = ir.commands().keys().map(ToString::to_string).collect();
    let profiled_commands: BTreeSet<_> = profile
        .intents
        .iter()
        .map(|intent| intent.command.clone())
        .collect();
    if semantic_commands != profiled_commands {
        let missing_profile: Vec<_> = semantic_commands.difference(&profiled_commands).collect();
        let missing_ess: Vec<_> = profiled_commands.difference(&semantic_commands).collect();
        bail!(
            "ESS/profile command drift: absent from profile {missing_profile:?}; absent from ESS {missing_ess:?}"
        );
    }
    let expected = std::fs::read_to_string(root.join("generated/ess/ir.json"))
        .context("reading generated/ess/ir.json; regenerate it with ESS")?;
    if expected != ir.to_canonical_json() {
        bail!("generated/ess/ir.json has drifted; regenerate it with the pinned ESS revision");
    }
    Ok(ir)
}

fn realization_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut files = std::fs::read_dir(root.join("realizations"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "yaml")
    });
    files.sort();
    Ok(files)
}

fn render_realizations(root: &Path, ir: &ess_compiler::EssIr) -> Result<String> {
    let mut output = String::from(
        "# Running AgentIDE\n\nThese modes are generated from two `ess-realization/1` declarations bound to the same exact AgentIDE ESS. The declarations, rather than this page, are the authority for implementation artifacts, semantic surfaces, attachment boundaries, availability, and runtime requirements.\n\n",
    );
    for path in realization_files(root)? {
        let text = std::fs::read_to_string(&path)?;
        let specification = ess_realization::RealizationSpec::from_yaml(&text)
            .with_context(|| format!("reading realization {}", path.display()))?;
        let realization = ess_realization::compile(&specification, ir)
            .map_err(|errors| anyhow!("realization {} was refused:\n{errors}", path.display()))?;
        let generated = realization.to_markdown();
        let body = generated
            .strip_prefix("# Running modes\n\n")
            .ok_or_else(|| anyhow!("ESS realization Markdown has an unexpected heading"))?;
        output.push_str("## Realization: `");
        output.push_str(realization.id().as_str());
        output.push_str("`\n\n");
        for line in body.lines() {
            if let Some(heading) = line.strip_prefix("## ") {
                output.push_str("### ");
                output.push_str(heading);
            } else {
                output.push_str(line);
            }
            output.push('\n');
        }
        output.push('\n');
    }
    let content_len = output.trim_end().len();
    output.truncate(content_len);
    output.push('\n');
    Ok(output)
}

fn validate_realizations(root: &Path, ir: &ess_compiler::EssIr) -> Result<()> {
    let expected = render_realizations(root, ir)?;
    let path = root.join("docs/running-modes.md");
    let observed = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "reading {}; regenerate it with `cargo xtask generate-realizations`",
            path.display()
        )
    })?;
    if observed != expected {
        bail!(
            "realization documentation has drifted; regenerate it with `cargo xtask generate-realizations`"
        );
    }
    Ok(())
}

fn validate_fixtures(root: &Path) -> Result<()> {
    let directory = root.join("fixtures/sessions");
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&path)?;
        for forbidden in [
            "/home/",
            "/Users/",
            "BEGIN PRIVATE",
            "access_token",
            "reasoning",
            "prompt",
        ] {
            if text
                .to_ascii_lowercase()
                .contains(&forbidden.to_ascii_lowercase())
            {
                bail!(
                    "fixture {} contains forbidden marker `{forbidden}`",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn validate_generated_ess(root: &Path) -> Result<()> {
    let temporary = tempfile::tempdir()?;
    let out = temporary.path();
    let specification = root.join("spec/agentide");
    let ess = std::env::var_os("AGENTIDE_ESS_BIN").unwrap_or_else(|| "ess".into());
    let compile = Command::new(&ess)
        .args(["compile", "--path"])
        .arg(&specification)
        .args(["--out"])
        .arg(out.join("ir.json"))
        .status()
        .with_context(
            || "starting the pinned ESS CLI; install tag 0.9.2 or set AGENTIDE_ESS_BIN",
        )?;
    if !compile.success() {
        bail!("ESS canonical IR generation failed with {compile}");
    }
    for kind in ["docs", "schema", "openapi", "asyncapi"] {
        let status = Command::new(&ess)
            .args(["generate", "--path"])
            .arg(&specification)
            .args(["--kind", kind, "--out"])
            .arg(out)
            .status()
            .with_context(|| format!("starting ESS {kind} generation"))?;
        if !status.success() {
            bail!("ESS {kind} generation failed with {status}");
        }
    }
    let expected = read_tree(&root.join("generated/ess"))?;
    let observed = read_tree(out)?;
    if expected != observed {
        let missing: Vec<_> = observed
            .keys()
            .filter(|path| !expected.contains_key(*path))
            .collect();
        let stale: Vec<_> = expected
            .keys()
            .filter(|path| !observed.contains_key(*path))
            .collect();
        let changed: Vec<_> = observed
            .keys()
            .filter(|path| {
                expected
                    .get(*path)
                    .is_some_and(|bytes| bytes != observed.get(*path).expect("present"))
            })
            .collect();
        bail!("generated ESS drift: missing {missing:?}; stale {stale:?}; changed {changed:?}");
    }
    Ok(())
}

fn validate_generated_service(root: &Path) -> Result<()> {
    let package = service_builder::package::ServicePackage::read(&root.join("service.yaml"))
        .context("loading the AgentIDE Service SDK package")?;
    let build = service_builder::build_package(&package)
        .context("building the AgentIDE Service SDK package")?;
    let generated = root.join("generated/service");
    let drift = build.artifacts.check(&generated)?;
    if !drift.is_empty() {
        bail!("generated Service SDK package has drifted; regenerate generated/service: {drift:?}");
    }

    let temporary = tempfile::tempdir()?;
    let disposable = temporary.path().join("agentide-generated-service");
    copy_tree(&generated, &disposable)?;
    let manifest = disposable.join("rust/Cargo.toml");
    let target = root.join("target/generated-service");
    println!("+ cargo test --manifest-path {}", manifest.display());
    let status = Command::new("cargo")
        .args(["test", "--manifest-path"])
        .arg(&manifest)
        .env("CARGO_TARGET_DIR", target)
        .status()
        .context("checking the generated AgentIDE Service SDK package")?;
    if !status.success() {
        bail!("generated AgentIDE Service SDK package tests failed with {status}");
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            bail!(
                "generated Service SDK package contains symlink {}",
                source_path.display()
            );
        }
        if file_type.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            std::fs::copy(&source_path, &destination_path)?;
        } else {
            bail!(
                "generated Service SDK package contains unsupported entry {}",
                source_path.display()
            );
        }
    }
    Ok(())
}

fn read_tree(root: &Path) -> Result<std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>> {
    let mut files = std::collections::BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.is_file() {
                files.insert(path.strip_prefix(root)?.to_path_buf(), std::fs::read(path)?);
            }
        }
    }
    Ok(files)
}

fn run(directory: &Path, program: &str, arguments: &[&str]) -> Result<()> {
    println!("+ {program} {}", arguments.join(" "));
    let status = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .status()
        .with_context(|| format!("starting `{program}`"))?;
    if !status.success() {
        bail!("`{program} {}` failed with {status}", arguments.join(" "));
    }
    Ok(())
}
