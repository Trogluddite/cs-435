use std::io::{BufReader, Read, Write};
use std::{result, thread};          // later: env
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::net::{TcpListener, TcpStream, Shutdown};

use::lurk_messages::{ErrorType, Message, MessageType};
use::lurk_gamestate::{Character, CharacterFlags, GameState, Room};

type Result<T> = result::Result<T, ()>;

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


fn main() -> Result<()> {
    let address = format!("{}:{}",SERVER_ADDRESS, SERVER_PORT);
    let listener = TcpListener::bind(&address).map_err( |_err| {
        println!("[SERVER MESSAGE]: Error: could not bind to address {address}");
    })?;
    println!("[SERVER MESSAGE]: running on socket: {address}");

    /************* Add some rooms *****************/
    let joy_room : Room = Room::new(
        0,
        String::from("Joy Room"),
        String::from("A realm filled with happiness, rainbows and unicorns that poop cotton candy"),
        vec![1,2,3,4],
    );
    let fear_tomb : Room = Room::new(
        1,
        String::from("Fear Tomb"),
        String::from("A terrifying vault redolent with unspeakable horrors. Someone has microwaved fish here."),
        vec![0],
    );
    let goblin_bathhouse : Room = Room::new(
        2,
        String::from("Goblin Bathhouse"),
        String::from("Boiling vats of goblin-slime have been super-heated for the pleasure of green monsters."),
        vec![0],
    );
    let treasurebox : Room = Room::new(
        3,
        String::from("Treasure Box"),
        String::from("*Obviously* you want to go here, right?"),
        vec![0],
    );
    let doom_chute : Room = Room::new(
        4,
        String::from("Doom Chute"),
        String::from("This appears to be a slide covered in acid. Why would anyone build such a terrible thing?"),
        vec![0],
    );
    let mut game_state = GameState::new();
    game_state.add_room(joy_room);
    game_state.add_room(fear_tomb);
    game_state.add_room(goblin_bathhouse);
    game_state.add_room(treasurebox);
    game_state.add_room(doom_chute);
    /************* end of add some rooms *****************/

    /************* Add some monsters *****************/
    let monster_names = [
        String::from("Sad Goblin"),
        String::from("Shrieking Wombat"),
        String::from("Tiny Fur Demon"),
        String::from("Cowardly Kitten"),
    ];
    let monster_descriptions = [
        String::from("A sobbing monstrosity. Have mercy: He hates his job."),
        String::from("His poops are cubes, and he won't stop talking about it."),
        String::from("Is it a cat? Is it a bat? It's some kind of mammal and it sure is mad about that."),
        String::from("She's biting my toes OMG."),
    ];

    for i in 0..monster_names.len(){
        let mut monster = Character::new_monster(
            String::clone(&monster_names[i]),
            String::clone(&monster_descriptions[i]),
            25, 25, 25, 25, //generic stats for now
            100,            // some gold to loot
            (i + 1) as u16, //just pepper the first four rooms with some monsters,
        );
        game_state.add_character(&mut monster);
    }
    /************* end of Add some monsters *****************/


    let game_state = Arc::new(Mutex::new(game_state));

    let (sender, receiver) = channel();
    let receiver = Arc::new(Mutex::new(receiver));  //shadow 'receiver' w/ ARC & mutex
    thread::spawn(move || handle_mpsc_thread_messages(receiver)); // spawn server thread

    //listen for incoming connections
    for stream in listener.incoming() {
        match stream{
            Ok(stream) => {
                let stream = Arc::new(stream);
                let sender = sender.clone();
                let game_state = Arc::clone(&game_state);
                println!("[SERVER MESSAGE]: New connection, spawning thread for client {:?}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_client(stream, sender, game_state));
            }
            Err(e) => {
                println!("Error: {}",e);
            }
        }
    }
    Ok(())
}

//thread receiver -- MPSC 'sends' will be received here
fn handle_mpsc_thread_messages(receiver: Arc<Mutex<Receiver<Message>>>) -> Result<()> {
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
                println!("[MPSC RECEIVED] Accept message from: {:?}", author.peer_addr().unwrap());
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
            Message::Error { author, message_type, error_code, message_len, message } => {
                println!("[MPSC RECEIVED] Error message from: {:?}", author.peer_addr().unwrap());
                let mut send_message: Vec<u8> = Vec::new();
                send_message.push(message_type);
                send_message.push(error_code);
                send_message.extend(message_len.to_le_bytes());
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
fn handle_client(
    stream: Arc<TcpStream>,
    message: Sender<Message>,
    game_state: Arc<Mutex<GameState>>) -> Result<()> {
    /***************** < server state params> *****************/
    // these will be defaults for each connecting client
    let stat_limit : u16 = 5000;
    let initial_points : u16 = 300;
    let mut game_started : bool = false;
    let mut player_joined : bool = false;
    /***************** < server state params> *****************/

    let mut character : Character = Character::new(String::new(), String::new());
    println!("[SERVER_MESSAGE] Adding character to hashmap");
    let character_ref = &mut character;

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
                let mut message_data = [0u8; 47]; // 47 bytes + 1 (message type already read)
                reader.read_exact(&mut message_data).map_err(|err|{
                    println!("[GAME SERVER] Could not read character message; error was {err}");
                })?;

                // note on ranges -- we've already popped the first byte out of the stream, so
                // so we read protocol positions shifted 1 byte left (e.g., byte 1 in protocol
                // is now byte 0.
                let c_name : String = String::from_utf8_lossy(&message_data[0..32]).to_string();
                match game_state.lock().unwrap().character_map.get(&c_name){
                //match charater_map.lock().unwrap().get(&c_name){
                    Some(_) => {
                        println!("[SERVER_MESSAGE] character {c_name} already joined!");
                        let estr : String = String::from("Error: Character already joined");
                        let emsg = Message::Error {
                            author: stream.clone(),
                            message_type: MessageType::ERROR,
                            error_code: ErrorType::PLAYER_EXISTS,
                            message_len: estr.len() as u16,
                            message: estr.into_bytes(),
                        };
                        message.send(emsg).map_err(|err| {
                            println!("Could not send error message to client {c_name}; Error was {err}");
                        })?;
                        continue;
                    },
                    None => println!("[SERVER_MESSAGE] character{c_name} joining")
                }
                let flags    : u8 = message_data[32];
                let attack   : u16 = u16::from_le_bytes([message_data[33], message_data[34]]);
                let defense  : u16 = u16::from_le_bytes([message_data[35], message_data[36]]);
                let regen    : u16 = u16::from_le_bytes([message_data[37], message_data[38]]);
                let health   : i16 = i16::from_le_bytes([message_data[39], message_data[40]]);
                let gold     : u16 = u16::from_le_bytes([message_data[41], message_data[42]]);
                let _room    : u16 = u16::from_le_bytes([message_data[43], message_data[44]]);
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
                        message_len: estr.len() as u16,
                        message: estr.into_bytes(),
                    };
                    message.send(emsg).map_err(|err| {
                        println!("Could not send error message to client {c_name}; Error was {err}");
                    })?;
                    continue;
                };

                //set stats & return character message
                if flags == CharacterFlags::ALL_FLAGS_SET || flags == CharacterFlags::NO_FLAGS_SET {
                    character_ref.flags = CharacterFlags::IS_ALIVE | CharacterFlags::IS_READY;
                }
                //TODO: Handle reserved flags set??
                else{
                    character_ref.flags = flags
                }
                character_ref.name = if c_name == "" {String::from("DEFAULT MEAT")} else {c_name};
                character_ref.desc = String::from_utf8_lossy(&desc).to_string();
                character_ref.is_active = true;
                character_ref.attack = attack;
                character_ref.defense = defense;
                character_ref.regen = regen;
                character_ref.health = health;
                character_ref.gold = gold;
                character_ref.curr_room = 0;
                player_joined = true;

                game_state.lock().unwrap().add_character(character_ref);
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
                    println!("[SERVER MESSAGE] player with name {:?} attempted to start before character was accepted", character_ref.name);
                    let estr : String = String::from("Error: your character has not been accepted to the server");
                    let emesg = Message::Error {
                        author: stream.clone(),
                        message_type: MessageType::ERROR,
                        error_code: ErrorType::NOT_READY,
                        message_len: estr.len() as u16,
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
                    namebuff[..32].clone_from_slice(character_ref.name[0..32].as_bytes());
                    let cmesg = Message::Character {
                        author: stream.clone(),
                        message_type: MessageType::CHARACTER,
                        character_name: namebuff,
                        flags: character_ref.flags,
                        attack: character_ref.attack,
                        defense: character_ref.defense,
                        regen: character_ref.regen,
                        health: character_ref.health,
                        gold: character_ref.gold,
                        curr_room: character_ref.curr_room,
                        desc_len: character_ref.desc.len() as u16,
                        desc: character_ref.desc.as_bytes().to_vec(),
                    };
                    message.send(cmesg).map_err(|err| {
                        println!("Could not send error message to client; Error was {err}");
                    })?;

                    println!("[MPSC Send] Sending Room message");
                    let game_state = game_state.lock().unwrap();
                    let start_roomnum = character_ref.curr_room;
                    let curr_room = game_state.room_hashmap.get(&start_roomnum).unwrap(); //may panic
                    let mut room_name = [0u8;32];
                    room_name[..curr_room.name.len()].clone_from_slice(curr_room.name.as_bytes());

                    let room_mesg = Message::Room {
                        author: stream.clone(),
                        message_type: MessageType::ROOM,
                        room_number: character_ref.curr_room,
                        room_name,
                        desc_len: curr_room.desc.len() as u16,
                        room_desc: curr_room.desc.as_bytes().to_vec(),
                    };
                    message.send(room_mesg).map_err(|err|{
                        println!("Could not send room message to client; Error was {err}");
                    })?;
                    /***** end room message *****/


                    println!("[MPSC Send] Sending connection message for connected room_ids {:?}", curr_room.connections);
                    for room_id in &curr_room.connections{
                        let conn_room = game_state.room_hashmap.get(&room_id).unwrap();
                        let mut conn_room_name = [0u8;32];
                        conn_room_name[..conn_room.name.len()].clone_from_slice(conn_room.name.as_bytes());
                        let conn_mesg = Message::Connection {
                            author: stream.clone(),
                            message_type: MessageType::CONNECTION,
                            room_number: *room_id,
                            room_name: conn_room_name,
                            desc_len: conn_room.desc.len() as u16,
                            room_desc: conn_room.desc.as_bytes().to_vec(),
                        };
                        message.send(conn_mesg).map_err(|err|{
                            println!("Could not connection message to client; Error was {err}");
                        })?;
                    }
                    /***** end connection messages *****/
                    game_started = true;
                }
            }
            MessageType::CHANGEROOM => {
                let mut t_room_buff = [0u8; 2]; // 2 bytes for u16
                reader.read_exact(&mut t_room_buff).map_err(|err|{
                    println!("[GAME SERVER] Could ChangeRoom message; error was {err}");
                })?;
                let target_room = u16::from_le_bytes([t_room_buff[0], t_room_buff[1]]);
                //TODO: guard on invalid room

                let mut game_state = game_state.lock().unwrap();
                let character : &mut Character = game_state.character_map.get_mut(&character_ref.name).unwrap();
                //TODO: This is a dumb borrow-checker workaround ... I couldn't modify the
                //game-state object the way I wanted :/
                character.curr_room = target_room;
                character_ref.curr_room = target_room;
                /********** Send Room message ***********/
                println!("[MPSC Send] Sending Room message");
                let character_roomnum = target_room;
                let curr_room = game_state.room_hashmap.get(&character_roomnum).unwrap(); //may panic
                let mut room_name = [0u8;32];
                room_name[..curr_room.name.len()].clone_from_slice(curr_room.name.as_bytes());

                let room_mesg = Message::Room {
                    author: stream.clone(),
                    message_type: MessageType::ROOM,
                    room_number: character_ref.curr_room,
                    room_name,
                    desc_len: curr_room.desc.len() as u16,
                    room_desc: curr_room.desc.as_bytes().to_vec(),
                };
                message.send(room_mesg).map_err(|err|{
                    println!("Could not send room message to client; Error was {err}");
                })?;
                /***** end room message *****/

                /********** Send Connection messages ***********/
                println!("[MPSC Send] Sending connection messages for connected room_ids {:?}", curr_room.connections);
                for room_id in &curr_room.connections{
                    let conn_room = game_state.room_hashmap.get(&room_id).unwrap();
                    let mut conn_room_name = [0u8;32];
                    conn_room_name[..conn_room.name.len()].clone_from_slice(conn_room.name.as_bytes());
                    let conn_mesg = Message::Connection {
                        author: stream.clone(),
                        message_type: MessageType::CONNECTION,
                        room_number: *room_id,
                        room_name: conn_room_name,
                        desc_len: conn_room.desc.len() as u16,
                        room_desc: conn_room.desc.as_bytes().to_vec(),
                    };
                    message.send(conn_mesg).map_err(|err|{
                        println!("Could not connection message to client; Error was {err}");
                    })?;
                }
                /***** end connection messages *****/

                /********** Send Character messages ***********/
                println!("[MPSC Send] Sending Character messages");
                for character in game_state.character_map.keys(){
                    let character = game_state.character_map.get(character).unwrap();
                    if character.curr_room != curr_room.id_num{
                        continue;
                    }
                    let mut namebuff = [0u8;32];
                    namebuff[..character.name.len()].clone_from_slice(character.name[..character.name.len()].as_bytes());
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
                }
                /********** End of Send Character messages ***********/
            }
            MessageType::FIGHT => {
                println!("Got fight message");
                let game_state = &mut game_state.lock().unwrap();
                let curr_room = character_ref.curr_room;
                let mut monster_hits : Vec<u16> = Vec::new();

                for (k,v) in &mut game_state.character_map{
                    if (v.flags & CharacterFlags::IS_MONSTER) == CharacterFlags::IS_MONSTER{
                        if v.curr_room != curr_room {
                            println!("character {} is not in room {}", k, curr_room);
                            continue;
                        }
                        if (v.flags & CharacterFlags::IS_ALIVE) != CharacterFlags::IS_ALIVE{
                            continue;
                        }
                        else{
                            let (mut attack_remain,a_underflow) = v.attack.overflowing_sub(character_ref.defense);
                            if a_underflow{
                                attack_remain = 0;
                            }
                            monster_hits.push(attack_remain);
                            let (mut hit_minus_def, underflow) = v.defense.overflowing_sub(character_ref.attack);
                            if underflow {
                                hit_minus_def = 0;
                            }
                            println!("health before: {}",v.health);
                            println!("v.health: {}, hit_minus_def: {}",v.health, hit_minus_def);
                            v.health = v.health - hit_minus_def as i16;
                            if v.health <= 0 {
                                v.flags = v.flags & !CharacterFlags::IS_ALIVE;
                            }
                            println!("health after: {}", v.health);
                        }
                    }

                }
                let hit_sum : u16 = monster_hits.iter().sum();
                character_ref.health = character_ref.health - hit_sum as i16;
                game_state.character_map.get_mut(&character_ref.name).unwrap().health = character_ref.health;
                if character_ref.health <= 0{
                    character_ref.flags = character_ref.flags & !CharacterFlags::IS_ALIVE;
                    game_state.character_map.get_mut(&character_ref.name).unwrap().flags = character_ref.flags;
                }

                /********** Send Character messages ***********/
                println!("[MPSC Send] Sending Character messages");
                for character in game_state.character_map.keys(){
                    let character = game_state.character_map.get(character).unwrap();
                    if character.curr_room != character_ref.curr_room{
                        println!("Character {} is not in the player's current room", character.name);
                        continue;
                    }
                    let mut namebuff = [0u8;32];
                    namebuff[..character.name.len()].clone_from_slice(character.name[..character.name.len()].as_bytes());
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
                }
                /********** End of Send Character messages ***********/

            }
            MessageType::LEAVE => {
                if !player_joined{
                    println!("[SERVER_MESSAGE] Received 'leave' message, but no player joined. Doing nothing.");
                }
                else{
                    println!("[SERVER_MESSAGE] Player {:?} disconnected", character_ref.name);
                    stream.shutdown(Shutdown::Both).expect("Could not close TCP stream");
                    break;
                }
            }
            MessageType::LOOT => {}
            MessageType::MESSAGE => {}
            MessageType::PVPFIGHT => {
                println!("[SERVER_MESSAGE] snding PVP Disabled message");
                let estr : String = String::from("Error: PVP_DISABLED");
                let emsg = Message::Error {
                    author: stream.clone(),
                    message_type: MessageType::ERROR,
                    error_code: ErrorType::PVP_DISABLED,
                    message_len: estr.len() as u16,
                    message: estr.into_bytes(),
                };
                message.send(emsg).map_err(|err| {
                    println!("Could not send error message; Error was {err}");
                })?;
            }

            /**************** < non-client message types>*******************/
            MessageType::VERSION => {
                println!("[SERVER_MESSAGE] The client sent a 'Version' message; we're ignoring it");
            }
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
