use entity::entities::message_component_data;
use sea_orm::prelude::Uuid;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serenity::builder::{CreateApplicationCommand, CreateButton};
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::prelude::ChannelId;

use crate::commands::{StoredDiscordTask, TaskType};

pub const COMMAND_NAME: &str = "create_interactive_prompt";

pub async fn run(
    _options: &[CommandDataOption],
    db: DatabaseConnection,
    channel_id: ChannelId,
    ctx: &serenity::client::Context,
) -> String {
    // Create a entry in the database to track the prompt
    let get_login_token_task = message_component_data::ActiveModel {
        id_uuid: Set(Uuid::new_v4()),
        payload: Set(
            serde_json::to_value(&StoredDiscordTask::Task(TaskType::CreateToken)).unwrap(),
        ),
        ..Default::default()
    }
    .insert(&db)
    .await
    .unwrap();

    // The menu that people will be able to use to get their login token
    let _m = channel_id
        .send_message(&ctx, |m| {
            m.content(
                "# Help menu\n\nYou can create a login token so that you can access the web ui.",
            )
            .components(|c| {
                c.create_action_row(|r| {
                    r.add_button(
                        CreateButton::default()
                            .custom_id(get_login_token_task.id_uuid)
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
    "".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(COMMAND_NAME)
        .description("Create a prompt box for others to use")
}
