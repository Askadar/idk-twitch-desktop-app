use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ChannelEmote {
    id: String,
    code: String,
    #[serde(rename = "imageType")]
    image_type: String,
    animated: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmoteData {
    id: String,
    #[serde(rename = "channelEmotes")]
    channel_emotes: Vec<ChannelEmote>,
    #[serde(rename = "sharedEmotes")]
    shared_emotes: Vec<ChannelEmote>,
}

pub struct BetterTVManager {
    client: reqwest::Client,
}

impl BetterTVManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl super::EmoteStorage for BetterTVManager {
    async fn fetch_for(
        &self,
        id: &str,
        platform: Option<&str>,
    ) -> Result<Vec<super::Emote>, String> {
        let platform = platform.unwrap_or("twitch");
        let url = format!("https://api.betterttv.net/3/cached/users/{platform}/{id}");
        let data = self.client.get(url).send().await;

        match data {
            Ok(resp) => {
                let emote_root = resp.json::<EmoteData>().await;

                match emote_root {
                    Ok(mut root) => {
                        let mut emotes = Vec::with_capacity(
                            root.channel_emotes.len() + root.shared_emotes.len(),
                        );
                        emotes.append(&mut root.channel_emotes);
                        emotes.append(&mut root.shared_emotes);

                        let emotes = emotes
                            .iter()
                            .map(|e| {
                                let id = e.id.clone();
                                let code = format!(":{}:", e.code);
                                let image_type = e.image_type.clone();
                                let size = 2;
                                super::Emote {
                                    id: id.clone(),
                                    code,
                                    url: format!(
                                        "https://cdn.betterttv.net/emote/{id}/{size}x.{image_type}"
                                    ),
                                }
                            })
                            .collect();

                        Ok(emotes)
                    }

                    Err(e) => Err(format!("Failed to parse emote data: {e}")),
                }
            }
            Err(e) => Err(format!("Failed to fetch emotes: {e}")),
        }
    }

    async fn fetch_global(&self) -> Result<Vec<super::Emote>, String> {
        let url = format!("https://api.betterttv.net/3/cached/global");
        println!("fetching {url}");
        let data = self.client.get(url).send().await;

        match data {
            Ok(resp) => {
                let emote_root = resp.json::<Vec<ChannelEmote>>().await;

                match emote_root {
                    Ok(emotes) => {
                        let emotes = emotes
                            .iter()
                            .map(|e| {
                                let id = e.id.clone();
                                let code = format!(":{}:", e.code);
                                let image_type = e.image_type.clone();
                                let size = 2;
                                super::Emote {
                                    id: id.clone(),
                                    code,
                                    url: format!(
                                        "https://cdn.betterttv.net/emote/{id}/{size}x.{image_type}"
                                    ),
                                }
                            })
                            .collect();

                        Ok(emotes)
                    }

                    Err(e) => Err(format!("Failed to parse emote data: {e}")),
                }
            }
            Err(e) => Err(format!("Failed to fetch emotes: {e}")),
        }
    }
}
