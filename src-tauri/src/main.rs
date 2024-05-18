// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use redis::{aio::ConnectionManager, AsyncCommands};
use tauri::State;

use crate::messages::message_listener;
type Error = Box<dyn std::error::Error>;

pub mod data;
pub mod messages;
pub mod token_storage;


#[tauri::command]
async fn greet(name: &str, state: State<'_, RedisClient>) -> Result<String, String> {
    let mut r = state.r.clone();
    r.incr::<_, _, i32>("test", 1).await.unwrap();

    let nutty = format!("Hello, {}! Time to get slappied!", name);
    Ok(nutty)
}

fn setup<'a>(app: &'a mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let r = messages::init_redis();
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move { message_listener(handle).await });

    Ok(())
}

struct RedisClient {
    // r: Arc<Mutex<simple_redis::client::Client>>,
    r: ConnectionManager,
    // sub: Mutex<redis::aio::Connection>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let r = messages::init_redis().get_connection_manager().await.unwrap();
    // let m = redis::aio::ConnectionManager
    // let sub = messages::init_redis().get_async_connection().await.unwrap();
    println!("penned");

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
