use crate::common::Harness;
use router::session::Rule;
use std::{error::Error, process::Child};

mod common;

struct AutoKill(Child);

impl Drop for AutoKill {
    fn drop(&mut self) {
        self.0.kill().expect("Could not kill child process");
    }
}

const CONFIG: &str = r#"{
  "protocol": "udp",
  "mode": "broadcast",
  "source": "127.0.0.1:8080",
  "destinations": ["127.0.0.1:8081", "127.0.0.1:8082"]
}"#;

/// Basic test of UDP broadcasting functionality.
#[test]
fn test_basic() -> Result<(), Box<dyn Error>> {
    let rule = Rule::from_json(CONFIG)?;
    let mut harness = Harness::new(rule, 2357);

    harness.start()?;

    // For each in a list of strings, send it and verify that it
    // arrives correctly to all destinations.
    harness.send_str(b"Just a test")?;
    harness.send_str(b"Another test")?;
    Ok(())
}
