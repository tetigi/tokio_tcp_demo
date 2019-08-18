#![feature(async_await)]

use async_trait::async_trait;
use std::env::args;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::net::tcp::split::TcpStreamWriteHalf;
use std::error;
use std::io::{Error, ErrorKind};
use std::time::{Instant, Duration};
use tokio::prelude::*;
use tokio::timer::Delay;

// use https://docs.rs/bytes/0.4.12/bytes/ for byte manipulation
// use https://docs.rs/tokio/0.2.0-alpha.1/tokio/net/struct.TcpStream.html for streaming

#[async_trait]
trait Client {
    async fn on_connect(&mut self);
    async fn handle(&mut self, msg: &[u8]);
}

struct TrollHandler {
    out: TcpStreamWriteHalf,
}

#[async_trait]
impl Client for TrollHandler {
    async fn on_connect(&mut self) {
        let msg = "hello!".as_bytes();

        println!("Sending on connect..");

        if let Err(e) = self.out.write_all(&msg).await {
            println!("Failed to write to socket; err = {:?}", e);
        }
    }

    async fn handle(&mut self, msg: &[u8]) {
        println!("Got back {:?}!", std::str::from_utf8(msg));

        let msg = "hello!".as_bytes();

        if let Err(e) = self.out.write_all(&msg).await {
            println!("Failed to write to socket; err = {:?}", e);
        }
    }
}

const ADDR: &str = "127.0.0.1:8080";

async fn run_server() -> Result<(), Box<dyn error::Error>> {
    let addr = ADDR.parse()?;
    let mut listener = TcpListener::bind(&addr).unwrap();

    println!("Starting server..");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => return, // socket was closed
                    Ok(n) => n,
                    Err(e) => {
                        println!("Failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                println!("Got {:?}!", std::str::from_utf8(&buf[0..n]));

                Delay::new(Instant::now().checked_add(Duration::from_millis(1000)).unwrap()).await;

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    println!("Failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

async fn run_client<T, F>(factory: F) -> Result<(), Box<dyn error::Error>>
    where T: Client,
          F: Fn(TcpStreamWriteHalf) -> T {
    let addr = ADDR.parse()?;

    let stream = TcpStream::connect(&addr).await?;
    let (mut read, write) = stream.split();
    let mut client = (factory)(write);

    println!("Connected!");
    client.on_connect().await;

    let mut buf = [0; 1024];

    loop {
        let n = match read.read(&mut buf).await {
            Ok(n) if n == 0 => break, // socket was closed
            Ok(n) => n,
            Err(e) => {
                println!("Failed to read from socket; err = {:?}", e);
                break;
            }
        };

        client.handle(&buf[0..n]).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = args().collect();

    println!("Hello, world!");

    if args.len() > 1 {
        match args[1].as_ref() {
            "client" => run_client(move |out| TrollHandler { out }).await,
            _ => run_server().await,
        }
    } else {
        let err: Error = Error::new(ErrorKind::InvalidInput, "Expected an argument (client/server)");
        let out_err: Box<dyn error::Error> = Box::new(err);

        Err(out_err)
    }
}
