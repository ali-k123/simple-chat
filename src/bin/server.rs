use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;

const IP : &str = "127.0.0.1";
const PORT : &str = "8888";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, _) = broadcast::channel::<(Vec<u8>,SocketAddr)>(16);
    let listener = TcpListener::bind(format!("{}:{}",IP,PORT)).await?;
    loop {
        let (socket,addr) = listener.accept().await?;
        let tx = tx.clone();
        tokio::spawn(async move {
            handler(socket,addr,tx).await;
        });
    }
}

async fn handler(mut socket : TcpStream, addr : SocketAddr, tx : broadcast::Sender<(Vec<u8>,SocketAddr)>){
    let mut rx = tx.subscribe();
    loop{
        let mut msg_size = [0u8;2];
        tokio::select!(
            _result = socket.read_exact(&mut msg_size) => {
                let msg_size = u16::from_le_bytes(msg_size);
                let mut msg = vec![0;msg_size as usize];
                socket.read_exact(&mut msg).await.unwrap();
                tx.send((msg, addr)).unwrap();
            }
            result = rx.recv() => {
                match result{
                    Ok(r) => {
                        let (msg, other_addr) = r;
                        if addr!=other_addr{
                            let msg_size : u16 = msg.len() as u16;
                            let msg_size = msg_size.to_le_bytes();
                            socket.write_all(&msg_size).await.unwrap();
                            socket.write_all(&msg).await.unwrap();
                        }
                    }
                    Err(_) => {
                        return
                    }
                }
            }
        );
    }
}
