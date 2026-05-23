
use serenity::{
    all::{ChannelId, Context, EventHandler, GatewayIntents, Member, Message, Ready, User, VoiceState}, builder::{CreateEmbed, CreateMessage}, prelude::*
};
use serenity::all::ActivityData;
use dotenv::dotenv;
use std::{env, time::Duration};
use tracing::{info, error};
use tracing_subscriber::{
    fmt,
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tokio::time::{sleep};

use crate::moderation::ModerationDecision;
mod ticket;
mod call;
mod channels;
mod memory;
mod ai;
mod moderation;

const FREE_MODELS: &[&str] = &[
    "deepseek/deepseek-v4-flash:free",
    "google/gemma-4-26b-a4b-it:free",
    "google/gemma-4-31b-it:free",
    "nvidia/nemotron-3-super-120b-a12b:free",
    "qwen/qwen3-next-80b-a3b-instruct:free",
    "qwen/qwen3-coder:free",
    "nousresearch/hermes-3-llama-3.1-405b:free"
];

struct BotState {
    message_count: u32,
    start_time: std::time::Instant,
}

impl TypeMapKey for BotState {
    type Value = BotState;
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected to {} server!!", ready.user.name);
        ctx.set_activity(Some(ActivityData::playing("with discord")));
        let mut data = ctx.data.write().await;
        data.insert::<BotState>(BotState {
            message_count: 0,
            start_time: std::time::Instant::now(),
        });
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let channel_id = ChannelId::new(1376958993582391599);
        let welcome_message = format!(
            "Benvenuto {}! \n\n
            Se hai bisogno di aiuto su un linguaggio \n
            usa `/aiuto [linguaggio]`
            ",
            new_member.user.name
        );
        channel_id.send_message(&ctx.http,
            CreateMessage::new().content(welcome_message)
        )
        .await
        .unwrap();
    }

    async fn guild_member_removal(&self, _ctx: Context, guild_id: serenity::all::GuildId, user: User, member_data: Option<Member>){
        info!("[LEAVE] {} ({}) left server {}", user.name, user.id.get(), guild_id.get());
        println!("[LEAVE] {} ({}) left server {}", user.name, user.id.get(), guild_id.get());
        if let Some(member) = member_data {
            if let Some(nickname) = member.nick {
                info!("[LEAVE] Nickname was {}", nickname);
                println!("[LEAVE] Nickname was {}", nickname);
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState){
        call::handle_voice_join(ctx, old, new).await
    }

    async fn message(&self, ctx: Context, msg: Message){
        if msg.author.bot {
            return;
        }
        for model in FREE_MODELS {
            match moderation::moderate_message(&ctx, &msg, &model).await {
                ModerationDecision::Delete => {
                    return;
                }
                ModerationDecision::Safe => {
                    return;
                }
                ModerationDecision::RetryLater => {
                    continue;
                }
            }
        }
        let vec = vec!["/ciao", "/aiuto", "/die", "/ticket", "/ai"];
        if msg.content.starts_with("/") && !vec.contains(&msg.content.to_lowercase().as_str()) 
            && !msg.content.starts_with("/ai ") 
        {
            
            let sender = msg.author.name.clone();
            let sent = msg.content.as_str();
            info!("unknown command from {} and the message was {}", sender, sent);
            let embed = CreateEmbed::new()
                .title("Info")
                .field(format!(
                        "Ciao {} il commando {} non esiste ancora",
                        sender, sent
                    ), "i commandi disponibili sono !ciao, !aiuto, !ticket", false)
                .color(0x0099FF);
                let message = CreateMessage::new().embed(embed);
                
                msg.channel_id.send_message(&ctx.http, 
                    message
                ).await.unwrap();
                sleep(Duration::from_secs(10)).await;
            msg.delete(&ctx.http)
            .await.unwrap();
        }
        let mut data = ctx.data.write().await;
        if let Some(state) = data.get_mut::<BotState>() {
            state.message_count += 1;
            state.start_time = std::time::Instant::now();
        }
        let mut channel_name = String::new();
        if let Ok(channel) = msg.channel(&ctx).await {
            if let serenity::all::Channel::Guild(guild_channel) = channel {
                // println!("channel name {}", guild_channel.name)
                channel_name = guild_channel.name;
            }
        }

        if msg.content == "/ciao" {
            msg.channel_id.broadcast_typing(&ctx.http).await.ok();

            let start = std::time::Instant::now();
            println!("start {:?}", start);
            let name = msg.author.name.clone();
            // println!("response {:?}", response);
            let _ = msg.channel_id
            .send_message(&ctx.http, CreateMessage::new().content(format!("ciao {}", name)))
            .await.unwrap();
        }
        if msg.content == "/die" 
            && msg.author.id == 1338291003085422594 
            && channel_name.starts_with("ticket-") {
                // println!("{}", channel_name);
                // println!("{}", msg.author.id);
                msg.channel_id.delete(&ctx.http)
                .await.unwrap();
                info!("closed the ticket {}", channel_name);
            }

        if msg.content == "/aiuto" {
            let embed = CreateEmbed::new()
                .title("Help")
                .field("aiuto in un linguaggio", "!aiuto [nome linguaggio]", false)
                .color(0x0099FF);
            let message = CreateMessage::new().embed(embed);
            msg.channel_id
            .send_message(&ctx.http, message)
            .await
            .unwrap();
        }
        if msg.content.starts_with("/aiuto "){
            let parts: Vec<&str> = msg.content.split_whitespace().collect();
                if let Some(lang) = parts.get(1) {
                    if !channel_name.contains(lang){
                        match lang.to_lowercase().as_ref() {
                            "rust" => {
                                let rust_channel_id:u64 = 1378299379840188466;
                                msg.channel_id.send_message(&ctx.http, 
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", rust_channel_id))
                                )
                                .await
                                .unwrap();
                            }
                            "python" => {
                                let python_channel_id:u64 = 1378298909914304685;
                                msg.channel_id.send_message(&ctx.http, 
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", python_channel_id))
                                )
                                .await
                                .unwrap();
                            }
                            "java" => {
                                let java_channel_id:u64 = 1378299259933298739;
                                msg.channel_id.send_message(&ctx.http, 
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", java_channel_id))
                                )
                                .await
                                .unwrap();
                            }
                            "javascript" => {
                                let javascript_channel_id:u64 = 1378299337398030437;
                                msg.channel_id.send_message(&ctx.http, 
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", javascript_channel_id))
                                )
                                .await
                                .unwrap();
                            }
                            "php" => {
                                let php_channel_id:u64 = 1378299468474093628;
                                msg.channel_id.send_message(&ctx.http, 
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", php_channel_id))
                                )
                                .await
                                .unwrap();
                            }
                            "cpp" | "c++" => {
                                let cpp_channel_id:u64 = 1378299138508324885;
                                msg.channel_id.send_message(&ctx.http, 
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", cpp_channel_id))
                                )
                                .await
                                .unwrap();
                            }
                            "csharp" | "c#" => {
                                let csharp_channel_id:u64 = 1378299599566929991;
                                msg.channel_id.send_message(&ctx.http, 
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", csharp_channel_id))
                                )
                                .await
                                .unwrap();
                            }
                            _ => {
                                let other_help_id:u64 = 1378300028296237157;
                                msg.channel_id.send_message(&ctx.http,
                                    CreateMessage::new().content(format!("chiedi aiuto dentro il canale <#{}>", other_help_id))
                                )
                                .await
                                .unwrap();
                            }
                        }
                    }
                }
        }

        if msg.content == "/ticket" {
            info!("{} opened a ticket", msg.author.name.clone());
            ticket::create_ticket(&ctx, &msg).await;
        }

        if msg.content.starts_with("/ai ") {
            let sender_id = msg.author.id.to_string();
            match memory::check_rate_limit(sender_id, msg.clone()).await {
                Ok(_) => {
                    let question = msg.content.replace("/ai", "");
                    let _ = msg.channel_id
                        .broadcast_typing(&ctx.http)
                        .await;
                    let channel_id = msg.channel_id.get();
                    for model in FREE_MODELS {
                        match ai::ask_ai(&question, &model, channel_id).await {
                            Ok(answer) => {
                                // println!("answer {}", answer);
                                // println!("model {}", model);
                                if answer.len() > 1900 {
                                    let chunks = answer.as_bytes().chunks_exact(1900);

                                    for chunk in chunks {
                                        let text = String::from_utf8_lossy(chunk);
                                        let _ = msg.channel_id.send_message(&ctx.http, 
                                            CreateMessage::new().content(text)
                                        ).await;
                                    }
                                }else {
                                    let _ = msg.channel_id.send_message(&ctx.http, 
                                        CreateMessage::new().content(answer)
                                    ).await;
                                }
                                break;
                            }
                            Err(err) => {
                                // println!("Ai error {}", err);
                                if err.contains("429") || err.contains("402") {
                                    continue;
                                }
                                msg.channel_id.send_message(&ctx.http,
                                    CreateMessage::new().content(
                                        "Errore durante la chiamata all'ai "
                                    )
                                ).await
                                .unwrap();
                            }
                        }
                    }
                },
                Err(err) => {
                    let _ = msg.channel_id.send_message(&ctx, 
                        CreateMessage::new().content(err)
                    ).await;
                }
            }
            
        }
    }
}


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::registry()
    .with(
        EnvFilter::new("learning=info")
    )
    .with(
        fmt::layer()
    )

    .with(
        fmt::layer()
        .with_ansi(false)
        .with_writer(
            std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("TraitBot.log")
            .unwrap()
        )
    )
    .init();

    let token = env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN!");

    let intents = GatewayIntents::GUILDS
    | GatewayIntents::GUILD_MEMBERS
    | GatewayIntents::MESSAGE_CONTENT
    | GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .type_map_insert::<BotState>(BotState {
            message_count: 0,
            start_time: std::time::Instant::now(),
        })
        .await
        .expect("Error creating client connection");

    if let Err(why) = client.start().await {
        error!("client error {:?}", why)
    }

    Ok(())
}
