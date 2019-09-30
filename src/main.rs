use std::io::prelude::*;
use bufstream::BufStream;

use std::net::TcpStream;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BuildCommand {
    structure: String,
    location: String
}

#[derive(Serialize, Deserialize)]
struct ServerResponse {
    model: String,
    attributes: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct ServerMessage {
    message: String,
}

fn main() -> std::io::Result<()> {

    let stream = TcpStream::connect("localhost:10006")?;
    let mut buf_stream = BufStream::new(&stream);

    buf_stream.write(b"Rust\r\n")?; // register as user Rust
    buf_stream.flush()?;
    println!("Connected! Waiting for game to start...");

    let build_village = BuildCommand {
        structure: String::from("village"),
        location: String::from("([1,2],[2,1],[2,2])"),
    };

    let build_street = BuildCommand {
        structure: String::from("street"),
        location: String::from("([1,2],[2,1])"),
    };


    loop {
        if let Some(response) = read_tcp_input(&mut buf_stream) {

            let response: ServerResponse = serde_json::from_str(&response)?; 
            println!("response model {} ", response.model);

            // from the model we can derive the type of the attributes which we can put in the corresponding object. 
            match response.model.as_str() {
                "board"  => {
                    // send the command
                    let commands = vec!(&build_village, &build_street);
                    serde_json::to_writer(&stream, &commands)?;
                    buf_stream.write(b"\r\n")?; // send a newline to indicate we are done
                    buf_stream.flush()?;

                    // show what we have` sent
                    let output_string = serde_json::to_string(&commands)?;
                    println!("Response sent: {}", &output_string);
                },
                "response" => {
                    println!("Got a reponse");
                },
                _ => {
                    println!("Got something unknown");

                }
            };
        }
    }
} // the stream is closed here


/// Reads the TCP input and extracts a json object from it.
/// Returns Some(String) if the string is a properly formatted json object,
/// otherwise returns None
fn read_tcp_input(buf_stream: &mut BufStream<&TcpStream>) -> Option<String> {
    let mut buffer = String::new();
    if let Ok(_buffer_size) = buf_stream.read_line(&mut buffer) {
        let model = String::from(&buffer[..]);
        return Some(model)
    }
    None
}