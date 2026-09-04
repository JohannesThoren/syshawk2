use futures_util::{SinkExt, StreamExt};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use shawk_common::TerminalControlMsg;
use std::io::{Read, Write};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

/// Spawns an interactive shell in a PTY and connects it to the server's
/// terminal-handoff socket for `session_id`, relaying bytes both ways until
/// either side closes.
pub async fn open_terminal_session(
    ws_base: &str,
    token: &str,
    session_id: Uuid,
    cols: u16,
    rows: u16,
    shell: &str,
) -> anyhow::Result<()> {
    let url = format!(
        "{}/api/probe/terminal-connect?token={}&session_id={}",
        ws_base, token, session_id
    );
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
    let (mut ws_write, mut ws_read) = ws_stream.split();

    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(shell);
    // portable-pty falls back to $HOME for the child's cwd if none is set
    // explicitly, without checking it actually exists. Under systemd,
    // $HOME is set from the service account's passwd entry - and a
    // dedicated service account created with --no-create-home has no such
    // directory, which fails the spawn with ENOENT. Fall back to this
    // process's own cwd (WorkingDirectory=/opt/shawk under the systemd
    // unit) instead, which is guaranteed to exist.
    let fallback_cwd = std::env::current_dir().unwrap_or_else(|_| "/".into());
    cmd.cwd(fallback_cwd);
    let mut child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut pty_reader = pair.master.try_clone_reader()?;
    let mut pty_writer = pair.master.take_writer()?;
    let master = pair.master;

    // Blocking PTY reads happen on a dedicated thread; forward bytes to the
    // websocket via a channel so the async side stays non-blocking.
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match pty_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if out_tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    loop {
        tokio::select! {
            data = out_rx.recv() => {
                match data {
                    Some(bytes) => {
                        if ws_write.send(Message::Binary(bytes)).await.is_err() {
                            break;
                        }
                    }
                    None => break, // shell exited
                }
            }
            msg = ws_read.next() => {
                match msg {
                    Some(Ok(Message::Binary(bytes))) => {
                        if pty_writer.write_all(&bytes).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(TerminalControlMsg::Resize { cols, rows }) =
                            serde_json::from_str(&text)
                        {
                            let _ = master.resize(PtySize {
                                rows,
                                cols,
                                pixel_width: 0,
                                pixel_height: 0,
                            });
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }

    let _ = child.kill();
    Ok(())
}
