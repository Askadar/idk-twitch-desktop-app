use std::sync::Arc;

use futures::StreamExt;
use redis::{AsyncCommands, Client};
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc::UnboundedReceiver;
use twitch_api::{
    twitch_oauth2::{AccessToken, UserToken},
    HelixClient,
};
use twitch_irc::{
    login::{RefreshingLoginCredentials, TokenStorage}, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

use crate::data::{ChannelData, Emote, Message};
use crate::token_storage::RedisStorage;

pub fn init_redis() -> Client {
    redis::Client::open("redis://127.0.0.1/").expect("Failed to create redis client")
}

async fn init_irc(
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

async fn init_helix(mut s: RedisStorage) -> (HelixClient<'static, reqwest::Client>, UserToken) {
    let token = s.load_token().await.expect("Failed to retrieve token");

    let client: HelixClient<reqwest::Client> = HelixClient::default();

    let token = UserToken::from_token(&client, AccessToken::from(token.access_token.clone()))
        .await
        .unwrap();

    return (client, token);
}

pub async fn message_listener(app: AppHandle) {
    let app_rc = Arc::new(app);

    let r = init_redis();
    let con = r.get_async_connection().await.unwrap();
    let mut sub = con.into_pubsub();
    sub.subscribe("channels").await.unwrap();
    println!("subscribed to channels");
    let stream = sub.on_message();
    let cm = r
        .get_connection_manager()
        .await
        .expect("Failed to init redis connection manager");

    let handler = move |message: redis::Msg| {
        let s = RedisStorage::seed(cm.clone());
        let s2 = RedisStorage::seed(cm.clone());
        let app = app_rc.clone();
        let cm = cm.clone();

        async move {
            let (mut irc, irc_client) = init_irc(s).await;
            let (t, token) = init_helix(s2).await;

            let channel = message.get_payload::<String>().unwrap();
            let channels = t
                .req_get(
                    twitch_api::helix::streams::GetStreamsRequest::user_logins(&[
                        twitch_api::types::NicknameRef::from_str(&channel),
                    ]),
                    &token,
                )
                .await
                .unwrap()
                .data;
            let channel_data = channels.get(0);

            let Some(channel) = channel_data else {
                println!("Failed to retrieve data for {channel}");
                return;
            };

            let channel_data = ChannelData {
                user_login: channel.user_login.to_string(),
                id: channel.id.to_string(),
            };

            println!("detailed data: {:#?}", channel);

            let join_handle = tauri::async_runtime::spawn(async move {
                println!("Threading");

                while let Some(message) = irc.recv().await {
                    handle_message(message, cm.clone(), &channel_data, &app).await;
                }
            });

            irc_client.join(channel.user_login.to_string()).unwrap();

            println!("listening");
            join_handle.await.unwrap();

            ()
        }
    };

    stream.for_each_concurrent(None, handler).await;
}

async fn handle_message(
    message: ServerMessage,
    mut r: redis::aio::ConnectionManager,
    channel_data: &ChannelData,
    app: &AppHandle,
) -> () {
    match message {
        ServerMessage::Privmsg(m) => {
            let formatted_message = format!("{}: {}", m.sender.name, m.message_text);
            let message = Message {
                name: m.sender.name,
                message: m.message_text,
                emotes: m
                    .emotes
                    .iter()
                    .map(|e| Emote {
                        id: e.id.clone(),
                        code: e.code.clone(),
                        char_range: e.char_range.clone(),
                    })
                    .collect(),
            };
            app.emit("new-message", message).expect("failed to emit");

            r.sadd::<_, _, i32>(
                &format!("messages:{}:{}", m.channel_login, channel_data.id,),
                &formatted_message,
            )
            .await
            .unwrap();
        }
        _ => {}
    }
}
