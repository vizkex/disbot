use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;
const INFO: &'static str = "hello im vizkex bot";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content == "!commands" {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, "the commands for the bot is the following")
                .await
            {
                println!("ERROR sending the massage: {why:?}");
            }
        } else if msg.content == "!info" {
            if let Err(why) = msg.channel_id.say(&ctx.http, INFO).await {
                println!("ERROR sending the massage: {why:?}");
            }
        }
    }
}
#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
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
