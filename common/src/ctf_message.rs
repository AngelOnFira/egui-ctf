use entity::entities::{hacker, team};
use iter_tools::Itertools;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CTFMessage {
    /// A subset of the information stored in the CTF state, to be passed to the client
    CTFClientState(CTFClientStateComponent),
    SubmitFlag(String),
    /// Tell a specific client that something that matters to them has happened
    /// (They submitted a flag correctly a team member went offline, etc.)
    ClientUpdate(ClientUpdate),
    /// Login token being submitted
    Login(String),
}

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
    pub fn get_client_state(&self, client_id: Uuid, db: &DatabaseConnection) -> CTFClientState {
        // Build the team and client data. First, see if the client is logged
        // in, then see if they're on a team.
        // let logged_in
        // let client_data, team_data = match 


        CTFClientState {
            global_data: GlobalData {
                hacker_teams: self.hacker_teams.clone()
            },
            game_data: GameData { challenges: Vec::new() },
            team_data: TeamData {
                team: HackerTeam { name: (), hackers: () },
            },
            client_data: ClientData {
            },
        }
    }

    /// Rebuild the state from the database. It might be better to just update
    /// state then flush it to the database or something, but whatever, it's a
    /// cheap operation on this size of data.
    pub async fn rebuild_state(db: &DatabaseConnection) -> Self {
        // Get all the teams from the database
        let teams = team::Entity::find()
            .all(db)
            .await
            .expect("Failed to get all teams");

        // Get all the players from the database
        let hackers = hacker::Entity::find()
            .all(db)
            .await
            .expect("Failed to get all players");

        // Sort the teams by username
        let teams = teams
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

// This struct is used to store all client-side state data about the CTF. It
// won't be passed over the network as-is, since sometimes only updates to
// certain fields might need to be made.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CTFClientState {
    // Any data that anyone can see about the CTF
    pub global_data: GlobalData,
    // Any data that only logged in players can see about the CTF. This will be
    // none if the client is not logged in.
    pub game_data: GameData,
    // Any data that only this client's team can see. This will be none if the
    // client is not logged in, or not on a team.
    pub team_data: Option<TeamData>,
    // Any data that only this client can see about the CTF, such as their
    // settings. This will be none if the client is not logged in.
    pub client_data: Option<ClientData>,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameData {
    pub challenges: Vec<CTFChallenge>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamData {
    team: HackerTeam,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientData {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HackerTeam {
    pub name: String,
    pub hackers: Vec<Hacker>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hacker {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CTFChallenge {}

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
}
