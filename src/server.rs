use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServerInput {
    pub model: String,
    pub attributes: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct ServerMessage {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ServerResponse {
    pub code: i16,
    pub title: String,
    pub description: String,
    pub additional_info: String,
    pub is_error: bool,
}

#[derive(Serialize, Deserialize)]
pub struct JoinMessage {
    pub id: i16,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ClientResponse {
    pub response: serde_json::Value
}
