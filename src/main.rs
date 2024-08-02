// #[warn(unused_imports)]
pub mod commands;
// pub mod mafiawolf;
// pub mod rock;

use commands::start;
use dotenv::dotenv;

use serenity::async_trait;

use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;

// TODO impl a simple rook seser paper
struct Handler;
// const INFO: &'static str = "hello im vizkex bot";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // Move the environment variable capture outside the async context
        let channel_id = env::var("CHANNEL_ID")
            .expect("Expected a channel ID in the environment")
            .parse::<u64>()
            .expect("CHANNEL_ID must be a u64");

        // Send "Hello" message to the specified channel
        if let Err(why) = ChannelId::new(channel_id).say(&ctx.http, "Hello").await {
            println!("Error sending message: {:?}", why);
        }
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    start().await;
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
