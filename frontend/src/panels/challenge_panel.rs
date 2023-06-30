use common::{
    ctf_message::{CTFMessage, GameData},
    NetworkMessage,
};
use eframe::egui;

use serde::{Deserialize, Serialize};

use crate::app::{ClientState, ConnectionState};

#[derive(Deserialize, Serialize)]
pub struct ChallengePanel {
    flag: String,
}

impl Default for ChallengePanel {
    fn default() -> Self {
        Self {
            flag: String::new(),
        }
    }
}

impl ChallengePanel {
    fn name(&self) -> &'static str {
        "Challenges"
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        ctf_state: &ClientState,
        visible_challenge: &Option<String>,
        connection_state: &mut ConnectionState,
    ) {
        egui::Window::new(self.name())
            // .open(open)
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui, ctf_state, visible_challenge, connection_state);
            });
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        ctf_state: &ClientState,
        visible_challenge: &Option<String>,
        connection_state: &mut ConnectionState,
    ) {
        if let Some(challenge_name) = visible_challenge {
            // Check if there is a challenge with this name in the game state
            if let GameData::LoggedIn { challenges } = &ctf_state.ctf_state.game_data {
                if let Some(challenge) = challenges.iter().find(|c| &c.title == challenge_name) {
                    // Show the challenge
                    ui.heading(&challenge.title);
                    ui.label(&format!("Category: {}", challenge.category));
                    ui.label(&format!("Points: {}", challenge.points));
                    ui.separator();
                    if ui.label(&challenge.description).clicked() {
                        ui.output_mut(|o| {
                            o.copied_text = challenge.description.clone();
                        })
                    };

                    // Copy description button
                    if ui.button("Copy to clipboard").clicked() {
                        ui.output_mut(|o| o.copied_text = challenge.description.clone());
                    }

                    // If there is a link, show it
                    if !challenge.link.is_empty() {
                        ui.label("");
                        ui.hyperlink(&challenge.link);
                    }

                    ui.separator();

                    // Login form
                    ui.horizontal(|ui| {
                        ui.label("Flag:");
                        ui.text_edit_singleline(&mut self.flag);
                    });

                    // Submit button
                    if ui.button("Submit").clicked() {
                        // Send the submission to the server if it's not empty
                        if !self.flag.is_empty() {
                            connection_state.send_message(NetworkMessage::CTFMessage(
                                CTFMessage::SubmitFlag {
                                    challenge_name: challenge_name.clone(),
                                    flag: self.flag.clone(),
                                },
                            ));
                        }
                    }
                }
            }
        }
    }
}
