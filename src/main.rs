mod config;
mod schedule;

use std::time::Duration;
use chrono::Utc;
use tokio::{time};
use crate::config::Config;
const DEFAULT_TICK_SEC: u64 = 20;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new("./config.yaml")?;
    let ts = config.tick_seconds.unwrap_or(DEFAULT_TICK_SEC);
    let schedules = config.schedules()?;
    let mut interval = time::interval(Duration::from_secs(ts));
    loop {
        interval.tick().await;
        for sc in &schedules {
            if sc.is_due(Utc::now()) {
                println!("DUE DUE: {:?}", &sc);
            }
        }
    }
}

