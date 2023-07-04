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

    pub fn window(&mut self, ctx: &egui::Context, ctf_state: &ClientState) {
        egui::Window::new(self.name())
            // .open(open)
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui, ctf_state);
            });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctf_state: &ClientState) {
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.heading("Online hackers");

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

            // Make sure we have a ctf state. If we don't, set visible to
            // false.
            match &ctf_state.ctf_state.global_data {
                Some(global_data) => {
                    for hacker_team in &global_data.hacker_teams {
                        for hacker in &hacker_team.hackers {
                            ui.label(format!("{} [{}]", &hacker.name, hacker_team.name));
                        }
                    }
                }
                None => {
                    self.enabled = false;
                }
            }
        });
    }
}
