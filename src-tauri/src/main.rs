// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;

use redis::Commands;
use tauri::{AppHandle, Manager};
use twitch_api::twitch_oauth2::{AccessToken, UserToken};
use twitch_api::HelixClient;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::{irc, ClientConfig, SecureTCPTransport, TwitchIRCClient};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Time to get slappied!", name)
}

fn setup<'a>(app: &'a mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    println!("setting up");
    tauri::async_runtime::spawn(async move { message_listener(handle).await });
    println!("spawned deez nuts");

    // tauri::async_runtime::spawn(async move {
    //     // also added move here
    //     let verify_result = verify_local_server().await;
    //     match verify_result {
    //         Ok(_) => {
    //             println!("Local Server is running");
    //         }
    //         Err(err) => {
    //             handle.emit_all("local-server-down", ()); // changed this to handle.
    //             println!("Local Server is not running");
    //             println!("{}", err);
    //         }
    //     }
    // });
    Ok(())
}

async fn message_listener(app: AppHandle) {
    let mut r = redis::Client::open("redis://127.0.0.1/").unwrap();
    println!("connected to rrr");

    let login_name = std::env::var("BOT_USERNAME").expect("Missing env 'BOT_USERNAME'");
    let oauth_token = std::env::var("BOT_TOKEN").expect("Missing env 'BOT_TOKEN'");

    let channels: Vec<String> = r.lrange("channels", 0, -1).unwrap();

    println!("channeleed: {:?}", channels);

    let t: HelixClient<reqwest::Client> = HelixClient::default();

    let token = UserToken::from_token(&t, AccessToken::from(oauth_token.clone()))
        .await
        .unwrap();

    let config =
        ClientConfig::new_simple(StaticLoginCredentials::new(login_name, Some(oauth_token)));

    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let channels_info = t
        .req_get(
            twitch_api::helix::streams::GetStreamsRequest::user_logins(
                channels
                    .iter()
                    .map(|a| twitch_api::types::NicknameRef::from_str(a))
                    .collect::<Vec<_>>(),
            ),
            &token,
        )
        .await
        .unwrap()
        .data;

    let login_live_hash: HashMap<String, String> = channels_info
        .iter()
        .map(|s| (s.user_login.clone().to_string(), s.id.clone().to_string()))
        .collect();

    let join_handle = tauri::async_runtime::spawn(async move {
        println!("Threading");
        while let Some(message) = incoming_messages.recv().await {
            handle_message(message, &mut r, &login_live_hash, &app).await;
        }
    });

    channels_info.iter().for_each(|channel| {
        client
            .join(channel.user_login.to_string().to_owned())
            .unwrap()
    });
    client
        .send_message(irc!["CAP REQ", "twitch.tv/membership"])
        .await
        .unwrap();

    println!("{:?}", channels_info);
    println!(
        "Prepped on {:?}",
        channels_info
            .iter()
            .map(|s| s.user_login.to_string())
            .collect::<Vec<_>>()
    );

    join_handle.await.unwrap();
}

fn main() {
    dotenv::dotenv().unwrap();

    tauri::Builder::default()
        .setup(setup)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(serde::Serialize, Clone)]
struct Message {
    name: String,
    message: String,
}

async fn handle_message(
    message: ServerMessage,
    r: &mut redis::Client,
    login_live_hash: &HashMap<String, String>,
    app: &AppHandle,
) -> () {
    match message {
        ServerMessage::Privmsg(m) => {
            let formatted_message = format!("{}: {}", m.sender.name, m.message_text);
            let message = Message {
                name: m.sender.name,
                message: m.message_text,
            };
            app.emit("new-message", message).expect("failed to emit");

            r.sadd::<String, String, ()>(
                format!(
                    "messages:{}:{}",
                    m.channel_login,
                    login_live_hash.get(&m.channel_login).unwrap()
                ),
                formatted_message,
            )
            .unwrap();
        }
        ServerMessage::Join(m) => {
            let added = r
                .sadd::<String, String, usize>(
                    format!(
                        "joins:{}:{}",
                        m.channel_login,
                        login_live_hash.get(&m.channel_login).unwrap()
                    ),
                    m.user_login.clone(),
                )
                .unwrap();
            if added == 1 {
                r.zincr::<_, _, _, _>(format!("watchStreaks:{}", m.channel_login), m.user_login, 1)
                    .unwrap()
            }
        }
        // ServerMessage::Part(m) => {
        //     print!("\n#!# {} left {} #!# ", m.user_login, m.channel_login);
        //     std::io::stdout().flush().unwrap();
        // }
        _ => {}
    }
}
