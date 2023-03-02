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

    let mut client1 = Client::new( TcpStream::connect("localhost:8000").await? );
    let mut channel1 = client1.channel("Test1").await.unwrap();

    sleep(Duration::from_secs(1) ).await;

    let mut client2 = Client::new( TcpStream::connect("localhost:8000").await? );
    let mut channel2 = client2.channel("Test2").await.unwrap();

    let _ = channel1.send("maj sa").await;
    let _ = channel2.send("ahoj").await;

    let _ = channel1.send("maj sa1").await;
    let _ = channel2.send("ahoj1").await;

    let _ = channel1.send("maj sa2").await;
    let _ = channel2.send("ahoj2").await;

    println!(">>> Starting sleep from main thread");
    sleep(Duration::from_secs(2) ).await;
    println!(">>> Sleep passed");

    server.quit().await;

    // let _ = channel1.send("client1-end").await;
    // let _ = channel2.send("client2-end").await;
    
    // println!("Client1 received:");
    // loop {
    //     let msg_res = channel1.receive().await;
    //     let msg;
    //     match msg_res {
    //         Ok(message) => { msg = message; },
    //         Err(err) => {
    //             println!("{}", err.to_string());
    //             break;
    //         }
    //     }
    //     println!("> {}", msg);
    // }

    println!();

    println!("Client2 received:");
    loop {
        let msg_res = channel2.receive().await;
        let msg;
        match msg_res {
            Ok(message) => { msg = message; },
            Err(err) => {
                println!("{}", err.to_string());
                break;
            }
        }
        println!(">> {}", msg);
    }

    Ok(())
}

