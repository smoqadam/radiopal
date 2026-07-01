mod ganjoor;
mod static_action;
mod youtube;

pub use ganjoor::GanjoorConfig;
pub use static_action::StaticConfig;
pub use youtube::YoutubeConfig;

use crate::selector::Candidate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Static(StaticConfig),
    Youtube(YoutubeConfig),
    Ganjoor(GanjoorConfig),
}

impl Action {
    pub async fn candidates(&self) -> anyhow::Result<Vec<Candidate>> {
        match self {
            Action::Static(cfg) => cfg.candidates(),
            Action::Youtube(cfg) => cfg.candidates().await,
            Action::Ganjoor(cfg) => cfg.candidates().await,
        }
    }

    pub async fn materialize(&self, chosen: &Candidate) -> anyhow::Result<PathBuf> {
        match self {
            Action::Static(cfg) => cfg.materialize(chosen),
            Action::Youtube(cfg) => cfg.materialize(chosen).await,
            Action::Ganjoor(cfg) => cfg.materialize(chosen).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_action_parses() {
        let action: Action = noyalib::from_str("type: static\ndir: short_stories\n").unwrap();
        assert_eq!(
            action,
            Action::Static(StaticConfig {
                dir: "short_stories".to_string()
            })
        );
    }

    #[test]
    fn youtube_action_parses_with_default_cache() {
        let action: Action =
            noyalib::from_str("type: youtube\nurl: https://www.youtube.com/@some\n").unwrap();
        assert_eq!(
            action,
            Action::Youtube(YoutubeConfig {
                url: "https://www.youtube.com/@some".to_string(),
                cache: "media/cache".to_string(),
            })
        );
    }

    #[test]
    fn ganjoor_action_parses_with_defaults() {
        let action: Action = noyalib::from_str("type: ganjoor\n").unwrap();
        assert_eq!(
            action,
            Action::Ganjoor(GanjoorConfig {
                poet_id: 0,
                cache: "media/cache".to_string(),
            })
        );
    }

    #[test]
    fn ganjoor_action_parses_poet_id() {
        let action: Action = noyalib::from_str("type: ganjoor\npoet_id: 7\n").unwrap();
        assert_eq!(
            action,
            Action::Ganjoor(GanjoorConfig {
                poet_id: 7,
                cache: "media/cache".to_string(),
            })
        );
    }
}
