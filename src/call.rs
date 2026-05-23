
use serenity::{
    all::{
        ChannelId,
        Context,
        CreateMessage,
        UserId,
        VoiceState
    },
};

use tokio::time::{sleep, Duration};
use tracing::info;

pub async fn handle_voice_join(
    ctx: Context,
    old: Option<VoiceState>,
    new: VoiceState,
) {
    if old.as_ref().and_then(|v| v.channel_id).is_none()
        && new.channel_id.is_some()
    {
        if new.channel_id == Some(ChannelId::new(1504922714471792821)) {
            info!("A user joined the support voice chat");

            let voice_channel_id = new.channel_id.unwrap();
            let user_id = new.user_id;
            let guild_id = match new.guild_id {
                Some(id) => id,
                None => return,
            };

            let ctx_clone = ctx.clone();

            tokio::spawn(async move {
                info!("Waiting for 60 seconds");
                sleep(Duration::from_secs(60)).await;

                info!("Checking if user {:?} is still waiting", user_id);

                let is_still_waiting = if let Some(guild) = ctx_clone.cache.guild(guild_id) {
                    if let Some(vs) = guild.voice_states.get(&user_id) {
                        vs.channel_id == Some(voice_channel_id)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_still_waiting {
                    info!("User {:?} is still waiting", user_id);

                    let notify_channel = ChannelId::new(1504942144358449242);
                    let my_user_id = UserId::new(1338291003085422594);

                    let _ = notify_channel
                        .send_message(
                            &ctx_clone.http,
                            CreateMessage::new().content(format!(
                                "<@{}> someone is waiting in the support call",
                                my_user_id.get()
                            )),
                        )
                        .await;
                }
            });
        }
    }
}