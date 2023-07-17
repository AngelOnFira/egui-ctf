use actix::Addr;
use chrono::NaiveDateTime;
use common::ctf_message::CTFMessage;
use entity::{
    entities::{hacker, submission, team},
    helpers::get_team_unsolved_challenges,
};
use rand::seq::SliceRandom;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::messages::AnonymousCTFRequest;

use super::{handlers::handle_request, ActixRecipient, ActixRequest, Auth, HandleData, RequestID, CTFServer};

#[derive(Debug, Clone)]
pub struct AITeams;

impl AITeams {
    pub fn new() -> Self {
        AITeams
    }

    // Each time this is run, roll for a chance of a team solving a challenge.
    // It will be rolled once a second, and each team should solve a challenge
    // once every 1 minute on average.
    pub async fn run(&self, db: &DatabaseConnection, addr: &Addr<CTFServer>) {
        // Start by getting the list of teams
        let teams = team::Entity::find().all(db).await.unwrap();

        // Iterate over each team
        for team in teams {
            // Randomly roll for if they solve a challenge

            // Chance is how many seconds it should take on average to solve a challenge
            let seconds_for_chance = 10.0;

            if rand::random::<f32>() < ((1.0 * 5.0) / seconds_for_chance) {
                // Find a random player on this team to be the solver of the
                // challenge. If there isn't any player, create one.
                let hacker: hacker::Model = match hacker::Entity::find()
                    .filter(hacker::Column::FkTeamId.eq(team.id))
                    .all(db)
                    .await
                {
                    Ok(hacker_list) => {
                        // If there is a hacker, pick one at random
                        let random_hacker = hacker_list.choose(&mut rand::thread_rng());
                        match random_hacker {
                            Some(hacker) => hacker.clone(),
                            None => {
                                // If there isn't a hacker, create one
                                let hacker = hacker::ActiveModel {
                                    fk_team_id: Set(Some(team.id)),
                                    // Choose a random discord id, 18 digits long
                                    discord_id: Set(rand::random::<i64>()),
                                    username: Set("AI Hacker".to_string()),
                                };
                                let hacker = hacker.insert(db).await.unwrap();
                                hacker
                            }
                        }
                    }
                    Err(_) => {
                        // Panic idk
                        panic!("Failed to get hackers");
                    }
                };

                // Find a challenge that this team hasn't solved. Do this by
                // getting a list of all the challenges they have unsolved, and
                // pick one of them at random.
                let unsolved_challenges = get_team_unsolved_challenges(db, team.id).await;

                // Pick a random challenge
                let challenge = match unsolved_challenges.choose(&mut rand::thread_rng()) {
                    Some(challenge) => challenge,
                    None => {
                        // If there are no unsolved challenges, skip this team
                        continue;
                    }
                };

                // Send the message to the CTFServer actor
                addr.do_send(AnonymousCTFRequest {
                    ctf_message: CTFMessage::SubmitFlag {
                        challenge_name: challenge.title.clone(),
                        flag: challenge.flag.clone(),
                    },
                    discord_id: hacker.discord_id,
                });

                // // Handle solving it
                // handle_request(
                //     Auth::Hacker {
                //         discord_id: hacker.discord_id,
                //     },
                //     HandleData {
                //         db_clone: db.clone(),
                //         tasks: &mut Vec::new(),
                //         request: ActixRequest {
                //             id: RequestID::Anonymous,
                //             ctf_message: CTFMessage::SubmitFlag {
                //                 challenge_name: challenge.title.clone(),
                //                 flag: challenge.flag.clone(),
                //             },
                //         },
                //         recipient: ActixRecipient::Anonymous,
                //     },
                // )
                // .await;

                // // Create a submission for this challenge
                // submission::ActiveModel {
                //     fk_challenge_id: Set(Some(challenge.id)),
                //     fk_team_id: Set(Some(team.id)),
                //     flag: Set("".to_string()),
                //     // Set the time to now
                //     time: Set(NaiveDateTime::from_timestamp_opt(
                //         now.as_secs() as i64,
                //         now.subsec_nanos(),
                //     )
                //     .unwrap()),
                //     correct: Set(true),
                //     ..Default::default()
                // }
                // .insert(db)
                // .await
                // .unwrap();
            }
        }
    }
}
