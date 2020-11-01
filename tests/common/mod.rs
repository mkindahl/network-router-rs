extern crate router;

use log::debug;
use router::config::{self, Config};
use router::storage::{Mode, Rule};
use std::error;
use std::fmt::{self, Display};
use std::io::{self, BufRead, BufReader};
use std::net::UdpSocket;
use std::process::{Child, Command, Stdio};
use std::str::from_utf8;

/// A test harness that can be used to test the router. It allow a
/// test to start the router with a single rule and attach sockets to
/// the source and destination.
///
/// # Example
///
/// ```
/// const CONFIG: &str = r#"{
///   "protocol": "Udp",
///   "mode": "Broadcast",
///   "source": "127.0.0.1:8080",
///   "destinations": ["127.0.0.1:8081", "127.0.0.1:8082"]
/// }"#;
///
/// #[test]
/// fn test_basic() -> Result<(), Box<dyn Error>> {
///   let rule = config::Rule::from_json(CONFIG)?;
///   let mut harness = Harness::new(rule)?;
///   harness.start()?;
///   harness.test_send_str("Just a test")?;
///   Ok(())
/// }
/// ```
pub struct Harness {
    runtime: Option<Runtime>,
    rule: Rule,
}

/// Test runtime.
struct Runtime {
    child: Child,
    sender: UdpSocket,
    receivers: Vec<UdpSocket>,
}

/// Test harness error.
#[derive(Debug)]
pub struct Error(String);

impl Harness {
    /// Create a new harness with a single rule.
    ///
    ///
    pub(crate) fn new(rule: Rule) -> Harness {
        Harness {
            rule,
            runtime: None,
        }
    }

    pub(crate) fn start(&mut self) -> Result<(), Error> {
        // Set up listeners on destinations.
        let receivers: Result<Vec<_>, _> =
            self.rule.destinations.iter().map(UdpSocket::bind).collect();

        let receivers = match receivers {
            Ok(recv) => recv,
            Err(err) => return Err(Error(format!("Error: {}", err))),
        };

        // Open a sender socket
        let sender = UdpSocket::bind("0.0.0.0:0")?;
        let config = Config {
            rules: vec![self.rule.clone()],
        };
        // Spawn the router if all above worked out fine.
        let config_str = format!(r#"--config={}"#, config.to_json()?);
        println!("Config: {}", config_str);
        let child = wait_until_started(
            Command::new(env!("CARGO_BIN_EXE_network-router"))
                .arg(config_str)
                .stderr(Stdio::piped())
                .env("RUST_LOG", "info")
                .spawn()
                .expect("unable to start network router"),
        )?;

        self.runtime = Some(Runtime {
            child,
            sender,
            receivers,
        });
        Ok(())
    }

    pub(crate) fn test_send_str(&mut self, packet: &str) -> Result<(), Error> {
        match self.runtime {
            Some(ref runtime) => match self.rule.mode {
                Mode::Broadcast => {
                    runtime
                        .sender
                        .send_to(packet.as_bytes(), self.rule.source)?;
                    for receiver in &runtime.receivers {
                        let mut buf = [0; 1500];
                        let bytes = receiver.recv(&mut buf)?;
                        assert_eq!(Ok(packet), from_utf8(&buf[0..bytes]));
                    }
                    Ok(())
                }
                Mode::RoundRobin => {
                    todo!();
                }
            },
            None => Err(Error(format!("not started"))),
        }
    }
}

impl From<config::Error> for Error {
    fn from(err: config::Error) -> Self {
        Error(format!("{}", err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(format!("{}", err))
    }
}

impl error::Error for Error {}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(mut rt) = self.runtime.take() {
            if let Err(err) = rt.child.kill() {
                panic!("Cannot kill child: {}", err);
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

/// Wait for the router to start.
///
/// Read the output until a "router started" message is found, or
/// the router exited with an error.
fn wait_until_started(mut child: Child) -> Result<Child, Error> {
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let mut buf = String::new();
    while let Ok(bytes) = stderr.read_line(&mut buf) {
        if let Ok(Some(status)) = child.try_wait() {
            let output = child.wait_with_output();
            println!("Output: {:?}", output);
            return Err(Error(format!("Router exited with status {}", status)));
        }

        debug!("{}", buf);
        if bytes == 0 || buf.contains("session started") {
            break;
        }
        buf.clear();
    }
    Ok(child)
}
