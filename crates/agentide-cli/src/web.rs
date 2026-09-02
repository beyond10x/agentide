//! Embedded browser projection over the same engine used by the CLI and TUI.

use std::net::SocketAddr;
use std::sync::Arc;

use agentide_core::{Engine, Refusal};
use agentide_substrate::SubstratePort;
use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(RustEmbed)]
#[folder = "../../web/dist/"]
struct Assets;

#[derive(Clone)]
struct AppState {
    engine: Arc<Engine<SubstratePort>>,
    session_id: String,
}

#[derive(Debug, Deserialize)]
struct IntentRequest {
    #[serde(default = "empty_object")]
    input: Value,
}

fn empty_object() -> Value {
    json!({})
}

/// Serves a session-scoped local workbench.
pub async fn serve(
    engine: Arc<Engine<SubstratePort>>,
    session_id: String,
    listen: &str,
) -> Result<()> {
    let address: SocketAddr = listen
        .parse()
        .with_context(|| format!("`{listen}` is not a numeric socket address"))?;
    if !address.ip().is_loopback() {
        anyhow::bail!("the embedded workbench serves loopback addresses only");
    }
    let state = AppState { engine, session_id };
    let app = Router::new()
        .route("/", get(index))
        .route("/assets/{*path}", get(asset))
        .route("/api/snapshot", get(snapshot))
        .route("/api/events", get(events))
        .route("/api/intents", get(intents))
        .route("/api/intents/{intent}/preview", post(preview))
        .route("/api/intents/{intent}/call", post(call))
        .route("/api/plans/{digest}/resume", post(resume))
        .route("/api/approvals/{digest}", post(grant))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("binding AgentIDE web surface to {listen}"))?;
    println!("AgentIDE: http://{listen}");
    axum::serve(listener, app).await.context("serving AgentIDE")
}

async fn index() -> Response {
    embedded("index.html")
}

async fn asset(Path(path): Path<String>) -> Response {
    embedded(&path)
}

fn embedded(path: &str) -> Response {
    Assets::get(path).map_or_else(
        || (StatusCode::NOT_FOUND, "asset not found").into_response(),
        |asset| {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], asset.data).into_response()
        },
    )
}

async fn snapshot(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let engine = Arc::clone(&state.engine);
    let session = state.session_id;
    let snapshot = blocking(move || engine.snapshot(&session)).await?;
    Ok(Json(serde_json::to_value(snapshot)?))
}

async fn events(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let engine = Arc::clone(&state.engine);
    let session = state.session_id;
    let events = blocking(move || engine.events(&session, 0, 1_000)).await?;
    Ok(Json(serde_json::to_value(events)?))
}

async fn intents(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::to_value(&state.engine.profile().intents)?))
}

async fn preview(
    State(state): State<AppState>,
    Path(intent): Path<String>,
    Json(request): Json<IntentRequest>,
) -> Result<Json<Value>, ApiError> {
    let engine = Arc::clone(&state.engine);
    let session = state.session_id;
    let plan = blocking(move || engine.preview(&session, &intent, request.input)).await?;
    Ok(Json(serde_json::to_value(plan)?))
}

async fn call(
    State(state): State<AppState>,
    Path(intent): Path<String>,
    Json(request): Json<IntentRequest>,
) -> Result<Json<Value>, ApiError> {
    let engine = Arc::clone(&state.engine);
    let session = state.session_id;
    Ok(Json(
        blocking(move || engine.call(&session, &intent, request.input)).await?,
    ))
}

async fn grant(
    State(state): State<AppState>,
    Path(digest): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let engine = Arc::clone(&state.engine);
    let session = state.session_id;
    let granted_digest = digest.clone();
    blocking(move || engine.grant(&session, &granted_digest)).await?;
    Ok(Json(json!({"status": "granted", "plan_digest": digest})))
}

async fn resume(
    State(state): State<AppState>,
    Path(digest): Path<String>,
    Json(request): Json<IntentRequest>,
) -> Result<Json<Value>, ApiError> {
    let engine = Arc::clone(&state.engine);
    let session = state.session_id;
    Ok(Json(
        blocking(move || engine.resume(&session, &digest, request.input)).await?,
    ))
}

async fn blocking<T, F>(operation: F) -> Result<T, ApiError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, Refusal> + Send + 'static,
{
    Ok(tokio::task::spawn_blocking(operation).await??)
}

struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let refusal = self.0.downcast_ref::<Refusal>();
        let status = refusal.map_or(StatusCode::INTERNAL_SERVER_ERROR, |_| StatusCode::CONFLICT);
        let value = refusal.map_or_else(
            || json!({"code": "agentide.failed", "message": self.0.to_string(), "retryable": false}),
            |error| json!({"code": error.code, "message": error.message, "retryable": error.retryable}),
        );
        (status, Json(value)).into_response()
    }
}
