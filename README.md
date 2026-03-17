# RLP over TCP

Two-port RLP serialization simulation using Tokio.

## Architecture

```
Server (port 9999)                    Client (port 9998)
┌─────────────────┐                  ┌─────────────────┐
│ Listens on 9999 │◄── ack channel ──│ Connects to 9999│
│                 │                  │                 │
│ Connects to 9998│── data channel ─►│ Listens on 9998 │
└─────────────────┘                  └─────────────────┘
```

- **Port 9999:** Server listens, receives acks from client
- **Port 9998:** Client listens, receives RLP-encoded transactions from server

Two independent TCP connections. Data flows server → client on one, acks flow client → server on the other.

## Run

Build:

```bash
cargo build --release
```

Terminal 1 (start the server first):

```bash
cargo run --bin server
```

Terminal 2:

```bash
cargo run --bin client
```

## Usage

Once both are connected, type transactions in the server terminal:

**Send string data**

```
> data
> data hello
> data hello world
> quit
```

Format: `data <to_address_hex> <value>`

**Send Transaction data**

```
> tx deadbeef 1000000
> tx cafe 500
> tx ab01cd02ef03 999
> quit
```

Format: `tx <to_address_hex> <value>`

The server encodes each transaction as RLP, sends it over TCP to the client's port. The client decodes it, prints the raw bytes and decoded fields, then sends an ack back to the server's port.

## Dependencies

- `tokio` — async runtime and TCP
- `rlp` — RLP encoding/decoding (same crate the Ethereum ecosystem uses)
- `hex` — hex string parsing for addresses
