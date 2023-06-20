use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::app::ClientState;

pub struct HackerList {
    enabled: bool,
    visible: bool,
}

impl Default for HackerList {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
        }
    }
}

impl HackerList {
    fn name(&self) -> &'static str {
        "Hacker list"
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
            ui.set_visible(self.visible);

            let table = TableBuilder::new(ui)
                // .striped(self.striped)
                // .resizable(self.resizable)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::initial(100.0).range(40.0..=300.0))
                .column(Column::initial(100.0).at_least(40.0).clip(true))
                .column(Column::remainder())
                .min_scrolled_height(0.0);

            // if let Some(row_nr) = self.scroll_to_row.take() {
            //     table = table.scroll_to_row(row_nr, None);
            // }

            // Login form
            // Make sure we have a ctf state. If we don't, set visible to
            // false.
            match &ctf_state.ctf_state {
                Some(client_state) => {
                    // self.enabled = true;
                    table
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Team");
                            });
                            header.col(|ui| {
                                ui.strong("Player");
                            });
                        })
                        .body(|mut body| {
                            for hacker_team in &client_state.hacker_teams {
                                for hacker in &hacker_team.hackers {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(&hacker_team.name);
                                        });
                                        row.col(|ui| {
                                            ui.label(&hacker.name);
                                        });
                                    });
                                }
                            }
                        });
                }
                None => {
                    self.enabled = false;
                    return;
                }
            }
        });
    }
}
