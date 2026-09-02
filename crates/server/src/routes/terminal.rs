use crate::{auth::hash_token, db, state::AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use shawk_common::ProbeControlMsg;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TokenQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct TerminalConnectQuery {
    pub token: String,
    pub session_id: Uuid,
}

/// The probe dials this once at startup and keeps it open; the server uses
/// it to push `OpenTerminal` requests down to that probe on demand.
pub async fn probe_control(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(q): Query<TokenQuery>,
) -> impl IntoResponse {
    let token_hash = hash_token(&q.token);
    let probe = match db::find_probe_by_token_hash(&state.pool, &token_hash).await {
        Ok(Some(p)) => p,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    ws.on_upgrade(move |socket| handle_probe_control(socket, state, probe.id))
        .into_response()
}

async fn handle_probe_control(mut socket: WebSocket, state: Arc<AppState>, probe_id: Uuid) {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    state.probe_control.write().unwrap().insert(probe_id, tx);
    tracing::info!(%probe_id, "probe control channel connected");

    loop {
        tokio::select! {
            outgoing = rx.recv() => match outgoing {
                Some(text) => {
                    if socket.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
                None => break,
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(_)) => break,
                _ => {}
            },
        }
    }

    state.probe_control.write().unwrap().remove(&probe_id);
    tracing::info!(%probe_id, "probe control channel disconnected");
}

/// The probe dials this after receiving an `OpenTerminal` control message,
/// handing its raw PTY socket off to whichever dashboard client is waiting
/// on that session id.
pub async fn probe_terminal_connect(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(q): Query<TerminalConnectQuery>,
) -> impl IntoResponse {
    let token_hash = hash_token(&q.token);
    let found = db::find_probe_by_token_hash(&state.pool, &token_hash)
        .await
        .ok()
        .flatten();
    if found.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    ws.on_upgrade(move |socket| async move {
        let waiter = state
            .terminal_waiters
            .write()
            .unwrap()
            .remove(&q.session_id);
        match waiter {
            Some(sender) => {
                let _ = sender.send(socket);
            }
            None => {
                let _ = socket.close().await;
            }
        }
    })
    .into_response()
}

/// Dashboard-facing: opens a terminal on `probe_id`. Waits for the probe to
/// dial back with its PTY socket, then relays raw frames between the two
/// sockets untouched until either side closes.
pub async fn dashboard_terminal(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(probe_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| relay_terminal(socket, state, probe_id))
}

async fn relay_terminal(mut dashboard_ws: WebSocket, state: Arc<AppState>, probe_id: Uuid) {
    let session_id = Uuid::new_v4();
    let (tx, rx) = oneshot::channel();
    state.terminal_waiters.write().unwrap().insert(session_id, tx);

    let control_tx = state.probe_control.read().unwrap().get(&probe_id).cloned();

    let Some(control_tx) = control_tx else {
        state.terminal_waiters.write().unwrap().remove(&session_id);
        let _ = dashboard_ws
            .send(Message::Text("probe is not connected".into()))
            .await;
        let _ = dashboard_ws.close().await;
        return;
    };

    let open_msg = ProbeControlMsg::OpenTerminal {
        session_id,
        cols: 80,
        rows: 24,
    };
    let Ok(json) = serde_json::to_string(&open_msg) else {
        return;
    };
    if control_tx.send(json).is_err() {
        state.terminal_waiters.write().unwrap().remove(&session_id);
        let _ = dashboard_ws.close().await;
        return;
    }

    let probe_ws = match tokio::time::timeout(Duration::from_secs(10), rx).await {
        Ok(Ok(socket)) => socket,
        _ => {
            state.terminal_waiters.write().unwrap().remove(&session_id);
            let _ = dashboard_ws
                .send(Message::Text("terminal session timed out".into()))
                .await;
            let _ = dashboard_ws.close().await;
            return;
        }
    };

    let (mut dash_tx, mut dash_rx) = dashboard_ws.split();
    let (mut probe_tx, mut probe_rx) = probe_ws.split();

    let to_probe = async {
        while let Some(Ok(msg)) = dash_rx.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
            if probe_tx.send(msg).await.is_err() {
                break;
            }
        }
    };
    let to_dashboard = async {
        while let Some(Ok(msg)) = probe_rx.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
            if dash_tx.send(msg).await.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = to_probe => {}
        _ = to_dashboard => {}
    }
    tracing::info!(%probe_id, %session_id, "terminal session ended");
}
