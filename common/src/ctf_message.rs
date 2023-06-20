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
    pub score: u32,
}

impl CTFMessageTag for CTFState {}
