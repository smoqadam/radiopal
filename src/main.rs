mod action;
mod config;
mod liquidsoap;
mod schedule;
mod scheduler;
mod selector;
mod store;

use crate::config::Config;
use crate::scheduler::Scheduler;

const DEFAULT_TICK_SEC: u64 = 20;
const DEFAULT_LIQUIDSOAP_ADDR: &str = "127.0.0.1:1234";
const DEFAULT_STATE_FILE: &str = "selector_state.json";


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new("./config.yaml")?;

    Scheduler::run(&config).await?;

    Ok(())
}

