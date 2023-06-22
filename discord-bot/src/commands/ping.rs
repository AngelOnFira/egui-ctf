use sea_orm::DatabaseConnection;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

pub fn run(_options: &[CommandDataOption]) -> String {
    "Hey, I'm alive!".to_string()
}

pub fn register(command: &mut CreateApplicationCommand, db: DatabaseConnection) -> &mut CreateApplicationCommand {
    command.name("ping").description("A ping command")
}
