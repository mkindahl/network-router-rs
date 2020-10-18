# Simpel Connection or Packet Router in Rust

This is a simple connection/port-based network router implemented in
Rust using Tokio using async/await.

For UDP it currently implement simple UDP packet forwarding. It
can listen on one port and forward packets received to one or more
other addresses.

For TCP it currently implement simple round-robin selection of
incoming connections. It listens on one port and for every connection
request arriving, it will establish an outgoing connection to a list
of other addresses in a round-robin fashion. Packets will then simply
be forwarded when they arrive in both directions independently of each
others.

Routes right now have to be set up statically, but the intention is to
implement a good API for dynamically setting up and changing routes.

# How to build

```
cd network-router-rs
cargo build
```

# How to run

* Edit the `config.yaml` file.

* Start the router. 

  ```
  target/debug/network-router config.yaml
  ```

# Configuration file format

The confuguration file is in YAML and is split into separate sections
(documents in YAML terminology) with one section for each forwarding
configuration:

```
---
protocol: udp
mode: broadcast
sources: [ 127.0.0.1:8080 ]
destinations: [ 127.0.0.1:8081, 127.0.0.1:8082 ]
```

Each section can contain four different attributes:

- **protocol** is the protocol that the section should use. It can be
  either `udp` or `tcp`.
- **mode** can be either `broadcast` or `round-robin` and the default
  is `broadcast` for UDP and `round-robin` for TCP.
  
  - In broadcast mode, each packet will be sent to all destinations,
    which only make sense for UDP.

  - In round-robin mode, each packet will be sent to or connection
    established with one target at a time in a round-robin fashion.

- **sources** is a list of source addresses that the router should
  listen on.
  
- **destinations** is a list of destination addresses that the router
  should send packets or establish connections with.

# Caveat

For the TCP connection, shutdown does not currently work since the
"shutdown" function for a `TcpStream` does not do anything at all
(!). See [Issue #852](https://github.com/tokio-rs/tokio/issues/852) in
(tokio-rs/tokio)[https://github.com/tokio-rs/tokio].

A simple (but ugly) way around this is to apply the following patch to
`tokio` repository:

```
diff --git a/tokio-tcp/src/stream.rs b/tokio-tcp/src/stream.rs
index 1a00679..ea0205d 100644
--- a/tokio-tcp/src/stream.rs
+++ b/tokio-tcp/src/stream.rs
@@ -844,6 +844,7 @@ impl<'a> AsyncRead for &'a TcpStream {
 
 impl<'a> AsyncWrite for &'a TcpStream {
     fn shutdown(&mut self) -> Poll<(), io::Error> {
+        self.io.get_ref().shutdown(Shutdown::Write)?;
         Ok(().into())
     }
```

# Contribution

The code is distributed using Apache License Version 2.0. See the
`LICENCE` file for more information.
