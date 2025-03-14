# Ruvchain-connector

Ruvchain-connector is an independent project that aims to transparently reduce latency of a connection over Ruvchain network, utilizing NAT traversal to bypass intermediary nodes. It periodically probes for active sessions and automatically establishes direct peerings over internet with remote nodes running Ruvchain-connector without requiring any firewall configuration or port mapping.

## Features

* Peer-to-peer level latency for any traffic between a pair of peers, running the connector.
* Both TCP and UDP (QUIC) protocols are supported.
* Automatic NAT/Firewall traversal (aka hole-punching).
* Neither firewall configuration nor port mapping is required.
* No configuration of the connector is required by default.
* Seamless and transparent integration with ruvchain router.

## How it works

By default, `ruvchain` routes data only through explicitly connected peers and doesn't attempt to reach other nodes accessible over the internet. Therefore, path usually contains 1-2 intermediary nodes, namely public peers of both sides. If both you and your peer use internet, you can send traffic directly (aka peer-to-peer), thus reducing latency.

* Connector connects to [Admin API] of the running router. And monitors active sessions (peers you have data exchange over Ruvchain network with).
* Once any such session appears, connector tries to connect to associated peer, assuming it has another connector running on the same `listen_port`.
* Both connectors exchange their external internet addresses and use [NAT traversal] technique to instantiate direct bridge over the internet.
* If previous step was successful, connector will relay all data passing the bridge to the router until session is closed or other error occurs.

[STUN]: https://en.wikipedia.org/wiki/STUN
[Admin API]: https://ruvcha.in/admin.html
[NAT traversal]: https://en.wikipedia.org/wiki/NAT_traversal

## Usage

Connector can run without additional configuration. All it needs is access to [Admin API] of the router and to the IP network.

```shell
$ ruvchain-connector --loglevel info # off/error/warn/info/debug
...
```

It may be helpful to know what the defaults are.

```shell
$ ruvchain-connector --print-default
...
# List of possible admin socket locations
ruvchain_admin_listen = [
  "unix:///var/run/ruvchain/ruvchain.sock",
  "unix:///var/run/ruvchain.sock",
  "tcp://localhost:5001",
]
...
# List of allowed ruvchain peer addresses
# Uncomment to apply
#whitelist = [ ]
...
# List of peering protocols
# Supported are "tcp", "quic", "tls"
ruvchain_protocols = [ "tcp", "quic","tls" ]

# List of ruvchain listen addresses
# Known in the ruvchain config as `Listen`
ruvchain_listen = [ ]
...
# Default connect/listen port on ruvchain network
listen_port = 6868
...
# List of STUN servers
stun_servers = [
  ...
]
...
```

You can also overwrite some if needed.

```shell
$ ruvchain-connector --config <path> # standard input is read if path is "-"
...
```

## Installation

- **Downloading:** Check the [Releases page](https://github.com/ruvcoindev/ruvchain-connector/releases).
- **Compiling:**
  ```shell
  $ git clone https://github.com/ruvchain/ruvchain-connector
  $ cd ruvchain-connector
  $ cargo build --bin ruvchain-connector --release
  $ sudo cp target/release/ruvchain-connector /usr/local/bin/ruvchain-connector
  ```

## Details

<details>
<summary>External address lookup</summary>

In order to know what address to use with [NAT traversal], connector must know self external internet address and port. This task is performed using [STUN] protocol with TCP extension, hence not every [STUN] server is supported. [STUN] standard is quite broad, but connector utilities only address lookup feature.

You can check compatibility using `stun-test` binary from this repository.

```shell
$ cargo build --bin stun-test --release
$ # ./target/release/stun-test
```

`stun-test` takes network protocol and [STUN] server(s) as argument and outputs resolved address.

```shell
$ stun-test --tcp --print-servers stunserver.stunprotocol.org:3478
stunserver.stunprotocol.org:3478 244.13.30.107:28674
```

You can also take servers from hardcoded defaults or your configuration.

```shell
$ stun-test --tcp --default
244.13.30.107:28674
...
```

If `stun-test` fails to connect to any server it will print error and exit with code `1`.
```shell
$ stun-test --tcp stunserver.stunprotocol.org:3478 127.0.0.1:3478
244.13.30.107:28674
ERROR While resolving {server=127.0.0.1:3478}: Failed to connect: Connection refused
```

It also checks whether all servers return same address. You can skip this check by passing `--no-check` argument.

```shell
$ stun-test --tcp stunserver.stunprotocol.org:3478 false.resolver
244.13.30.107:28674
ERROR While resolving {server=false.resolver}: {received=0.0.0.0:0}: Previously resolved addresses do not match
```

</details>

<details>
<summary>Establishing direct connection over the internet</summary>

NAT traversal procedure is described in [this paper](https://bford.info/pub/net/p2pnat), here is a short summary:

- Create and bind listen and connection sockets to the same port (using `SO_REUSEADDR` and `SO_REUSEPORT` flags).
- Lookup self external address and port.
- Exchange external addresses with the peer.
- Try to connect to the peer and listen for connection simultaneously.

</details>
