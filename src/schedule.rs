use crate::config::ScheduleConfig;
use crate::schedule::ScheduleError::{BadEvery, BadTime, Both, Empty};
use crate::selector::Selector;
use chrono::prelude::*;
use chrono::{DateTime, Duration};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub enum Schedule {
    At(NaiveTime),
    Every(Duration),
}

/// The next time this schedule should fire, at or after `now`.
pub fn next_slot(schedule: &Schedule, now: DateTime<Local>) -> DateTime<Local> {
    let slot = match schedule {
        Schedule::At(t) => {
            let today = now.date_naive().and_time(*t);
            if today >= now.naive_local() {
                today
            } else {
                today + Duration::days(1)
            }
        }
        Schedule::Every(d) => {
            let midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
            let elapsed = now.time() - midnight.time();
            let step = d.num_seconds();
            let n = (elapsed.num_seconds() + step - 1) / step; // ceil
            midnight + *d * (n as i32)
        }
    };
    slot.and_local_timezone(Local).unwrap()
}

impl Schedule {
    pub fn from_config(cfg: &ScheduleConfig) -> Result<Schedule, ScheduleError> {
        // The key trick: match on BOTH options as a tuple.
        match (&cfg.every, &cfg.time) {
            (Some(_), Some(_)) => Err(Both),
            (None, None) => Err(Empty),

            (None, Some(t)) => {
                if t.is_empty() {
                    return Err(BadTime("time is empty".to_string()));
                }
                let nt = NaiveTime::parse_from_str(t, "%H:%M")
                    .map_err(|_| BadTime(format!("bad time: {}", t.clone())))?;
                Ok(Schedule::At(nt))
            }

            (Some(e), None) => {
                let dur = parse_every(e)?;
                Ok(Schedule::Every(dur))
            }
        }
    }
}

fn parse_every(s: &str) -> Result<Duration, ScheduleError> {
    if s.is_empty() {
        return Err(Empty);
    }
    let s = s.trim();

    let (num, unit) = s.split_at(s.len() - 1);

    let n: i64 = num.parse().map_err(|_| BadEvery(s.to_string()))?;

    match unit {
        "h" => Ok(Duration::hours(n)),
        "m" => Ok(Duration::minutes(n)),
        "s" => Ok(Duration::seconds(n)),
        _ => Err(BadEvery(s.to_string())),
    }
}

#[derive(Debug, PartialEq)]
pub struct ScheduledEntry {
    pub config: ScheduleConfig,
    pub schedule: Schedule,
    pub last_fired: Option<DateTime<Local>>,
    pub stage: Stage,
    pub selector: Selector,
}

#[derive(Debug, PartialEq)]
pub enum Stage {
    Idle,
    Preparing (DateTime<Local>),
    Ready(DateTime<Local>, PathBuf),
}

#[derive(Debug, PartialEq)]
pub enum ScheduleError {
    Empty,            // neither `every` nor `time`
    Both,             // both set
    BadTime(String),  // "14:30" didn't parse
    BadEvery(String), // "2h" didn't parse
}

impl Display for ScheduleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Empty => write!(f, "neither `every` nor `time`"),

            Both => write!(f, "both `every` and `time`"),

            BadTime(t) => write!(f, "{}", t),

            BadEvery(e) => write!(f, "{}", e),
        }
    }
}

impl Error for ScheduleError {}

impl ScheduledEntry {
    pub fn new(config: ScheduleConfig) -> Result<Self, ScheduleError> {
        let sc: Schedule = Schedule::from_config(&config)?;
        let selector = Selector::from_kind(config.select);

        Ok(ScheduledEntry {
            last_fired: None,
            stage: Stage::Idle,
            selector,
            config,
            schedule: sc,
        })
    }

    pub fn due(&self, now: DateTime<Local>) -> DateTime<Local> {
        next_slot(&self.schedule, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, StaticConfig};
    use crate::config::Lane;
    use crate::selector::SelectKind;

    #[test]
    fn test_valid() {
        let cases: &[(Option<String>, Option<String>, Schedule)] = &[
            (Some("22:00".to_string()), None, Schedule::At(NaiveTime::from_hms_opt(22, 0, 0).unwrap())),
            (Some("2:00".to_string()), None, Schedule::At(NaiveTime::from_hms_opt(2, 0, 0).unwrap())),
            (Some("10:00".to_string()), None, Schedule::At(NaiveTime::from_hms_opt(10, 0, 0).unwrap())),
            (None, Some("3h".to_string()), Schedule::Every(Duration::hours(3))),
            (None, Some("10m".to_string()), Schedule::Every(Duration::minutes(10))),
            (None, Some("10s".to_string()), Schedule::Every(Duration::seconds(10))),
        ];


        for case in cases {
            let c = cfg(case.0.clone(), case.1.clone());
            let se = ScheduledEntry::new(c.clone()).unwrap();
            assert_eq!(se.schedule, case.2, "input = {c:?}");
        }
    }


    #[test]
    fn test_invalid() {
        let cases: &[(Option<String>, Option<String>, ScheduleError)] = &[
            (None, None, Empty),
            (Some("2:00".to_string()), Some("22:00".to_string()), Both),
            (Some("1".to_string()), None, BadTime("bad time: 1".to_string())),
            (Some("kjh".to_string()), None, BadTime("bad time: kjh".to_string())),
            (Some("200".to_string()), None, BadTime("bad time: 200".to_string())),
            (Some("10:00h".to_string()), None, BadTime("bad time: 10:00h".to_string())),
            (None, Some("22:00".to_string()), BadEvery("22:00".to_string())),
            (None, Some("2e".to_string()), BadEvery("2e".to_string())),
            (None, Some("22H".to_string()), BadEvery("22H".to_string())),
            (None, Some("1".to_string()), BadEvery("1".to_string())),
            (None, Some("test".to_string()), BadEvery("test".to_string())),
        ];


        for case in cases {
            let c = cfg(case.0.clone(), case.1.clone());
            let se = ScheduledEntry::new(c).err().unwrap();
            assert_eq!(se, case.2, "input = {case:?}");
        }
    }


    fn cfg(time: Option<String>, every: Option<String>) -> ScheduleConfig {
        ScheduleConfig {
            name: "".to_string(),
            title: None,
            lane: Lane::Next,
            lead: None,
            every,
            time,
            select: SelectKind::Random,
            action: Action::Static(StaticConfig {
                dir: "".to_string(),
            }),
        }
    }
}
