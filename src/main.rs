#![feature(async_await)]

use std::env::args;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::error;
use std::io::{Error, ErrorKind};
use std::time::{Instant, Duration};
use tokio::prelude::*;
use tokio::timer::Delay;

// use https://docs.rs/bytes/0.4.12/bytes/ for byte manipulation
// use https://docs.rs/tokio/0.2.0-alpha.1/tokio/net/struct.TcpStream.html for streaming

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

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    println!("Failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

async fn run_client() -> Result<(), Box<dyn error::Error>> {
    let addr = ADDR.parse()?;

    let mut stream = TcpStream::connect(&addr).await?;

    println!("Connected!");

    let mut buf = [0; 1024];
    let msg: &str = "hello!";

    stream.write_all(msg.as_bytes()).await?;

    loop {
        let n = match stream.read(&mut buf).await {
            Ok(n) if n == 0 => break, // socket was closed
            Ok(n) => n,
            Err(e) => {
                println!("Failed to read from socket; err = {:?}", e);
                break;
            }
        };

        println!("Got back {:?}!", std::str::from_utf8(&buf[0..n]));

        Delay::new(Instant::now().checked_add(Duration::from_millis(1000)).unwrap()).await;

        if let Err(e) = stream.write_all(&buf[0..n]).await {
            println!("Failed to write to socket; err = {:?}", e);
            break;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = args().collect();

    println!("Hello, world!");

    if args.len() > 1 {
        match args[1].as_ref() {
            "client" => run_client().await,
            _ => run_server().await,
        }
    } else {
        let err: Error = Error::new(ErrorKind::InvalidInput, "Expected an argument (client/server)");
        let out_err: Box<dyn error::Error> = Box::new(err);

        Err(out_err)
    }
}
