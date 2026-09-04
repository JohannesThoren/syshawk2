use crate::{state::AppState, system_auth};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::Duration as TimeDuration;

use crate::session::SESSION_COOKIE;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub username: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<MeResponse>), StatusCode> {
    let username = req.username.trim().to_string();
    if username.is_empty() || req.password.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Run blocking PAM/NSS calls off the async runtime.
    let check_username = username.clone();
    let password = req.password.clone();
    let group = state.required_group.clone();
    let authorized = tokio::task::spawn_blocking(move || {
        system_auth::verify_password(&check_username, &password)
            && system_auth::is_member_of(&check_username, &group)
    })
    .await
    .unwrap_or(false);

    if !authorized {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = state.sessions.create(&username);
    let cookie = Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(TimeDuration::hours(12))
        .build();

    Ok((jar.add(cookie), Json(MeResponse { username })))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> (CookieJar, StatusCode) {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        state.sessions.revoke(cookie.value());
    }
    (jar.remove(SESSION_COOKIE), StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Json<MeResponse>, StatusCode> {
    let token = jar.get(SESSION_COOKIE).map(|c| c.value().to_string());
    let username = token
        .and_then(|t| state.sessions.validate(&t))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(MeResponse { username }))
}
