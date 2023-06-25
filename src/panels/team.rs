use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::app::ClientState;

pub struct TeamPanel {
    enabled: bool,
    visible: bool,
    team_name_field: String,
}

impl Default for TeamPanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            team_name_field: String::new(),
        }
    }
}

impl TeamPanel {
    fn name(&self) -> &'static str {
        "Team"
    }

    pub fn show(&mut self, ctx: &egui::Context, ctf_state: &ClientState) {
        egui::Window::new(self.name())
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui, ctf_state);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, ctf_state: &ClientState) {
        // Join a team
        ui.heading("Join Team");

        // Join team token field
        ui.horizontal(|ui| {
            ui.label("Team token:");
            ui.text_edit_singleline(&mut self.team_name_field);
        });

        // Join team button
        if ui.button("Join team").clicked() {
            // Send the submission to the server if it's not empty
            if !self.team_name_field.is_empty() {
                // if let Err(e) = connection_state.send_message(NetworkMessage::CTFMessage(
                //     CTFMessage::Login(self.token.clone()),
                // )) {
                //     eprintln!("Failed to send login token: {}", e);
                // }
            }
        }

        ui.separator();

        ui.heading("Create Team");

        // Login form
        ui.horizontal(|ui| {
            ui.label("Team name:");
            ui.text_edit_singleline(&mut self.team_name_field);
        });

        // Login button
        if ui.button("Create team").clicked() {
            // Send the submission to the server if it's not empty
            if !self.team_name_field.is_empty() {
                // if let Err(e) = connection_state.send_message(NetworkMessage::CTFMessage(
                //     CTFMessage::Login(self.token.clone()),
                // )) {
                //     eprintln!("Failed to send login token: {}", e);
                // }
            }
        }

        ui.separator();

        ui.heading("Team members");

        ui.add_enabled_ui(self.enabled, |ui| {
            let table = TableBuilder::new(ui)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::initial(100.0).range(40.0..=300.0))
                .column(Column::initial(100.0).at_least(40.0).clip(true))
                .column(Column::remainder())
                .min_scrolled_height(0.0);

            // Team member list
            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Member");
                    });
                    header.col(|ui| {
                        ui.strong("Challenge");
                    });
                    header.col(|ui| {
                        ui.strong("Status");
                    });
                })
                .body(|mut body| {
                    for hacker in vec!["hacker1", "hacker2", "hacker3"] {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(hacker);
                            });
                            row.col(|ui| {
                                ui.label("test");
                            });
                            row.col(|ui| {
                                ui.label("test");
                            });
                        });
                    }
                });
        });
    }
}
