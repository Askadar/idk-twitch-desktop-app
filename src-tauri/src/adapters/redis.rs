use redis::{AsyncCommands, Client};

pub fn init_redis() -> Client {
    redis::Client::open("redis://127.0.0.1/").expect("Failed to create redis client")
}

pub struct RedisStorage {
    r: redis::aio::ConnectionManager,
}

// dummy debug cause ConnectionMamager isn't impl
impl std::fmt::Debug for RedisStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("").finish()
    }
}

#[async_trait::async_trait]
impl twitch_irc::login::TokenStorage for RedisStorage {
    async fn load_token(&mut self) -> Result<twitch_irc::login::UserAccessToken, Self::LoadError> {
        let mut r = self.r.clone();
        let token_str = r.get::<_, String>("token").await;
        match token_str {
            Ok(slice) => match serde_json::from_str(&slice) {
                Ok(token) => Ok(token),
                Err(e) => Err(format!("Failed to parse token slice {e}")),
            },
            Err(e) => Err(format!("Failed to load token from redis {e}")),
        }
    }
    async fn update_token(
        &mut self,
        token: &twitch_irc::login::UserAccessToken,
    ) -> Result<(), Self::UpdateError> {
        let mut r = self.r.clone();

        let token_str = serde_json::to_string(token);
        match token_str {
            Ok(slice) => match r.set::<_, _, String>("token", &slice).await {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to set token in redis {e}")),
            },
            Err(e) => Err(format!("Failed to serialize token {e}")),
        }
    }

    type LoadError = String;
    type UpdateError = String;
}

impl RedisStorage {
    pub async fn new() -> Self {
        let r = init_redis()
            .get_connection_manager()
            .await
            .unwrap();
        RedisStorage { r }
    }

    pub fn seed(cm: redis::aio::ConnectionManager) -> Self {
        RedisStorage { r: cm }
    }
}
