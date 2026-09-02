//! Standalone bindings implemented only through Substrate's guarded boundaries.
#![allow(clippy::missing_errors_doc, clippy::doc_markdown)]

use std::path::Path;
use std::time::Duration;

use agentide_contracts::Binding;
use agentide_core::{IntentPort, Refusal};
use b10x_harness_substrate::{
    Backend, Embedded, ProcessWorkspaceAccess, Toolchain, process_workspace_access,
};
use b10x_harness_toolchain::Registry;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// The standalone implementation port.
#[derive(Debug)]
pub struct SubstratePort {
    backend: Embedded,
    workspace_id: String,
    toolchains: Vec<String>,
}

impl SubstratePort {
    /// Adopts an existing target workspace below its parent directory.
    pub fn adopt(workspace: &Path) -> Result<Self, Refusal> {
        let workspace = workspace
            .canonicalize()
            .map_err(|error| refused("workspace.unreadable", error))?;
        let parent = workspace.parent().ok_or_else(|| {
            Refusal::named("workspace.invalid", "workspace has no parent directory")
        })?;
        let name = workspace
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Refusal::named("workspace.invalid", "workspace name is not UTF-8"))?;
        let providers = Registry::builtins()
            .and_then(|registry| registry.resolve(&workspace, None))
            .map_err(|error| Refusal::named("toolchain.unavailable", error))?;
        let toolchains = providers
            .iter()
            .map(|provider| provider.name.clone())
            .collect();
        let toolchain = Toolchain::from_providers(providers)
            .map_err(|error| Refusal::named("toolchain.invalid", error))?;
        let backend = Embedded::open_with(parent, cgroup_root(), toolchain)
            .map_err(|error| refused("substrate.unavailable", error))?;
        let workspace_id = backend
            .workspace_adopt(name)
            .map_err(|error| refused("substrate.workspace_refused", error))?;
        Ok(Self {
            backend,
            workspace_id,
            toolchains,
        })
    }

    fn exec(&self, argv: &[String], writable: &[String]) -> Result<Value, Refusal> {
        if argv.is_empty() {
            return Err(Refusal::named("binding.invalid", "bound argv is empty"));
        }
        let access = process_workspace_access(writable)
            .map_err(|error| Refusal::named("binding.invalid", error))?;
        self.backend
            .exec(
                &self.workspace_id,
                argv,
                &access,
                Some(Duration::from_mins(15)),
            )
            .map_err(|error| refused("substrate.exec_refused", error))
    }
}

impl IntentPort for SubstratePort {
    fn invoke(&self, binding: &Binding, intent: &str, input: &Value) -> Result<Value, Refusal> {
        if binding.driver != "substrate" {
            return Err(Refusal::named(
                "binding.driver_unknown",
                format!("standalone port does not implement `{}`", binding.driver),
            ));
        }
        match binding.operation.as_str() {
            "file_read" => {
                let path = text(input, "path")?;
                let content = self
                    .backend
                    .file_read(&self.workspace_id, path)
                    .map_err(|error| refused("substrate.read_refused", error))?;
                Ok(json!({"path": path, "content": content, "sha256": digest(content.as_bytes())}))
            }
            "file_write" => {
                let path = text(input, "path")?;
                let content = text(input, "content")?;
                if let Some(expected) = input.get("expected_sha256").and_then(Value::as_str) {
                    let current = self
                        .backend
                        .file_read(&self.workspace_id, path)
                        .map_err(|error| refused("substrate.precondition_unreadable", error))?;
                    let actual = digest(current.as_bytes());
                    if actual != expected {
                        return Err(Refusal::named(
                            "workspace.precondition_failed",
                            format!("expected `{expected}`, observed `{actual}`"),
                        ));
                    }
                }
                let observation = self
                    .backend
                    .file_write(&self.workspace_id, path, content)
                    .map_err(|error| refused("substrate.write_refused", error))?;
                Ok(
                    json!({"path": path, "sha256": digest(content.as_bytes()), "observation": observation}),
                )
            }
            "search" => {
                let mut argv = vec!["rg".into(), "--json".into(), text(input, "pattern")?.into()];
                if let Some(paths) = input.get("paths").and_then(Value::as_array) {
                    for path in paths {
                        argv.push(
                            path.as_str()
                                .ok_or_else(|| {
                                    Refusal::named(
                                        "intent.input_invalid",
                                        "every search path must be a string",
                                    )
                                })?
                                .into(),
                        );
                    }
                }
                self.exec(&argv, &[])
            }
            "exec" => self.exec(&bound_argv(binding)?, &[]),
            "exec_profile" => {
                let profile = if intent == "code_verify" {
                    text(input, "level")?.to_ascii_lowercase()
                } else {
                    text(input, "profile")?.to_owned()
                };
                let profiles = binding
                    .options
                    .get("profiles")
                    .and_then(Value::as_object)
                    .ok_or_else(|| Refusal::named("binding.invalid", "profiles map is missing"))?;
                let argv = profiles.get(&profile).ok_or_else(|| {
                    Refusal::named(
                        "binding.profile_unavailable",
                        format!("semantic profile `{profile}` is not configured"),
                    )
                })?;
                let argv = string_array(argv, "profile argv")?;
                let writable = binding.options.get("writable_subtrees").map_or_else(
                    || Ok(Vec::new()),
                    |value| string_array(value, "writable_subtrees"),
                )?;
                self.exec(&argv, &writable)
            }
            operation => Err(Refusal::named(
                "binding.operation_unknown",
                format!("substrate operation `{operation}` is not implemented"),
            )),
        }
    }

    fn capabilities(&self) -> Value {
        match self.backend.machine() {
            Ok(facts) => json!({
                "driver": facts.driver,
                "driver_version": facts.driver_version,
                "facts": facts.facts,
                "workspace_id": self.workspace_id,
                "toolchains": self.toolchains,
            }),
            Err(error) => {
                json!({"error": error.to_string(), "workspace_id": self.workspace_id, "toolchains": self.toolchains})
            }
        }
    }
}

fn bound_argv(binding: &Binding) -> Result<Vec<String>, Refusal> {
    let argv = binding
        .options
        .get("argv")
        .ok_or_else(|| Refusal::named("binding.invalid", "bound argv is missing"))?;
    string_array(argv, "argv")
}

fn string_array(value: &Value, name: &str) -> Result<Vec<String>, Refusal> {
    value
        .as_array()
        .ok_or_else(|| Refusal::named("binding.invalid", format!("`{name}` must be an array")))?
        .iter()
        .map(|part| {
            part.as_str().map(Into::into).ok_or_else(|| {
                Refusal::named(
                    "binding.invalid",
                    format!("every `{name}` item must be a string"),
                )
            })
        })
        .collect()
}

fn text<'a>(input: &'a Value, name: &str) -> Result<&'a str, Refusal> {
    input
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| Refusal::named("intent.input_invalid", format!("`{name}` must be a string")))
}

fn cgroup_root() -> Option<std::path::PathBuf> {
    std::env::var_os("AGENTIDE_CGROUP_ROOT").map(Into::into)
}

fn refused(code: &str, error: impl std::fmt::Display) -> Refusal {
    Refusal::named(code, error.to_string())
}

fn digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

/// Read-only access constant exported for embedding tests and adapters.
#[must_use]
pub const fn read_only_access() -> ProcessWorkspaceAccess {
    ProcessWorkspaceAccess::ReadOnly
}
