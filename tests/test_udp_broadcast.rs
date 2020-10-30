mod common;

use crate::common::Harness;
use router::config;
use std::error::Error;

const CONFIG: &str = r#"{
  "protocol": "Udp",
  "mode": "Broadcast",
  "sources": ["127.0.0.1:8080"],
  "destinations": ["127.0.0.1:8081", "127.0.0.1:8082"]
}"#;

/// Basic test of UDP broadcasting functionality.
#[test]
fn test_basic() -> Result<(), Box<dyn Error>> {
    let rule = config::Rule::from_json(CONFIG)?;

    let msgs = vec!["Just a test", "Another test"];
    let mut harness = Harness::new(rule);

    harness.start()?;

    // For each in a list of strings, send it and verify that it
    // arrives correctly to all destinations.
    for msg in msgs {
        harness.test_send_str(msg)?;
    }
    Ok(())
}
