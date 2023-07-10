mod commands;

use std::env;

use commands::{create_interactive_prompt, token};
use sea_orm::{Database, DatabaseConnection};
use serenity::async_trait;

use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler {
    db: DatabaseConnection,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                token::COMMAND_NAME => {
                    token::run(
                        &command.data.options,
                        self.db.clone(),
                        &command.member.as_ref().unwrap().user.id,
                        &command.member.as_ref().unwrap().user.name,
                    )
                    .await
                }
                create_interactive_prompt::COMMAND_NAME => {
                    create_interactive_prompt::run(
                        &command.data.options,
                        self.db.clone(),
                        command.channel_id,
                        &ctx,
                    )
                    .await
                }
                _ => format!("not implemented :( {}", command.data.name),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| token::register(command))
                .create_application_command(|command| create_interactive_prompt::register(command))
        })
        .await;

        println!(
            "I now have the following guild slash commands: {:#?}",
            commands
        );
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let db = loop {
        let result =
            Database::connect("postgres://postgres:postgres@localhost:5432/postgres").await;

        // If there was an error connecting to the database, sleep for 5 seconds
        // and try again
        match result {
            Ok(db) => break db,
            Err(e) => {
                println!("Failed to connect to database: {}", e);
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    };

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler { db })
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

// #[shuttle_runtime::main]
// async fn serenity(
//     #[shuttle_secrets::Secrets] secret_store: SecretStore,
//     #[shuttle_shared_db::Postgres(
//         local_uri = "postgres://postgres:{secrets.PASSWORD}@localhost:16695/postgres"
//     )]
//     pool: PgPool,
// ) -> shuttle_serenity::ShuttleSerenity {
//     // Get the discord token set in `Secrets.toml`
//     let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

//     let db = Database::connect("postgres://postgres:password@localhost:5432/shuttle")
//         .await
//         .unwrap();

//     // Set gateway intents, which decides what events the bot will be notified about
//     let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

//     let client = Client::builder(&token, intents)
//         .event_handler(Bot)
//         .await
//         .expect("Err creating client");

//     Ok(client.into())
// }
