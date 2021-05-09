extern crate router;

use bytes::Buf;
use http::uri::{Authority, InvalidUri, Scheme};
use hyper::{header, Body, Client, Method, Request, StatusCode, Uri};
use router::{
    config::{self, Config, Web},
    session::{Mode, Rule},
};
use std::{
    env, error,
    fmt::{self, Display},
    io::{self, BufRead, BufReader},
    net::UdpSocket,
    process::{Child, Command, Stdio},
};
use tokio::runtime::Runtime;

/// A test harness that can be used to test the router. It allow a
/// test to start the router with a single rule and attach sockets to
/// the source and destination.
///
/// Right now it only supports UDP sockets in the configuration.
///
/// # Example
///
/// ```
/// const CONFIG: &str = r#"{
///   "protocol": "udp",
///   "mode": "broadcast",
///   "source": "127.0.0.1:8080",
///   "destinations": ["127.0.0.1:8081", "127.0.0.1:8082"]
/// }"#;
///
/// #[test]
/// fn test_basic() -> Result<(), Box<dyn Error>> {
///   let rule = config::Rule::from_json(CONFIG)?;
///   let mut harness = Harness::new(rule)?;
///   harness.start()?;
///   harness.send_str("Just a test")?;
///   Ok(())
/// }
/// ```
pub struct Harness {
    state: Option<State>,
    runtime: Option<Runtime>,
    connection: Connection,
    rule: Rule,
}

struct Connection {
    endpoint: Web,
}

/// Test runtime.
struct State {
    child: Child,
    sender: UdpSocket,
    receivers: Vec<UdpSocket>,
}

impl Harness {
    /// Create a new harness with a single rule and a web endpoint.
    pub fn new(rule: Rule, port: u16) -> Harness {
        Harness {
            rule,
            connection: Connection {
                endpoint: Web::Port(Some(port)),
            },
            runtime: None,
            state: None,
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        // Set up listeners on destinations.
        let receivers: Result<Vec<_>, _> =
            self.rule.destinations.iter().map(UdpSocket::bind).collect();

        let receivers = match receivers {
            Ok(recv) => recv,
            Err(err) => return Err(Error(format!("Error: {}", err))),
        };

        let config = Config {
            web: Some(self.connection.endpoint),
            rules: vec![self.rule.clone()],
        };

        // Spawn the router to use a random port.
        let config_str = format!(r#"--config={}"#, config.to_json()?);
        let child = wait_until_started(
            Command::new(env!("CARGO_BIN_EXE_network-router"))
                .arg(config_str)
                .stderr(Stdio::piped())
                .env(
                    "RUST_LOG",
                    env::var("RUST_LOG").unwrap_or("info".to_string()),
                )
                .spawn()
                .expect("unable to start network router"),
        )?;

        // Open a sender socket
        let sender = UdpSocket::bind("0.0.0.0:0")?;

        self.runtime = Some(Runtime::new()?);
        self.state = Some(State {
            child,
            sender: sender,
            receivers: receivers,
        });
        Ok(())
    }

    /// Send a string as a packet to the UDP port and check that it is
    /// received in the receive sockets.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn send_str(&mut self, packet: &[u8]) -> Result<(), Error> {
        match self.state {
            Some(ref state) => match self.rule.mode {
                Mode::Broadcast => {
                    state.sender.send_to(packet, self.rule.source)?;
                    for receiver in &state.receivers {
                        let mut buf = [0; 1500];
                        let bytes = receiver.recv(&mut buf)?;
                        assert_eq!(packet, &buf[0..bytes]);
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

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn send_request(
        &mut self,
        method: Method,
        resource: &str,
        body: Body,
    ) -> Result<(impl Buf, StatusCode), Error> {
        let result = self.connection.request(method, resource, body);
        self.runtime.as_mut().unwrap().block_on(result)
    }
}

impl Connection {
    #[cfg(test)]
    #[allow(dead_code)]
    async fn request(
        &self,
        method: Method,
        resource: &str,
        body: Body,
    ) -> Result<(impl Buf, StatusCode), Error> {
        let scheme: Scheme = "http".parse()?;
        let authority: Authority = self.endpoint.to_string().parse()?;
        let req = Request::builder()
            .method(method)
            .uri(
                Uri::builder()
                    .scheme(scheme)
                    .authority(authority)
                    .path_and_query(resource)
                    .build()?,
            )
            .header(header::ACCEPT, "application/json")
            .body(body)?;
        let client = Client::new();
        let resp = client.request(req).await?;
        let status = resp.status();
        let body = hyper::body::aggregate(resp).await?;
        Ok((body, status))
    }
}

impl From<config::Error> for Error {
    fn from(err: config::Error) -> Self {
        Error(format!("{}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error(format!("{}", err))
    }
}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error(format!("{}", err))
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Self {
        Error(format!("HTTP Error: {}", err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(format!("{}", err))
    }
}

impl From<InvalidUri> for Error {
    fn from(_err: InvalidUri) -> Self {
        Error("invalid URI".to_string())
    }
}

/// Test harness error.
#[derive(Debug, PartialEq)]
pub struct Error(String);

impl error::Error for Error {}

impl Drop for Harness {
    fn drop(&mut self) {
        if let Some(mut rt) = self.state.take() {
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

        if bytes == 0 || buf.contains("session started") {
            break;
        }
        buf.clear();
    }
    Ok(child)
}
