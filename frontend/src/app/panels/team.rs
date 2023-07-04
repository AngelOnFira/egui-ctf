use common::{
    ctf_message::{CTFMessage, TeamData},
    NetworkMessage,
};
use eframe::egui;
use egui::Align2;
use egui_extras::{Column, TableBuilder};

use crate::app::{ClientState, ConnectionState};

pub struct TeamPanel {
    enabled: bool,
    visible: bool,
    team_join_token_field: String,
    team_create_team_name_field: String,
}

impl Default for TeamPanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            team_join_token_field: String::new(),
            team_create_team_name_field: String::new(),
        }
    }
}

impl TeamPanel {
    fn name(&self) -> &'static str {
        "Team"
    }

    pub fn window(
        &mut self,
        ctx: &egui::Context,
        ctf_state: &ClientState,
        connection_state: &mut ConnectionState,
    ) {
        egui::Window::new(self.name())
            .resizable(true)
            .movable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                self.ui(ui, ctf_state, connection_state);
            });
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        ctf_state: &ClientState,
        connection_state: &mut ConnectionState,
    ) {
        match &ctf_state.ctf_state.team_data {
            TeamData::NoTeam => {
                // Join a team
                ui.heading("Join Team");

                // Join team token field
                ui.horizontal(|ui| {
                    ui.label("Team token:");
                    ui.text_edit_singleline(&mut self.team_join_token_field);
                });

                // Join team button
                if ui.button("Join team").clicked() {
                    // Send the submission to the server if it's not empty
                    if !self.team_join_token_field.is_empty() {
                        connection_state.send_message(NetworkMessage::CTFMessage(
                            CTFMessage::JoinTeam(self.team_join_token_field.clone()),
                        ));
                    }
                }

                ui.separator();

                ui.heading("Create Team");

                // Create team form
                ui.horizontal(|ui| {
                    ui.label("Team name:");
                    ui.text_edit_singleline(&mut self.team_create_team_name_field);
                });

                // Create team button
                if ui.button("Create team").clicked() {
                    // Send the submission to the server if it's not empty
                    if !self.team_create_team_name_field.is_empty() {
                        connection_state.send_message(NetworkMessage::CTFMessage(
                            CTFMessage::CreateTeam(self.team_create_team_name_field.clone()),
                        ));
                    }
                }
            }
            TeamData::OnTeam(hacker_team) => {
                ui.heading(&hacker_team.name);

                // Leave team button
                if ui.button("Leave team").clicked() {
                    // TODO: Leave team
                    connection_state
                        .send_message(NetworkMessage::CTFMessage(CTFMessage::LeaveTeam));
                }

                ui.separator();

                ui.heading("Team join token");

                if ui
                    .label(&hacker_team.join_token)
                    // .on_hover_text("Click to copy")
                    // TODO: this doesn't work. Probably has to do with moving a
                    // window when you click.
                    .clicked()
                {
                    ui.output_mut(|o| {
                        o.copied_text = hacker_team.join_token.clone();
                    })
                };

                // Copy join token button
                if ui.button("Copy to clipboard").clicked() {
                    ui.output_mut(|o| o.copied_text = hacker_team.join_token.clone());
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
                            for hacker in ["hacker1", "hacker2", "hacker3"] {
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
    }
}
