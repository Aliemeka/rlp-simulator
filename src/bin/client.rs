use rlp_simulator::rlp::{Decodable, Rlp};
use rlp_simulator::{Transaction, decode_string};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // Connect to server's ack port first
    let mut retries = 0;
    let mut ack_socket = loop {
        match TcpStream::connect("127.0.0.1:9999").await {
            Ok(s) => break s,
            Err(_) if retries < 10 => {
                retries += 1;
                eprintln!("[client] waiting for server on :9999...");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(e) => {
                eprintln!("[client] failed to connect to server ack port: {e}");
                return;
            }
        }
    };
    eprintln!("[client] connected to server ack port :9999");

    // Listen for data on 9998
    let data_listener = TcpListener::bind("127.0.0.1:9998").await.unwrap();
    eprintln!("[client] listening for data on :9998");

    // Wait for server to connect to our data port
    let (mut data_socket, addr) = data_listener.accept().await.unwrap();
    eprintln!("[client] server connected for data from {addr}");
    eprintln!("[client] ready, waiting for transactions...\n");

    loop {
        // Read length prefix (4 bytes, big-endian)
        let mut len_buf = [0u8; 4];
        if data_socket.read_exact(&mut len_buf).await.is_err() {
            eprintln!("[client] server disconnected");
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        // Read RLP payload
        let mut payload = vec![0u8; len];
        data_socket.read_exact(&mut payload).await.unwrap();

        // Decode
        let rlp = Rlp::new(&payload);

        // Check if transaction or ordinary data
        //

        match Transaction::decode(&rlp) {
            Ok(tx) => {
                eprintln!("[client] received raw transaction rlp stream: {rlp}");
                eprintln!(
                    "[client] decoded transaction data: nonce={} to=0x{} value={} ({} bytes RLP)",
                    tx.nonce,
                    hex::encode(&tx.to),
                    tx.value,
                    payload.len()
                );

                // Print the raw RLP bytes
                eprint!("[client] raw bytes: ");
                for byte in &payload {
                    eprint!("{:02x} ", byte);
                }
                eprintln!();
            }
            Err(_) => {
                if let Ok(s) = decode_string(&rlp) {
                    eprintln!("[client] received raw data rlp stream: {rlp}");
                    eprintln!(
                        "[client] decoded data: '{}' ({} bytes RLP)",
                        s,
                        payload.len()
                    );
                    // Print the raw RLP bytes
                    eprint!("[client] raw bytes: ");
                    for byte in &payload {
                        eprint!("{:02x} ", byte);
                    }
                    eprintln!();
                } else {
                    eprintln!(
                        "[client] received unknown data ({} bytes RLP)",
                        payload.len()
                    );
                }
            }
        }

        // Send ack back on the ack connection
        ack_socket.write_all(b"ok").await.unwrap();
        eprintln!("[client] ack sent\n");
    }
}
