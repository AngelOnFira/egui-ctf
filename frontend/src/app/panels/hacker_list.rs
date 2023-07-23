use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::app::ClientState;

pub struct HackerList {
    enabled: bool,
    visible: bool,
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Arc<Mutex<Option<String>>>,
}

impl Default for HackerList {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            dropped_files: Default::default(),
            picked_path: Default::default(),
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

            let _table = TableBuilder::new(ui)
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
                    // Go through all the hackers on a team
                    for hacker_team in &global_data.hacker_teams {
                        for hacker in &hacker_team.hackers {
                            ui.label(format!("{} [{}]", &hacker.name, hacker_team.name));
                        }
                    }

                    // Go though all the hackers not on a team
                    for hacker in &global_data.non_hacker_teams {
                        ui.label(format!("{} [{}]", &hacker.name, "No team"));
                    }
                }
                None => {
                    self.enabled = false;
                }
            }

            ui.label("Drag-and-drop files onto the window!");

            #[cfg(target_arch = "wasm32")]
            {
                wasm_bindgen_futures::spawn_local(async {
                    if ui.button("Open file...").clicked() {
                        if let Some(path) = rfd::AsyncFileDialog::new().pick_file().await {
                            self.picked_path = Some(path.file_name())
                        }
                    }
                });
            }

            if let Some(picked_path) = &self.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };
                        if let Some(bytes) = &file.bytes {
                            use std::fmt::Write as _;
                            write!(info, " ({} bytes)", bytes.len()).ok();
                        }
                        ui.label(info);
                    }
                });
            }
        });
    }
}
