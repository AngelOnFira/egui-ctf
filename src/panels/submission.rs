use common::{ctf_message::CTFMessage, NetworkMessage};

use crate::app::ConnectionState;

/// Shows off one example of each major type of widget.
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SubmissionPanel {
    enabled: bool,
    visible: bool,
    flag: String,
}

impl Default for SubmissionPanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            flag: String::new(),
        }
    }
}

impl SubmissionPanel {
    fn name(&self) -> &'static str {
        "Submit Flag"
    }

    pub fn show(&mut self, ctx: &egui::Context, connection_state: &mut ConnectionState) {
        egui::Window::new(self.name())
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui, connection_state);
            });
    }
}

impl SubmissionPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui, connection_state: &mut ConnectionState) {
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.set_visible(self.visible);

            // Login form
            ui.horizontal(|ui| {
                ui.label("Flag:");
                ui.text_edit_singleline(&mut self.flag);
            });
        });

        // Submit button
        if ui.button("Submit").clicked() {
            // Send the submission to the server if it's not empty
            if !self.flag.is_empty() {
                match connection_state.send_message(NetworkMessage::CTFMessage(
                    CTFMessage::SubmitFlag(self.flag.clone()),
                )) {
                    Ok(_) => {
                        self.flag.clear();
                    }
                    Err(e) => {
                        eprintln!("Failed to send flag: {}", e);
                    }
                }
            }
        }
    }
}
