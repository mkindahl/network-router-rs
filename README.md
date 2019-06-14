# Simpel Network Router in Rust

This is a simple connection/port-based network router implemented in Rust using Tokio.

Currently, it only implement simple UDP forwarding and listen on one
port and forward any packets received there to one or more other
addresses.

# How to build

```
cd network-router-rs
cargo build
```

# How to run

* Edit the `config.yaml` file.

* Start the router

  ```
  target/debug/router
  ```

# Contribution

The code is distributed using Apache License Version 2.0. See the
`LICENCE` file for more information.
