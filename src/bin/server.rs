use rlp::RlpStream;
use rlp_simulator::{Transaction, encode_string};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // Listen for ack connections on 9999
    let ack_listener = TcpListener::bind("127.0.0.1:9999").await.unwrap();
    eprintln!("[server] listening for acks on :9999");
    eprintln!("[server] waiting for client to start...\n");

    // Wait for client to connect to our ack port
    let (mut ack_socket, addr) = ack_listener.accept().await.unwrap();
    eprintln!("[server] client connected for acks from {addr}");

    // Now connect to client's data port
    let mut retries = 0;
    let mut data_socket = loop {
        match TcpStream::connect("127.0.0.1:9998").await {
            Ok(s) => break s,
            Err(_) if retries < 10 => {
                retries += 1;
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            Err(e) => {
                eprintln!("[server] failed to connect to client data port: {e}");
                return;
            }
        }
    };
    eprintln!("[server] connected to client data port :9998");

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut nonce: u64 = 0;

    eprintln!();
    eprintln!("Enter the input you want to send to the client.\n");
    eprintln!("Enter data as: data <value>");
    eprintln!("Example: data hello world\n");
    eprintln!("Enter transactions as: tx <to_hex> <value>");
    eprintln!("Example: tx deadbeef 1000000");
    eprintln!("Type 'quit' to exit\n");

    loop {
        eprint!("> ");

        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await.unwrap();

        if bytes_read == 0 || line.trim() == "quit" {
            eprintln!("Shutting down.");
            break;
        }

        let command = line.trim().split_whitespace().next();
        if command == Some("data") {
            let data = line.trim().strip_prefix("data ").unwrap_or("");
            let payload = encode_string(data.to_string()).out();

            // Send length prefix + RLP payload to client's data port
            let len = (payload.len() as u32).to_be_bytes();
            data_socket.write_all(&len).await.unwrap();
            data_socket.write_all(&payload).await.unwrap();

            eprintln!(
                "[server] sent raw data: '{}' ({} bytes)",
                data,
                payload.len()
            );

            // Wait for ack on our ack port
            let mut buf = [0u8; 2];
            ack_socket.read_exact(&mut buf).await.unwrap();
            eprintln!("[server] ack received: {}", String::from_utf8_lossy(&buf));
            continue;
        } else if command != Some("tx") {
            eprintln!("Unknown command. Use 'data' or 'tx' or 'quit'.");
            continue;
        }

        let next_parts = line.trim().strip_prefix("tx ").unwrap_or("");
        let parts: Vec<&str> = next_parts.split_whitespace().collect();
        if parts.len() != 2 {
            eprintln!("Usage: <to_hex> <value>");
            continue;
        }

        let to = match hex::decode(parts[0]) {
            Ok(bytes) => bytes,
            Err(_) => {
                eprintln!("Invalid hex for 'to' address");
                continue;
            }
        };

        let value: u64 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Invalid value, must be a number");
                continue;
            }
        };

        let tx = Transaction { nonce, to, value };

        // Encode
        let mut stream = RlpStream::new();
        stream.append(&tx);
        let payload = stream.out();

        // Send length prefix + RLP payload to client's data port
        let len = (payload.len() as u32).to_be_bytes();
        data_socket.write_all(&len).await.unwrap();
        data_socket.write_all(&payload).await.unwrap();

        eprintln!(
            "[server] sent nonce={} to=0x{} value={} ({} bytes RLP)",
            nonce,
            parts[0],
            value,
            payload.len()
        );

        // Wait for ack on our ack port
        let mut buf = [0u8; 2];
        ack_socket.read_exact(&mut buf).await.unwrap();
        eprintln!("[server] ack received: {}", String::from_utf8_lossy(&buf));

        nonce += 1;
    }
}
