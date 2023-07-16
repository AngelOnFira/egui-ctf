use crate::ctf_server::{ActorTask, ActorTaskTo, CTFServer, HandleData, SendNetworkMessage};

use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState, ClientUpdate},
    NetworkMessage,
};
use entity::entities::{hacker, team};

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

pub async fn handle<'a>(handle_data: &'a mut HandleData<'a>, token: String, agent: Agent) {
    if token.is_empty() {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                "Token cannot be empty".to_string(),
            ))),
            handle_data.recipient_clone.clone(),
        );

        // Return tasks
        return;
    }
    let team: Option<team::Model> = team::Entity::find()
        .filter(team::Column::JoinToken.eq(&token))
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to check if team exists");
    // Make sure the token isn't empty

    // See if there is a team with this token

    match team {
        // If no team exists with this token, return an error message
        None => {
            CTFServer::send_message_associated(
                NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                    "No team exists with this token".to_string(),
                ))),
                handle_data.recipient_clone.clone(),
            );

            // Return tasks
        }
        Some(team) => {
            // Get the hacker associated with this request
            let hacker: hacker::Model = hacker::Entity::find()
                .filter(hacker::Column::DiscordId.eq(discord_id))
                .one(&handle_data.db_clone)
                .await
                .expect("Failed to get hacker")
                .unwrap();

            // If this hacker is already on a team, return an error message
            if hacker.fk_team_id.is_some() {
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                        ClientUpdate::Notification("You are already on a team".to_string()),
                    )),
                    handle_data.recipient_clone.clone(),
                );

                // Return tasks
                return;
            }

            // Update the hacker's team id
            let mut hacker: hacker::ActiveModel = hacker.into();
            hacker.fk_team_id = Set(Some(team.id));
            let hacker_id = hacker.clone().discord_id.unwrap();
            hacker
                .save(&handle_data.db_clone)
                .await
                .expect("Failed to update hacker");

            // Send the hacker a message that they joined a team
            handle_data
                .tasks
                .push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                    to: ActorTaskTo::Session(handle_data.msg.id),
                    message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                        CTFClientStateComponent::ClientData(
                            CTFState::get_hacker_client_data(hacker_id, &handle_data.db_clone)
                                .await,
                        ),
                    )),
                }));

            // Send the hacker their team data
            handle_data
                .tasks
                .push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                    to: ActorTaskTo::Session(handle_data.msg.id),
                    message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                        CTFClientStateComponent::TeamData(
                            CTFState::get_hacker_team_data(hacker_id, &handle_data.db_clone).await,
                        ),
                    )),
                }));

            // Send the hacker a notification that they joined a team
            CTFServer::send_message_associated(
                NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                    format!("You joined team {}", team.name),
                ))),
                handle_data.recipient_clone.clone(),
            );
        }
    }
}
