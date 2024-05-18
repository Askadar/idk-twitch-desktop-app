use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct Emote {
    pub id: String,
    pub char_range: std::ops::Range<usize>,

    /// This is the text that this emote replaces, e.g. `Kappa` or `:)`.
    pub code: String,
}

#[derive(Serialize, Clone)]
pub struct Message {
    pub name: String,
    pub message: String,
    pub emotes: Vec<Emote>,
}

#[derive(Deserialize)]
pub struct ChannelData {
    pub user_login: String,
    pub id: String,
}
