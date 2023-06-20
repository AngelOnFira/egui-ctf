use entity::entities::hacker;
use iter_tools::Itertools;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CTFMessage {
    CTFClientState(CTFClientState),
}

pub trait CTFMessageTag {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CTFState {
    pub hacker_teams: Vec<HackerTeam>,
}

impl Default for CTFState {
    fn default() -> Self {
        Self {
            hacker_teams: vec![],
        }
    }
}

impl CTFState {
    /// Build a state for the client to see
    pub fn get_client_state(&self) -> CTFClientState {
        CTFClientState {
            hacker_teams: self.hacker_teams.clone(),
        }
    }

    ///
    pub async fn rebuild_state(db: &DatabaseConnection) -> Self {
        // Get all the teams from the database
        let teams = hacker::Entity::find()
            .all(db)
            .await
            .expect("Failed to get all teams");

        // Get all the players from the database
        let players = hacker::Entity::find()
            .all(db)
            .await
            .expect("Failed to get all players");

        // Sort the teams by username
        let teams = teams
            .iter()
            .sorted_by(|a, b| a.username.cmp(&b.username))
            .map(|team| {
                let hackers = players
                    .iter()
                    .filter(|player| player.fk_team_id == Some(team.id))
                    .map(|player| Hacker {
                        name: player.username.clone(),
                    })
                    .collect::<Vec<Hacker>>();

                HackerTeam {
                    name: team.username.clone(),
                    hackers,
                }
            })
            .collect::<Vec<HackerTeam>>();

        // Return the new state
        CTFState {
            hacker_teams: teams,
        }
    }
}

/// A subset of the information stored in the CTF state, to be passed to the client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CTFClientState {
    pub hacker_teams: Vec<HackerTeam>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HackerTeam {
    pub name: String,
    pub hackers: Vec<Hacker>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hacker {
    pub name: String,
}

impl CTFMessageTag for CTFState {}
