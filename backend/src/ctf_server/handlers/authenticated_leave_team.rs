use crate::ctf_server::{ActorTask, ActorTaskTo, HandleData, SendNetworkMessage};

use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState},
    NetworkMessage,
};
use entity::entities::hacker;

use sea_orm::{ActiveModelTrait, EntityTrait, Set};

pub async fn handle<'a>(handle_data: &'a mut HandleData<'a>, discord_id: i64) {
    // Extract the Discord ID from the agent
    // Check that this hacker is on a team
    let mut hacker: hacker::ActiveModel = hacker::Entity::find_by_id(discord_id)
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to get hacker")
        .unwrap()
        .into();

    // Set the hacker's team to empty
    hacker.fk_team_id = Set(None);

    // Save the hacker in the database
    hacker.update(&handle_data.db_clone).await.unwrap();

    // Broadcast this new GlobalData to every client
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

    // Update the client's TeamData on their hacker leaving a team
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
}
