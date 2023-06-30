use common::ctf_message::{CTFState, GameData};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::app::ClientState;

pub struct ChallengePanel {
    enabled: bool,
    visible: bool,
}

impl Default for ChallengePanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
        }
    }
}

impl ChallengePanel {
    fn name(&self) -> &'static str {
        "Challenges"
    }

    pub fn show(&mut self, ctx: &egui::Context, ctf_state: &ClientState) {
        egui::Window::new(self.name())
            // .open(open)
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui, ctf_state);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctf_state: &ClientState) {
        ui.add_enabled_ui(self.enabled, |ui| {
            // If we're logged in, show all the challenges
            if let GameData::LoggedIn { challenges } = &ctf_state.ctf_state.game_data {
                for challenge in challenges {
                    ui.label(&challenge.title);
                }
            }
        });
    }
}
