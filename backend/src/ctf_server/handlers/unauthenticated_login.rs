use crate::{
    ctf_server::{
        ActixRequest, ActorTask, ActorTaskTo, Auth, CTFServer, HandleData, SendNetworkMessage,
        UpdateState,
    },
    messages::IncomingCTFRequest,
};

use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState, ClientUpdate},
    NetworkMessage,
};
use entity::entities::{hacker, token};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn handle<'a>(handle_data: &'a mut HandleData<'a>, token: String) {
    // Find any tokens in the database that match this token
    let token = token::Entity::find()
        .filter(token::Column::Token.eq(token))
        // Token is a primary key, so only getting one is fine
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to get token");

    // If we have that token, then we can authenticate this websocket connection
    // as the user they say they are
    match token {
        Some(token) => {
            // Get the hacker associated with this token
            let hacker = hacker::Entity::find_by_id(token.fk_hacker_id.unwrap())
                .one(&handle_data.db_clone)
                .await
                .expect("Failed to get hacker");

            // If we have a hacker, then we can authenticate this websocket
            // connection as the user they say they are
            match hacker {
                Some(hacker) => {
                    update_authenticated_user(
                        handle_data.tasks,
                        &handle_data.request,
                        hacker,
                        token,
                        &handle_data.db_clone,
                    )
                    .await;
                }
                // If this token doesn't have a hacker associated with it,
                // something is wrong. This is unreachable.
                None => {
                    panic!("Token has no hacker associated with it");
                }
            }
        }
        None => {
            // If we don't have that token, then we can't authenticate this
            // websocket connection
            CTFServer::send_message_associated(
                NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::IncorrectToken)),
                handle_data.recipient.clone(),
            );
        }
    }
}

async fn update_authenticated_user(
    tasks: &mut Vec<ActorTask>,
    request: &ActixRequest,
    hacker: hacker::Model,
    token: token::Model,
    db_clone: &DatabaseConnection,
) {
    // Tell the client they are authenticated
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(request.id),
        message: NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
            ClientUpdate::Authenticated {
                discord_username: hacker.username.clone(),
                valid_token: token.token.clone(),
            },
        )),
    }));

    // Get the updated state from the database.
    let global_data = CTFState::get_global_data(db_clone).await;

    // Tell every other player that this player has logged in
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::BroadcastAll,
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GlobalData(global_data.clone()),
        )),
    }));

    // Send this client the current game state
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(request.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GameData(CTFState::get_game_data(db_clone).await),
        )),
    }));

    // Update this session's auth state
    tasks.push(ActorTask::UpdateState(UpdateState::SessionAuth {
        auth: Auth::Hacker {
            discord_id: hacker.discord_id,
        },
    }));

    // Update the team on their hacker coming online
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(request.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::TeamData(
                CTFState::get_hacker_team_data(hacker.discord_id, db_clone).await,
            ),
        )),
    }));

    // Update the client on their hacker coming online
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(request.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::ClientData(
                CTFState::get_hacker_client_data(hacker.discord_id, db_clone).await,
            ),
        )),
    }));

    // Update the client with the current scoreboard
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(request.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GlobalData(CTFState::get_global_data(db_clone).await),
        )),
    }));
}
