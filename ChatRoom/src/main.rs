mod solution;

use tokio::sync::Mutex;
use std::{error::Error, time::Duration};
use solution::{Server, Client};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
use std::sync::Arc;
//use std::thread::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 8000)).await?;
    let server = Server::new(listener);

    let client1 = Client::new( Arc::new(Mutex::new(TcpStream::connect("localhost:8000").await?)) );
    let channel1 = client1.channel("Marek").await.unwrap();

    let client2 = Client::new( Arc::new(Mutex::new(TcpStream::connect("localhost:8000").await?)) );
    let channel2 = client2.channel("Samo").await.unwrap();

    println!("Starting sleep from main thread");
    sleep(Duration::from_secs(5) ).await;
    println!("Sleep passed");

    server.quit().await;

    Ok(())
}

