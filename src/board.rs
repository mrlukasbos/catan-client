use serde::{Deserialize, Serialize};
use itertools::Itertools;
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

        let permutations = (0..2).permutations(2);
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

#[derive(Serialize, Deserialize, PartialEq)]
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

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub key: String,
    pub player: Option<u8>,
    pub road: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ServerInputPlayer {
    pub model: String,
    pub attributes: Player,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub id: i32,
    pub color: String,
    pub name: String,
//    pub resources: Vec<String>,
//    pub development_cards: Vec<String>,
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
