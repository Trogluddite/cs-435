use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn say_helo(stream : &mut TcpStream){
    stream.write_all(format!("{}", "plz to be giving your name: ").as_bytes());
}

fn handle_client(stream: TcpStream, clients: Arc<Mutex<HashMap<String, TcpStream>>>) {
    let mut reader: BufReader<&TcpStream> = BufReader::new(&stream);

    let mut username: String = String::new();
    if reader.read_line(&mut username).is_err() {
        return;
    }
    let username: String = username.trim().to_string();

    println!("{} connected", username);

    clients.lock().unwrap().insert(username.clone(), stream.try_clone().unwrap());

    loop {
        let mut msg: String = String::new();
        if reader.read_line(&mut msg).is_err() {
            break;
        }
        let msg: &str = msg.trim();

        if msg.is_empty() {
            continue;
        }

        println!("{}: {}", username, msg);

        for (client, client_stream) in clients.lock().unwrap().iter() {
            if *client != username {
                let mut client_writer: BufWriter<&TcpStream> = BufWriter::new(client_stream);
                if client_writer.write_all(format!("{} said: {}\n", username, msg).as_bytes()).is_err() {
                    break;
                }
                if client_writer.flush().is_err() {
                    break;
                }
            }
        }
    }

    println!("{} disconnected", username);
    clients.lock().unwrap().remove(&username);
}


fn main() {
    let clients: Arc<Mutex<HashMap<String, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let listener: TcpListener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                say_helo(&mut stream);
            }
            Err(e) => { /* connection failed */ }
        }
    }
    Ok(());

    println!("Server running on 0.0.0.0:8080");

    for stream in listener.incoming() {
        println!("incomming!\n");

        let clients: Arc<Mutex<HashMap<String, TcpStream>>> = Arc::clone(&clients);

        thread::spawn(move || {
            handle_client(stream.unwrap(), clients);
        });
    }
}

