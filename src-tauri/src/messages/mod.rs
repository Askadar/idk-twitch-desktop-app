mod handle_message;

use std::sync::Arc;

use futures::StreamExt;
use tauri::AppHandle;

use crate::adapters::redis::{init_redis, RedisStorage};
use crate::adapters::twitch::{init_helix, init_irc};
use crate::data::ChannelData;
use crate::emotes::Emote;
use crate::messages::handle_message::handle_message;

#[derive(serde::Serialize, Clone)]
pub struct Message {
    pub name: String,
    pub message: String,
    pub emotes: Vec<Emote>,
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

            let Some(stream) = channel_data else {
                println!("Failed to retrieve data for {channel}");
                return;
            };

            let emote_manager = crate::emotes::RemoteEmoteManager::new(stream.user_id.as_ref(), None).await;
            let channel_data = ChannelData {
                user_login: stream.user_login.to_string(),
                id: stream.id.to_string(),
            };
            // println!("detailed data: {:#?}", stream);

            let join_handle = tauri::async_runtime::spawn(async move {
                println!("Threading");

                while let Some(message) = irc.recv().await {
                    handle_message(message, cm.clone(), &channel_data, &app, &emote_manager).await;
                }
            });

            irc_client.join(stream.user_login.to_string()).unwrap();

            println!("listening");
            join_handle.await.unwrap();

            ()
        }
    };

    stream.for_each_concurrent(None, handler).await;
}
