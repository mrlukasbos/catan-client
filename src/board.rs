use serde::{Deserialize, Serialize};
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ServerInputBoard {
    pub model: String,
    pub attributes: Board,
}

#[derive(Serialize, Deserialize)]
pub struct Board {
    #[serde(default)]
    bandits: Vec<ServerInputBandit>,
    #[serde(default)]
    tiles: Vec<ServerInputTile>,
    #[serde(default)]
    nodes: Vec<ServerInputNode>,
    #[serde(default)]
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

    pub fn get_tile_by_key(&self, key: &str) -> Option<&Tile> {
        self.get_tiles().into_iter().find(|tile| tile.key == key)
    }

    pub fn get_node_by_key(&self, key: &str) -> Option<&Node> {
        self.get_nodes().into_iter().find(|node| node.key == key)
    }

    pub fn get_edge_by_key(&self, key: &str) -> Option<&Edge> {
        self.get_edges().into_iter().find(|edge| edge.key == key)
    }

    pub fn get_tiles_surrounding_node(&self, node: &Node) -> Vec<&Tile> { 
        let t = self.get_tile_by_key(node.t_key.as_str());
        let l = self.get_tile_by_key(node.l_key.as_str());
        let r = self.get_tile_by_key(node.r_key.as_str());
        let options = vec!(t, l, r);
        options.into_iter().flatten().collect()
    }

    // get edges leading to a node
    pub fn get_edges_surrounding_node(&self, node: &Node) -> Vec<&Edge> {
        let tiles = self.get_tiles_surrounding_node(node);
        let mut options: Vec<Option<&Edge>> = Vec::new();

        let permutations = (0..=2).permutations(2);
        for permutation in permutations {
            let edge_key = format!("({},{})", tiles[permutation[0]].key, tiles[permutation[1]].key);
            options.push(self.get_edge_by_key(edge_key.as_str()));
        }
        options.into_iter().flatten().collect()
    }

    // get all nodes that contain the edge as it surrounding edge
    pub fn get_nodes_surrounding_edge(&self, edge: &Edge) -> Vec<&Node> {
        self.get_nodes().into_iter().filter(|node| {
            self.get_edges_surrounding_node(node).contains(&edge)
        }).collect()
    }

    // get all nodes surrounding another node
    pub fn get_nodes_surrounding_node(&self, node: &Node) -> Vec<&Node> {
        let edges = self.get_edges_surrounding_node(node);
        let mut options: Vec<Option<&Node>> = Vec::new();
        for edge in edges { 
            let surrounding_node = self.get_nodes_surrounding_edge(edge).into_iter().find(|n| { n != &node });
            options.push(surrounding_node);
        }
        options.into_iter().flatten().collect()
    }

    pub fn get_nodes_from_player(&self, player: &Player) -> Vec<&Node> {
        self.get_nodes().into_iter().filter(|node| {
            if let Some(player_id) = node.player {
                return player_id == player.id && node.structure != ""
            }
            false
        }).collect()
    }

    pub fn get_cities_from_player(&self, player: &Player) -> Vec<&Node> {
        let nodes = self.get_nodes_from_player(player);
        nodes.into_iter().filter(|node| {
            node.structure == "city"
        }).collect()
    }

    pub fn get_villages_from_player(&self, player: &Player) -> Vec<&Node> {
        let nodes = self.get_nodes_from_player(player);
        nodes.into_iter().filter(|node| {
            node.structure == "village"
        }).collect()
    }

    pub fn get_edges_from_player(&self, player: &Player) -> Vec<&Edge> {
        self.get_edges().into_iter().filter(|edge| {
            edge.road // && edge.player.id == player.id
        }).collect()
    }

    // get all edges where the player could try to build a street
    pub fn get_potential_street_edges(&self, player: &Player) -> Vec<&Edge> {
        let player_streets = self.get_edges_from_player(player);

        // get the nodes around the players streets
        let nodes_connected_to_player_streets = player_streets.into_iter().map(|street| {
            self.get_nodes_surrounding_edge(street)
        }).concat();

        // get all edges surrounding the nodes to which the player has at least one edge
        let edges_around_nodes: Vec<&Edge> = nodes_connected_to_player_streets.into_iter().map(|node| {
            let edges = self.get_edges_surrounding_node(node);
            edges
        }).concat().into_iter().sorted().dedup().collect();

        // filter out the streets that the player already owns
        edges_around_nodes.into_iter().filter(|e| {
            e.player.is_none()
        }).collect()
    }

    // get all nodes where the player can build a village
    pub fn get_potential_village_nodes(&self, player: &Player) -> Vec<&Node> { 
        // First we collect all positions where no one can build a village 
        // Because there is already one or there is one next to the node.
        let mut invalid_nodes: Vec<&Node> = self.get_nodes().iter().cloned().filter(|n| {
            n.player.is_some()
        }).collect();
        let illegal_empty_node_positions: Vec<&Node> = invalid_nodes.iter().cloned().map(|n| {
            self.get_nodes_surrounding_node(n)
        }).concat();
        invalid_nodes.extend(&illegal_empty_node_positions);
    
        // now all generic invalid nodes are known, get all nodes connected to an edge of the
        // player, and subtract the invalid nodes from that.
        let player_streets = self.get_edges_from_player(player);
        let nodes_connected_to_player_streets = player_streets.into_iter().map(|street| {
            self.get_nodes_surrounding_edge(street)
        }).concat();
        nodes_connected_to_player_streets.iter().cloned().filter(|n| {
            !invalid_nodes.contains(n)    
        }).collect()
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
    pub x: u8,
    pub y: u8,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputNode {
    pub model: String,
    pub attributes: Node,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Node {
    pub key: String,
    pub structure: String,
    pub player: Option<usize>,
    pub t_key: String,
    pub r_key: String,
    pub l_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputEdge {
    pub model: String,
    pub attributes: Edge,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Debug)]
pub struct Edge {
    pub key: String,
    pub player: Option<u8>,
    pub road: bool,
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputPlayer {
    pub model: String,
    pub attributes: Player,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub color: String,
    pub name: String,
    pub resources: Vec<Resource>,
//    pub development_cards: Vec<String>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Resource {
    pub r#type: String,
    pub value: usize,
}

impl Player {
    pub fn is_me(&self) -> bool {
        self.name == "Rust" // todo check this with the ID from the ID_acknowledgment
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputBandit{
    pub model: String,
    pub attributes: Bandit,
}

#[derive(Serialize, Deserialize)]
pub struct Bandit {
    pub tile_key: String
}
