use std::io::prelude::*;
use bufstream::BufStream;
use std::net::TcpStream;
use serde::{Deserialize, Serialize};

use crate::server::*;
use crate::board::*;

mod board;
mod server;

#[derive(Serialize, Deserialize)]
struct BuildCommand {
    structure: String,
    location: String,
}

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("localhost:10006")?;
    let mut buf_stream = BufStream::new(&stream);

    // connect as user 'Rust';
    let name = String::from("Rust");
    transmit(&mut buf_stream, &stream, &name);

    println!("Connected! Waiting for game to start...");

    loop {
        if let Some(input) = read_tcp_input(&mut buf_stream) {

            // get the input as ServerInput to extract the model type
            // from the model we can derive the type of the attributes which we can put in the corresponding object.
            let response: ServerInput = serde_json::from_str(&input)?; 

            match response.model.as_str() {
                "board"  => {
                    let server_board: Board = serde_json::from_value(response.attributes)?;


                },
                "response" => {

                    // extract the response data
                    let server_response: ServerResponse = serde_json::from_value(response.attributes)?;

                    println!("Response: {} {} {} {}", server_response.code, server_response.title, server_response.description, server_response.additional_info);

                    if server_response.code == 102 {


                        // create a dummy command
                        let build_village = BuildCommand {
                            structure: String::from("village"),
                            location: String::from("([1,2],[2,1],[2,2])"),
                        };

                        let build_street = BuildCommand {
                            structure: String::from("street"),
                            location: String::from("([1,2],[2,1])"),
                        };
                        let commands = vec!(&build_village, &build_street);

                        transmit(&mut buf_stream, &stream, &commands);

                        // show what we have sent
                        let output_string = serde_json::to_string(&commands)?;
                        println!("Response sent: {}", &output_string);
                    }
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

/// Transmit a JSON object over the TCP connection and append a newline
fn transmit<T: ?Sized>(buf_stream: &mut BufStream<&TcpStream>, stream: &TcpStream, value: &T) -> Result<(), &'static str> where T: Serialize {
    serde_json::to_writer(stream, value).unwrap();
    buf_stream.write(b"\r\n").unwrap(); // send a newline to indicate we are done
    buf_stream.flush().unwrap();
    Ok(())
}