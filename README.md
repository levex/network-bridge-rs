# network-bridge [![Build Status](https://travis-ci.org/levex/network-bridge-rs.svg?branch=master)](https://travis-ci.org/levex/network-bridge-rs)
Rust crate (library) for creating and managing network bridges on Linux.

# Example

One can create a bridge using a simple builder pattern:

```rust
extern crate network_bridge;
use network_bridge::BridgeBuilder;

let bridge = BridgeBuilder::new("bridge_name")
		.interface("eth0")
		.interface("eth1")
		.build();
```

In the future, one will be able to set more properties of the bridge using this
crate.

# Disclaimer

This crate is licensed under:

- MIT License (see LICENSE-MIT); or
- Apache 2.0 License (see LICENSE-Apache-2.0),

at your option.

Please note that this crate is under heavy development, we will use sematic
versioning, but during the `0.1.*` phase, no guarantees are made about
backwards compatibility.

Regardless, check back often and thanks for taking a look!
