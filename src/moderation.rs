
use reqwest::Client;

use serde::{
    Deserialize,
    Serialize,
};

use serenity::all::{
        Context, Mentionable, Message
    };

use std::env;

#[derive(Serialize)]
struct ModerationRequest {
    model: String,
    messages: Vec<ModerationMessage>,
}

#[derive(Serialize)]
struct ModerationMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ModerationResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, PartialEq)]
pub enum ModerationDecision {
    Safe,
    Delete,
    RetryLater,
}

pub async fn moderate_message(ctx: &Context, msg: &Message, model: &str) -> ModerationDecision {
    if msg.author.bot {
        return ModerationDecision::Safe;
    }

    let cleaned_text = if msg.content.starts_with("/"){
        msg.content.split_whitespace().skip(1)
            .collect::<Vec<&str>>()
            .join(" ")
    }else {
        msg.content.clone()
    };

    if cleaned_text.trim().is_empty(){
        return ModerationDecision::Safe;
    };
    
    return ai_moderation(ctx, msg, &cleaned_text, model).await;
}

async fn ai_moderation(ctx: &Context, msg: &Message, content: &str, model: &str) -> ModerationDecision {

    let api_key = match env::var("TRAITBOT_AI_TOKEN") {
        Ok(k) => k,
        Err(_) => return ModerationDecision::Safe,
    };
    
    let client = Client::new();

    let body = ModerationRequest {
        model: model.to_string(),

        messages: vec![
            ModerationMessage {
                role: "system".to_string(),

                content:
                    "
                        You are an AI moderation system.
                        
                        Detect:
                        - toxicity
                        - harassment
                        - hate speech
                        - scams
                        - NSFW
                        - spam
                        - threats

                        Respond ONLY with:

                        SAFE 

                        or 

                        DELETE

                        note: the language of the message to moderate can be any
                    ".to_string(),
            },

            ModerationMessage {
                role: "user".to_string(),
                content: 
                    content.to_string(),
            }
        ],
    };
    
    let response = match client
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
        .header("X-Title", "TraitBot Moderation")
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return ModerationDecision::Safe,
    };

    if !response.status().is_success() {
        let _status = response.status();
        let text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        if text.contains("429") || text.contains("402") {
            return ModerationDecision::RetryLater;
        }else {
            return ModerationDecision::Safe;
        }
    }

    let json: ModerationResponse = match response.json().await {
        Ok(j) => j,
        Err(_) => return ModerationDecision::Safe,
    };

    if json.choices.is_empty() {
        return ModerationDecision::Safe;
    }

    // interpret first choice content: "DELETE" means moderation triggered
    let first = &json.choices[0].message.content;
    println!("{}", first);
    if first.trim().eq_ignore_ascii_case("DELETE") {
        let _ =  msg.delete(&ctx.http).await;
        let _ = msg.channel_id.say(&ctx.http, 
            format!("{} your message has been removed", 
                msg.author.mention()
            )
        ).await;
        return ModerationDecision::Delete;
    }

    ModerationDecision::Safe
}