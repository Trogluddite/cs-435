use std::io::{BufReader, Write, Read};
use std::{result, thread};          // later: env
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream, Shutdown};

const SERVER_PORT:u16 = 5005;
const SERVER_ADDRESS:&'static str = "0.0.0.0";
const WELCOME:&'static str = "        
           \\`-._           __
            \\\\  `-..____,.'  `.
             :`.         /    \\`.
             :  )       :      : \\
              ;'        '   ;  |  :
              )..      .. .:.`.;  :
             /::...  .:::...   ` ;
             ; _ '    __        /:\\
             `:o>   /\\o_>      ;:. `.
            `-`.__ ;   __..--- /:.   \\
            === \\_/   ;=====_.':.     ;
             ,/'`--'...`--....        ;
                  ;                    ;
                .'                      ;
              .'                        ;
            .'     ..     ,      .       ;
           :       ::..  /      ;::.     |
          /      `.;::.  |       ;:..    ;
         :         |:.   :       ;:.    ;
         :         ::     ;:..   |.    ;
          :       :;      :::....|     |
          /\\     ,/ \\      ;:::::;     ;
        .:. \\:..|    :     ; '.--|     ;
       ::.  :''  `-.,,;     ;'   ;     ;
    .-'. _.'\\      / `;      \\,__:      \\
    `---'    `----'   ;      /    \\,.,,,/
                       `----`              ";

struct MessageType;
impl MessageType{
    const ACCEPT:       u8 = 8;
    const CHANGEROOM:   u8 = 2;
    const CHARACTER:    u8 = 10;
    const CONNECTION:   u8 = 13;
    const ERROR:        u8 = 7;
    const FIGHT:        u8 = 3;
    const GAME:         u8 = 11;
    const LEAVE:        u8 = 12;
    const LOOT:         u8 = 5;
    const MESSAGE:      u8 = 1;
    const ROOM:         u8 = 9;
    const START:        u8 = 6;
    const PVPFIGHT:     u8 = 4;
    const VERSION:      u8 = 14;
}

struct ErrorType;
#[allow(dead_code)] //FIXME: later
impl ErrorType{
    const OTHER:        u8 = 0;
    const BAD_ROOM:     u8 = 1;
    const PLAYER_EXISTS:u8 = 2;
    const BAD_MONSTER:  u8 = 3;
    const STAT_ERROR:   u8 = 4;
    const NOT_READY:    u8 = 5;
    const NO_TARGET:    u8 = 6;
    const NO_FIGHT:     u8 = 7;
    const PVP_DISABLED: u8 = 8;
}

struct PlayerFlags;
#[allow(dead_code)] //FIXME: later
impl PlayerFlags{
    const IS_ALIVE:     u8 = 0b10000000;
    const JOIN_BATTLE:  u8 = 0b01000000;
    const IS_MONSTER:   u8 = 0b00100000;
    const IS_STARTED:   u8 = 0b00010000;
    const IS_READY:     u8 = 0b00001000;
    const ALL_FLAGS_SET:u8 = 0b11111111;
    const NO_FLAGS_SET: u8 = 0b00000000;
}

#[derive(Debug)]
#[allow(dead_code)] //FIXME: later
enum Message{
    Accept{
        author:         Arc<TcpStream>,
        message_type:   u8,
        accepted_type:  u8,
    },
    ChangeRoom{
        author:         Arc<TcpStream>,
        message_type:   u8,
        target_room:    u16,
    },
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
    Connection{
        author:         Arc<TcpStream>,
        message_type:   u8,
        room_number:    u16,
        room_name:      [u8; 32],
        desc_len:       u16,
        room_desc:      Vec<u8>,
    },
    Error{
        author:         Arc<TcpStream>,
        message_type:   u8,
        error_code:     u8,
        messaage_len:   u16,
        message:        Vec<u8>,
    },
    Fight{
        author:         Arc<TcpStream>,
        message_type:   u8,
    },
    Game{
        author:         Arc<TcpStream>,
        message_type:   u8,
        initial_points: u16,
        stat_limit:     u16,
        desc_len:       u16,
        game_desc:      Vec<u8>,  //to be treated as characters
    },
    Leave{
        author:         Arc<TcpStream>,
        message_type:   u8,
    },
    Loot{
        author:         Arc<TcpStream>,
        message_type:   u8,
        target_name:    Vec<u8>,
    },
    Message{
        author:         Arc<TcpStream>,
        message_type:   u8,
        message_len:    u16,
        recipient_name: [u8; 32],
        sender_name:    [u8; 30],
        end_marker:     u16,
        message:        Vec<u8>,
    },
    Room{
        author:         Arc<TcpStream>,
        message_type:   u8,
        room_number:    u16,
        room_name:      [u8; 32],
        desc_len:       u16,
        room_desc:      Vec<u8>,
    },
    Start{
        author:         Arc<TcpStream>,
        message_type:   u8,
    },
    PVPFight{
        author:         Arc<TcpStream>,
        message_type:   u8,
        target_name:    [u8; 32],
    },
    Version{
        author:         Arc<TcpStream>,
        message_type:   u8,
        major_revision: u8,
        minor_revision: u8,
        ext_len:        u16,
        ext_list:       Vec<u8>,
    },
}

#[allow(dead_code)] //FIXME later
struct Character{
    conn:       Arc<TcpStream>,
    name:       String,
    is_active:  bool,
    flags:      u8,
    attack:     u16,
    defense:    u16,
    regen:      u16,
    health:     i16,
    gold:       u16,
    curr_room:  u16,
    desc:       String,
}
impl Character{
    fn new(conn: Arc<TcpStream>, name: String, desc: String) -> Character{
        Character{
            conn,
            name,
            desc,
            is_active : true,
            flags: 0xFF,
            attack: 50,
            defense: 50,
            regen: 100,
            health: 100,
            gold: 0,
            curr_room: 0,
        }
    }
}

type Result<T> = result::Result<T, ()>;

fn main() -> Result<()> {
    //let _args: Vec<String> = env::args().collect();
    // assuming static settings for now; check this later
    let address = format!("{}:{}",SERVER_ADDRESS, SERVER_PORT);
    let listener = TcpListener::bind(&address).map_err( |_err| {
        println!("[SERVER MESSAGE]: Error: could not bind to address {address}");
    })?;
    println!("[SERVER MESSAGE]: running on socket: {address}");

    let (sender, receiver) = channel();
    let receiver = Arc::new(Mutex::new(receiver));  //shadow 'receiver' w/ ARC & mutex
    thread::spawn(move || handle_received_messages(receiver)); // spawn server thread

    //listen for incoming connections
    for stream in listener.incoming() {
        match stream{
            Ok(stream) => {
                let stream = Arc::new(stream);
                let sender = sender.clone();
                println!("[SERVER MESSAGE]: New connection, spawning thread for client {:?}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream, sender));
            }
            Err(e) => {
                println!("Error: {}",e);
            }
        }
    }
    Ok(())
}

//thread receiver
fn handle_received_messages(receiver: Arc<Mutex<Receiver<Message>>>) -> Result<()> {
    println!("[SERVER_MESSAGE]: handling incomming messages");
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
            Message::Accept{ author, message_type, accepted_type} => {
                println!("[MPCS RECEIVED] Accept message from: {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.push(accepted_type);
                author.as_ref().write_all(&message).map_err(|err| {
                    eprintln!("Couldn't send accept message to client, with error: {}", err);
                })?;
            }
            Message::Character { author, message_type, character_name, flags, attack, defense, regen, health, gold, curr_room, desc_len, desc } => {
                println!("[MPSC RECEIVED] Character message from: {:?}",author.peer_addr().unwrap());
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
            Message::Connection { author, message_type, room_number, room_name, desc_len, room_desc } => {
                println!("[MPSC RECEIVED] connection message from: {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(room_number.to_le_bytes());
                message.extend(room_name);
                message.extend(desc_len.to_le_bytes());
                message.extend(room_desc);

                author.as_ref().write_all(&message).map_err(|err| {
                    println!("Couldn't send connection message to client, with error {}", err);
                })?;
            }
            Message::Error { author, message_type, error_code, messaage_len, message } => {
                println!("[MPSC RECEIVED] Error message from: {:?}", author.peer_addr().unwrap());
                let mut send_message: Vec<u8> = Vec::new();
                send_message.push(message_type);
                send_message.push(error_code);
                send_message.extend(messaage_len.to_le_bytes());
                send_message.extend(message);

                author.as_ref().write_all(&send_message).map_err(|err| {
                    println!("Couldn't send connection message to client, with error {}", err);
                })?;
            }
            Message::Game{ author, message_type, initial_points, stat_limit, desc_len,  game_desc} => {
                println!("[MPSC RECEIVED] game message from: {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(initial_points.to_le_bytes());
                message.extend(stat_limit.to_le_bytes());
                message.extend(desc_len.to_le_bytes());
                message.extend(game_desc);

                println!("[SERVER_MESSAGE] Sending Game message");
                author.as_ref().write_all(&message).map_err(|err|{
                    println!("couldn't send Game message to client, with error {}", err);
                })?;
            }
            Message::Room { author, message_type, room_number, room_name, desc_len, room_desc } => {
                println!("[MPSC RECEIVED] room message from: {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(room_number.to_le_bytes());
                message.extend(room_name);
                message.extend(desc_len.to_le_bytes());
                message.extend(room_desc);

                println!("[SERVER_MESSAGE] Sending room message");
                author.as_ref().write_all(&message).map_err(|err|{
                    println!("Couldn't send Room message to client, with error {}", err);
                })?;
            }
            //extensions not currently implemented
            Message::Version{ author, message_type, major_revision, minor_revision, ext_len: _, ext_list: _} => {
                println!("[MPSC RECEIVED] Version message from:  {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(major_revision.to_le_bytes());
                message.extend(minor_revision.to_le_bytes());
                message.extend(0u16.to_le_bytes());

                println!("[SERVER_MESSAGE] Sending Version message");
                author.as_ref().write_all(&message).map_err(|err| {
                    eprintln!("Couldn't send Version message to client, with error: {}", err);
                })?;
            }
           _ => {
                println!("[RECEIVED] unhandled message");
            }
        }
    }
}

//tcp receiver
fn handle_client(stream: Arc<TcpStream>, message: Sender<Message>) -> Result<()> {
    /***************** < server state params> *****************/
    // these will be defaults for each connecting client
    let stat_limit : u16 = 5000;
    let initial_points : u16 = 300;
    let mut game_started : bool = false;
    let mut player_joined : bool = false;
    /***************** < server state params> *****************/
    let mut character : Character = Character::new(stream.clone(), String::new(), String::new());

    if stream.peer_addr().is_err() {
        println!("Error: couldn't get client's peer address.");
        return Err(());
    }
    else{
        println!("New game connection from {:?}", stream.peer_addr().unwrap());
    }

    /****** <Preamble>: Shove a version & description at every client *****/
    let server_version = Message::Version{
        author: stream.clone(),
        message_type: MessageType::VERSION,
        major_revision: 2,
        minor_revision: 3,
        ext_len: 0,
        ext_list: Vec::new(),
    };
    println!("[MPSC SEND] Version message from {:?}", thread::current().id());
    message.send(server_version).map_err(|err| {
        println!("couldn't send Version message to client. Err was: {}", err);
        std::process::exit(1);
    })?;

    let game_info = Message::Game{
        author: stream.clone(),
        message_type: MessageType::GAME,
        initial_points,
        stat_limit,
        desc_len: WELCOME.len() as u16,
        game_desc: WELCOME.as_bytes().to_vec(),
    };
    println!("[MPSC SEND] Game message from {:?}", thread::current().id());
    message.send(game_info).map_err(|err| {
        println!("couldn't send Game message to client. Err was: {}", err);
    })?;

    /****** </Preamble>: Shove a version & description at every client *****/

    /***** <Main loop> read from stream & react to messaages ******/
    let mut reader = BufReader::new(stream.as_ref());
    let mut message_type = [0u8];
    let mut bufr: Vec<u8> = Vec::new();

    loop{
        reader.read_exact(&mut message_type).map_err(|err|{
            eprintln!("[GAME SERVER]: couldn't receive message; assuming client disconnect. Error wasa {:?}", err);
            let _ = Message::Leave{
                author: stream.clone(),
                message_type: MessageType::LEAVE,
            };
        })?;

        match message_type[0] {
            MessageType::CHARACTER => {
                println!("matched a character message");
                let mut message_data = [0u8; 47]; // 47 bytes + 1 (message type already read)
                reader.read_exact(&mut message_data).map_err(|err|{
                    println!("[GAME SERVER] Could not read character message; error was {err}");
                })?;

                // note on ranges -- we've already popped the first byte out of the stream, so
                // so we read protocol positions shifted 1 byte left (e.g., byte 1 in protocol
                // is now byte 0.
                let c_name   : String = String::from_utf8_lossy(&message_data[0..32]).to_string();
                let flags    : u8 = message_data[32];
                let attack   : u16 = u16::from_le_bytes([message_data[33], message_data[34]]);
                let defense  : u16 = u16::from_le_bytes([message_data[35], message_data[36]]);
                let regen    : u16 = u16::from_le_bytes([message_data[37], message_data[38]]);
                let health   : i16 = i16::from_le_bytes([message_data[39], message_data[40]]);
                let gold     : u16 = u16::from_le_bytes([message_data[41], message_data[42]]);
                let _room     : u16 = u16::from_le_bytes([message_data[43], message_data[44]]);
                let desc_len : usize = u16::from_le_bytes([message_data[45], message_data[46]]) as usize;

                let mut desc = vec![0u8; desc_len];
                reader.read_exact(&mut desc).map_err(|err|{
                    println!("[GAME SERVER] Could not read character description; error was {err}");
                })?;

                //notify client if supplied stats exceed maximum
                let points = attack + defense + regen;
                if points > initial_points {
                    println!("[GAME SERVER] Player connected with stats exceeding the value of {initial_points}; returning error");
                    let estr : String = String::from("Error: stats set too high; Attack, Defense, and Regen should not exceed initial_points");
                    let emsg = Message::Error {
                        author: stream.clone(),
                        message_type: MessageType::ERROR,
                        error_code: ErrorType::STAT_ERROR,
                        messaage_len: estr.len() as u16,
                        message: estr.into_bytes(),
                    };
                    message.send(emsg).map_err(|err| {
                        println!("Could not send error message to client {c_name}; Error was {err}");
                    })?;
                    continue;
                };

                //set stats & return character message
                if flags == PlayerFlags::ALL_FLAGS_SET || flags == PlayerFlags::NO_FLAGS_SET {
                    character.flags = PlayerFlags::IS_ALIVE | PlayerFlags::IS_READY;
                }
                //TODO: Handle reserved flags set??
                else{
                    character.flags = flags
                }
                character.name = if c_name == "" {String::from("DEFAULT MEAT")} else {c_name};
                character.desc = String::from_utf8_lossy(&desc).to_string();
                character.is_active = true;
                character.attack = attack;
                character.defense = defense;
                character.regen = regen;
                character.health = health;
                character.gold = gold;
                character.curr_room = 0;

                player_joined = true;

                //Send accept to client
                println!("[MPSC Send] Sending Accept message");
                let acceptmsg = Message::Accept {
                    author: stream.clone(),
                    message_type: MessageType::ACCEPT,
                    accepted_type: MessageType::CHARACTER,
                };
                message.send(acceptmsg).map_err(|err| {
                    println!("Could not send error message to client; Error was {err}");
                })?;
            }
            MessageType::START => {
                if game_started{
                    println!("[SERVER_MESSAGE] received 'Start' message, but game was already started. Doing nothing.");
                }
                if !player_joined{
                    println!("[SERVER MESSAGE] player with name {:?} attempted to start before character was accepted", character.name);
                    let estr : String = String::from("Error: your character has not been accepted to the server");
                    let emesg = Message::Error {
                        author: stream.clone(),
                        message_type: MessageType::ERROR,
                        error_code: ErrorType::NOT_READY,
                        messaage_len: estr.len() as u16,
                        message: estr.into_bytes(),
                    };
                    println!("[MPSC Send] Sending Error message");
                    message.send(emesg).map_err(|err| {
                        println!("Could not send error message to client; Error was {err}");
                    })?;
                    continue;
                }
                else {
                    println!("[MPSC Send] Sending Character message");
                    let mut namebuff = [0u8;32];
                    namebuff[..32].clone_from_slice(&character.name[0..32].as_bytes());
                    let cmesg = Message::Character {
                        author: stream.clone(),
                        message_type: MessageType::CHARACTER,
                        character_name: namebuff,
                        flags: character.flags,
                        attack: character.attack,
                        defense: character.defense,
                        regen: character.regen,
                        health: character.health,
                        gold: character.gold,
                        curr_room: character.curr_room,
                        desc_len: character.desc.len() as u16,
                        desc: character.desc.as_bytes().to_vec(),
                    };
                    message.send(cmesg).map_err(|err| {
                        println!("Could not send error message to client; Error was {err}");
                    })?;
                    println!("[MPSC Send] Sending Room message");
                    let rdesc : String = String::from("Here's a room description");
                    let rname : [u8;32] = [0u8;32];
                    let rmesg = Message::Room {
                        author: stream.clone(),
                        message_type: MessageType::ROOM,
                        room_number: 1,
                        room_name: rname,
                        desc_len: rdesc.len() as u16,
                        room_desc: rdesc.as_bytes().to_vec(),
                    };
                    message.send(rmesg).map_err(|err|{
                        println!("Could not send room message to clienbt; Error was {err}");
                    })?;

                    game_started = true;
                }
            }
            MessageType::CHANGEROOM => {}
            MessageType::FIGHT => {}

            MessageType::LEAVE => {
                if !player_joined{
                    println!("[SERVER_MESSAGE] Received 'leave' message, but no player joined. Doing nothing.");
                }
                else{
                    println!("[SERVER_MESSAGE] Player {:?} disconnected", character.name);
                    stream.shutdown(Shutdown::Both).expect("Could not close TCP stream");
                    break;
                }
            }
            MessageType::LOOT => {}
            MessageType::MESSAGE => {}
            MessageType::PVPFIGHT => {}
            MessageType::VERSION => {}

            /**************** < non-client message types>*******************/
            MessageType::ACCEPT => {
                println!("[SERVER_MESSAGE] The client sent an 'Accept' message; we're ignoring it");
            }
            MessageType::CONNECTION => {
                println!("[SERVER_MESSAGE] The client sent a 'Connection' message; we're ignoring it");
            }
            MessageType::ERROR => {
                println!("[SERVER_MESSAGE] The client sent an 'Error' message; we're ignoring it");
            }
            MessageType::ROOM => {
                println!("[SERVER_MESSAGE] The client sent a 'Room' message; we're ignoring it");
            }
            /**************** < non-client message types>*******************/

            _ => {
                println!(
                    "[SERVER_MESSAGE] The client sent an unknown message type, with id: {:?}. Ignoring message contents.",
                    message_type[0]
                );
            }
        }
       bufr.clear();
    }
    Ok(())
}
