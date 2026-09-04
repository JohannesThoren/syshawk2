use crate::{auth, db, state::AppState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use shawk_common::{
    OFFLINE_THRESHOLD_SECS, ProbeStatus, ProbeSummary, RegisterProbeRequest, RegisterProbeResponse,
};
use std::sync::Arc;
use uuid::Uuid;

fn require_admin(state: &AppState, headers: &HeaderMap) -> Result<(), StatusCode> {
    let provided = headers
        .get("x-admin-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided == state.admin_token && !state.admin_token.is_empty() {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn status_from_last_seen(last_seen: Option<chrono::DateTime<Utc>>) -> ProbeStatus {
    match last_seen {
        None => ProbeStatus::Pending,
        Some(ts) => {
            if (Utc::now() - ts).num_seconds() <= OFFLINE_THRESHOLD_SECS {
                ProbeStatus::Online
            } else {
                ProbeStatus::Offline
            }
        }
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RegisterProbeRequest>,
) -> Result<Json<RegisterProbeResponse>, StatusCode> {
    require_admin(&state, &headers)?;

    let (token, token_hash) = auth::generate_token();
    let row = db::create_probe(&state.pool, &req.name, &token_hash)
        .await
        .map_err(internal_error)?;

    Ok(Json(RegisterProbeResponse {
        id: row.id,
        name: row.name,
        token,
    }))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProbeSummary>>, StatusCode> {
    let rows = db::list_probes(&state.pool).await.map_err(internal_error)?;
    let mut out = Vec::with_capacity(rows.len());

    for row in rows {
        let latest_row = db::latest_snapshot(&state.pool, row.id)
            .await
            .map_err(internal_error)?;
        let latest = match latest_row {
            Some(r) => Some(r.into_snapshot().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?),
            None => None,
        };
        out.push(ProbeSummary {
            id: row.id,
            name: row.name,
            hostname: latest.as_ref().map(|s| s.hostname.clone()),
            status: status_from_last_seen(row.last_seen),
            last_seen: row.last_seen,
            latest,
        });
    }

    Ok(Json(out))
}

pub async fn history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<shawk_common::Snapshot>>, StatusCode> {
    let since = Utc::now() - chrono::Duration::hours(1);
    let rows = db::history(&state.pool, id, since)
        .await
        .map_err(internal_error)?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(row.into_snapshot().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }
    Ok(Json(out))
}

fn internal_error(e: sqlx::Error) -> StatusCode {
    tracing::error!(error = %e, "database error");
    StatusCode::INTERNAL_SERVER_ERROR
}
