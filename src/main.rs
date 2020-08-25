use std::io::prelude::*;
use bufstream::BufStream;
use std::net::TcpStream;
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;
use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;
use std::{thread, time};

use crate::server::*;
use crate::board::*;
use crate::commands::*;

mod board;
mod server;
mod commands;

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
    events: Option<Vec<ServerInputEvent>>,
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

    // returns the player belonging to this codebase, if it exists in the data
    fn me(&self) -> Option<&Player> {
        self.get_players().into_iter().find(|p| { p.is_me() })
    }

    fn get_player_by_id(&self, id: usize) -> Option<&Player> {
        self.get_players().into_iter().find(|p| { p.id == id })
    }
}

fn main() -> std::io::Result<()> {

    let mut stream;

    loop {
        stream = TcpStream::connect("localhost:10006")?;
        let mut buf_stream = BufStream::new(&stream);

        let names = vec!("Luke Skywalker", "Darth Vader", "Yoda", "Obi-Wan Kenobi", "Han Solo", "Leia Skywalker", "Anakin Skywalker", "Mace Windu");
        let name = String::from(names.choose(&mut rand::thread_rng()).unwrap().clone()) + "\r\n";

        // connect as user with random name
        buf_stream.write(name.as_bytes()).unwrap();
        buf_stream.flush().unwrap();

        println!("Connected as {}Waiting for game to start...", name);
        let mut game: Option<Game> = None;
        loop {

            // limit speed of rust
            let sleep_time = time::Duration::from_millis(1000);
            thread::sleep(sleep_time);

            if let Some(input) = read_tcp_input(&mut buf_stream) {
            
                let response: ServerInput  = match serde_json::from_str(&input) {
                    Ok(response)  => response,
                    Err(_) => {
                        break;
                    },
                };

                match response.model.as_str() {
                    "game"  => {
                        let val = serde_json::from_value(response.attributes)?;
                        game = Some(val)
                    },
                    "response" => {
                        // println!("Received input: {}", &input);
                        let server_response: ServerResponse = serde_json::from_value(response.attributes)?;
                        if let Some(g) = &game {
                            handle_server_response(&stream, &mut buf_stream, server_response, &g)
                        }
                    },
                    _ => {
                        println!("Got something unknown");
                    }
                };
            }
        }
    }
}

#[derive(FromPrimitive)]
enum ResponseCode {
    Ok = 0,
    IdAcknowledgment = 1,
    TradeRequest = 100,
    BuildRequest = 101,
    InitialBuildRequest = 102,
    MoveBanditRequest = 103,
    ForceDiscardRequest = 104,
}

// Handles a server response with either a print statement in the console or a transmission back. 
fn handle_server_response(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>, server_response: ServerResponse, game : &Game) -> () {
    match FromPrimitive::from_i16(server_response.code) {
        Some(ResponseCode::Ok) => println!("Success!"),
        Some(ResponseCode::IdAcknowledgment) => {
            let id: i16 = server_response.additional_info.parse().unwrap_or(-1);
            println!("Our id is: {}", id)
        },
        Some(ResponseCode::TradeRequest) => send_trade_command(&stream, &mut buf_stream, game).unwrap(),
        Some(ResponseCode::BuildRequest) => send_build_command(&stream, &mut buf_stream, game).unwrap(),
        Some(ResponseCode::InitialBuildRequest) => send_initial_build_command(&stream, &mut buf_stream, game).unwrap(),
        Some(ResponseCode::MoveBanditRequest) => move_bandit_command(&stream, &mut buf_stream, game).unwrap(),
        Some(ResponseCode::ForceDiscardRequest) => send_force_discard_command(&stream, &mut buf_stream, game).unwrap(),

        _ => println!("Got error: {}", server_response.code)
    }
}

// Send a command that responds to a TradeRequest
fn send_trade_command(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>, game: &Game) -> Result<(), &'static str> {
    let all_resources = vec!("ore", "grain", "wool", "wood", "stone");
    let wanted_resources = vec!("wood", "stone", "grain", "wool");
    let random_trade = TradeCommand {
        from: String::from(all_resources.choose(&mut rand::thread_rng()).unwrap().clone()),
        to: String::from(wanted_resources.choose(&mut rand::thread_rng()).unwrap().clone()),
    };
    let trade_commands : Vec<&TradeCommand> = vec!(&random_trade);
    transmit(&mut buf_stream, &stream, &trade_commands)
}

// Send a command that responds to a ForceDiscardRequest
// Discards all resources.
fn send_force_discard_command(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>, game: &Game) -> Result<(), &'static str> {
    let me  = game.me().unwrap();
    transmit(&mut buf_stream, &stream, & me.resources)
}

// Send a command that response to a MoveBanditRequest
// Places the bandit on a random tile.
fn move_bandit_command(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>, game: &Game) -> Result<(), &'static str> {
    let board = game.get_board().unwrap();
    let tiles = board.get_tiles();
    let random_tile = tiles.choose(&mut rand::thread_rng()).unwrap();
    let bandit_cmd = MoveBanditCommand {
        location: random_tile.key.clone()
    };
    let bandit_commands: Vec<&MoveBanditCommand> = vec!(&bandit_cmd);
    transmit(&mut buf_stream, &stream, &bandit_commands)
}

// send a command that responds to a InitialBuildRequest
// Create a village and street at a random location
fn send_initial_build_command(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>, game: &Game) -> Result<(), &'static str> {
    let board = game.get_board().unwrap();
    let nodes = board.get_nodes();
    let random_node = nodes.choose(&mut rand::thread_rng()).unwrap();
    let surrounding_edges = board.get_edges_surrounding_node(random_node);
    let random_street = surrounding_edges.choose(&mut rand::thread_rng()).unwrap();
    let build_village = BuildCommand {
        structure: String::from("village"),
        location: (*random_node).key.clone()
    };
    let build_street = BuildCommand {
        structure: String::from("street"),
        location: (*random_street).key.clone()
    };
    let commands = vec!(&build_village, &build_street);
    transmit(&mut buf_stream, &stream, &commands)
}

// Send a build command that responds to BuildRequest.
// If a village can be built, build a village. Otherwise try building a street, otherwise do
// nothing. 
fn send_build_command(stream: &TcpStream, mut buf_stream: &mut BufStream<&TcpStream>, game: &Game) -> Result<(), &'static str> {
    let board = game.get_board().unwrap();

    println!("my resources are: {:?}", game.me().unwrap().resources);

    let potential_villages = board.get_potential_village_nodes(game.me().unwrap());
    println!("I have potential villages: {:?}", potential_villages);
    if potential_villages.len() != 0 {
        let random_village = potential_villages.choose(&mut rand::thread_rng()).unwrap();
        let build_village = BuildCommand {
            structure: String::from("village"),
            location: (*random_village).key.clone()
        };
        let commands = vec!(&build_village);
        return transmit(&mut buf_stream, &stream, &commands)
    }

    let potential_streets = board.get_potential_street_edges(game.me().unwrap());
    println!("I have potential streets: {:?}", potential_streets);
    if potential_streets.len() == 0 {
        let commands: Vec<BuildCommand> = Vec::new();
        return transmit(&mut buf_stream, &stream, &commands)
    }
    let random_street = potential_streets.choose(&mut rand::thread_rng()).unwrap();
    let build_street = BuildCommand {
        structure: String::from("street"),
        location: (*random_street).key.clone()
    };
    let commands = vec!(&build_street);
    transmit(&mut buf_stream, &stream, &commands)
}

// Reads the TCP input and extracts a json object from it.
// Returns Some(String) if the string is a properly formatted json object,
// otherwise returns None
fn read_tcp_input(buf_stream: &mut BufStream<&TcpStream>) -> Option<String> {
    let mut buffer = String::new();
    if let Ok(_buffer_size) = buf_stream.read_line(&mut buffer) {
        return Some(buffer)
    }
    None
}

// Transmit a JSON object over the TCP connection and append a newline
fn transmit<T: ?Sized>(buf_stream: &mut BufStream<&TcpStream>, stream: &TcpStream, value: &T) -> Result<(), &'static str> where T: Serialize {
    serde_json::to_writer(stream, value).unwrap_or_default();
    buf_stream.write(b"\r\n").unwrap_or_default(); // send a newline to indicate we are done
    buf_stream.flush().unwrap_or_default();
    Ok(())
}
