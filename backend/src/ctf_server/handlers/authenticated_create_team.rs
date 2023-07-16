use crate::ctf_server::{ActorTask, ActorTaskTo, CTFServer, HandleData, SendNetworkMessage};

use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState, ClientUpdate},
    NetworkMessage,
};
use entity::entities::{hacker, team};

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use uuid::Uuid;

pub async fn handle<'a>(handle_data: &'a mut HandleData<'a>, team_name: String, discord_id: i64) {
    if team_name.is_empty() {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                "Team name cannot be empty".to_string(),
            ))),
            handle_data.recipient_clone.clone(),
        );

        // Return tasks
        return;
    }
    let team_exists: bool = team::Entity::find()
        .filter(team::Column::Name.eq(&team_name))
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to check if team exists")
        .is_some();
    if team_exists {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                format!("Team '{}' already exists", team_name),
            ))),
            handle_data.recipient_clone.clone(),
        );

        // Return tasks
        return;
    }
    let team = team::ActiveModel {
        name: Set(team_name),
        join_token: Set(Uuid::new_v4().as_simple().to_string()),
        ..Default::default()
    }
    .insert(&handle_data.db_clone)
    .await
    .unwrap();
    let mut hacker: hacker::ActiveModel = hacker::Entity::find_by_id(discord_id)
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to get hacker")
        .unwrap()
        .into();
    hacker.fk_team_id = Set(Some(team.id));
    hacker.update(&handle_data.db_clone).await.unwrap();
    handle_data
        .tasks
        .push(ActorTask::SendNetworkMessage(SendNetworkMessage {
            to: ActorTaskTo::Team(Vec::new()),
            message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                CTFClientStateComponent::GlobalData(
                    CTFState::get_global_data(&handle_data.db_clone).await,
                ),
            )),
        }));
    handle_data
        .tasks
        .push(ActorTask::SendNetworkMessage(SendNetworkMessage {
            to: ActorTaskTo::Session(handle_data.request.id),
            message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                CTFClientStateComponent::TeamData(
                    CTFState::get_hacker_team_data(discord_id, &handle_data.db_clone).await,
                ),
            )),
        }));
    // TODO: Check if this user is already on a team

    // If the team name is empty, return an error message

    // Check if a team by this name already exists in the database

    // If a team by this name already exists, return an error message

    // Create a new team in the database

    // Set this team as the hacker's team

    // Set the hacker's team

    // Save the hacker in the database

    // Broadcast this new GlobalData to every client

    // Update the client's TeamData on their hacker joining a team
}
