use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CTFMessage {
    GuessTheNumber(GuessTheNumber),
}

pub trait GameMessageTag {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GuessTheNumber {
    Guess { guess: u32 },
    Start { max: u32 },
    Stop,
}

impl GameMessageTag for GuessTheNumber {}
