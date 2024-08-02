use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use serenity::builder::CreateMessage;
use tokio::sync::RwLock;
// use serenity::model::prelude::UserId;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    pub log_channel: RwLock<Option<serenity::ChannelId>>,
    pub boost_channel: RwLock<Option<serenity::ChannelId>>,
    // ... other fields ...
}
/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}
#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
async fn set_boost_channel(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "Channel to send boost notifications"] channel: serenity::Channel,
) -> Result<(), Error> {
    if let serenity::Channel::Guild(channel) = channel {
        let mut boost_channel = ctx.data().boost_channel.write().await;
        *boost_channel = Some(channel.id);
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
#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
async fn set_log_channel(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "Channel to send log messages"] channel: serenity::Channel,
) -> Result<(), Error> {
    if let serenity::Channel::Guild(channel) = channel {
        let mut log_channel = ctx.data().log_channel.write().await;
        *log_channel = Some(channel.id);
        ctx.say(format!("Log channel set to {}", channel.name))
            .await?;
    } else {
        ctx.say("Please provide a valid guild text channel.")
            .await?;
    }

    Ok(())
}

#[poise::command(slash_command, guild_only, required_permissions = "BAN_MEMBERS")]
async fn ban(
    ctx: poise::ApplicationContext<'_, Data, Error>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Reason for banning"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let reason = reason.unwrap_or_else(|| "No reason provided".to_string());
    let admin = ctx.author();

    guild_id
        .ban_with_reason(&ctx.serenity_context(), user.id, 0, &reason)
        .await?;
    ctx.say(format!("Banned {} for: {}", user.name, reason))
        .await?;

    // Send log message to the specified log channel
    if let Some(log_channel_id) = ctx.data().log_channel.read().await.as_ref() {
        let content = format!(
            "User {} (ID: {}) was banned by {} (ID: {}) for: {}",
            user.name, user.id, admin.name, admin.id, reason
        );
        log_channel_id
            .send_message(
                ctx.serenity_context(),
                CreateMessage::new().content(content),
            )
            .await?;
    }

    Ok(())
}
//fn event_handler<'a>(
//    ctx: &'a serenity::Context,
//    event: &'a poise::Event<'a>,
//    _framework: poise::FrameworkContext<'a, Data, Error>,
//    data: &'a Data,
//) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Error>> + Send + 'a>> {
//    Box::pin(async move {
//        match event {
//            poise::Event::GuildMemberUpdate {
//               old_if_available,
//                new,
//            } => {
//                if let (Some(old), Some(boost_channel)) =
//                    (old_if_available, *data.boost_channel.read().await)
//                {
//                    if old.premium_since.is_none() && new.premium_since.is_some() {
//                        let content = format!(
//                            "🎉 Thank you <@{}> for boosting the server! 🚀",
//                            new.user.id
//                        );
//                       let _ = boost_channel
//                            .send_message(&ctx.http, |m| m.content(content))
//                            .await;
//                    }
//                }
//            }
//            // ... other event handlers ...
//            _ => {}
//        }
//       Ok(())
//    })
//}
pub async fn start() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![set_log_channel(), age(), ban()],
            //event_handler: |ctx, event, framework, data| {
            //    Box::pin(event_handler(ctx, event, framework, data))
            //},
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    log_channel: RwLock::new(None),
                    boost_channel: RwLock::new(None),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
