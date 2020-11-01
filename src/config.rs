//! Configuration parser.
//!
//! Configurations are written in JSON and consists of a set of
//! forwarding rules. Rules are described in the rules module.
//!
//! # Example
//!
//! Here is a simple configuration that will broadcast UDP traffic
//! from port 9080 and forward to ports 9081 and 9082 and forward TCP
//! connections in a round-robin fashion from port 9090 to 9091 and
//! 9092.
//!
//! ```json
//! {
//!     "rules": [
//!         {
//!             "protocol":"Udp",
//!             "mode":"Broadcast",
//!             "source": "127.0.0.1:9080",
//!             "destinations": [
//!                 "127.0.0.1:9081",
//!                 "127.0.0.1:9082"
//!             ]
//!         },
//!         {
//!             "protocol":"Tcp",
//!             "mode":"RoundRobin",
//!             "source": "127.0.0.1:9090",
//!             "destinations": [
//!                 "127.0.0.1:9081",
//!                 "127.0.0.1:9082"
//!             ]
//!         },
//!     ]
//! }

use crate::storage::Rule;
use crate::strategy;
use serde::{Deserialize, Serialize};
use std::fs;

/// Configuration with rules.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    pub rules: Vec<Rule>,
}

#[derive(PartialEq, Debug)]
pub enum Error {
    IoError(String),
    JsonError(String),
    ConfigError(String),
    SyntaxError(String),
}

impl Config {
    pub fn from_json(json: &str) -> Result<Config> {
        let config: Config = serde_json::from_str(json)?;
        Ok(config)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|err| Error::JsonError(format!("JSON Error: {}", err)))
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "I/O error: {}", err),
            Error::JsonError(err) => write!(f, "JSON error: {}", err),
            Error::ConfigError(ref txt) => write!(f, "Config error: {}", txt),
            Error::SyntaxError(ref txt) => write!(f, "Syntax error: {}", txt),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::IoError(_) => "I/O error",
            Error::JsonError(_) => "JSON error",
            Error::ConfigError(_) => "config error",
            Error::SyntaxError(_) => "syntax error",
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(format!("{}", error))
    }
}

impl std::convert::From<strategy::Error> for Error {
    fn from(err: strategy::Error) -> Self {
        Error::SyntaxError(format!("{}", err))
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError(format!("{}", error))
    }
}

impl std::convert::From<std::net::AddrParseError> for Error {
    fn from(_: std::net::AddrParseError) -> Self {
        Error::ConfigError("not an address".to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl Rule {
    pub fn from_json(data: &str) -> Result<Rule> {
        let rule: Rule = serde_json::from_str(data)?;
        Ok(rule)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|err| Error::JsonError(format!("JSON Error: {}", err)))
    }
}

impl Config {
    /// Create a new empty configuration.
    pub fn new() -> Config {
        Config { rules: Vec::new() }
    }

    /// Add a rule to the configuration.
    pub fn add(&mut self, rule: Rule) {
        self.rules.push(rule)
    }

    /// Read a JSON configuration from a file name.
    pub fn from_file(filename: &str) -> Result<Config> {
        info!("Loading configuration from {}", filename);
        let config = serde_json::from_str(&fs::read_to_string(filename)?)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl std::str::FromStr for Rule {
    type Err = Error;
    fn from_str(text: &str) -> Result<Self> {
        serde_json::from_str(text).map_err(|err| Error::JsonError(format!("{}", err)))
    }
}

impl std::str::FromStr for Config {
    type Err = Error;
    fn from_str(text: &str) -> Result<Self> {
        serde_json::from_str(text).map_err(|err| Error::JsonError(format!("{}", err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{Mode, Protocol};

    #[test]
    fn test_rule_parse() {
        let rule: Result<Rule> =
            r#"{"protocol": "Udp", "mode": "Broadcast", "source": "127.0.0.1:8080", "destinations": []}"#
                .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:8080".parse().unwrap(),
                destinations: vec![]
            })
        );

        let rule: Result<Rule> = r#"{"protocol":"Udp",
                "mode":"Broadcast", "source": "127.0.0.1:9080",
                "destinations": []}"#
            .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:9080".parse().unwrap(),
                destinations: vec![]
            })
        );

        let rule: Result<Rule> = r#"{"protocol":"Udp",
                "mode":"Broadcast", "source": "127.0.0.1:9080",
                "destinations": ["127.0.0.1:9081", "127.0.0.1:9082"]}"#
            .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:9080".parse().unwrap(),
                destinations: vec![
                    "127.0.0.1:9081".parse().unwrap(),
                    "127.0.0.1:9082".parse().unwrap()
                ]
            })
        );

        let rule: Result<Rule> = r#"{"protocol":"Udp",
                "mode":"Broadcast", "source": "127.0.0.1:9080",
                "destinations": ["127.0.0.1:9081"]}"#
            .parse();
        assert_eq!(
            rule,
            Ok(Rule {
                protocol: Protocol::Udp,
                mode: Mode::Broadcast,
                source: "127.0.0.1:9080".parse().unwrap(),
                destinations: vec!["127.0.0.1:9081".parse().unwrap()]
            })
        );
    }

    #[test]
    fn test_config_parse() {
        let config: Result<Config> = r#"
{
    "rules": [
	{
	    "protocol":"Udp",
	    "mode":"Broadcast",
	    "source": "127.0.0.1:9080",
	    "destinations": ["127.0.0.1:9081", "127.0.0.1:9082"]
	}
    ]
}
"#
        .parse();
        assert_eq!(
            config,
            Ok(Config {
                rules: vec![Rule {
                    protocol: Protocol::Udp,
                    mode: Mode::Broadcast,
                    source: "127.0.0.1:9080".parse().unwrap(),
                    destinations: vec![
                        "127.0.0.1:9081".parse().unwrap(),
                        "127.0.0.1:9082".parse().unwrap()
                    ]
                }]
            })
        );
    }
}
