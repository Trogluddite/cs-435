use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    let mut client_writer: BufWriter<&TcpStream> = BufWriter::new(&stream);

    let x:[u8;7] = [14, 2, 3, 0, 0, 0, 0];
    let y:[u8;7] = [123, 7, 6, 5, 4, 3, 1];
    client_writer.write_all(&x);

    client_writer.write_all(&y);

}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8123")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream.unwrap());
    }
    Ok(())
}
