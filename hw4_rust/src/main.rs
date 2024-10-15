use std::{
    io::{BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

struct MessageType;
impl MessageType{
    const C_SET_VALUE:    u8 = 1;
    const C_CONV_S_TO_D:  u8 = 2;
    const C_CONV_D_TO_S:  u8 = 3;
    const S_SET_OK:       u8 = 4;
    const S_CONV_RES:     u8 = 5;
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5006").unwrap();
    let mut sheep_price : f64 = 1.0;

    for stream in listener.incoming(){
        let stream = stream.unwrap();
        handle_connection(stream, &mut sheep_price);
    }
}

fn handle_connection(mut stream: TcpStream, sheep_price: &mut f64){
    let mut bufreader = BufReader::new(&mut stream);
    let mut m_type = [0u8];
    bufreader.read_exact(&mut m_type).map_err(|err|{
        println!("Couldn't read type from buffer; error was {err}");
    }).ok();


    //todo: bounds checking, chump
    match m_type[0] {
        MessageType::C_SET_VALUE => {
            println!("[SERVER] Got a C_SET_VALUE message (byte with value {:?})", MessageType::C_SET_VALUE);
            let mut data: [u8;8] = [0u8;8];
            bufreader.read_exact(&mut data).map_err(|err|{
                println!("[SERVER] Couldn't read data for C_SET_VALUE message, error was {err}")
            }).ok();
            *sheep_price = f64::from_le_bytes(data[0..8].try_into().unwrap());
            println!("[SERVER] New sheep price set to {:?}", *sheep_price);
            
            //just one byte -- reference to single-element slice containing message type
            stream.write( &[MessageType::S_SET_OK;1] ).ok();
        },
        MessageType::C_CONV_S_TO_D => {
            println!("[SERVER] Got a C_CONV_S_TO_D message (byte with value {:?})", MessageType::C_CONV_S_TO_D);
            let mut data: [u8;8] = [0u8;8];
            bufreader.read_exact(&mut data).map_err(|err|{
                println!("[SERVER] Couldn't read data for C_CONV_S_TO_D message, error was {err}")
            }).ok();

            let num_sheep : f64 = f64::from_le_bytes(data[0..8].try_into().unwrap());
            let curr_price = *sheep_price; //just because the deref looks wonky
            let tot_cost = num_sheep * curr_price;

            let mut outdata: [u8;9] = [0u8;9];
            outdata[0] = MessageType::S_CONV_RES;
            let floatbytes : [u8;8] = f64::to_le_bytes(tot_cost);
            let mut i = 1;
            for byte in floatbytes.iter(){
                outdata[i] = *byte;
                i = i+1;
            }
            stream.write(&outdata).ok();
            println!("[SERVER] Sent total cost (${tot_cost}) to client");
        },
        MessageType::C_CONV_D_TO_S => {
            println!("[SERVER] got a C_CONV_D_TO_S message (byte with value {:?})", MessageType::C_CONV_D_TO_S);
            let mut data: [u8;8] = [0u8;8];
            bufreader.read_exact(&mut data).map_err(|err|{
                println!("[SERVER] Couldn't read data for C_CONV_S_TO_D message, error was {err}")
            }).ok();

            let how_many_monies : f64 = f64::from_le_bytes(data[0..8].try_into().unwrap());
            let curr_price = *sheep_price;
            let num_sheep = how_many_monies / curr_price;

            let mut outdata: [u8;9] = [0u8;9];
            outdata[0] = MessageType::S_CONV_RES;
            let floatbytes: [u8;8] = f64::to_le_bytes(num_sheep);
            let mut i = 1;
            for byte in floatbytes.iter(){
                outdata[i] = *byte;
                i = i+1;
            }
            stream.write(&outdata).ok();
            println!("[SERVER] sent number of sheep ({num_sheep}) to client");
        },
        _ => {
            println!("[SERVER] Got an unknown message type");
        }
    }
}
