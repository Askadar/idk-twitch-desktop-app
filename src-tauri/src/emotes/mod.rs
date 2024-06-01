use std::collections::HashMap;

use async_trait::async_trait;
use futures::future::join_all;

pub mod better_tv;
pub mod seven_tv;
pub mod twitch;

#[derive(serde::Serialize, Clone, Debug)]
pub struct Emote {
    pub code: String,
    pub id: String,
    pub url: String,
}

impl Default for Emote {
    fn default() -> Self {
        Self {
            code: "NaN".to_string(),
            id: "000".to_string(),
            url: "https://nowhere.com".to_string(),
        }
    }
}

#[async_trait]
pub trait EmoteStorage {
    async fn fetch_for(&self, id: &str, platform: Option<&str>) -> Result<Vec<Emote>, String>;
    async fn fetch_global(&self) -> Result<Vec<Emote>, String>;
}

pub struct RemoteEmoteManager {
    sources: Vec<Box<dyn EmoteStorage + Send + Sync>>,
    emotes: HashMap<String, Emote>,
}

impl RemoteEmoteManager {
    pub async fn new(user: &str, platform: Option<&str>) -> Self {
        let sources: Vec<Box<dyn EmoteStorage + Send + Sync>> = vec![
            Box::new(better_tv::BetterTVManager::new()),
            Box::new(seven_tv::SevenTVManager::new()),
        ];

        let emotes = join_all(sources.iter().map(|source| async move {
            let emotes = join_all(vec![
                source.fetch_for(user, platform),
                source.fetch_global(),
            ])
            .await;

            emotes
                .into_iter()
                .flat_map(|emotes| match emotes {
                    Ok(emotes) => emotes.into_iter().map(|e| (e.code.clone(), e)).collect(),
                    Err(e) => {
                        eprintln!("Failed to fetch emotes from {}", e);
                        vec![]
                    }
                })
                .collect::<Vec<(String, Emote)>>()
        }))
        .await
        .into_iter()
        .flatten()
        .collect();

        // println!("Full emote list {:?}", &emotes);

        Self { sources, emotes }
    }

    pub fn test_emote(&self, code: &str) -> Option<&Emote> {
        self.emotes.get(code)
    }
}
