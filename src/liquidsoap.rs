use crate::config::Lane;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub fn queue_name(lane: &Lane) -> &'static str {
    match lane {
        Lane::Next => "next",
        Lane::Duck => "duck",
        Lane::Takeover => "takeover",
    }
}

pub async fn push(addr: &str, lane: &Lane, clip: &Path) -> anyhow::Result<String> {
    let mut stream = TcpStream::connect(addr).await?;

    let command = format!("{}.push {}\n", queue_name(lane), clip.display());
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"quit\n").await?;
    stream.flush().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(response.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    #[test]
    fn queue_name_maps_each_lane() {
        assert_eq!(queue_name(&Lane::Next), "next");
        assert_eq!(queue_name(&Lane::Duck), "duck");
        assert_eq!(queue_name(&Lane::Takeover), "takeover");
    }

    #[tokio::test]
    async fn push_sends_lane_push_then_quit() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        let server = tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 256];
            let mut received = String::new();
            loop {
                let n = sock.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                received.push_str(&String::from_utf8_lossy(&buf[..n]));
                if received.contains("quit") {
                    break;
                }
            }
            received
        });

        push(&addr, &Lane::Duck, Path::new("/content/music/one.mp3"))
            .await
            .unwrap();

        let received = server.await.unwrap();
        assert!(received.contains("duck.push /content/music/one.mp3"));
        assert!(received.contains("quit"));
    }
}
