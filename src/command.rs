use anyhow::Result;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serenity::builder::CreateMessage;

use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

type Context<'a> = poise::Context<'a, Data, anyhow::Error>;

pub struct Data {
    pub config: Arc<RwLock<ConfigData>>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct ConfigData {
    pub log_channel: Option<u64>,
    pub boost_channel: Option<u64>,
}

const CONFIG_FILE: &str = "bot_config.json";

impl ConfigData {
    async fn load() -> Result<Self> {
        if Path::new(CONFIG_FILE).exists() {
            let content = fs::read_to_string(CONFIG_FILE).await?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(ConfigData::default())
        }
    }

    async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(CONFIG_FILE, content).await?;
        Ok(())
    }
}

impl Data {
    async fn new() -> Result<Self> {
        let config = ConfigData::load().await?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
        })
    }

    async fn save_config(&self) -> Result<()> {
        let config = self.config.read().await;
        config.save().await
    }
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<()> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
async fn set_log_channel(
    ctx: poise::ApplicationContext<'_, Data, anyhow::Error>,
    #[description = "Channel to send log messages"] channel: serenity::Channel,
) -> Result<()> {
    if let serenity::Channel::Guild(channel) = channel {
        let mut config = ctx.data().config.write().await;
        config.log_channel = Some(channel.id.get());
        drop(config); // Release the lock before saving
        ctx.data().save_config().await?;
        ctx.say(format!("Log channel set to {}", channel.name))
            .await?;
    } else {
        ctx.say("Please provide a valid guild text channel.")
            .await?;
    }
    Ok(())
}

#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
async fn set_boost_channel(
    ctx: poise::ApplicationContext<'_, Data, anyhow::Error>,
    #[description = "Channel to send boost notifications"] channel: serenity::Channel,
) -> Result<()> {
    if let serenity::Channel::Guild(channel) = channel {
        let mut config = ctx.data().config.write().await;
        config.boost_channel = Some(channel.id.get());
        drop(config); // Release the lock before saving
        ctx.data().save_config().await?;
        ctx.say(format!(
            "Boost notification channel set to {}",
            channel.name
        ))
        .await?;
    } else {
        ctx.say("Please provide a valid guild text channel.")
            .await?;
    }
    Ok(())
}

#[poise::command(slash_command, guild_only, required_permissions = "BAN_MEMBERS")]
async fn ban(
    ctx: poise::ApplicationContext<'_, Data, anyhow::Error>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Reason for banning"] reason: Option<String>,
) -> Result<()> {
    let guild_id = ctx.guild_id().unwrap();
    let reason = reason.unwrap_or_else(|| "No reason provided".to_string());
    let admin = ctx.author();

    guild_id
        .ban_with_reason(&ctx.serenity_context(), user.id, 0, &reason)
        .await?;
    ctx.say(format!("Banned {} for: {}", user.name, reason))
        .await?;

    // Send log message to the specified log channel
    if let Some(log_channel_id) = ctx.data().config.read().await.log_channel {
        let content = format!(
            "User {} (ID: {}) was banned by {} (ID: {}) for: {}",
            user.name, user.id, admin.name, admin.id, reason
        );
        let channel_id = serenity::ChannelId::new(log_channel_id);
        channel_id
            .send_message(
                &ctx.serenity_context().http,
                CreateMessage::new().content(content),
            )
            .await?;
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn send_message(
    ctx: Context<'_>,
    #[description = "The channel to send the message to"] channel: serenity::Channel,
    #[description = "The message content (use \\n for new lines)"] message: String,
) -> Result<()> {
    let formatted_message = message.replace("\\n", "\n");

    if let serenity::Channel::Guild(guild_channel) = channel {
        if let Err(why) = guild_channel.say(&ctx.http(), &formatted_message).await {
            ctx.say(format!("Error sending message: {:?}", why)).await?;
        } else {
            ctx.say("Message sent successfully!").await?;
        }
    } else {
        ctx.say("Please provide a valid guild channel.").await?;
    }
    Ok(())
}
#[warn(unused_variables)]
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, anyhow::Error>,
    data: &Data,
) -> Result<()> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::GuildMemberUpdate {
            old_if_available,
            new,
            event: _,
        } => {
            let boost_channel_id = data.config.read().await.boost_channel;
            if let Some(channel_id) = boost_channel_id {
                if let Some(new_member) = new {
                    if new_member.premium_since.is_some() && old_if_available.is_none() {
                        let content = format!("{} has boosted the server!", new_member.user.name);
                        let channel_id = serenity::ChannelId::new(channel_id);
                        channel_id
                            .send_message(&ctx.http, CreateMessage::new().content(content))
                            .await?;
                    }
                }
            }
        }
        //        #[allow(unreachable_patterns)]
        //       serenity::FullEvent::GuildMemberUpdate {
        //            old_if_available,
        //            new,
        //            ..
        //        } => {
        //            let boost_channel_id = data.config.read().await.boost_channel;
        //            if let Some(channel_id) = boost_channel_id {
        //                if let Some(new_member) = new {
        //                    if new_member.premium_since.is_some() && old_if_available.is_none() {
        //                        let content =
        //                           format!("{} has boosted the server! thx bro", new_member.user.name);
        //                        let channel_id = serenity::ChannelId::new(channel_id);
        //                        channel_id
        //                            .send_message(&ctx.http, CreateMessage::new().content(content))
        //                            .await?;
        //                    }
        //                }
        //            }
        //       }
        serenity::FullEvent::ChannelUpdate { old, new } => {
            let log_channel_id = data.config.read().await.log_channel;
            if let Some(channel_id) = log_channel_id {
                let message = match (old, new) {
                    (Some(old_channel), new_channel) => {
                        let mut changes = Vec::new();

                        if old_channel.name != new_channel.name() {
                            changes.push(format!(
                                "Name: '{}' -> '{}'",
                                old_channel.name,
                                new_channel.name()
                            ));
                        }
                        if old_channel.topic != new_channel.topic {
                            changes.push(format!(
                                "Topic: '{:?}' -> '{:?}'",
                                old_channel.topic, new_channel.topic
                            ));
                        }
                        if old_channel.nsfw != new_channel.nsfw {
                            changes.push(format!(
                                "NSFW: {} -> {}",
                                old_channel.nsfw, new_channel.nsfw
                            ));
                        }
                        // Access rate_limit_per_user as a field
                        if old_channel.rate_limit_per_user != new_channel.rate_limit_per_user {
                            changes.push(format!(
                                "Slowmode: {:?} -> {:?}",
                                old_channel.rate_limit_per_user, new_channel.rate_limit_per_user
                            ));
                        }

                        if changes.is_empty() {
                            format!("Channel '{}' (ID: {}) was updated, but no visible changes were detected.", new_channel.name(), new_channel.id)
                        } else {
                            format!(
                                "Channel '{}' (ID: {}) was updated. Changes:\n{}",
                                new_channel.name(),
                                new_channel.id,
                                changes.join("\n")
                            )
                        }
                    }
                    (None, new_channel) => {
                        format!(
                            "Channel '{}' (ID: {}) was updated, but old data is not available.",
                            new_channel.name(),
                            new_channel.id
                        )
                    }
                };

                serenity::ChannelId::new(channel_id)
                    .send_message(&ctx.http, CreateMessage::new().content(message))
                    .await?;
            }
        }
        // serenity::FullEvent::ChannelUpdate { old, new } => {
        //    let notification_channel = data.config.read().await.log_channel;
        //    if let Some(channel_id) = notification_channel {
        //        let message = match old {
        //            Some(old_channel) => format!("Channel {} has been updated!", old_channel.name),
        //            None => format!(
        //                "A channel (now named {}) has been updated, but old data is not available.",
        //                new.name()
        //            ),
        //        };
        //        serenity::ChannelId::new(channel_id)
        //            .send_message(&ctx.http, CreateMessage::new().content(message))
        //            .await?;
        //    }
        //}
        _ => {}
    }
    Ok(())
}

pub async fn start() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                send_message(),
                set_boost_channel(),
                set_log_channel(),
                age(),
                ban(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let data = Data::new().await.expect("Failed to initialize bot data");
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
