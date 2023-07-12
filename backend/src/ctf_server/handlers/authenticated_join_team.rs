use crate::{
    ctf_server::{ActorTask, ActorTaskTo, CTFServer, SendNetworkMessage},
    messages::{IncomingCTFRequest, WsActorMessage},
};
use actix::prelude::Recipient;
use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState, ClientUpdate},
    NetworkMessage,
};
use entity::entities::{hacker, team};

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub async fn handle(
    token: String,
    recipient_clone: &Recipient<WsActorMessage>,
    tasks: &mut Vec<ActorTask>,
    db_clone: &DatabaseConnection,
    discord_id: i64,
    msg: &IncomingCTFRequest,
) -> Option<Vec<ActorTask>> {
    if token.is_empty() {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                "Token cannot be empty".to_string(),
            ))),
            recipient_clone.clone(),
        );

        // Return tasks
        return Some(tasks.clone());
    }
    let team: Option<team::Model> = team::Entity::find()
        .filter(team::Column::JoinToken.eq(&token))
        .one(db_clone)
        .await
        .expect("Failed to check if team exists");
    // Make sure the token isn't empty

    // See if there is a team with this token

    match team {
        // If no team exists with this token, return an
        // error message
        None => {
            CTFServer::send_message_associated(
                NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                    "No team exists with this token".to_string(),
                ))),
                recipient_clone.clone(),
            );

            // Return tasks
            return Some(tasks.clone());
        }
        Some(team) => {
            // Get the hacker associated with this
            // request
            let hacker: hacker::Model = hacker::Entity::find()
                .filter(hacker::Column::DiscordId.eq(discord_id))
                .one(db_clone)
                .await
                .expect("Failed to get hacker")
                .unwrap();

            // If this hacker is already on a team,
            // return an error message
            if hacker.fk_team_id.is_some() {
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                        ClientUpdate::Notification("You are already on a team".to_string()),
                    )),
                    recipient_clone.clone(),
                );

                // Return tasks
                return Some(tasks.clone());
            }

            // Update the hacker's team id
            let mut hacker: hacker::ActiveModel = hacker.into();
            hacker.fk_team_id = Set(Some(team.id));
            let hacker_id = hacker.clone().discord_id.unwrap();
            hacker
                .save(db_clone)
                .await
                .expect("Failed to update hacker");

            // Send the hacker a message that they
            // joined a team
            tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                to: ActorTaskTo::Session(msg.id),
                message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                    CTFClientStateComponent::ClientData(
                        CTFState::get_hacker_client_data(hacker_id, db_clone).await,
                    ),
                )),
            }));

            // Send the hacker their team data
            tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                to: ActorTaskTo::Session(msg.id),
                message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                    CTFClientStateComponent::TeamData(
                        CTFState::get_hacker_team_data(hacker_id, db_clone).await,
                    ),
                )),
            }));

            // Send the hacker a notification that they
            // joined a team
            CTFServer::send_message_associated(
                NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                    format!("You joined team {}", team.name),
                ))),
                recipient_clone.clone(),
            );
        }
    }
    None
}
