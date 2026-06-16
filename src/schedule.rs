use crate::config::ScheduleConfig;
use crate::schedule::ScheduleError::{BadEvery, BadTime, Both, Empty};
use chrono::prelude::*;
use chrono::{DateTime, Duration, Utc};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
enum Schedule {
    At(NaiveTime),
    Every(Duration),
}

impl Schedule {
    pub fn from_config(cfg: &ScheduleConfig) -> Result<Schedule, ScheduleError> {
        // The key trick: match on BOTH options as a tuple.
        match (&cfg.every, &cfg.time) {
            (Some(_), Some(_)) => Err(Both),
            (None, None) => Err(Empty),

            (None, Some(t)) => {
                if t.len() == 0 {
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
    if s.len() == 0 {
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
pub struct ScheduledEntry<'a> {
    config: &'a ScheduleConfig,
    schedule: Schedule,
    played_at: Option<DateTime<Utc>>,
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

impl<'a> ScheduledEntry<'a> {
    pub fn new(config: &'a ScheduleConfig) -> Result<Self, ScheduleError> {
        let sc: Schedule = Schedule::from_config(&config)?;

        Ok(ScheduledEntry {
            config,
            schedule: sc,
            played_at: None,
        })
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        let lead = Duration::seconds(self.config.lead.unwrap_or(0) as i64);

        let slot = match &self.schedule {
            Schedule::At(t) => now.date_naive().and_time(*t).and_utc(),
            Schedule::Every(d) => {
                let midnight = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                let elapsed = now - midnight;
                let step = d.num_seconds();

                let n = (elapsed.num_seconds() + step - 1) / step; // ceil

                midnight + *d * (n as i32)
            }
        };

        now >= slot - lead && now <= slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Lane};

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
            let c = &cfg(case.0.clone(), case.1.clone());
            let se = ScheduledEntry::new(&c).unwrap();
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
            let c = &cfg(case.0.clone(), case.1.clone());
            let se = ScheduledEntry::new(&c).err().unwrap();
            assert_eq!(se, case.2, "input = {case:?}");
        }
    }


    fn cfg(time: Option<String>, every: Option<String>) -> ScheduleConfig {
        ScheduleConfig {
            name: "".to_string(),
            action: "".to_string(),
            lane: Lane::Next,
            lead: None,
            every,
            time,
            params: Default::default(),
        }
    }
}
