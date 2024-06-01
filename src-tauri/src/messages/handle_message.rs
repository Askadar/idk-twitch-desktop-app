use super::Message;
use crate::data::ChannelData;
use crate::emotes::{twitch::transform_emotes, Emote};

use redis::AsyncCommands;
use tauri::{AppHandle, Manager};
use twitch_irc::message::ServerMessage;

pub async fn handle_message(
    message: ServerMessage,
    mut r: redis::aio::ConnectionManager,
    channel_data: &ChannelData,
    app: &AppHandle,
    emote_manager: &crate::emotes::RemoteEmoteManager,
) -> () {
    match message {
        ServerMessage::Privmsg(m) => {
            let formatted_message = format!("{}: {}", m.sender.name, m.message_text);
            let mut emotes = transform_emotes(&m.emotes);
            let mut remote_emotes: Vec<Emote> = m
                .message_text
                .split(" ")
                // TODO maybe deal with lifetimes in Message struct instead of cloning emotes
                .filter_map(|w| emote_manager.test_emote(w).map(|e| e.to_owned()))
                .collect();
            emotes.append(&mut remote_emotes);

            let message = Message {
                name: m.sender.name,
                message: m.message_text,
                emotes,
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
