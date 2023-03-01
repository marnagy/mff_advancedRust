extern crate tokio;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use std::future::Future;
use std::net::SocketAddr;
//use futures::executor;
use std::sync::Arc;

// #[derive(Debug)]
pub struct Server {
    listener: Arc<TcpListener>,
    streams: Arc<Mutex<Vec<TcpStream>>>,
    live_loop: Arc<Option<Box<JoinHandle<()>>>>
}

impl Clone for Server {
    fn clone(&self) -> Self {
        // TODO: CONTINUE HERE
        Server {
            listener: self.listener.clone(),
            streams: self.streams.clone(),
            live_loop: self.live_loop.clone()
        }
    }
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        let mut server = Server {
            listener: Arc::new(listener),
            streams: Arc::new(Mutex::new(Vec::new())),
            live_loop: Arc::new(None)
        };

        // shallow copy
        let server_clone = server.clone();

        let handle = tokio::spawn(async move {
            server_clone.process_connection().await;
        });
        //let fut = server_clone.run();
        server.live_loop = Arc::new(Some(Box::new(handle)));

        server

    }
    async fn process_connection(&self) {
        println!("Server is waiting for connection...");

        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    println!("new client: {:?}", addr);
                    // TODO: process new connection
                    //tokio::spawn();
                },
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    }
    pub async fn quit(self) {
        let mut streams = self.streams.lock().await;
        streams.clear();
    }
}

struct Channel {
    stream: TcpStream
}

// impl Channel {
//     pub async fn send(&self, message: &str) -> Result<(), E> {
//         let msg_encoded = message.as_bytes();

//         self.stream.write_u32(msg_encoded.len() as u32);
//         self.stream.write_all(message.as_bytes()).await?;

//         Ok(())
//     }
//     pub async fn receive(&self) -> Result<String, Error> {
//         let msg_length = self.stream.read_u32().await?;
//         let mut buffer = vec![0, msg_length];
//         let bytes_read = self.stream.try_read_vectored(&mut buffer);

//         Ok(())
//     }
// }

struct Client {

}

// impl Client {
//     pub fn new(stream: TcpStream) -> Self {

//     }
//     pub async fn channel(&self, name: &str) -> Result<Channel, Error> {

//     }
// }
