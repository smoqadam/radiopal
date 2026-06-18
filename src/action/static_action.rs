use crate::selector::Candidate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a", "aac", "opus"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticConfig {
    pub dir: String,
}

impl StaticConfig {
    pub(crate) fn candidates(&self) -> anyhow::Result<Vec<Candidate>> {
        let candidates = std::fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_audio(p))
            .map(|p| Candidate::new(p.to_string_lossy().into_owned()))
            .collect();
        Ok(candidates)
    }

    pub(crate) fn materialize(&self, chosen: &Candidate) -> anyhow::Result<PathBuf> {
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

    #[test]
    fn candidates_lists_only_audio() {
        let dir = temp_dir("cands");
        std::fs::write(dir.join("one.mp3"), b"").unwrap();
        std::fs::write(dir.join("two.wav"), b"").unwrap();
        std::fs::write(dir.join("notes.txt"), b"").unwrap();

        let cfg = StaticConfig {
            dir: dir.to_string_lossy().into_owned(),
        };
        let mut uris: Vec<String> = cfg
            .candidates()
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

    #[test]
    fn materialize_returns_local_path() {
        let cfg = StaticConfig {
            dir: "short_stories".to_string(),
        };
        let path = cfg.materialize(&Candidate::new("short_stories/one.mp3")).unwrap();
        assert_eq!(path, PathBuf::from("short_stories/one.mp3"));
    }

    #[test]
    fn candidates_missing_dir_errors() {
        let cfg = StaticConfig {
            dir: "/no/such/dir".to_string(),
        };
        assert!(cfg.candidates().is_err());
    }
}
