use sea_orm::prelude::*;

use crate::entities::{challenge, submission};

/// Get all the challenges this team has solved
pub async fn get_team_solved_challenges(
    db: &DatabaseConnection,
    team_id: i32,
) -> Vec<(submission::Model, challenge::Model)> {
    let mut challenges = Vec::new();
    for solve in submission::Entity::find()
        .filter(submission::Column::Correct.eq(true))
        .filter(submission::Column::FkTeamId.eq(team_id))
        .all(db)
        .await
        .expect("Failed to get all submissions")
    {
        let challenge_id = solve.fk_challenge_id.unwrap();

        challenges.push((
            solve,
            challenge::Entity::find()
                .filter(challenge::Column::Id.eq(challenge_id))
                .one(db)
                .await
                .expect("Failed to get challenge")
                .unwrap(),
        ))
    }

    challenges
}

/// Get all the challenges this team hasn't solved
pub async fn get_team_unsolved_challenges(
    db: &DatabaseConnection,
    team_id: i32,
) -> Vec<challenge::Model> {
    // Get all challenges
    let challenges = crate::entities::challenge::Entity::find()
        .all(db)
        .await
        .expect("Failed to get all challenges");

    // Get all the challenges this team has solved
    let solved_challenges: Vec<(submission::Model, challenge::Model)> =
        get_team_solved_challenges(db, team_id).await;

    // Get all the challenges this team hasn't solved
    let challenges = challenges
        .into_iter()
        .filter(|challenge| {
            !solved_challenges
                .iter()
                .any(|(_, solved_challenge)| solved_challenge.id == challenge.id)
        })
        .collect::<Vec<challenge::Model>>();

    challenges
}
