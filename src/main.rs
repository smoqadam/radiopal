mod config;

use std::time::Duration;
use tokio::{time};
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        println!("ticke");
        let config = match Config::new("./config.yaml") {
            Ok(c) => c,
            Err(err) => { println!("{}", err); continue },
        };
        println!("Config: {:?}", config);
    }
}

