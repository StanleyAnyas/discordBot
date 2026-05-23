use once_cell::sync::Lazy;
use serde_json;
use serenity::all::Message;
use std::{ 
    collections::HashMap, fs, time::{Duration, Instant} 
};
use tokio::sync::Mutex;
use tracing::{error};

static RATE_LIMITS: Lazy<Mutex<HashMap<String, Instant>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn check_rate_limit(user_id: String, msg: Message) -> Result<(), String>{
    let sender = msg.author.name.clone();
    let mut limits = RATE_LIMITS.lock().await;
    let now = Instant::now();

    if let Some(last) = limits.get(&user_id) {
        if now.duration_since(*last) < Duration::from_secs(25) {
            return Err(format!("{} Please wait a few seconds before asking again", sender).to_string());
        }
    }

    limits.insert(user_id, now);
    Ok(())
}

pub static CHANNEL_HISTORY: Lazy<Mutex<HashMap<u64, Vec<String>>>> = 
    Lazy::new(|| {
        let memory = load_memory();
        Mutex::new(memory)
    });


fn load_memory() -> HashMap<u64, Vec<String>>{
    let file_content = fs::read_to_string("memory.json");

    match file_content {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_default()
        },
        Err(_) =>{
            HashMap::new()
        }
    }
}

pub async fn save_memory() {
    // println!("memory");
    let memory = CHANNEL_HISTORY.lock().await;
    // println!("memory2");
    let json = serde_json::to_string_pretty(&*memory)
        .unwrap_or_else(|_| "{}".to_string());

    if let Err(e) = fs::write("memory.json", json){
        error!("Failed to save file {}", e)
    }
}