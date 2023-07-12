use serde::{Deserialize, Serialize};

pub mod create_interactive_prompt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StoredDiscordTask {
    Task(TaskType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TaskType {
    CreateToken,
}
