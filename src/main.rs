mod action;
mod config;
mod liquidsoap;
mod schedule;
mod scheduler;
mod selector;
mod store;
mod web;

use crate::config::Config;
use crate::scheduler::Scheduler;
use anyhow::Context;

const DEFAULT_TICK_SEC: u64 = 20;
const DEFAULT_LIQUIDSOAP_ADDR: &str = "127.0.0.1:1234";
const DEFAULT_STATE_FILE: &str = "selector_state.json";
const DEFAULT_CONFIG_PATH: &str = "config/config.yaml";
const DEFAULT_WEB_ADDR: &str = "0.0.0.0:8080";
const DEFAULT_STREAM_URL: &str = "/stream";
const DEFAULT_STATION_NAME: &str = "RadioPal";


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path =
        std::env::var("RADIOPAL_CONFIG").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
    let config =
        Config::new(&config_path).with_context(|| format!("loading config from {config_path}"))?;

    Scheduler::run(&config).await?;

    Ok(())
}

