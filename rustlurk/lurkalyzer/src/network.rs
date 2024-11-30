#[allow(unused_imports)] //FIXME: later
use std::io::{BufReader, Write, Read};
use std::{result, thread};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Reciever, Sender};
use std::net::{TcpListener, TcpStream, Shutdown};

fn main() -> Result<()> {
    println!("Hello!");
}
