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

pub async fn push(addr: &str, lane: &Lane, clip: &Path, title: &str) -> anyhow::Result<String> {
    let mut stream = TcpStream::connect(addr).await?;

    // annotate: attaches metadata to the request so Icecast broadcasts the
    // correct StreamTitle immediately, instead of falling back to the file's
    // (often missing) ID3 tags and showing the previous track.
    let request = format!("annotate:title={title:?}:{}", clip.display());
    let command = format!("{}.push {request}\n", queue_name(lane));
    stream.write_all(command.as_bytes()).await?;
    stream.write_all(b"quit\n").await?;
    stream.flush().await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(response.trim().to_string())
}

/// What Liquidsoap is currently streaming (music bed or a pushed program),
/// via the telnet `request.on_air` + `request.metadata` commands.
#[derive(Debug, Clone)]
pub struct Track {
    pub title: Option<String>,
    pub filename: Option<String>,
}

pub async fn current(addr: &str) -> anyhow::Result<Option<Track>> {
    let mut stream = TcpStream::connect(addr).await?;

    let on_air = telnet(&mut stream, "request.on_air").await?;
    let rid = on_air.split_whitespace().last().unwrap_or_default().to_string();
    if rid.parse::<u64>().is_err() {
        let _ = stream.write_all(b"quit\n").await;
        return Ok(None);
    }

    let meta = telnet(&mut stream, &format!("request.metadata {rid}")).await?;
    let _ = stream.write_all(b"quit\n").await;
    Ok(Some(parse_metadata(&meta)))
}

/// Send one telnet command and read the response up to Liquidsoap's `END` line.
async fn telnet(stream: &mut TcpStream, cmd: &str) -> anyhow::Result<String> {
    stream.write_all(format!("{cmd}\n").as_bytes()).await?;
    stream.flush().await?;

    let mut buf = [0u8; 2048];
    let mut acc = String::new();
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        acc.push_str(&String::from_utf8_lossy(&buf[..n]));
        if acc.lines().any(|line| line.trim() == "END") {
            break;
        }
    }
    Ok(acc
        .lines()
        .take_while(|line| line.trim() != "END")
        .collect::<Vec<_>>()
        .join("\n"))
}

fn parse_metadata(s: &str) -> Track {
    let mut title = None;
    let mut filename = None;
    for line in s.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"').to_string();
            if value.is_empty() {
                continue;
            }
            match key.trim() {
                "title" => title = Some(value),
                "filename" if filename.is_none() => filename = Some(value),
                _ => {}
            }
        }
    }
    Track { title, filename }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

        push(&addr, &Lane::Duck, Path::new("/content/music/one.mp3"), "calm")
            .await
            .unwrap();

        let received = server.await.unwrap();
        assert!(received.contains("duck.push annotate:title=\"calm\":/content/music/one.mp3"));
        assert!(received.contains("quit"));
    }

    #[test]
    fn parse_metadata_extracts_title_and_filename() {
        let raw = "title=\"Ghazal 387\"\nartist=\"Hafez\"\nfilename=\"/app/content/x.mp3\"";
        let t = parse_metadata(raw);
        assert_eq!(t.title.as_deref(), Some("Ghazal 387"));
        assert_eq!(t.filename.as_deref(), Some("/app/content/x.mp3"));
    }

    #[tokio::test]
    async fn current_reads_on_air_metadata() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap().to_string();

        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 256];
            // respond to request.on_air
            sock.read(&mut buf).await.unwrap();
            sock.write_all(b"7\nEND\n").await.unwrap();
            // respond to request.metadata 7
            sock.read(&mut buf).await.unwrap();
            sock.write_all(b"title=\"calm\"\nfilename=\"/c/x.mp3\"\nEND\n")
                .await
                .unwrap();
        });

        let track = current(&addr).await.unwrap().unwrap();
        assert_eq!(track.title.as_deref(), Some("calm"));
        assert_eq!(track.filename.as_deref(), Some("/c/x.mp3"));
    }
}
