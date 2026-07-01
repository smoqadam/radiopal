use crate::action::Action;
use crate::config::Lane;
use crate::liquidsoap;
use crate::schedule::{Schedule, ScheduledEntry, next_slot};
use crate::selector::SelectKind;
use chrono::Local;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::spawn;

const INDEX_HTML: &str = include_str!("../web/index.html");

#[derive(Clone, Serialize)]
pub struct NowPlaying {
    pub name: String,
    pub clip: String,
    pub since: u64,
}

pub struct ScheduleView {
    pub title: String,
    pub lane: String,
    pub timing: String,
    pub select: String,
    pub action: String,
    schedule: Schedule,
}

impl ScheduleView {
    /// Serialize with a live `next_run` (unix seconds) so the UI can count down.
    fn to_json(&self) -> serde_json::Value {
        let next = next_slot(&self.schedule, Local::now())
            .timestamp()
            .max(0) as u64;
        serde_json::json!({
            "title": self.title,
            "lane": self.lane,
            "timing": self.timing,
            "select": self.select,
            "action": self.action,
            "next_run": next,
        })
    }
}

impl From<&ScheduledEntry> for ScheduleView {
    fn from(sc: &ScheduledEntry) -> Self {
        let c = &sc.config;
        let timing = match (&c.every, &c.time) {
            (Some(e), _) => format!("every {e}"),
            (_, Some(t)) => format!("at {t}"),
            _ => "-".to_string(),
        };
        ScheduleView {
            title: c.display(),
            lane: match c.lane {
                Lane::Next => "next",
                Lane::Duck => "duck",
                Lane::Takeover => "takeover",
            }
            .to_string(),
            timing,
            select: match c.select {
                SelectKind::Random => "random",
                SelectKind::Shuffle => "shuffle",
                SelectKind::Sequential => "sequential",
            }
            .to_string(),
            action: match c.action {
                Action::Static(_) => "static",
                Action::Youtube(_) => "youtube",
                Action::Ganjoor(_) => "ganjoor",
            }
            .to_string(),
            schedule: sc.schedule.clone(),
        }
    }
}

#[derive(Clone)]
pub struct WebState {
    pub station: String,
    pub stream_url: String,
    pub liq_addr: String,
    pub schedules: Arc<Vec<ScheduleView>>,
    pub now: Arc<Mutex<Option<NowPlaying>>>,
}

impl WebState {
    async fn state_json(&self) -> String {
        let schedules: Vec<_> = self.schedules.iter().map(ScheduleView::to_json).collect();
        serde_json::json!({
            "station": self.station,
            "stream_url": self.stream_url,
            "now": self.current_now().await,
            "schedules": schedules,
        })
        .to_string()
    }

    /// Ask Liquidsoap what is actually on air (music bed or a pushed program);
    /// that's the source of truth. Fall back to the last clip we pushed only if
    /// Liquidsoap can't be reached.
    async fn current_now(&self) -> Option<serde_json::Value> {
        let live = tokio::time::timeout(Duration::from_secs(2), liquidsoap::current(&self.liq_addr))
            .await;
        match live {
            Ok(Ok(Some(track))) => {
                let name = track
                    .title
                    .clone()
                    .or_else(|| track.filename.as_deref().map(basename))
                    .unwrap_or_else(|| "on air".to_string());
                Some(serde_json::json!({ "name": name, "clip": track.filename }))
            }
            Ok(Ok(None)) => None, // nothing on air
            _ => self.now.lock().ok().and_then(|g| g.clone()).map(|n| {
                serde_json::json!({ "name": n.name, "clip": n.clip, "since": n.since })
            }),
        }
    }
}

fn basename(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

pub fn now_playing(name: String, clip: String) -> NowPlaying {
    NowPlaying {
        name,
        clip,
        since: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    }
}

pub async fn serve(addr: String, state: WebState) {
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(err) => {
            eprintln!("web: failed to bind {addr}: {err}");
            return;
        }
    };
    println!("web ui on http://{addr}");
    loop {
        match listener.accept().await {
            Ok((sock, _)) => {
                let state = state.clone();
                spawn(async move {
                    let _ = handle(sock, &state).await;
                });
            }
            Err(err) => eprintln!("web: accept failed: {err}"),
        }
    }
}

async fn handle(mut sock: tokio::net::TcpStream, state: &WebState) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];
    let n = sock.read(&mut buf).await?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .map(|p| p.split('?').next().unwrap_or(p))
        .unwrap_or("/");

    let (status, ctype, body) = match path {
        "/" => ("200 OK", "text/html; charset=utf-8", INDEX_HTML.to_string()),
        "/api/state" => ("200 OK", "application/json", state.state_json().await),
        _ => ("404 Not Found", "text/plain", "not found".to_string()),
    };

    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    sock.write_all(resp.as_bytes()).await
}
