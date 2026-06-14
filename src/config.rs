use std::collections::HashMap;
use std::{fmt, fs};
use std::fmt::{Error, Formatter};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Lane {
    #[serde(rename = "next")]
    Next,

    #[serde(rename = "duck")]
    Duck,

    #[serde(rename = "takeover")]
    Takeover
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    lead: Option<i16>,
    schedules: Vec<ScheduleConfig>
}
pub type ScheduleConfigParams = HashMap<String, String>;
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ScheduleConfig {
    name: String,
    action: String,
    lane: Lane,
    every: Option<String>,
    time: Option<String>,
    params: ScheduleConfigParams,
}

#[derive(Debug)]
pub enum   ConfigError {
    NotFound(String),
    InvalidFormat(String),
    Validation(Vec<String>),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        println!("config error");
        Ok(())
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        println!("Value: {:?}", value);
        ConfigError::NotFound("test".to_string())
    }
}

impl std::error::Error for ConfigError{}


impl Config {
    pub fn new(path: &str) -> Result<Config, ConfigError> {
        let yml = fs::read_to_string(path)?;
        Config::validate(&yml)
    }


    fn validate(yml: &str) -> Result<Config, ConfigError> {
        // validate duplicated names
        // and either "every" or "time" must be there
        let cfg: Config = noyalib::from_str(yml)
            .map_err(|err| ConfigError::InvalidFormat( format!("err: {:?}", err).to_string()))?;

        let mut errs: Vec<String> = Vec::new();
        let mut actions: Vec<String> = Vec::new();
        for action in cfg.schedules.iter() {
            let name = action.name.clone();
            if action.every.is_none() && action.time.is_none() {
                errs.push(format!("schedule {} must have either 'every' or 'time'", &name));
            }

            if actions.contains(&name) {
                errs.push(format!("the name {} already exists", &name));

            }

            actions.push(action.name.clone());
        }

        if errs.len() > 0 {
            return Err(ConfigError::Validation(errs));
        }

        Ok(cfg)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let yml_str = r"
lead: 30

schedules:
  - name: short_stories
    action: static
    lane: duck
    every: '2h'
    params: { dir: short_stories, select: shuffle }

        ";
        let mut params = HashMap::new();
        params.insert("dir".to_string(), "short_stories".to_string());
        params.insert("select".to_string(), "shuffle".to_string());
        let expected = Config {
            lead: Some(30),
            schedules: vec![
                ScheduleConfig{
                    name: "short_stories".to_string(),
                    action: "static".to_string(),
                    lane: Lane::Duck,
                    every: Some("2h".to_string()),
                    time: None,
                    params: params,
                }
            ],

        };
        let cfg = Config::validate(yml_str).unwrap();
        assert_eq!(cfg, expected);
    }
}