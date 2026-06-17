mod config;
mod schedule;
mod scheduler;

use std::time::Duration;
use chrono::Utc;
use tokio::{time};
use crate::config::Config;
use crate::scheduler::Scheduler;

const DEFAULT_TICK_SEC: u64 = 20;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new("./config.yaml")?;

    Scheduler::run(&config).await?;

    Ok(())
}

