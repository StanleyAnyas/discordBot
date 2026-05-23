use serenity::{
    all::{
        Context,
        GuildChannel,
        Message,
    },
    builder::{
        CreateEmbed,
        CreateMessage,
    },
};

use tokio::time::{sleep, Duration};


#[allow(dead_code)]
pub async fn send_channel_intro(ctx: Context, msg: Message, guild_member: GuildChannel){
    if msg.author.bot {
        return;
    }

    if msg.content.starts_with("!") {
        return;
    }

    let (
        title,
        description,
        color, 
        footer,
    ) = match guild_member.name.as_str() {
        "rust-help" => (
            "Rust help",
            "Ask your Rust questions here \n\n
            show your code \n\
            use code blocks",
            0xf74c00,
            "TraitBot Rust Assistance",
        ),

        _ => {
            return;
        }
    };

    let _ = msg.channel_id.broadcast_typing(&ctx.http).await;
    let embed = CreateEmbed::new()
    .title(title)
    .description(description)
    .color(color)
    .field("User", 
    format!("@<{}>", msg.author.id.get()), true)
    .field("Channel", 
    format!("@<{}>", guild_member.id.get()), true)
    .footer(
        serenity::builder::CreateEmbedFooter::new(footer)
    );
    
    let send_message = msg.channel_id.send_message(&ctx, 
        CreateMessage::new().embed(embed)
    ).await;

    if let Ok(message) = send_message {
        sleep(Duration::from_secs(10)).await;
        let _ = message.delete(&ctx.http).await;
    }
}
