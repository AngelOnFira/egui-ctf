use common::{ctf_message::CTFMessage, NetworkMessage};
use egui::Align2;

use crate::app::ConnectionState;

/// Shows off one example of each major type of widget.
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LoginPanel {
    enabled: bool,
    visible: bool,
    pub token: String,
}

impl Default for LoginPanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            token: String::new(),
        }
    }
}

impl LoginPanel {
    fn name(&self) -> &'static str {
        "🔑 Login"
    }

    pub fn window(&mut self, ctx: &egui::Context, connection_state: &mut ConnectionState) {
        egui::Window::new(self.name())
            .resizable(false)
            .movable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                self.ui(ui, connection_state);
            });
    }
}

impl LoginPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui, connection_state: &mut ConnectionState) {
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.set_visible(self.visible);

            // Login form
            ui.horizontal(|ui| {
                ui.label("Token:");
                ui.text_edit_singleline(&mut self.token);
            });
        });

        // Login button
        if ui.button("Login").clicked() {
            // Send the submission to the server if it's not empty
            if !self.token.is_empty() {
                connection_state.send_message(NetworkMessage::CTFMessage(CTFMessage::Login(
                    self.token.clone(),
                )));
            }
        }
    }
}
