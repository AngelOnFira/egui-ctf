use chrono::NaiveDateTime;
use entity::{
    entities::{submission, team},
    helpers::get_team_unsolved_challenges,
};
use rand::seq::SliceRandom;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

#[derive(Debug, Clone)]
pub struct AITeams {}

impl AITeams {
    pub fn new() -> Self {
        AITeams {}
    }

    // Each time this is run, roll for a chance of a team solving a challenge.
    // It will be rolled once a second, and each team should solve a challenge
    // once every 1 minute on average.
    pub async fn run(&self, db: &DatabaseConnection) {
        // Start by getting the list of teams
        let teams = team::Entity::find().all(db).await.unwrap();

        // Iterate over each team
        for team in teams {
            // Randomly roll for if they solve a challenge

            // Chance is how many seconds it should take on average to solve a challenge
            let seconds_for_chance = 10.0;

            if rand::random::<f32>() < ((1.0 * 5.0) / seconds_for_chance) {
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

                let now: std::time::Duration = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap();

                // Create a submission for this challenge
                submission::ActiveModel {
                    fk_challenge_id: Set(Some(challenge.id)),
                    fk_team_id: Set(Some(team.id)),
                    flag: Set("".to_string()),
                    // Set the time to now
                    time: Set(NaiveDateTime::from_timestamp_opt(
                        now.as_secs() as i64,
                        now.subsec_nanos(),
                    )
                    .unwrap()),
                    correct: Set(true),
                    ..Default::default()
                }
                .insert(db)
                .await
                .unwrap();
            }
        }
    }
}
