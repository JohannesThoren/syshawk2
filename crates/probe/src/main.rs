mod collector;
mod config;
mod control;
mod terminal;

use collector::Collector;
use config::ProbeConfig;
use shawk_common::IngestRequest;
use std::time::Duration;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config_path = std::env::args().nth(1).unwrap_or_else(|| "probe".to_string());
    let cfg = ProbeConfig::load(&config_path)?;
    info!(server = %cfg.server_url, interval = cfg.interval_secs, "shawk-probe starting");

    let ws_base = cfg
        .server_url
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);

    tokio::spawn(control::run(ws_base, cfg.token.clone()));

    run_metrics_loop(cfg).await
}

async fn run_metrics_loop(cfg: ProbeConfig) -> anyhow::Result<()> {

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut collector = Collector::new(cfg.top_processes);
    let ingest_url = format!("{}/api/ingest", cfg.server_url.trim_end_matches('/'));
    let mut interval = tokio::time::interval(Duration::from_secs(cfg.interval_secs));

    loop {
        interval.tick().await;
        let snapshot = collector.collect();
        let body = IngestRequest { snapshot };

        match client
            .post(&ingest_url)
            .bearer_auth(&cfg.token)
            .json(&body)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                info!("snapshot sent");
            }
            Ok(resp) => {
                warn!(status = %resp.status(), "server rejected snapshot");
            }
            Err(e) => {
                error!(error = %e, "failed to reach server");
            }
        }
    }
}
