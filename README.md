# Simple Connection and Packet Router in Rust

This is a simple connection/port-based network router implemented in
Rust using Tokio and async/await.

For UDP it currently implement simple UDP packet forwarding. It can
listen on one port and forward packets received to one or more other
addresses.

For TCP it currently implement simple round-robin selection of
incoming connections. It listens on one port and for every connection
request arriving, it will establish an outgoing connection to a list
of other addresses in a round-robin fashion. Packets will then simply
be forwarded when they arrive in both directions independently of each
others.

# How to build

```
cd network-router-rs
cargo build
```

# How to run

* Edit the `config.json` file.

* Start the router. 

```
cargo run --bin network-router -- --config-file=config.json
```

You can get a list of command-line options using `--help`.

# Configuration file format

The configuration file is in JSON and is split into separate sections
with one section for each forwarding configuration:

```json
{
    "sections": [
	{
	    "protocol":"udp",
	    "mode":"broadcast",
	    "source": "127.0.0.1:8080",
	    "destinations": ["127.0.0.1:8081"]
	}
    ]
}
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

- **source** is a source addresses that the router should
  listen on.
  
- **destinations** is a list of destination addresses that the router
  should send packets or establish connections with.

# Contribution

The code is distributed using Apache License Version 2.0. See the
`LICENSE` file for more information.
