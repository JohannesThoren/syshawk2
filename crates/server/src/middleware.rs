use crate::{session::SESSION_COOKIE, state::AppState};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

/// Gates dashboard-facing routes behind a valid session cookie. Probe-facing
/// routes (ingest, control, terminal handoff) use their own bearer-token
/// auth and never pass through this layer.
pub async fn require_session(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if state.sessions.validate(&token).is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
