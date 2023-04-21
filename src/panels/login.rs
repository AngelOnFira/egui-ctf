/// Shows off one example of each major type of widget.
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LoginPanel {
    enabled: bool,
    visible: bool,
    team_token: String,
}

impl Default for LoginPanel {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            team_token: String::new(),
        }
    }
}

impl LoginPanel {
    fn name(&self) -> &'static str {
        "🔑 Login"
    }

    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}

impl LoginPanel {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.enabled, |ui| {
            ui.set_visible(self.visible);

            // Login form
            ui.horizontal(|ui| {
                ui.label("Team Token:");
                ui.text_edit_singleline(&mut self.team_token);
            });
        });

        // Login button
        if ui.button("Login").clicked() {
            panic!("Login button clicked");
        }
    }
}
