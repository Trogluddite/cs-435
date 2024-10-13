use std::io::{BufReader, Write, Read};
use std::{result, thread};          // later: env
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream};

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
    const ACCEPT: u8 = 8;
    const CHANGEROOM: u8 = 2;
    const CHARACTER: u8 = 10;
    const CONNECTION: u8 = 13;
    const ERROR: u8 = 7;
    const FIGHT: u8 = 3;
    const GAME: u8 = 11;
    const LEAVE: u8 = 12;
    const LOOT: u8 = 5;
    const MESSAGE: u8 = 1;
    const ROOM: u8 = 9;
    const START: u8 = 6;
    const PVPFIGHT: u8 = 4;
    const VERSION: u8 = 14;
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
            Message::Version{ author, message_type, major_revision, minor_revision, ext_len: _, ext_list: _} => {
                println!("[MPSC RECEIVED] Version message from:  {:?}", author.peer_addr().unwrap());
                let mut message: Vec<u8> = Vec::new();
                message.push(message_type);
                message.extend(major_revision.to_le_bytes());
                message.extend(minor_revision.to_le_bytes());
                message.extend(0u16.to_le_bytes());

                author.as_ref().write_all(&message).map_err(|err| {
                    eprintln!("Couldn't send Version message to client, with error: {}", err);
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

                author.as_ref().write_all(&message).map_err(|err|{
                    println!("couldn't send Game message to client, with error {}", err);
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
    let initial_c_point_limit : u16 = 300;
    let initial_c_stat_limit : u16 = 500;
    let mut game_started : bool = false;
    let mut player_joined : bool = false;
    /***************** < server state params> *****************/
    let t_character : Character = Character::new(stream.clone(), String::new(), String::new());

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
        initial_points: 500,
        stat_limit: 300,
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

        // to be shaadowed in match arms
        //let message_to_send: Message;

        match message_type[0] {
            /*accept_msg => {
                println!("accept msg");
                continue;
            }*/

            MessageType::CHARACTER => {
                println!("matched a character message");
                let mut message_data = [0u8; 47]; // character message is 48 bytees;
                reader.read_exact(&mut message_data).map_err(|err|{
                    println!("[GAME SERVER] Could not read character message; error was {err}");
                })?;

                let c_name = String::from_utf8_lossy(&message_data[1..31]);
                let flags    : u8 = message_data[32];
                let attack   : u16 = u16::from_le_bytes([message_data[33], message_data[34]]);
                let defense  : u16 = u16::from_le_bytes([message_data[35], message_data[36]]);
                let regen    : u16 = u16::from_le_bytes([message_data[37], message_data[38]]);
                let health   : i16 = i16::from_le_bytes([message_data[39], message_data[40]]);
                let gold     : u16 = u16::from_le_bytes([message_data[41], message_data[42]]);
                let room     : u16 = u16::from_le_bytes([message_data[43], message_data[44]]);
                let desc_len : usize = u16::from_le_bytes([message_data[45], message_data[46]]) as usize;
                let mut desc : Vec<u8> = message_data[47..(47 + desc_len)].to_vec();
                reader.read_exact(&mut desc).map_err(|err|{
                    println!("[GAME SERVER] Could not read description; error was {err}");
                })?;

                println!("name: {c_name}");
                println!("flags: {:#010b}", flags);
                println!("attack: {attack}");
                println!("defense: {defense}");
                println!("regen: {regen}");
                println!("health: {health}");
                println!("gold: {gold}");
                println!("room: {room}");
                println!("desc_len: {desc_len}");
                let s_desc = String::from_utf8_lossy(&desc);
                println!("Description: {s_desc}");
                println!("[GAME SERVER] player {c_name} connected");
            }

            MessageType::START => {
                println!("got a start message");
            }

            MessageType::ACCEPT => {}
            MessageType::CHANGEROOM => {}
            MessageType::CONNECTION => {}
            MessageType::ERROR => {}
            MessageType::FIGHT => {}
            MessageType::GAME => {}
            MessageType::LEAVE => {}
            MessageType::LOOT => {}
            MessageType::MESSAGE => {}
            MessageType::ROOM => {}
            MessageType::PVPFIGHT => {}
            MessageType::VERSION => {}

            _ => {}
        }
 
    /*
    let c_desc = String::from("I am definitely not a plumber in search of a princess");
    let c_name = String::from("ItsaMe,Oiram"); //placeholder name, definitely not Mario
    let mut name_c_array = [0u8; 32];           // pre-pad name array
    // shove whatever will fit into name_c_array
    name_c_array[..c_name.len()].copy_from_slice(c_name.as_bytes());
    let character_info = Message::Character {
        author: stream.clone(),
        message_type: MessageType::new().character,
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

    let r_desc = String::from("this is a room mkay");
    let mut r_name_arr = [0u8; 32];
    let r_name = String::from("MURDERSH3D");
    r_name_arr[..r_name.len()].copy_from_slice(r_name.as_bytes());
    let conn_info = Message::Connection { 
        author: stream.clone(),
        message_type: MessageType::new().connection,
        room_number: 0,
        room_name: r_name_arr,
        desc_len: r_desc.len() as u16,
        room_desc: r_desc.as_bytes().to_vec(),
    };


    println!("[SENT] Character to {:?}", stream.peer_addr().unwrap());
    message.send(character_info).map_err(|err| {
        println!("couldn't send Character message to the client. Err was: {}", err);
    })?;

    println!("[SENT] Conn to {:?}", stream.peer_addr().unwrap());
    message.send(conn_info).map_err(|err| {
        println!("couldn't send Connection message to the client. Err was: {}", err);
    })?;*/


        bufr.clear();
    }
    //Ok(())
}


/*
* send character  message example
*
                let c_desc = String::from("I am definitely not a plumber in search of a princess");
                let c_name = String::from("ItsaMe,Oiram"); //placeholder name, definitely not Mario
                let mut name_c_array = [0u8; 32];           // pre-pad name array
                // shove whatever will fit into name_c_array
                name_c_array[..c_name.len()].copy_from_slice(c_name.as_bytes());
                let character_info = Message::Character {
                    author: stream.clone(),
                    message_type: MessageType::new().character,
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

                message.send(character_info).map_err(|err|{
                    eprintln!("couldn't do the thing, err was: {:?}",err);
                })?;
                continue;
            }

*/

