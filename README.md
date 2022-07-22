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

# Description

The goals for the network router is to be a simple, flexible,
efficient, and open-source packet and connection router that can be
used as part of a larger distributed application. 

- **Support both UDP and TCP.** These two protocols are central to all
  applications and hence need to be supported.
- **Easy to deploy.** To make the router each to deploy it is a single
  binary with sensible defaults. No extra configuration file is
  required.
- **Configurability.** Even though the defaults are sensible, it can
  be necessary to configure specific instances of the router
  differently, so it is possible to add a configuration file that will
  be picked up by the router.
- **Integration.** The router need to integrate with a larger system,
  so an API is needed to be able to configure and re-configure the
  router as required by the changes in the topology of the system. For
  this reason, it supports a ReST API that can be used to add and
  remove routes and rules and extract information about the status of
  the router.
- **Customizable.** Different applications have different needs, so
  the defaults can be changed for custom builds and features can be
  added or removed. If you do not need the ReST API, it can be
  removed. If you do not need configuration file support, it can be
  removed.
- **Compact.** It is focused on being compact, so some tradeoffs have
  been made to ensure that it is compact. For example, since JSON is
  needed for the API, it is also the language used for the
  configuration files.
- **Fast.** Using tokio to be able to handle a lot of connections and
  not have to manage threads separately.

# Features

The router was developed to be easy to integrate into larger systems
where some jobs are automated, but also be usable in a smaller system
where there is no automation, but it might be desirable to have simple
ways to configure the router.

- **JSON API.** Proxies/routers are part of a bigger system and it is
  important to integrate the router with other systems. Since the most
  common and flexible format is JSON over HTTP, this is what we use.

- **Web Interface.** For simple applications, it is important to be
  able to get information from it in an easy format. For example, if
  it is deployed on a small network it is more convenient to just
  access it using a browser.

- **Small and configurable.** Avoiding duplication of libraries and
  allowing features to be used or compiled out. This means that we use
  JSON for the configuration file since it is required to provide a
  ReST interface.

- **Easy to deploy.** A single binary to install. No configuration
  files or template files are necessary and the default configuration
  can be compiled in.
  
  If browser support is added, the templates are compiled in using
  Askama and not deployed as separate files.

- **Efficient use of resources.** Using Tokio instead of hand-built
  thread management.

# Configuration file format

The configuration file is in JSON and is split into several sections.

## The `rules` section

The rules section contains one rule for each forwarding
configuration. For example:


```json
{
    "rules": [
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
  either `udp` or `tcp`. There is no default and this field is
  required for each rule.

- **mode** can be either `broadcast` or `round-robin` and the default
  is `broadcast` for UDP and `round-robin` for TCP.
  
  - In broadcast mode, each packet will be sent to all destinations,
    which only make sense for UDP.

  - In round-robin mode, each packet will be sent to or connection
    established with one target at a time in a round-robin fashion.

- **source** is a source addresses that the router should listen
  on. If zero is used for the port, it will pick the first available
  port.
  
- **destinations** is a list of destination addresses that the router
  should send packets or establish connections with.

# Contribution

The code is distributed using Apache License Version 2.0. See the
`LICENSE` file for more information.
