use std::io::{BufReader,Write};
use std::{env, result, thread};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream};

const SERVER_PORT:u16 = 5005;

enum Message{
    Version{
        author: Arc<TcpStream>,
        message_type: u8,
        major_revision: u8,
        minor_revision: u8,
        ext_len: u16,
        ext_list: Vec<u8>,
    }
}

type Result<T> = result::Result<T, ()>;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    // assuming static settings for now; check this later
    let address = "127.0.0.1:5005";
    let listener = TcpListener::bind(&address).map_err( |_err| {
        println!("Error: could not bind to address {address}");
    })?;
    println!("running on socket: {address}");
   
    let (sender, receiver) = channel();
    let receiver = Arc::new(Mutex::new(receiver));  //shadow 'receiver' w/ ARC & mutex
    thread::spawn(move || handle_server(receiver)); // spawn server thread
    
    //listen for incoming connections
    for stream in listener.incoming() {
        match stream{
            Ok(stream) => {
                let stream = Arc::new(stream);
                let sender = sender.clone();
                println!("New connection, spawning thread for client");
                thread::spawn(move || handle_client(stream, sender));
            }
            Err(e) => {
                println!("Error: {}",e); 
            }
        }
    }
    Ok(())
}

fn handle_server(receiver: Arc<Mutex<Receiver<Message>>>) -> Result<()> {
    loop{
        let rec = receiver.lock();
        let message = rec
                .unwrap()
                .recv()
                .map_err( |err| {
            println!("Couldn't receive message, got error: {}", err);
            std::process::exit(1);  //oh noes
        })?;

        match message{
            Message::Version{ author, message_type, major_revision, minor_revision, ext_len: _, ext_list: _} => {
                println!("Received message from:  {:?}", author.peer_addr());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(major_revision.to_le_bytes());
                message.extend(minor_revision.to_le_bytes());
                message.extend(0u16.to_le_bytes());

                author.as_ref().write_all(&message).map_err(|err| {
                    println!("Couldn't send message to client, with error: {}", err);
                })?;
            }
        }
    }
}

fn handle_client(stream: Arc<TcpStream>, message: Sender<Message>) -> Result<()> {
    println!("handling client message");
    let mut reader = BufReader::new(stream.as_ref());
    let mut message_type = [0u8];
    let mut bufr: Vec<u8> = Vec::new();

    if stream.peer_addr().is_err() {
        println!("Error: couldn't get client's peer address.");
        return Err(());
    } 
    else{
        println!("New connection from {:?}", stream.peer_addr());
    }

    let server_version = Message::Version{
        author: stream.clone(),
        message_type: 14,
        major_revision: 2,
        minor_revision: 3,
        ext_len: 0,
        ext_list: Vec::new(),
    };

    message.send(server_version).map_err(|err| {
        println!("couldn't send version message to client. Err was: {}", err);
        std::process::exit(1);
    })?;

    Ok(())
}
