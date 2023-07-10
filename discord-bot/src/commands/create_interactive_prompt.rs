use entity::entities::{hacker, token};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::prelude::{ChannelId, UserId};

pub const COMMAND_NAME: &str = "create_interactive_prompt";

pub async fn run(
    _options: &[CommandDataOption],
    db: DatabaseConnection,
    channel_id: ChannelId,
    ctx: &serenity::client::Context,
) -> String {
    // Setup the window

    // Ask the user for its favorite animal
    let m = channel_id
        .send_message(&ctx, |m| {
            m.content("Please select your favorite animal")
                .components(|c| {
                    c.create_action_row(|row| {
                        // An action row can only contain one select menu!
                        row.create_select_menu(|menu| {
                            menu.custom_id("animal_select");
                            menu.placeholder("No animal selected");
                            menu.options(|f| {
                                f.create_option(|o| o.label("🐈 meow").value("Cat"));
                                f.create_option(|o| o.label("🐕 woof").value("Dog"));
                                f.create_option(|o| o.label("🐎 neigh").value("Horse"));
                                f.create_option(|o| o.label("🦙 hoooooooonk").value("Alpaca"));
                                f.create_option(|o| o.label("🦀 crab rave").value("Ferris"))
                            })
                        })
                    })
                })
        })
        .await
        .unwrap();

    // Return the token to the user
    format!("Your token is")
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(COMMAND_NAME)
        .description("Create a prompt box for others to use")
}
