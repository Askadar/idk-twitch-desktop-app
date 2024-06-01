use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct HostFile {
    name: String,
    static_name: String,
    width: u32,
    height: u32,
    frame_count: u32,
    size: u32,
    format: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmoteData {
    id: String,
    name: String,
    listed: bool,
    animated: bool,
    host: Host,
}

#[derive(Serialize, Deserialize, Debug)]
struct Host {
    files: Vec<HostFile>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Emote {
    id: String,
    name: String,
    timestamp: u64,
    data: EmoteData,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmoteSet {
    id: String,
    name: String,
    emotes: Vec<Emote>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Root {
    id: String,
    platform: String,
    username: String,
    display_name: String,
    emote_set: EmoteSet,
}

pub struct SevenTVManager {
    client: reqwest::Client,
}

impl SevenTVManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl super::EmoteStorage for SevenTVManager {
    async fn fetch_for(
        &self,
        id: &str,
        platform: Option<&str>,
    ) -> Result<Vec<super::Emote>, String> {
        let platform = platform.unwrap_or("twitch");
        let data = self
            .client
            .get(format!("https://7tv.io/v3/users/{platform}/{id}"))
            .send()
            .await;

        match data {
            Ok(resp) => {
                let emote_root = resp.json::<Root>().await;
                match emote_root {
                    Ok(root) => Ok(root
                        .emote_set
                        .emotes
                        .iter()
                        .map(|e| super::Emote {
                            id: e.id.clone(),
                            code: e.data.name.clone(),
                            url: format!("https://cdn.7tv.app/emote/{}/2x.webp", e.id),
                        })
                        .collect()),
                    Err(e) => Err(format!("Failed to parse emote data: {e}")),
                }
            }
            Err(e) => Err(format!("Failed to fetch emotes: {e}")),
        }
    }
    async fn fetch_global(&self) -> Result<Vec<super::Emote>, String> {
        let data = self
            .client
            .get(format!(
                "https://7tv.io/v3/emote-sets/62cdd34e72a832540de95857"
            ))
            .send()
            .await;

        match data {
            Ok(resp) => {
                let emote_root = resp.json::<EmoteSet>().await;
                match emote_root {
                    Ok(emote_set) => Ok(emote_set
                        .emotes
                        .iter()
                        .map(|e| super::Emote {
                            id: e.id.clone(),
                            code: e.data.name.clone(),
                            url: format!("https://cdn.7tv.app/emote/{}/2x.webp", e.id),
                        })
                        .collect()),
                    Err(e) => Err(format!("Failed to parse emote data: {e}")),
                }
            }
            Err(e) => Err(format!("Failed to fetch emotes: {e}")),
        }
    }
}
