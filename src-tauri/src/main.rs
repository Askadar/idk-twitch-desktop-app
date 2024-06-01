// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use redis::{aio::ConnectionManager, AsyncCommands};
use tauri::State;

use adapters::redis::init_redis;
use messages::message_listener;

// type Error = Box<dyn std::error::Error>;

pub mod adapters;
pub mod data;
pub mod emotes;
pub mod messages;

#[tauri::command]
async fn greet(name: &str, state: State<'_, RedisClient>) -> Result<String, String> {
    let mut r = state.r.clone();
    r.incr::<_, _, i32>("test", 1).await.unwrap();

    let nutty = format!("Hello, {}!", name);
    Ok(nutty)
}

fn setup<'a>(app: &'a mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move { message_listener(handle).await });

    Ok(())
}

struct RedisClient {
    r: ConnectionManager,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let r = init_redis().get_connection_manager().await.unwrap();

    tauri::Builder::default()
        .manage(RedisClient {
            r,
            // sub: Mutex::new(sub),
        })
        .setup(setup)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
