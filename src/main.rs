mod config;

use std::time::Duration;
use tokio::{time};
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_secs(2));
    loop {
        println!("ticke");
        let config = Config::new("./config.yaml")?;
        println!("Config: {:?}", config);
        interval.tick().await;
    }
}

