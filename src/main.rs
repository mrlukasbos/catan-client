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

#[derive(Serialize, Deserialize)]
struct ServerInputEvent {
    model: String,
    attributes: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct Event {
    event_type: String,
    move_count: Option<u32>,
    message: String,
    player: u8,
    resources: Vec<String>,
    structures: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Game {
    move_count: Option<u32>,
    players: Vec<ServerInputPlayer>,
    status: String,
    board: Option<ServerInputBoard>,
    events: Vec<ServerInputEvent>,
    last_dice_throw: Option<u8>,
    phase: Option<String>,
    current_player: Option<u8>
}

impl Game {
    fn get_board(&self) -> Option<&Board> {
        if let Some(board) = &self.board {
            Some(&board.attributes)
        } else {
            None
        }
    }

    fn get_players(&self) -> Vec<&Player> {
        self.players.iter().map(|player_model| {
            &player_model.attributes
        }).collect()
    }
}

fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("localhost:10006")?;
    let mut buf_stream = BufStream::new(&stream);

    // connect as user 'Rust';
    buf_stream.write(b"Rust\r\n").unwrap();
    buf_stream.flush().unwrap();

    println!("Connected! Waiting for game to start...");

    loop {
        if let Some(input) = read_tcp_input(&mut buf_stream) {
            let response: ServerInput = serde_json::from_str(&input)?;

            match response.model.as_str() {
                "game"  => {
                    let game: Game = serde_json::from_value(response.attributes)?;
                },
                "response" => {
                    let server_response: ServerResponse = serde_json::from_value(response.attributes)?;
                    handle_server_response(&stream, &mut buf_stream, server_response)
                },
                _ => {
                    println!("Got something unknown");
                }
            };
        }
    }
}

fn handle_server_response(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>, server_response: ServerResponse) -> () {
    match server_response.code {
        0 => println!("Success!"),
        1 => {
            let id: i16 = server_response.additional_info.parse().unwrap_or(-1);
            println!("Our id is: {}", id)
        },
        102 => send_dummy_command(&stream, &mut buf_stream).unwrap(),
        _ => println!("We don't yet care about this: {}", server_response.code)
    }
}

/// Send a dummy command.
/// This command will build a street and a village at a hardcoded location
fn send_dummy_command(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>) -> Result<(), &'static str> {
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
    Ok(())
}

/// Reads the TCP input and extracts a json object from it.
/// Returns Some(String) if the string is a properly formatted json object,
/// otherwise returns None
fn read_tcp_input(buf_stream: &mut BufStream<&TcpStream>) -> Option<String> {
    let mut buffer = String::new();
    if let Ok(_buffer_size) = buf_stream.read_line(&mut buffer) {
        return Some(buffer)
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