use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use dashmap::DashMap;
use std::sync::Arc;

const IP : &str = "127.0.0.1";
const PORT : &str = "8888";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("{}:{}",IP,PORT)).await?;
    let user_list : Arc<DashMap<u16,Mutex<tokio::io::WriteHalf<TcpStream>>>> = Arc::new(DashMap::new());
    loop {
        let (socket,_)= listener.accept().await?;
        let  l =user_list.clone();
        tokio::spawn(async move {
            handler(socket, l).await;
        });
    }
}

async fn handler(socket : tokio::net::TcpStream, user_list : Arc<DashMap<u16,Mutex<tokio::io::WriteHalf<TcpStream>>>>){
    let (mut r, w) = tokio::io::split(socket);
    user_list.insert(1,Mutex::new(w));
    loop{
        let msg_size = r.read_u16().await.unwrap();
        let user_id = r.read_u16().await.unwrap();
        let mut msg = vec![0;msg_size as usize-2];
        r.read_exact(&mut msg).await.unwrap();
        let l = user_list.clone();
        tokio::spawn(async move {
            send_to_user(l, user_id, msg).await;
        });
    }
}

async fn send_to_user(user_list : Arc<DashMap<u16,Mutex<tokio::io::WriteHalf<TcpStream>>>>, user_id : u16, msg : Vec<u8>){
    let guard = &*user_list.get(&user_id).unwrap();
    let mut w = guard.lock().await;
    w.write_all(&(msg.len() as u16).to_be_bytes()).await.unwrap();
    w.write_all(&msg).await.unwrap();
}

/* 
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

*/