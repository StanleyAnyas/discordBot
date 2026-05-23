use reqwest::Client;
#[allow(unused_imports)]
use serenity::{
    all::{
        Message,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use std::env;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>
}


#[derive(Deserialize)]
struct ResponseMessage {
    content: String
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage
}

use crate::memory::{
    CHANNEL_HISTORY,
    save_memory
};

pub async fn ask_ai(question: &str, model: &str, channel_id: u64) -> Result<String, String>{
    let mut memory = CHANNEL_HISTORY.lock().await;
    let history = memory.entry(channel_id).or_insert(Vec::new());
    

    if history.len() > 10 {
        history.remove(0);
    }

    let api_key = env::var("TRAITBOT_AI_TOKEN")
        .map_err(|_| {
            "Missing api key".to_string()
        })?;

    if question.trim().is_empty() {
        return Err(
            "Question cannot be empty".to_string()
        );
    }

    let client = Client::new();
    let behaviour = "You are TraitBot, a funny and smart AI assistant for a Discord programming server. 
                    Keep answers concise and beginner friendly."
                    .to_string();
    
    let prev_history = history.join("\n");

    let body = ChatRequest {
        model: model.to_string(),

        messages: vec![

            ChatMessage {
                role: "system".to_string(),
                content: 
                format!(
                    "
                    {}

                    Previous questions and answer
                    {}
                    ", behaviour, prev_history
                )
                
            },

            ChatMessage {
                role: "user".to_string(),
                //TODO  send also the name of the person that sent the message
                content: question.trim().to_string(),
            }
        ],

    };

    let response = client 
        .post(
            "https://openrouter.ai/api/v1/chat/completions"
        )
        .header(
            "Authorization", 
        format!("Bearer {}", api_key)
        )
        .header("HTTP-Referer", 
        "http://localhost:3000"
        )
        .header("X-Title", "TraitBot")
        .json(&body)
        .send()
        .await

        .map_err(|err| {
            format!(
                "Failed to send question: {}",
                err
            )
        })?;
        if !response.status().is_success() {
            let status = response.status();

            let text = response
                .text()
                .await
                .unwrap_or_else(|_| {
                    "Unknown error".to_string()
                });

                return Err(
                    format!(
                        "There was an error calling the AI ({}): {}",
                        status,
                        text
                    )
                )
        }

        let json: ChatResponse = response 
            .json()
            .await

            .map_err(|err| {
                format!(
                    "Failed to parsed response: {}",
                    err
                )
            })?;

        if json.choices.is_empty() {
            return Err(
                "AI returned no response".to_string()
            );
        }

        let answer = json.choices[0]
        .message
        .content
        .trim()
        .to_string();
        history.push(
            format!("User: {}", question)
        );
        history.push(
            format!("AI: {}", answer)
        );
        if answer.is_empty() {
            return Err(
                "AI returned an empty message".to_string()
            );
        }
        drop(memory);
        save_memory().await;
        Ok(answer)
}