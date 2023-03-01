mod solution;

use std::{error::Error, time::Duration};
use solution::Server;
use tokio::net::TcpListener;
use tokio::time::sleep;
//use std::thread::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 8000)).await?;
    let server = Server::new(listener);

    //let fut = server.run();

    println!("Starting sleep from main thread");
    sleep(Duration::from_secs(5) ).await;
    println!("Sleep passed");

    //fut.await;

    //server.quit().await;

    Ok(())
}

