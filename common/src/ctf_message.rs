use std::collections::HashMap;

use entity::entities::{challenge, hacker, submission, team};
use iter_tools::Itertools;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CTFMessage {
    /// A client wants to connect and get information about the game, but isn't
    /// authenticated
    Connect,
    /// Login token being submitted
    Login(String),
    /// A client wants to be logged out
    Logout,
    /// A subset of the information stored in the CTF state, to be passed to the client
    CTFClientStateComponent(CTFClientStateComponent),
    SubmitFlag {
        challenge_name: String,
        flag: String,
    },
    /// Tell a specific client that something that matters to them has happened
    /// (They submitted a flag correctly a team member went offline, etc.)
    ClientUpdate(ClientUpdate),
    /// Team token being submitted by player
    JoinTeam(String),
    /// Team name being submitted by player
    CreateTeam(String),
    /// Player leaving their team
    LeaveTeam,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CTFState {
    pub hacker_teams: Vec<HackerTeam>,
}

impl CTFState {
    // /// Build a state for the client to see
    // pub fn get_client_state(&self, client_id: Uuid, db: &DatabaseConnection) -> CTFClientState {
    //     // Build the team and client data. First, see if the client is logged
    //     // in, then see if they're on a team.
    //     // let logged_in
    //     // let client_data, team_data = match

    //     CTFClientState {
    //         global_data: GlobalData {
    //             hacker_teams: self.hacker_teams.clone(),
    //         },
    //         game_data: GameData {
    //             challenges: Vec::new(),
    //         },
    //         team_data: TeamData {
    //             team: HackerTeam {
    //                 name: (),
    //                 hackers: (),
    //             },
    //         },
    //         client_data: ClientData {},
    //     }
    // }

    /// Build a hackers's team data
    pub async fn get_hacker_team_data(client_id: &String, db: &DatabaseConnection) -> TeamData {
        // Get the hacker
        let hacker = hacker::Entity::find()
            .filter(hacker::Column::DiscordId.eq(client_id))
            .one(db)
            .await
            .expect("Failed to get hacker")
            .unwrap();

        // If the hacker isn't on a team, return that
        if hacker.fk_team_id.is_none() {
            return TeamData::NoTeam;
        }

        // Get the team from the hacker
        let team = team::Entity::find()
            .filter(team::Column::Id.eq(hacker.fk_team_id.unwrap()))
            .one(db)
            .await
            .expect("Failed to get team")
            .unwrap();

        // Find all the hackers that are on this team
        let hackers = hacker::Entity::find()
            .filter(hacker::Column::FkTeamId.eq(team.id))
            .all(db)
            .await
            .expect("Failed to get hackers");

        TeamData::OnTeam(HackerTeam {
            name: team.name,
            join_token: team.join_token,
            hackers: hackers
                .iter()
                .map(|h| Hacker {
                    name: h.username.clone(),
                })
                .collect(),
        })
    }

    // Build a hacker's client data
    pub async fn get_hacker_client_data(client_id: &String, db: &DatabaseConnection) -> ClientData {
        // Get the hacker
        let hacker = hacker::Entity::find()
            .filter(hacker::Column::DiscordId.eq(client_id))
            .one(db)
            .await
            .expect("Failed to get hacker")
            .unwrap();

        ClientData::LoggedIn {
            username: hacker.username,
        }
    }

    // Build the game data
    pub async fn get_game_data(db: &DatabaseConnection) -> GameData {
        // Get all the challenges
        let challenges = challenge::Entity::find()
            .all(db)
            .await
            .expect("Failed to get challenges");

        // TODO: Only send them challenges that their team has unlocked if there
        // are pre-requisites
        GameData::LoggedIn {
            challenges: challenges
                .iter()
                .filter(|challenge| challenge.active)
                .map(|challenge| CTFChallenge {
                    title: challenge.title.clone(),
                    category: challenge.category.clone(),
                    description: challenge.description.clone(),
                    link: challenge.link.clone(),
                    points: challenge.points,
                    author: challenge.author.clone(),
                })
                .collect(),
        }
    }

    /// Rebuild the state from the database. It might be better to just update
    /// state then flush it to the database or something, but whatever, it's a
    /// cheap operation on this size of data.
    pub async fn get_global_data(db: &DatabaseConnection) -> GlobalData {
        // Get all the teams from the database
        let database_teams = team::Entity::find()
            .all(db)
            .await
            .expect("Failed to get all teams");

        // Get all the players from the database
        let hackers = hacker::Entity::find()
            .all(db)
            .await
            .expect("Failed to get all players");

        // Sort the teams by username
        let teams = database_teams
            .clone()
            .iter()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .map(|team| {
                let hackers = hackers
                    .iter()
                    .filter(|player| player.fk_team_id == Some(team.id))
                    .map(|player| Hacker {
                        name: player.username.clone(),
                    })
                    .collect::<Vec<Hacker>>();

                HackerTeam {
                    name: team.name.clone(),
                    join_token: team.join_token.clone(),
                    hackers,
                }
            })
            .collect::<Vec<HackerTeam>>();

        // Get the scoreboard data

        // Create a map from team id to team name
        let team_names = database_teams
            .clone()
            .iter()
            .map(|team| (team.name.clone(), team.id))
            .collect::<HashMap<TeamName, TeamId>>();

        // Find all the correct submissions
        let _solves = submission::Entity::find()
            .filter(submission::Column::Correct.eq(true))
            .all(db)
            .await
            .expect("Failed to get all submissions");

        let mut scoreboard: Scoreboard = Scoreboard {
            teams: HashMap::new(),
        };
        for team in teams.clone() {
            // Get all the solves this team has made
            let solves = submission::Entity::find()
                .filter(submission::Column::Correct.eq(true))
                .filter(submission::Column::FkTeamId.eq(*team_names.get(&team.name).unwrap()))
                .all(db)
                .await
                .expect("Failed to get all submissions");

            for solve in solves {
                // TODO: Check that we're not giving multiple points for the
                // same challenge

                // Get the challenge from the database
                let challenge = challenge::Entity::find()
                    .filter(challenge::Column::Id.eq(solve.fk_challenge_id))
                    .one(db)
                    .await
                    .expect("Failed to get challenge")
                    .unwrap();

                scoreboard
                    .teams
                    .entry(team.name.clone())
                    .or_insert(Vec::new())
                    .push(Solve {
                        points: challenge.points as u32,
                        time: solve.time.parse().unwrap(),
                    });
            }
        }

        // Return the new state
        GlobalData {
            hacker_teams: teams,
            scoreboard,
        }
    }
}

// This struct is used to store all client-side state data about the CTF. It
// won't be passed over the network as-is, since sometimes only updates to
// certain fields might need to be made.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CTFClientState {
    // Any data that anyone can see about the CTF
    pub global_data: Option<GlobalData>,
    // Any data that only logged in players can see about the CTF.
    pub game_data: GameData,
    // Any data that only this client's team can see.
    pub team_data: TeamData,
    // Any data that only this client can see about the CTF, such as their
    // settings.
    pub client_data: ClientData,
}

impl Default for CTFClientState {
    fn default() -> Self {
        Self {
            global_data: None,
            game_data: GameData::LoggedOut,
            team_data: TeamData::NoTeam,
            client_data: ClientData::LoggedOut,
        }
    }
}

// This is the counterpart to the CTFClientState above. It's used to send
// updates to the client about the CTF, and will only contain the data that
// needs to be updated.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CTFClientStateComponent {
    GlobalData(GlobalData),
    GameData(GameData),
    TeamData(TeamData),
    ClientData(ClientData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalData {
    pub hacker_teams: Vec<HackerTeam>,
    pub scoreboard: Scoreboard,
}

pub type TeamId = i32;
pub type TeamName = String;

/// For the scoreboard, we're going to need to know what solves the team has
/// made, and at what times.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scoreboard {
    pub teams: HashMap<TeamName, Vec<Solve>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Solve {
    pub points: u32,
    pub time: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameData {
    LoggedOut,
    LoggedIn { challenges: Vec<CTFChallenge> },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TeamData {
    NoTeam,
    OnTeam(HackerTeam),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientData {
    LoggedOut,
    LoggedIn { username: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HackerTeam {
    pub name: String,
    pub join_token: String,
    pub hackers: Vec<Hacker>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hacker {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CTFChallenge {
    pub title: String,
    pub category: String,
    pub description: String,
    pub link: String,
    pub points: i32,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientUpdate {
    /// This client correctly authenticated with a token
    Authenticated {
        discord_username: String,
        valid_token: String,
    }, // <-- TODO: Send them their discord info
    /// This client entered an incorrect token
    IncorrectToken,
    /// This client scored a point
    ScoredPoint(String),
    /// This client's team scored a point
    TeamScoredPoint,
    /// This client submitted an incorrect flag
    IncorrectFlag(String),
    /// General notification
    Notification(String),
}
