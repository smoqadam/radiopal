use rand::RngExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub uri: String,
}

impl Candidate {
    pub fn new(uri: impl Into<String>) -> Self {
        Candidate { uri: uri.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectKind {
    Random,
    Shuffle,
    Sequential,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum Selector {
    Random,
    Sequential { last: Option<String> },
    Shuffle { remaining: Vec<String> },
}

impl Selector {
    pub fn from_kind(kind: SelectKind) -> Self {
        match kind {
            SelectKind::Random => Selector::Random,
            SelectKind::Sequential => Selector::Sequential { last: None },
            SelectKind::Shuffle => Selector::Shuffle {
                remaining: Vec::new(),
            },
        }
    }

    pub fn pick(&mut self, candidates: &[Candidate]) -> Option<Candidate> {
        if candidates.is_empty() {
            return None;
        }
        match self {
            Selector::Random => {
                let idx = rand::rng().random_range(0..candidates.len());
                Some(candidates[idx].clone())
            }
            Selector::Sequential { last } => {
                let mut uris: Vec<&str> = candidates.iter().map(|c| c.uri.as_str()).collect();
                uris.sort_unstable();
                let next = match last.as_deref().and_then(|p| uris.iter().position(|u| *u == p)) {
                    Some(i) => (i + 1) % uris.len(),
                    None => 0,
                };
                let chosen = uris[next].to_string();
                *last = Some(chosen.clone());
                candidates.iter().find(|c| c.uri == chosen).cloned()
            }
            Selector::Shuffle { remaining } => {
                let present: Vec<&str> = candidates.iter().map(|c| c.uri.as_str()).collect();
                remaining.retain(|u| present.contains(&u.as_str()));
                if remaining.is_empty() {
                    let mut pool: Vec<String> = present.iter().map(|u| u.to_string()).collect();
                    shuffle(&mut pool);
                    *remaining = pool;
                }
                let chosen = remaining.pop()?;
                candidates.iter().find(|c| c.uri == chosen).cloned()
            }
        }
    }
}

fn shuffle(items: &mut [String]) {
    let mut rng = rand::rng();
    for i in (1..items.len()).rev() {
        let j = rng.random_range(0..=i);
        items.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates(uris: &[&str]) -> Vec<Candidate> {
        uris.iter().map(|u| Candidate::new(*u)).collect()
    }

    #[test]
    fn random_picks_a_candidate() {
        let cands = candidates(&["a", "b", "c"]);
        let mut sel = Selector::from_kind(SelectKind::Random);
        let pick = sel.pick(&cands).unwrap();
        assert!(cands.contains(&pick));
    }

    #[test]
    fn empty_candidates_pick_none() {
        let mut sel = Selector::from_kind(SelectKind::Random);
        assert!(sel.pick(&[]).is_none());
    }

    #[test]
    fn sequential_advances_in_sorted_order_and_wraps() {
        let cands = candidates(&["c", "a", "b"]);
        let mut sel = Selector::from_kind(SelectKind::Sequential);
        assert_eq!(sel.pick(&cands).unwrap().uri, "a");
        assert_eq!(sel.pick(&cands).unwrap().uri, "b");
        assert_eq!(sel.pick(&cands).unwrap().uri, "c");
        assert_eq!(sel.pick(&cands).unwrap().uri, "a");
    }

    #[test]
    fn shuffle_exhausts_before_repeating() {
        let cands = candidates(&["a", "b", "c"]);
        let mut sel = Selector::from_kind(SelectKind::Shuffle);
        let mut seen = vec![
            sel.pick(&cands).unwrap().uri,
            sel.pick(&cands).unwrap().uri,
            sel.pick(&cands).unwrap().uri,
        ];
        seen.sort();
        assert_eq!(seen, vec!["a", "b", "c"]);
    }

    #[test]
    fn shuffle_drops_missing_uris_from_state() {
        let mut sel = Selector::Shuffle {
            remaining: vec!["gone".to_string(), "b".to_string()],
        };
        let cands = candidates(&["a", "b"]);
        let pick = sel.pick(&cands).unwrap();
        assert!(cands.contains(&pick));
    }

    #[test]
    fn selector_state_round_trips_through_json() {
        let sel = Selector::Sequential {
            last: Some("a".to_string()),
        };
        let json = serde_json::to_string(&sel).unwrap();
        let back: Selector = serde_json::from_str(&json).unwrap();
        assert_eq!(sel, back);
    }
}
