use std::io::{BufWriter, Write};
use std::net::{TcpListener, TcpStream};

fn handle_client(stream: TcpStream) {
    let mut client_writer: BufWriter<&TcpStream> = BufWriter::new(&stream);

    let x:[u8;7] = [14, 2, 3, 0, 0, 0, 0];

    let y = [7, 4, 1, 0];
    let err = "A SAD HAS HAPPEN";

    let _ = client_writer.write_all(&x);
    let _ = client_writer.write_all(&y);
    let _ = client_writer.write_all(&err.as_bytes());

}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8123")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream.unwrap());
    }
    Ok(())
}
