use crate::selector::Candidate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YoutubeConfig {
    pub url: String,
    #[serde(default = "default_cache")]
    pub cache: String,
}

fn default_cache() -> String {
    "media/cache".to_string()
}

impl YoutubeConfig {
    pub(crate) async fn candidates(&self) -> anyhow::Result<Vec<Candidate>> {
        let output = Command::new("yt-dlp")
            .args(["--flat-playlist", "--print", "id", &self.url])
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!(
                "yt-dlp failed to list {}: {}",
                self.url,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }

        Ok(parse_ids(&String::from_utf8_lossy(&output.stdout)))
    }

    pub(crate) async fn materialize(&self, chosen: &Candidate) -> anyhow::Result<PathBuf> {
        let out = self.cache_path(&chosen.uri);
        if out.exists() {
            return Ok(out);
        }

        std::fs::create_dir_all(&self.cache)?;
        let template = format!("{}/{}.%(ext)s", self.cache, chosen.uri);
        let url = format!("https://www.youtube.com/watch?v={}", chosen.uri);
        let status = Command::new("yt-dlp")
            .args(["-x", "--audio-format", "mp3", "-o", &template, &url])
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("yt-dlp failed to download {}", chosen.uri);
        }
        Ok(out)
    }

    fn cache_path(&self, id: &str) -> PathBuf {
        Path::new(&self.cache).join(format!("{id}.mp3"))
    }
}

fn parse_ids(stdout: &str) -> Vec<Candidate> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(Candidate::new)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "radiopal-{}-{}-{:?}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_ids_trims_and_filters_blanks() {
        let ids: Vec<String> = parse_ids("abc123\n  def456 \n\n")
            .into_iter()
            .map(|c| c.uri)
            .collect();
        assert_eq!(ids, vec!["abc123", "def456"]);
    }

    #[test]
    fn cache_path_is_id_keyed_mp3() {
        let cfg = YoutubeConfig {
            url: "x".to_string(),
            cache: "media/cache".to_string(),
        };
        assert_eq!(cfg.cache_path("vid42"), PathBuf::from("media/cache/vid42.mp3"));
    }

    #[tokio::test]
    async fn materialize_returns_cached_file_without_download() {
        let dir = temp_dir("yt-cache");
        let id = "cachedvid42";
        std::fs::write(dir.join(format!("{id}.mp3")), b"").unwrap();

        let cfg = YoutubeConfig {
            url: "https://www.youtube.com/@whatever".to_string(),
            cache: dir.to_string_lossy().into_owned(),
        };
        let path = cfg.materialize(&Candidate::new(id)).await.unwrap();

        assert_eq!(path, dir.join(format!("{id}.mp3")));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
