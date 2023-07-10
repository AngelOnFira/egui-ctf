use entity::entities::{hacker, token};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::prelude::UserId;

pub const COMMAND_NAME: &str = "token";

pub async fn run(
    _options: &[CommandDataOption],
    db: DatabaseConnection,
    discord_user_id: &UserId,
    discord_username: &String,
) -> String {
    // Get the hacker from the database, or create a hacker if they aren't in
    // already
    let hacker = hacker::Entity::find()
        .filter(hacker::Column::DiscordId.eq(discord_user_id.0))
        .one(&db)
        .await
        .unwrap();

    let hacker: hacker::Model = match hacker {
        Some(hacker) => hacker,
        None => {
            let hacker = hacker::ActiveModel {
                discord_id: Set(discord_user_id.0 as i64),
                username: Set(discord_username.to_string()),
                ..Default::default()
            };
            dbg!(hacker.clone());
            hacker.insert(&db).await.unwrap()
        }
    };

    // Generate a token for the hacker
    let token: token::Model = token::ActiveModel {
        fk_hacker_id: Set(Some(hacker.discord_id)),
        token: Set(Uuid::new_v4().as_simple().to_string()),
        // 20 minutes from now
        expiry: Set((chrono::Utc::now() + chrono::Duration::minutes(20)).naive_local()),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // Return the token to the user
    format!("Your token is: {}", token.token)
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name(COMMAND_NAME).description("Get a login token")
}
