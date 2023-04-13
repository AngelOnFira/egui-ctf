use crate::game_server::game_room::{Game, GameData, GameRoom, PlayerData};
use actix::{AsyncContext, Context};
use std::collections::HashMap;

use super::{BattlesnakeDirection, Coordinate};

pub struct Snake {
    pub body: Vec<Coordinate>,
    pub state: SnakeState,
}

pub enum SnakeState {
    Alive,
    Dead,
}

impl Snake {
    pub fn new() -> Self {
        Snake {
            body: Vec::new(),
            state: SnakeState::Alive,
        }
    }

    pub fn head(&self) -> Coordinate {
        self.body[0]
    }

    pub fn neck(&self) -> Coordinate {
        self.body[1]
    }

    pub fn tail(&self) -> Coordinate {
        self.body[self.body.len() - 1]
    }

    pub fn move_direction(&mut self, direction: BattlesnakeDirection) {
        let new_head = match direction {
            BattlesnakeDirection::Up => (self.head().0, self.head().1 + 1),
            BattlesnakeDirection::Down => (self.head().0, self.head().1 - 1),
            BattlesnakeDirection::Left => (self.head().0 - 1, self.head().1),
            BattlesnakeDirection::Right => (self.head().0 + 1, self.head().1),
        };

        self.body.insert(0, new_head);
        self.body.pop();
    }
}