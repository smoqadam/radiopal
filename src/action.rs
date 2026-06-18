use crate::selector::Candidate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a", "aac", "opus"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Static(StaticConfig),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticConfig {
    pub dir: String,
}

impl Action {
    pub async fn candidates(&self) -> anyhow::Result<Vec<Candidate>> {
        match self {
            Action::Static(cfg) => cfg.candidates(),
        }
    }

    pub async fn materialize(&self, chosen: &Candidate) -> anyhow::Result<PathBuf> {
        match self {
            Action::Static(cfg) => cfg.materialize(chosen),
        }
    }
}

impl StaticConfig {
    fn candidates(&self) -> anyhow::Result<Vec<Candidate>> {
        let candidates = std::fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_audio(p))
            .map(|p| Candidate::new(p.to_string_lossy().into_owned()))
            .collect();
        Ok(candidates)
    }

    fn materialize(&self, chosen: &Candidate) -> anyhow::Result<PathBuf> {
        Ok(PathBuf::from(&chosen.uri))
    }
}

fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|ext| AUDIO_EXTENSIONS.contains(&ext.as_str()))
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
    fn is_audio_matches_known_extensions_case_insensitively() {
        assert!(is_audio(Path::new("a.mp3")));
        assert!(is_audio(Path::new("a.MP3")));
        assert!(is_audio(Path::new("a.FlAc")));
        assert!(!is_audio(Path::new("a.txt")));
        assert!(!is_audio(Path::new("noext")));
    }

    #[tokio::test]
    async fn static_candidates_lists_only_audio() {
        let dir = temp_dir("cands");
        std::fs::write(dir.join("one.mp3"), b"").unwrap();
        std::fs::write(dir.join("two.wav"), b"").unwrap();
        std::fs::write(dir.join("notes.txt"), b"").unwrap();

        let action = Action::Static(StaticConfig {
            dir: dir.to_string_lossy().into_owned(),
        });
        let mut uris: Vec<String> = action
            .candidates()
            .await
            .unwrap()
            .into_iter()
            .map(|c| c.uri)
            .collect();
        uris.sort();

        assert_eq!(uris.len(), 2);
        assert!(uris[0].ends_with("one.mp3"));
        assert!(uris[1].ends_with("two.wav"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn static_materialize_returns_local_path() {
        let action = Action::Static(StaticConfig {
            dir: "short_stories".to_string(),
        });
        let path = action
            .materialize(&Candidate::new("short_stories/one.mp3"))
            .await
            .unwrap();
        assert_eq!(path, PathBuf::from("short_stories/one.mp3"));
    }

    #[tokio::test]
    async fn static_candidates_missing_dir_errors() {
        let action = Action::Static(StaticConfig {
            dir: "/no/such/dir".to_string(),
        });
        assert!(action.candidates().await.is_err());
    }
}
