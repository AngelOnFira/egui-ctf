use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::app::ClientState;

pub struct TeamPanel {
    enabled: bool,
    visible: bool,
}

impl Default for TeamPanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
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
        ui.label("Team members");

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
