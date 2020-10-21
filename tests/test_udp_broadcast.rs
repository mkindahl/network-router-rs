use std::error::Error;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, UdpSocket};
use std::process::Child;
use std::process::{Command, Stdio};
use std::str::from_utf8;

struct AutoKill(Child);

impl Drop for AutoKill {
    fn drop(&mut self) {
        self.0.kill().expect("Could not kill child process");
    }
}

const CONFIG: &str = r#"{
"rules": [{
  "protocol": "Udp",
  "mode": "Broadcast",
  "sources": ["127.0.0.1:8080"],
  "destinations": ["127.0.0.1:8081", "127.0.0.1:8082"]}
]}"#;

/// Basic test of UDP broadcasting functionality.
#[test]
fn test_basic() -> Result<(), Box<dyn Error>> {
    let msgs = vec!["Just a test", "Another test"];

    // Start router reading from 8080 writing 8081 and 8082.
    let mut child = AutoKill(
        Command::new(env!("CARGO_BIN_EXE_network-router"))
            .arg(format!("--config={}", CONFIG))
            .stderr(Stdio::piped())
            .env("RUST_LOG", "info")
            .spawn()
            .expect("unable to start network router"),
    );

    // Wait for the router to start. Read the output until a "router
    // started" message is found.
    let mut stderr = BufReader::new(child.0.stderr.take().unwrap());
    let mut buf = String::new();
    while let Ok(bytes) = stderr.read_line(&mut buf) {
        if let Ok(Some(status)) = child.0.try_wait() {
            panic!("Router exited with status {}", status);
        }

        if bytes == 0 || buf.contains("session started") {
            break;
        }
        buf.clear();
    }

    // Set up two listeners on 8081 and 8082.
    let receivers = vec![
        UdpSocket::bind("0.0.0.0:8081")?,
        UdpSocket::bind("0.0.0.0:8082")?,
    ];

    // Open a sender socket
    let sender = UdpSocket::bind("0.0.0.0:0")?;
    let destination: SocketAddr = "127.0.0.1:8080".parse()?;

    // For each in a list of strings, send it and verify that it
    // arrives correctly to all destinations.
    for msg in msgs {
        sender.send_to(msg.as_bytes(), &destination)?;
        for receiver in &receivers {
            let mut buf = [0; 1500];
            let bytes = receiver.recv(&mut buf)?;
            assert_eq!(Ok(msg), from_utf8(&buf[0..bytes]));
        }
    }
    Ok(())
}
