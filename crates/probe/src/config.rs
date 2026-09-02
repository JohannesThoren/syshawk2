use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ProbeConfig {
    /// Base URL of the shawk-server, e.g. "https://monitor.lgjt.xyz"
    pub server_url: String,
    /// Token issued when this probe was registered on the server.
    pub token: String,
    /// Seconds between snapshots.
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    /// How many top processes (by CPU) to include per snapshot.
    #[serde(default = "default_top_n")]
    pub top_processes: usize,
}

fn default_interval() -> u64 {
    5
}

fn default_top_n() -> usize {
    15
}

impl ProbeConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("SHAWK_PROBE"))
            .build()?;
        Ok(settings.try_deserialize()?)
    }
}
