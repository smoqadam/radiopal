mod action;
mod config;
mod liquidsoap;
mod schedule;
mod scheduler;
mod selector;
mod store;

use crate::config::Config;
use crate::scheduler::Scheduler;
use anyhow::Context;

const DEFAULT_TICK_SEC: u64 = 20;
const DEFAULT_LIQUIDSOAP_ADDR: &str = "127.0.0.1:1234";
const DEFAULT_STATE_FILE: &str = "selector_state.json";
const DEFAULT_CONFIG_PATH: &str = "config/config.yaml";


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path =
        std::env::var("RADIOPAL_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    let config =
        Config::new(&config_path).with_context(|| format!("loading config from {config_path}"))?;

    Scheduler::run(&config).await?;

    Ok(())
}

