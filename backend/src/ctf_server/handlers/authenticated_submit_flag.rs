use crate::{
    ctf_server::{ActorTask, ActorTaskTo, CTFServer, HandleData, SendNetworkMessage},
    messages::WsActorMessage,
};
use actix::prelude::Recipient;
use chrono::NaiveDateTime;
use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState, ClientUpdate},
    NetworkMessage,
};
use entity::entities::{challenge, hacker, submission, team};

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn handle<'a>(
    handle_data: &'a mut HandleData<'a>,
    challenge_name: String,
    discord_id: i64,
    flag: String,
) -> Option<Vec<ActorTask>> {
    let challenge = challenge::Entity::find()
        .filter(challenge::Column::Title.eq(&challenge_name))
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to get challenge");

    let hacker = hacker::Entity::find_by_id(discord_id)
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to get hacker");

    if hacker.as_ref().unwrap().fk_team_id.is_none() {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                "You are not on a team, you can't submit a flag".to_string(),
            ))),
            handle_data.recipient_clone.clone(),
        );
        return Some(handle_data.tasks.clone());
    }

    let team = team::Entity::find_by_id(hacker.as_ref().unwrap().fk_team_id.unwrap())
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to get team");

    let existing_correct_submission = submission::Entity::find()
        .filter(submission::Column::FkChallengeId.eq(challenge.as_ref().unwrap().id))
        .filter(submission::Column::FkTeamId.eq(team.as_ref().unwrap().id))
        .filter(submission::Column::Correct.eq(true))
        .one(&handle_data.db_clone)
        .await
        .expect("Failed to get existing correct submission");

    if existing_correct_submission.is_some() {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                "Your team has already solved this challenge!".to_string(),
            ))),
            handle_data.recipient_clone.clone(),
        );
        return Some(handle_data.tasks.clone());
    }

    let now: std::time::Duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();

    let mut submission = submission::ActiveModel {
        flag: Set(flag.clone()),
        // Get the current time as a string
        time: Set(
            NaiveDateTime::from_timestamp_opt(now.as_secs() as i64, now.subsec_nanos()).unwrap(),
        ),
        fk_hacker_id: Set(Some(hacker.unwrap().discord_id)),
        fk_team_id: Set(Some(team.unwrap().id)),
        ..Default::default()
    };
    // Check the database to see if there are any challenges with this name

    // Get the hacker that made this submission

    // If they aren't on a team, then they can't submit a flag. Tell them that.

    // Get the hacker's team

    // Make sure there isn't already a submission for this challenge by this
    // team that is correct

    // If there is already a correct submission, then tell them that they can't
    // submit another flag for the challenge

    // Prepare the submission to be saved to the database

    match challenge {
        Some(challenge) => {
            submission.fk_challenge_id = Set(Some(challenge.id));

            // See if this channel's flag matches the flag they submitted
            if challenge.flag == flag ||
            // TODO: Remove this lol
            flag == "flag"
            {
                let recipient_clone = handle_data.recipient_clone.clone();
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                        ClientUpdate::ScoredPoint(format!(
                            "You solved {} for {} points!",
                            challenge.title, challenge.points
                        )),
                    )),
                    recipient_clone,
                );

                // Change the submission
                submission.correct = Set(true);
            } else {
                let recipient_clone = handle_data.recipient_clone.clone();
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                        ClientUpdate::ScoredPoint(format!(
                            "That flag didn't solve {}",
                            challenge_name
                        )),
                    )),
                    recipient_clone,
                );

                // Change the submission
                submission.correct = Set(false);
            }

            let solved = *submission.correct.as_ref();

            // Save the submission to the database
            submission.insert(&handle_data.db_clone).await.unwrap();

            if solved {
                // Notify all the online clients about a scoreboard update
                handle_data
                    .tasks
                    .push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                        to: ActorTaskTo::BroadcastAll,
                        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                            CTFClientStateComponent::GlobalData(
                                CTFState::get_global_data(&handle_data.db_clone).await,
                            ),
                        )),
                    }));
            }
        }
        None => {
            // Tell them that this challenge doesn't exist
            let recipient_clone = handle_data.recipient_clone.clone();
            CTFServer::send_message_associated(
                NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::ScoredPoint(
                    "That challenge does not exist".to_string(),
                ))),
                recipient_clone,
            )
        }
    }
    None
}
