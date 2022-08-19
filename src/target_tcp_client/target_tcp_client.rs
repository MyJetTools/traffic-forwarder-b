use std::sync::atomic::AtomicBool;
use std::{sync::Arc, time::Duration};

use tokio::sync::mpsc::UnboundedSender;
use tokio::{net::TcpStream, sync::mpsc::UnboundedReceiver};

use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};

use super::TargetTcpCallbacks;

pub struct TargetTcpClient {
    pub id: u32,
    sender: UnboundedSender<Option<Vec<u8>>>,
    disconnected: AtomicBool,
}

impl TargetTcpClient {
    pub async fn new(
        callbacks: Arc<TargetTcpCallbacks>,
        id: u32,
        host_port: String,
    ) -> Result<Arc<Self>, String> {
        let connect_result = TcpStream::connect(host_port.as_str()).await;

        match connect_result {
            Ok(tcp_stream) => {
                let (read_stream, write_stream) = tokio::io::split(tcp_stream);
                let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

                let result = Self {
                    id,
                    sender,
                    disconnected: AtomicBool::new(false),
                };
                let result = Arc::new(result);

                tokio::spawn(read_loop(
                    callbacks.clone(),
                    read_stream,
                    1024 * 1024 * 5,
                    result.clone(),
                ));

                tokio::spawn(tcp_send_loop(id, receiver, write_stream));

                return Ok(result);
            }
            Err(err) => Err(format!(
                "Can not connect to target {}. Err: {}",
                host_port, err
            )),
        }
    }

    pub fn send_payload(&self, payload: Vec<u8>) {
        let _ = self.sender.send(Some(payload));
    }

    pub fn disconnect(&self) {
        let before_was_connected = self
            .disconnected
            .swap(true, std::sync::atomic::Ordering::SeqCst);

        if before_was_connected {
            let _ = self.sender.send(None);
        }
    }
}

async fn read_loop(
    callbacks: Arc<TargetTcpCallbacks>,
    mut read_stream: ReadHalf<TcpStream>,
    buffer_size: usize,
    tcp_connection: Arc<TargetTcpClient>,
) {
    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);

    loop {
        unsafe {
            buffer.set_len(buffer_size);
        }

        match read_stream.read(&mut buffer).await {
            Ok(read_amount) => {
                if read_amount == 0 {
                    println!(
                        "Socket {} got 0 bytes. Stopping read_stream",
                        tcp_connection.id,
                    );
                    break;
                }

                if !callbacks
                    .on_payload(&tcp_connection, buffer[..read_amount].to_vec())
                    .await
                {
                    println!(
                        "Tunnel has not connection anymore. Stopping read_stream of socket {}",
                        tcp_connection.id,
                    );
                    break;
                }
            }
            Err(err) => {
                println!(
                    "Error reading from socket. Err:{}. Stopping read_stream",
                    err
                );
                break;
            }
        }
    }

    callbacks.on_disconnected(tcp_connection).await;
}

async fn tcp_send_loop(
    id: u32,
    mut receiver: UnboundedReceiver<Option<Vec<u8>>>,
    mut tcp_stream: WriteHalf<TcpStream>,
) {
    let send_timeout = Duration::from_secs(15);
    while let Some(next) = receiver.recv().await {
        match next {
            Some(payload) => {
                let future = tcp_stream.write_all(payload.as_slice());

                let result = tokio::time::timeout(send_timeout, future).await;

                if result.is_err() {
                    println!("TcpConnection:{}: send timeout", id);
                    break;
                }

                let result = result.unwrap();

                if let Err(err) = result {
                    println!("TcpConnection:{} has error {}", id, err);
                    break;
                }
            }
            None => {
                break;
            }
        }
    }

    let _ = tcp_stream.shutdown().await;
}
