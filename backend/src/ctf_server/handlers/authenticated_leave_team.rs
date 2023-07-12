use crate::{
    ctf_server::{ActorTask, ActorTaskTo, SendNetworkMessage},
    messages::IncomingCTFRequest,
};

use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState},
    NetworkMessage,
};
use entity::entities::hacker;

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub async fn handle(
    discord_id: i64,
    db_clone: DatabaseConnection,
    tasks: &mut Vec<ActorTask>,
    msg: &IncomingCTFRequest,
) {
    // check that this hacker is on a team

    let mut hacker: hacker::ActiveModel = hacker::Entity::find_by_id(discord_id)
        .one(&db_clone)
        .await
        .expect("Failed to get hacker")
        .unwrap()
        .into();

    // Set the hacker's team to empty
    hacker.fk_team_id = Set(None);

    // Save the hacker in the database
    hacker.update(&db_clone).await.unwrap();

    // Broadcast this new GlobalData to every client
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Team(Vec::new()),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GlobalData(CTFState::get_global_data(&db_clone).await),
        )),
    }));

    // Update the client's TeamData on their hacker leaving a team
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(msg.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::TeamData(
                CTFState::get_hacker_team_data(discord_id, &db_clone).await,
            ),
        )),
    }));
}
