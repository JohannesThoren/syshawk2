use crate::terminal;
use futures_util::StreamExt;
use shawk_common::ProbeControlMsg;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

/// Keeps a persistent websocket open to the server so it can push
/// `OpenTerminal` requests down to us on demand. Reconnects with a fixed
/// backoff if the connection drops.
pub async fn run(ws_base: String, token: String) {
    loop {
        if let Err(e) = connect_once(&ws_base, &token).await {
            warn!(error = %e, "probe control connection lost, retrying");
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_once(ws_base: &str, token: &str) -> anyhow::Result<()> {
    let url = format!("{}/api/probe/control?token={}", ws_base, token);
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
    info!("probe control channel connected");
    let (_write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ProbeControlMsg>(&text) {
                    Ok(ProbeControlMsg::OpenTerminal {
                        session_id,
                        cols,
                        rows,
                    }) => {
                        let ws_base = ws_base.to_string();
                        let token = token.to_string();
                        tokio::spawn(async move {
                            if let Err(e) = terminal::open_terminal_session(
                                &ws_base, &token, session_id, cols, rows,
                            )
                            .await
                            {
                                error!(error = %e, "terminal session failed");
                            }
                        });
                    }
                    Err(e) => warn!(error = %e, "unrecognized control message"),
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                warn!(error = %e, "control channel error");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
