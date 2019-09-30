use std::io::prelude::*;
use std::net::TcpStream;
use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::{thread, time};

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

    let mut stream = TcpStream::connect("localhost:10006")?;
    stream.write(b"Rust\r\n")?; // register as user Rust
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
        // Read a line and put it in the buffer
        let mut reader = BufReader::new(&stream);
        let mut buffer = String::new();

        if let Ok(_buffer_size) = reader.read_line(&mut buffer) {
            if !buffer.is_empty() {
                println!("{}", &buffer);

                // parse the message we got
                let start = buffer.find('{').unwrap_or_else(|| {
                    buffer.len()
                });

                let model = &buffer[start..]; // slice the first two chars



                let response: ServerResponse = serde_json::from_str(&model)?; 
                print!("response model {} ", response.model);


                // from the model we can derive the type of the attributes which we can put in the corresponding object. 

                if response.model == "board" {
                    // send the command
                    let commands = vec!(&build_village, &build_street);
                    serde_json::to_writer(&stream, &commands)?;
                    stream.write(b"\r\n")?; // send a newline to indicate we are done

                    // show what we have sent
                    let output_string = serde_json::to_string(&commands)?;
                    print!("Response sent: {}", &output_string);
                }
            }
        }
    }
} // the stream is closed here


// fn readTCPInput() {

// }