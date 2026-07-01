use crate::selector::Candidate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

const API: &str = "https://api.ganjoor.net/api/ganjoor";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GanjoorConfig {
    /// Ganjoor poet id. 0 means a fully random poet.
    #[serde(default)]
    pub poet_id: u32,
    #[serde(default = "default_cache")]
    pub cache: String,
}

fn default_cache() -> String {
    "media/cache".to_string()
}

impl GanjoorConfig {
    /// Fetch one random poem and expose its id as the sole candidate.
    pub(crate) async fn candidates(&self) -> anyhow::Result<Vec<Candidate>> {
        let url = if self.poet_id == 0 {
            format!("{API}/poem/random")
        } else {
            format!("{API}/poem/random?poetId={}", self.poet_id)
        };
        let poem = get_json(&url).await?;
        let id = poem
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("ganjoor: no poem id in response"))?;
        Ok(vec![Candidate::new(id.to_string())])
    }

    /// Resolve the poem's first recitation and download its mp3.
    /// Errors (including "no recitation") simply mean nothing plays this slot.
    pub(crate) async fn materialize(&self, chosen: &Candidate) -> anyhow::Result<PathBuf> {
        let id = &chosen.uri;
        let out = Path::new(&self.cache).join(format!("ganjoor_{id}.mp3"));
        if out.exists() {
            return Ok(out);
        }

        let recitations = get_json(&format!("{API}/poem/{id}/recitations")).await?;
        let mp3_url = recitations
            .get(0)
            .and_then(|r| r.get("mp3Url"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("ganjoor: no recitation for poem {id}"))?;

        std::fs::create_dir_all(&self.cache)?;
        let tmp = out.with_extension("mp3.part");
        let status = Command::new("curl")
            .args(["-fsSL", mp3_url, "-o"])
            .arg(&tmp)
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("ganjoor: failed to download {mp3_url}");
        }
        std::fs::rename(&tmp, &out)?;
        Ok(out)
    }
}

async fn get_json(url: &str) -> anyhow::Result<serde_json::Value> {
    let output = Command::new("curl")
        .args(["-fsSL", "-H", "accept: application/json", url])
        .output()
        .await?;
    if !output.status.success() {
        anyhow::bail!(
            "ganjoor: curl failed for {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_path_is_poem_keyed_mp3() {
        let out = Path::new("media/cache").join(format!("ganjoor_{}.mp3", "1234"));
        assert_eq!(out, PathBuf::from("media/cache/ganjoor_1234.mp3"));
    }
}
