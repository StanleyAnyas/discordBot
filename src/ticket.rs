
use serenity::{
    all::{
        ChannelId, ChannelType, Context, CreateChannel, Mentionable, Message, PermissionOverwrite, PermissionOverwriteType, Permissions
    }, builder::CreateMessage
};
use tracing::{info, error};

pub async fn create_ticket(ctx: &Context, msg: &Message){
    if let Some(guild_id) = msg.guild_id {
        let channel_name = format!(
            "ticket-{}",
            msg.author.name.to_lowercase()
        );

        let everyone_role = guild_id.everyone_role();

        let ticket_channel = guild_id.create_channel(
            &ctx.http, 
            CreateChannel::new(channel_name.clone()).kind(ChannelType::Text)
            .category(ChannelId::new(1503847756299632860))
            .permissions(
                vec![
                    PermissionOverwrite{
                        allow: Permissions::empty(),
                        deny: Permissions::VIEW_CHANNEL,
                        kind: PermissionOverwriteType::Role(everyone_role),
                    },

                    PermissionOverwrite {
                        allow:
                            Permissions::VIEW_CHANNEL
                            | Permissions::SEND_MESSAGES
                            | Permissions::READ_MESSAGE_HISTORY,
                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Member(
                            msg.author.id
                        ),
                    },

                    PermissionOverwrite {
                        allow: 
                        Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::READ_MESSAGE_HISTORY,

                        deny: Permissions::empty(),
                        kind: PermissionOverwriteType::Member(
                            1501279350215938258u64.into()
                        ),
                    }
                ])
        ).await;

        match ticket_channel {
            Ok(channel) => {
                channel.send_message(&ctx.http, 
                    CreateMessage::new().content(
                        format!(
                            "Ticket created: {}",
                            channel.mention()
                        )
                    )   
                ).await.unwrap();
                info!("created ticket {}", channel.mention());
                
            }

            Err(err) => {
                msg.channel_id.send_message(&ctx.http, 
                    CreateMessage::new().content(
                        format!("error creating {} try again", channel_name)
                    )
                ).await.unwrap();
                // println!("Ticket error: {}", err)
                error!("Ticket error {}", err);
                eprintln!("Ticket error: {}", err)
            }
        }
    }
}