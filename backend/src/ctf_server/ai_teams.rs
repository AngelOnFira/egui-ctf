use std::collections::HashMap;

use entity::entities::team;
use sea_orm::{DatabaseConnection, EntityTrait};

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
            let seconds_for_chance = 60.0;

            if rand::random::<f32>() < (1.0 / seconds_for_chance) {
                // Find a challenge that this team hasn't solved. Do this by
                // getting a list of all the challenges they have unsolved, and
                // pick one of them at random.
                
            }
        }
    }
}
