use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::{Error, Formatter, Write};
use std::{fmt, fs};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
enum Lane {
    #[serde(rename = "next")]
    Next,

    #[serde(rename = "duck")]
    Duck,

    #[serde(rename = "takeover")]
    Takeover,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    lead_seconds: Option<u32>,
    schedules: Vec<ScheduleConfig>,
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

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    NotFound(String),
    InvalidFormat(String),
    Validation(Vec<String>),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotFound(err) => f.write_str(err.as_str()),
            ConfigError::InvalidFormat(err) => f.write_str(err.as_str()),
            ConfigError::Validation(errs) => {
                // let err_str = errs.iter().map(|e| e.as_str()).collect::<Vec<&str>>().join("\n");
                // f.write_str(err_str.as_str())
                for e in errs.iter() {
                    write!(f, "{}", e)?;
                }
                Ok(())
            }
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::NotFound(value.to_string())
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn new(path: &str) -> Result<Config, ConfigError> {
        let yml = fs::read_to_string(path)?;
        let cfg = Self::parse(&yml)?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn parse(yml: &str) -> Result<Config, ConfigError> {
        let cfg = noyalib::from_str(yml)
            .map_err(|err| ConfigError::InvalidFormat(format!("err: {:?}", err).to_string()))?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<&Config, ConfigError> {
        // validate duplicated names
        // and either "every" or "time" must be there
        let mut errs: Vec<String> = Vec::new();
        let mut actions: Vec<String> = Vec::new();
        for action in self.schedules.iter() {
            let name = action.name.clone();
            if action.every.is_none() && action.time.is_none() {
                errs.push(format!(
                    "schedule {} must have either 'every' or 'time'",
                    &name
                ));
            }

            if actions.contains(&name) {
                errs.push(format!("the name {} already exists", &name));
            }

            actions.push(action.name.clone());
        }

        if !errs.is_empty() {
            return Err(ConfigError::Validation(errs));
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let yml_str = r"
lead_seconds: 30

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
            lead_seconds: Some(30),
            schedules: vec![ScheduleConfig {
                name: "short_stories".to_string(),
                action: "static".to_string(),
                lane: Lane::Duck,
                every: Some("2h".to_string()),
                time: None,
                params: params,
            }],
        };
        let cfg = Config::parse(yml_str).unwrap();
        assert_eq!(cfg, expected);
    }

    #[test]
    fn test_validation_err() {
        let yml_str = r"
lead: 30

schedules:
  - name: short_stories
    action: static
    lane: duck
    params: { dir: short_stories, select: shuffle }
        ";
        let cfg = Config::parse(yml_str).unwrap();

        matches!(cfg.validate().unwrap_err(), ConfigError::Validation(_));
    }
}
