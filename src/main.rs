mod config;
mod schedule;

use std::time::Duration;
use chrono::Utc;
use tokio::{time};
use crate::config::Config;
use crate::schedule::{ScheduledEntry};
const DEFAULT_TICK_SEC: u64 = 20;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new("./config.yaml")?;
    let ts = config.tick_seconds.unwrap_or_else(|| DEFAULT_TICK_SEC);

    let mut interval = time::interval(Duration::from_secs(ts));
    loop {
        interval.tick().await;
        println!("tick");
        let config = match Config::new("./config.yaml") {
            Ok(c) => c,
            Err(err) => {
                println!("{}", err);
                continue
            },
        };
        for sc in config.schedules.iter() {

            let schedule = match ScheduledEntry::new(sc ) {
                Ok(s) => s,
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            };
            println!("scheduleentry: {:?}", schedule);
            if schedule.is_due(Utc::now()) {
                println!("DUE DUE: {:?}", &schedule);
            }
        }

        println!("Config {}: {:?}", Utc::now().to_string(), config);
    }
}

