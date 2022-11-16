use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt,stdin};
use std::str;

const IP : &str = "0.0.0.0";
const PORT : &str = "8888";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect(format!("{}:{}",IP,PORT)).await?;
    let mut input = stdin();
    let mut msg_size = [0u8;2];
    let mut text : Vec<u8> = Vec::new();
    loop{
        tokio::select!(
            _result = socket.read_exact(&mut msg_size) => {
                let msg_size = u16::from_le_bytes(msg_size);
                let mut msg = vec![0;msg_size as usize];
                socket.read_exact(&mut msg).await.unwrap();
                let s = match str::from_utf8(&msg) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                };
                println!("{}",s);
            }
            _result = input.read(&mut text) => {
                let msg_size : u16 = text.len() as u16;
                let msg_size = msg_size.to_le_bytes();
                socket.write_all(&msg_size).await.unwrap();
                socket.write_all(&text).await.unwrap();                
            }
        );
    }
}
