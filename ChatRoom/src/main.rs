mod solution;

use std::{error::Error, time::Duration};
use solution::{Server, Client, VERBOSE};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
//use std::thread::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(("127.0.0.1", 8000)).await?;
    let server = Server::new(listener);

    let mut client1 = Client::new( TcpStream::connect("localhost:8000").await? );
    let mut channel1 = client1.channel("Test1").await.unwrap();
    
    let mut client2 = Client::new( TcpStream::connect("localhost:8000").await? );
    let mut channel2 = client2.channel("Test2").await.unwrap();

    let mut client3 = Client::new( TcpStream::connect("localhost:8000").await? );
    let mut channel3 = client3.channel("Test3").await.unwrap();

    sleep(Duration::from_secs(1) ).await;

    for i in 0..5 {
        let _ = channel1.send( format!("Hello{}", i).as_str() ).await;
        let _ = channel2.send( format!("Hi{}", i).as_str() ).await;
        let _ = channel3.send( format!("Good Day{}", i).as_str() ).await;
    }
    
    if VERBOSE {
        println!(">> Starting sleep from main thread");
    }
    sleep(Duration::from_secs(5) ).await;
    if VERBOSE {
        println!(">> Sleep passed");
    }
    
    server.quit().await;
    
    sleep(Duration::from_secs(2) ).await;
    
    if VERBOSE {
        println!();
    }
    
    println!("Client1 received:");
    loop {
        let msg_res = channel1.receive().await;
        let msg;
        match msg_res {
            Ok(message) => { msg = message; },
            Err(err) => {
                println!("{}", err.to_string());
                break;
            }
        }
        println!("{}", msg);
    }
    
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
        println!("{}", msg);
    }  
    
    println!();
    
    println!("Client3 received:");
    loop {
        let msg_res = channel3.receive().await;
        let msg;
        match msg_res {
            Ok(message) => { msg = message; },
            Err(err) => {
                println!("{}", err.to_string());
                break;
            }
        }
        println!("{}", msg);
    } 
    
    
    Ok(())
}

