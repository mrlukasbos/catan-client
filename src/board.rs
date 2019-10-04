use serde::{Deserialize, Serialize};
use crate::server::ServerInput;

#[derive(Serialize, Deserialize)]
pub struct Board {
    pub bandits: Vec<ServerInput>,
    tiles: Vec<ServerInputTile>,
    nodes: Vec<ServerInputNode>,
    edges: Vec<ServerInputEdge>,
}

impl Board {
    pub fn get_tiles(&self) -> Vec<&Tile> {
        self.tiles.iter().map(|server_tile| { &server_tile.attributes }).collect()
    }

    pub fn get_nodes(&self) -> Vec<&Node> {
        self.nodes.iter().map(|server_node| { &server_node.attributes}).collect()
    }

    pub fn get_edges(&self) -> Vec<&Edge> {
        self.edges.iter().map(|server_edge| { &server_edge.attributes}).collect()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputTile {
    pub model: String,
    pub attributes: Tile,
}

#[derive(Serialize, Deserialize)]
pub struct Tile {
    pub key: String,
    pub resource_type: String,
    pub number: u32,
    pub orientation: String,
    pub x: u8,
    pub y: u8,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputNode {
    pub model: String,
    pub attributes: Node,
}

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub key: String,
    pub structure: String,
    pub player: Option<u8>,
    pub t_key: String,
    pub r_key: String,
    pub l_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputEdge {
    pub model: String,
    pub attributes: Edge,
}

#[derive(Serialize, Deserialize)]
pub struct Edge {
    pub key: String,
    pub player: Option<u8>,
    pub road: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub color: String,
    pub name: String,
//    resources: Vec<Resource>,
//    development_cards: Vec<DevelopmentCard>,
}

#[derive(Serialize, Deserialize)]
pub struct Bandit {
    pub tile_key: String
}