use tokio::sync::mpsc::UnboundedReceiver;
use twitch_api::{
    twitch_oauth2::{AccessToken, UserToken},
    HelixClient,
};
use twitch_irc::{
    login::{RefreshingLoginCredentials, TokenStorage}, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

use super::redis::RedisStorage;


pub async fn init_irc(
    s: RedisStorage,
) -> (
    UnboundedReceiver<ServerMessage>,
    TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<RedisStorage>>,
) {
    let login_name = std::env::var("IRC_BOT_USERNAME").ok();

    let client_id = std::env::var("TWITCH_CLIENT_ID").expect("Missing env 'TWITCH_CLIENT_ID'");
    let client_secret =
        std::env::var("TWITCH_CLIENT_SECRET").expect("Missing env 'TWITCH_CLIENT_SECRET'");

    let creds =
        RefreshingLoginCredentials::init_with_username(login_name, client_id, client_secret, s);

    let config = ClientConfig::new_simple(creds);

    TwitchIRCClient::<SecureTCPTransport, RefreshingLoginCredentials<RedisStorage>>::new(config)
}

pub async fn init_helix(mut s: RedisStorage) -> (HelixClient<'static, reqwest::Client>, UserToken) {
    let token = s.load_token().await.expect("Failed to retrieve token");

    let client: HelixClient<reqwest::Client> = HelixClient::default();

    let token = UserToken::from_token(&client, AccessToken::from(token.access_token.clone()))
        .await
        .unwrap();

    return (client, token);
}
