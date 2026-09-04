use crate::{auth::hash_token, db, state::AppState};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use shawk_common::{IngestRequest, IngestResponse, WsEvent};
use std::sync::Arc;

pub async fn ingest(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, StatusCode> {
    let token = bearer_token(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let token_hash = hash_token(&token);

    let probe = db::find_probe_by_token_hash(&state.pool, &token_hash)
        .await
        .map_err(internal_error)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let now = Utc::now();
    db::insert_snapshot(&state.pool, probe.id, &req.snapshot)
        .await
        .map_err(internal_error)?;
    db::touch_last_seen(&state.pool, probe.id, now)
        .await
        .map_err(internal_error)?;

    // Best-effort broadcast to any connected dashboards; ignore if nobody's listening.
    let _ = state.events.send(WsEvent::Snapshot {
        probe_id: probe.id,
        snapshot: req.snapshot,
    });

    Ok(Json(IngestResponse { accepted: true }))
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    let value = headers.get("authorization")?.to_str().ok()?;
    value.strip_prefix("Bearer ").map(|s| s.to_string())
}

fn internal_error(e: sqlx::Error) -> StatusCode {
    tracing::error!(error = %e, "database error");
    StatusCode::INTERNAL_SERVER_ERROR
}
