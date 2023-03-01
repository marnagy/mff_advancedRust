class Client {
    stream: Arc<Mutex<TcpStream>>,
}

impl Client {

    pub fn new(stream: TcpStream) -> Self { 
	return Client{ stream: Arc::new(Mutex::new(stream)) }	
    }

    pub async fn channel(&self, name: &str) -> Result<Channel, Error> { 
	if(self.stream == none) {
	    return Error
	}
	else {
	    return Channel{ stream: stream.clone() }
	}
    }
}


class Channel {
    stream: Arc<Mutex<TcpStream>>,
}

impl Channel {
    pub async fn send(&self, message: &str) -> Result<(), Error> { 
	let send_stream = self.stream.lock().unwrap();
	send_stream.write_all(message).await?;	
    }

    pub async fn receive(&self) -> Result<String, Error> {
	//pfuuuuuu
	// ASI treba mat spravy nabufferovane u seba, lebo mam vediet recievnut aj ked je server uz mrtvy
	// Alebo mozno to z toho mojho streamu je schopne potiahnut i ked server je uz mrtvy? IDK...
    }
}

//absolutne netusim... asi potrebujem vediet kto je na mna pripojeny,
//aby som vedel potom spravy posielat zvysku Clientov
impl Server {
    pub fn new(listener: TcpListener) -> Self { ... }
    pub async fn quit(self) { ... }
}


#[tokio::main]
main {
    let listener = TcpListener::bind(("127.0.0.1", 8000)).await?;
    let server = Server::new(listener);

    //takto sa dostane stream, ktory sa dava konstruktoru klienta
    let mut stream = TcpStream::connect("127.0.0.1:8000").await?;
    //a takto by sa mal robit klient
    client: Client = Client::new(stream: TcpStream)
}