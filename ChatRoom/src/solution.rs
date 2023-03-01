extern crate tokio;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::borrow::BorrowMut;
use std::fmt::Display;
use std::net::SocketAddr;
use std::ops::Deref;
//use std::error::Error;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct Error {
    msg: String
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl std::error::Error for Error {

}


// #[derive(Debug)]
pub struct Server {
    listener: Arc<TcpListener>,
    streams: Arc<Mutex<Vec<(Box<Channel>, SocketAddr)>>>,
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
            server_clone.start_live_loop().await;
        });
        //let fut = server_clone.run();
        server.live_loop = Arc::new(Some(Box::new(handle)));

        server

    }
    async fn start_live_loop(&self) {
        println!("Server is waiting for connection...");

        
        loop {
            let server_clone = self.clone();
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    println!("new client: {:?}", addr);
                    // process new connection
                    tokio::spawn(async move {
                        server_clone.process_connection(socket, addr).await;
                    });
                },
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    }
    async fn process_connection(&self, socket: TcpStream, addr: SocketAddr){
        let channel = Channel::new(Arc::new(Mutex::new(socket)));
        let mut boxed_channel = Box::new(channel);
        //let channel_cloned = channel.copy();
        {
            let mut streams = self.streams.lock().await;
            streams.push( (boxed_channel.clone(), addr.clone()) );
        }

        // TODO: 
        loop {
            let received_msg_from_client = boxed_channel.as_mut().receive().await.unwrap();
            println!("Received msg: \"{}\" from {}", received_msg_from_client, addr);
        }
    }
    pub async fn quit(self) {
        let mut streams = self.streams.lock().await;
        streams.clear();
    }
}

#[derive(Debug)]
pub struct Channel {
    stream: Arc<Mutex<TcpStream>>,
    buffer: [u8; 1024]
}

impl Clone for Channel {
    fn clone(&self) -> Self {
        Channel {
            stream: self.stream.clone(),
            buffer: self.buffer.clone()
        }
    }
}

impl Channel {
    fn new(stream: Arc<Mutex<TcpStream>>) -> Self {
        Channel {
            stream: stream.clone(),
            buffer: [0u8; 1024]
        }
    }
    pub async fn send(&mut self, message: &str) -> Result<(), Error> {
        let null_terminated_msg = message.to_string() + String::from_utf8(vec![0x0]).unwrap().as_str();
        let msg_encoded = null_terminated_msg.as_bytes();

        let res = match self.stream.borrow_mut().lock().await.write_u32(msg_encoded.len() as u32).await {
            Ok(_) => None,
            Err(err_temp) => Some(Error{ msg: err_temp.to_string() })
        };
        if let Some(error) = res {
            return Err(error);
        }

        let res = match self.stream.lock().await.write_all(message.as_bytes()).await {
            Ok(_) => None,
            Err(err_temp) => Some(Error{ msg: err_temp.to_string() })
        };
        if let Some(error) = res {
            return Err(error);
        }

        Ok(())
    }
    pub async fn receive(&mut self) -> Result<String, Error> {
        let bytes_read = self.stream.lock().await.try_read(&mut self.buffer).unwrap();
        let mut msg_bytes = Vec::from_iter(self.buffer[..bytes_read].iter().cloned());

        // check if null terminated string
        assert!(msg_bytes.last().unwrap() == &0u8);
        msg_bytes.remove(msg_bytes.len() - 1);

        let msg = String::from_utf8(msg_bytes).unwrap();

        Ok(msg)
    }
}

pub struct Client {
    stream: Arc<Mutex<TcpStream>>
}

impl Client {
    pub fn new(stream: Arc<Mutex<TcpStream>>) -> Self {
        Client { stream: stream.clone() }
    }
    pub async fn channel(&self, name: &str) -> Result<Channel, Error> {
        let mut channel = Channel::new(self.stream.clone());

        match channel.send(name).await {
            Ok(_) => Ok(channel),
            Err(err) => Err(Error { msg: err.to_string() })
        }

        //Ok()
    }
}
