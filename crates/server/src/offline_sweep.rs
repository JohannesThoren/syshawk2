use crate::{db, state::AppState};
use shawk_common::{OFFLINE_THRESHOLD_SECS, ProbeStatus, WsEvent};
use std::{collections::HashMap, sync::Arc, time::Duration};
use uuid::Uuid;

/// Periodically checks every probe's last_seen and broadcasts a StatusChanged
/// event when a probe crosses the online/offline threshold, so the dashboard
/// doesn't need to poll to notice a dead host.
pub async fn run(state: Arc<AppState>) {
    let mut known_status: HashMap<Uuid, ProbeStatus> = HashMap::new();
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        let Ok(rows) = db::list_probes(&state.pool).await else {
            continue;
        };

        for row in rows {
            let status = match row.last_seen {
                None => ProbeStatus::Pending,
                Some(ts) => {
                    if (chrono::Utc::now() - ts).num_seconds() <= OFFLINE_THRESHOLD_SECS {
                        ProbeStatus::Online
                    } else {
                        ProbeStatus::Offline
                    }
                }
            };

            let changed = known_status
                .get(&row.id)
                .map(|prev| *prev != status)
                .unwrap_or(true);

            if changed {
                known_status.insert(row.id, status);
                let _ = state.events.send(WsEvent::StatusChanged {
                    probe_id: row.id,
                    status,
                });
            }
        }
    }
}
