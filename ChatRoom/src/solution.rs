extern crate tokio;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::io::{AsyncBufRead, AsyncWrite, BufReader, BufWriter, ReadHalf, WriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;
use std::borrow::BorrowMut;
use std::fmt::Display;
//use std::io::{BufWriter, BufReader};
use std::net::SocketAddr;
use std::sync::Arc;
//use std::futures::executor;

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
    streams: Arc<Mutex<Vec<(String, Arc<Mutex<Channel>>, SocketAddr)>>>,
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
        //let (mut read, mut write) = tokio::io::split(socket);
        let mut channel = Arc::new(
            Mutex::new(
                Channel::new(
                    socket
                )
            )
        );
        // read name from channel
        let name = {
            channel.lock().await.receive().await.unwrap()
        };

        println!("Received name: {}", name);

        //let channel_cloned = channel.copy();
        let connection_index;
        {
            let mut streams = self.streams.lock().await;
            connection_index = streams.len();
            streams.push( (name, channel.clone(), addr.clone()) );
        }

        // TODO: 
        loop {
            let received_msg_from_client =  channel.lock().await.receive().await.unwrap();
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
    //stream: Arc<Mutex<TcpStream>>,
    writer: WriteHalf<TcpStream>,
    reader: ReadHalf<TcpStream>,
    buffer: [u8; 1024]
}

// impl Clone for Channel {
//     fn clone(&self) -> Self {
//         let stream_mutex_guard = Runtime::new().unwrap().block_on(async {
//             self.stream.lock().await
//         });
//         let (mut reader, mut writer) = tokio::io::split(stream_mutex_guard);
//         Channel {
//             stream: self.stream.clone(),
//             writer: writer,
//             reader: reader,
//             buffer: self.buffer.clone()
//         }
//     }
// }

impl Channel {
    fn new(stream: TcpStream) -> Self {
        let (mut reader, mut writer) = tokio::io::split(stream);
        Channel {
            //stream: Arc::new(Mutex::new(stream)),
            writer: writer,
            reader: reader,
            buffer: [0u8; 1024]
        }
    }
    pub async fn send(&mut self, message: &str) -> Result<(), Error> {
        let null_terminated_msg = message.to_string() + String::from_utf8(vec![0x0]).unwrap().as_str();
        let msg_encoded = null_terminated_msg.as_bytes();

        let res = match self.writer.write_all(msg_encoded).await {
            Ok(_) => None,
            Err(err_temp) => Some(Error{ msg: err_temp.to_string() })
        };
        if let Some(error) = res {
            return Err(error);
        }

        // let res = match self.stream.write_all(message.as_bytes()).await {
        //     Ok(_) => None,
        //     Err(err_temp) => Some(Error{ msg: err_temp.to_string() })
        // };
        // if let Some(error) = res {
        //     return Err(error);
        // }

        Ok(())
    }
    pub async fn receive(&mut self) -> Result<String, Error> {
        let (bytes_read, maybe_err) = match self.reader.read(&mut self.buffer).await {
            Ok(bytes_read) => (bytes_read, None),
            Err(err_msg) => (0, Some(Err(Error {msg: err_msg.to_string()})))
        };
        if let Some(err) = maybe_err {
            return err;
        }
        println!("Passed reading buffer");
        //let bytes_read = bytes_read;
        let mut msg_bytes = Vec::from_iter(self.buffer[..bytes_read].iter().cloned());

        // check if null terminated string
        assert!(msg_bytes.last().unwrap() == &0u8);
        msg_bytes.remove(msg_bytes.len() - 1);

        let msg = String::from_utf8(msg_bytes).unwrap();


        Ok(msg)
    }
}

pub struct Client {
    stream: Option<TcpStream>
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream: Some(stream) }
    }
    pub async fn channel(&mut self, name: &str) -> Result<Channel, Error> {
        // get stream out of Client
        // let (stream_inst, err) = match &self.stream {
        //     Some(stream) => (Some(stream), None),
        //     None => (None, Some(Err(Error { msg: "failed to get stream from client.".to_string() })))
        // };
        // if let Some(err) = err {
        //     return err;
        // }
        let stream_inst;
        stream_inst = self.stream.take().unwrap();

        let mut channel = Channel::new(stream_inst);

        match channel.send(name).await {
            Ok(_) => Ok(channel),
            Err(err) => Err(Error { msg: err.to_string() })
        }

        //Ok()
    }
}
