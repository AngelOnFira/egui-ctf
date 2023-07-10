use entity::entities::{hacker, token};
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serenity::builder::{CreateApplicationCommand, CreateButton};
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::prelude::{ChannelId, UserId, ReactionType};

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
                    c.create_action_row(|r| {
                        // add_XXX methods are an alternative to create_XXX methods
                        r.add_button(
                            CreateButton::default()
                                .custom_id("cat")
                                .label("Get login token")
                                .style(ButtonStyle::Primary)
                                .to_owned(),
                        )
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
