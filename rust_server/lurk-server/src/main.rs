use std::io::{BufReader,Write};
use std::{env, result, thread};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream};

const SERVER_PORT:u16 = 5005;

struct MessageTypeMap{
    Character: u8,
    Game: u8,
    Version: u8,
}
impl MessageTypeMap{
    fn new() -> MessageTypeMap{
        MessageTypeMap{
            Character:  10,
            Game:       11,
            Version:    14,
        }
    }
}

enum Message{
    Character{
        author:         Arc<TcpStream>,
        message_type:   u8,
        character_name: [u8; 32], //expect exactly 32 characters, null-terminated
        flags:          u8,
        attack:         u16,
        defense:        u16,
        regen:          u16,
        health:         i16,
        gold:           u16,
        curr_room:      u16,
        desc_len:       u16,
        desc:           Vec<u8>,
    },
    Game{
        author: Arc<TcpStream>,
        message_type: u8,
        initial_points: u16,
        stat_limit: u16,
        desc_len: u16,
        game_desc: Vec<u8>,  //to be treated as characters
    },
    Version{
        author: Arc<TcpStream>,
        message_type: u8,
        major_revision: u8,
        minor_revision: u8,
        ext_len: u16,
        ext_list: Vec<u8>,
    },
}

type Result<T> = result::Result<T, ()>;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    // assuming static settings for now; check this later
    let address = "0.0.0.0:5005";
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
                println!("Received Version message from:  {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(major_revision.to_le_bytes());
                message.extend(minor_revision.to_le_bytes());
                message.extend(0u16.to_le_bytes());

                author.as_ref().write_all(&message).map_err(|err| {
                    println!("Couldn't send Version message to client, with error: {}", err);
                })?;
            }
            Message::Game{ author, message_type, initial_points, stat_limit, desc_len,  game_desc} => {
                println!("Received Game message from: {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(initial_points.to_le_bytes());
                message.extend(stat_limit.to_le_bytes());
                message.extend(desc_len.to_le_bytes());
                message.extend(game_desc);

                author.as_ref().write_all(&message).map_err(|err|{
                    println!("couldn't send Game message to client, with error {}", err);
                })?;
            }
            Message::Character { author, message_type, character_name, flags, attack, defense, regen, health, gold, curr_room, desc_len, desc } => {
                println!("received Character message from: {:?}",author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(character_name);
                message.extend(flags.to_le_bytes());
                message.extend(attack.to_le_bytes());
                message.extend(defense.to_le_bytes());
                message.extend(regen.to_le_bytes());
                message.extend(health.to_le_bytes());
                message.extend(gold.to_le_bytes());
                message.extend(curr_room.to_le_bytes());
                message.extend(desc_len.to_le_bytes());
                message.extend(desc);

                author.as_ref().write_all(&message).map_err(|err|{
                    println!("couldn't send Character message to client, with error: {}", err);
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
        println!("New connection from {:?}", stream.peer_addr().unwrap());
    }

    let server_version = Message::Version{
        author: stream.clone(),
        message_type: MessageTypeMap::new().Version,
        major_revision: 2,
        minor_revision: 3,
        ext_len: 0,
        ext_list: Vec::new(),
    };

    let g_desc = String::from("Henlo, dis iz Lurk game kthx");
    let game_info = Message::Game{
        author: stream.clone(),
        message_type: MessageTypeMap::new().Game,
        initial_points: 500,
        stat_limit: 300,
        desc_len: g_desc.len() as u16,
        game_desc: g_desc.as_bytes().to_vec(),
    };

    let c_desc = String::from("I am definitely not a plumber in search of a princess");
    let c_name = String::from("ItsaMe,Oiram"); //placeholder name, definitely not Mario
    let mut name_c_array = [0u8; 32];           // pre-pad name array
    // shove whatever will fit into name_c_array
    name_c_array[..c_name.len()].copy_from_slice(c_name.as_bytes());
    let character_info = Message::Character {
        author: stream.clone(),
        message_type: MessageTypeMap::new().Character,
        character_name: name_c_array,
        flags: 0b10000000,
        attack: 500,
        defense: 500,
        regen: 500,
        health: 500,
        gold: 12,
        curr_room: 0,
        desc_len: c_desc.len() as u16,
        desc: c_desc.as_bytes().to_vec(),
    };

    //TODO: create game-state object
    //TODO: create map objects
    //TODO: listen-loop for stream

    message.send(server_version).map_err(|err| {
        println!("couldn't send version message to client. Err was: {}", err);
        std::process::exit(1);
    })?;
    message.send(game_info).map_err(|err| {
        println!("couldn't send game info to client. Err was: {}", err);
    })?;
    message.send(character_info).map_err(|err| {
        println!("couldn't send character info to the client. Err was: {}", err);
    })?;

    Ok(())
}
