#[warn(unused_imports)]
use dotenv::dotenv;
mod roulette;
use roulette::Roulette;
use serenity::async_trait;
use serenity::builder::{
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
    CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};
use serenity::futures::StreamExt;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;
use std::time::Duration;

struct Handler;
const INFO: &'static str = "hello im vizkex bot";
fn sound_button(name: &str, emoji: ReactionType) -> CreateButton {
    // To add an emoji to buttons, use .emoji(). The method accepts anything ReactionType or
    // anything that can be converted to it. For a list of that, search Trait Implementations in
    // the docs for From<...>.
    CreateButton::new(name).emoji(emoji)
}
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
        } else if msg.content == "!animal" {
            // Ask the user for its favorite animal
            let m = msg
                .channel_id
                .send_message(
                    &ctx,
                    CreateMessage::new()
                        .content("Please select your favorite animal")
                        .select_menu(
                            CreateSelectMenu::new(
                                "animal_select",
                                CreateSelectMenuKind::String {
                                    options: vec![
                                        CreateSelectMenuOption::new("🐈 meow", "Cat"),
                                        CreateSelectMenuOption::new("🐕 woof", "Dog"),
                                        CreateSelectMenuOption::new("🐎 neigh", "Horse"),
                                        CreateSelectMenuOption::new("🦙 hoooooooonk", "Alpaca"),
                                        CreateSelectMenuOption::new("🦀 crab rave", "Ferris"),
                                    ],
                                },
                            )
                            .custom_id("animal_select")
                            .placeholder("No animal selected"),
                        ),
                )
                .await
                .unwrap();

            // Wait for the user to make a selection
            // This uses a collector to wait for an incoming event without needing to listen for it
            // manually in the EventHandler.
            let interaction = match m
                .await_component_interaction(&ctx.shard)
                .timeout(Duration::from_secs(60 * 3))
                .await
            {
                Some(x) => x,
                None => {
                    m.reply(&ctx, "Timed out").await.unwrap();
                    return;
                }
            };

            // data.values contains the selected value from each select menus. We only have one menu,
            // so we retrieve the first
            let animal = match &interaction.data.kind {
                ComponentInteractionDataKind::StringSelect { values } => &values[0],
                _ => panic!("unexpected interaction data kind"),
            };

            // Acknowledge the interaction and edit the message
            interaction
                .create_response(
                    &ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .content(format!("You chose: **{animal}**\nNow choose a sound!"))
                            .button(sound_button("meow", "🐈".parse().unwrap()))
                            .button(sound_button("woof", "🐕".parse().unwrap()))
                            .button(sound_button("neigh", "🐎".parse().unwrap()))
                            .button(sound_button("hoooooooonk", "🦙".parse().unwrap()))
                            .button(sound_button(
                                "crab rave",
                                // Custom emojis in Discord are represented with
                                // `<:EMOJI_NAME:EMOJI_ID>`. You can see this by posting an emoji in
                                // your server and putting a backslash before the emoji.
                                //
                                // Because ReactionType implements FromStr, we can use .parse() to
                                // convert the textual emoji representation to ReactionType
                                "<:ferris:381919740114763787>".parse().unwrap(),
                            )),
                    ),
                )
                .await
                .unwrap();

            // Wait for multiple interactions
            let mut interaction_stream = m
                .await_component_interaction(&ctx.shard)
                .timeout(Duration::from_secs(60 * 3))
                .stream();

            while let Some(interaction) = interaction_stream.next().await {
                let sound = &interaction.data.custom_id;
                // Acknowledge the interaction and send a reply
                interaction
                    .create_response(
                        &ctx,
                        // This time we dont edit the message but reply to it
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::default()
                                // Make the message hidden for other users by setting `ephemeral(true)`.
                                .ephemeral(true)
                                .content(format!("The **{animal}** says __{sound}__")),
                        ),
                    )
                    .await
                    .unwrap();
            }

            // Delete the orig message or there will be dangling components (components that still
            // exist, but no collector is running so any user who presses them sees an error)
            m.delete(&ctx).await.unwrap()
        } else if msg.content.starts_with("!roulette") {
            let args: Vec<&str> = msg.content.split_whitespace().collect();
            if args.len() != 3 {
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "Usage: !roulette <bet_number> <bet_amount>")
                    .await
                {
                    println!("Error sending message: {:?}", why);
                }
                return;
            }

            let bet_number: i32 = match args[1].parse() {
                Ok(num) => num,
                Err(_) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Invalid bet number").await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
            };

            let bet_amount: i32 = match args[2].parse() {
                Ok(num) => num,
                Err(_) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, "Invalid bet amount").await {
                        println!("Error sending message: {:?}", why);
                    }
                    return;
                }
            };

            let roulette = Roulette::new();
            let (win, payout) = roulette.bet(bet_number, bet_amount);

            let response = if win {
                format!(
                    "You won! The ball landed on {}. You won {}.",
                    bet_number, payout
                )
            } else {
                format!("You lost. The ball landed on {}.", roulette.spin())
            };

            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
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
