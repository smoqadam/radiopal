use crate::selector::Selector;
use std::collections::HashMap;
use std::path::PathBuf;

pub type SelectorState = HashMap<String, Selector>;

pub struct SelectorStore {
    path: PathBuf,
}

impl SelectorStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        SelectorStore { path: path.into() }
    }

    pub fn load(&self) -> SelectorState {
        match std::fs::read_to_string(&self.path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => SelectorState::new(),
        }
    }

    pub fn save(&self, state: &SelectorState) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(state)?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, data)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "radiopal-store-{}-{}-{:?}.json",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let store = SelectorStore::new(temp_path("missing"));
        assert!(store.load().is_empty());
    }

    #[test]
    fn save_then_load_round_trips() {
        let path = temp_path("round");
        let store = SelectorStore::new(&path);

        let mut state = SelectorState::new();
        state.insert(
            "short_stories".to_string(),
            Selector::Sequential {
                last: Some("a.mp3".to_string()),
            },
        );
        store.save(&state).unwrap();

        let loaded = store.load();
        assert_eq!(loaded, state);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn load_corrupt_file_returns_empty() {
        let path = temp_path("corrupt");
        std::fs::write(&path, b"{ not json").unwrap();
        let store = SelectorStore::new(&path);
        assert!(store.load().is_empty());
        std::fs::remove_file(&path).ok();
    }
}
