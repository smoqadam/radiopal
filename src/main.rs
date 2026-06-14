use std::time::Duration;
use tokio::{time};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_secs(2));
    loop {
        println!("ticke");
        interval.tick().await;
    }

    Ok(())
}
