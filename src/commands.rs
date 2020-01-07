use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BuildCommand {
    pub structure: String,
    pub location: String,
}

#[derive(Serialize, Deserialize)]
pub struct TradeCommand {
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Deserialize)]
pub struct MoveBanditCommand {
    pub location: String
}
