extern crate tokio;

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Runtime;
use std::borrow::BorrowMut;
use std::collections::{VecDeque, LinkedList};
use std::fmt::Display;
//use std::io::{BufWriter, BufReader};
use std::net::SocketAddr;
use std::rc::Rc;
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
    streams: Arc<Mutex<Vec<Mutex<(String, Arc<Mutex<Channel>>, SocketAddr)>>>>,
    live_loops: Arc<Mutex<Vec<JoinHandle<()>>>>
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Server {
            listener: self.listener.clone(),
            streams: self.streams.clone(),
            live_loops: self.live_loops.clone()
        }
    }
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        let mut server = Server {
            listener: Arc::new(listener),
            streams: Arc::new(Mutex::new(Vec::new())),
            live_loops: Arc::new(Mutex::new(Vec::new()))
        };

        // shallow copy
        let server_clone = server.clone();

        let handle = tokio::spawn(async move {
            server_clone.start_live_loop().await;
        });

        // shallow copy
        let server_clone = server.clone();

        tokio::spawn(async move {
            server_clone.live_loops.lock().await.push(handle);
        });
        //let fut = server_clone.run();

        server

    }
    async fn start_live_loop(&self) {
        println!("Server is waiting for connection...");

        
        loop {
            let server_clone = self.clone();
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    // process new connection
                    let joinhandle = tokio::spawn(async move {
                        server_clone.process_connection(socket, addr).await;
                    });
                    self.live_loops.lock().await.push(joinhandle);
                },
                Err(e) => println!("couldn't get client: {:?}", e),
            }
        }
    }
    async fn process_connection(&self, socket: TcpStream, addr: SocketAddr){
        //let (mut read, mut write) = tokio::io::split(socket);
        let channel = Arc::new(
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

        println!(">>> Registered user {}", name);

        {
            let mut streams = self.streams.lock().await;
            streams.push( Mutex::new( (name.clone(), channel.clone(), addr.clone()) ) );
        } 

        loop {
            let received_msg_from_client =  channel.lock().await.receive().await.unwrap();
            println!(">>> Received msg: \"{}\" from {}", received_msg_from_client, name);
            let msg = format!("{x} -> {y}", x = name, y = received_msg_from_client);
            {
                println!(">>> Locking Server.streams from {}...", name);
                let streams = &self.streams.lock().await;
                let length = streams.len();
                for index in 0..length {
                    let items = &streams.get(index).unwrap().lock().await;
                    if items.0 == name {
                        continue;
                    }
                    let client_channel = items.1.clone();
                    println!(">>> Sending message {} to {}", msg, items.0);
                    let mut channel = client_channel.lock().await;
                    channel.send(msg.as_str()).await.unwrap();
                }
                println!(">>> Unlocking Server.streams from {}...", name);
            }
        }
    }
    pub async fn quit(&self) {
        // kill all live loops
        for live_loop in self.live_loops.lock().await.iter() {
            live_loop.abort();
        }

        //println!(">>> Aborted all live loops.");

        // end all streams
        let mut streams = self.streams.lock().await;
        streams.clear();
    }
}

#[derive(Debug)]
pub struct Channel {
    //stream: Arc<Mutex<TcpStream>>,
    writer: WriteHalf<TcpStream>,
    reader: ReadHalf<TcpStream>,
    message_queue: LinkedList<String>
    //buffer: [u8; 1024]
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
        let (reader, writer) = tokio::io::split(stream);
        Channel {
            //stream: Arc::new(Mutex::new(stream)),
            writer: writer,
            reader: reader,
            message_queue: LinkedList::new()
            //buffer: [0u8; 1024]
        }
    }
    pub async fn send(&mut self, message: &str) -> Result<(), Error> {
        let null_terminated_msg = message.to_string() + String::from_utf8(vec![0x0]).unwrap().as_str();
        //println!("Sending message: {}", null_terminated_msg);
        let msg_encoded = null_terminated_msg.as_bytes().to_vec();

        let res = match self.writer.write_all(&msg_encoded[..]).await {
            Ok(_) => None,
            Err(err_temp) => Some(Error{ msg: err_temp.to_string() })
        };
        if let Some(error) = res {
            return Err(error);
        }

        let _ = self.writer.flush().await;

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
        if self.message_queue.len() > 0 {
            let msg = self.message_queue.pop_front().unwrap();
            return Ok(msg);
        }

        const BUFFER_SIZE: usize = 1024;
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut bytes_read = buffer.len(); 
        let mut maybe_err;
        let mut msg_buffer = "".to_string();

        let mut bytes_counter = 0;
        while bytes_read == BUFFER_SIZE {
            (bytes_read, maybe_err) = match self.reader.read(&mut buffer).await {
                Ok(bytes_read) => (bytes_read, None),
                Err(err_msg) => (0, Some(Err(Error {msg: err_msg.to_string()})))
            };
            if let Some(err) = maybe_err {
                return err;
            }

            if bytes_read == 0 {
                let error_msg = format!("Read {} bytes. Stream ended.", bytes_counter);
                return Err(Error { msg: error_msg });
            }

            bytes_counter += bytes_read;

            let mut current_msg_bytes = buffer[..bytes_read].to_vec();
            msg_buffer = msg_buffer + String::from_utf8(current_msg_bytes).unwrap().as_str();
        }

        //println!("Read {} bytes from stream -> {}", bytes_read, msg_buffer);

        // split if loaded multiple messages
        let mut messages = Vec::new();
        
        while msg_buffer.len() > 0 {
            let mut msg_cache = Vec::new();
            loop {
                let value = msg_buffer.remove(0);
                //println!("Processing {} as char -> {}", value as u8, value);

                // end of message
                if value == 0u8 as char {
                    break;
                }
                
                msg_cache.push(value as u8);
            }
            let single_msg = String::from_utf8(msg_cache).unwrap();
            //println!("Parsed message: \"{}\"", single_msg);
            messages.push(single_msg);
        }



        for msg in messages {
            self.message_queue.push_back(msg);
        }
        
        match self.message_queue.pop_front() {
            Some(msg) => Ok(msg),
            None => Err( Error { msg: "No message in queue.".to_string() } )
        }

        // // check if null terminated string
        // assert!(msg_bytes.last().unwrap() == &0u8);
        // msg_bytes.remove(msg_bytes.len() - 1);

        // let msg = String::from_utf8(msg_bytes).unwrap();

        // println!("Received msg >>> {}", msg);

        //Ok(msg)
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
