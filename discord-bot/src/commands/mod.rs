use serde::{Serialize, Deserialize};


pub mod create_interactive_prompt;
pub mod token;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StoredDiscordTask {
    Task(TaskType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TaskType {
    CreateToken,
}
