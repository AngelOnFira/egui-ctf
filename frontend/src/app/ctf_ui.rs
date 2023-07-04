use common::ctf_message::TeamData;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;

use crate::CTFApp;

use super::{connection_state::ConnectionStateEnum, AuthenticationStateEnum, CTFUiWindow, UiTheme};

pub fn ctf_ui(ctf_app: &mut CTFApp, ctx: &egui::Context) {
    egui::SidePanel::left("side_panel").show(ctx, |ui| {
        // Set the egui theme
        catppuccin_egui::set_theme(
            ctx,
            match ctf_app.ui_theme {
                UiTheme::Latte => catppuccin_egui::LATTE,
                UiTheme::Mocha => catppuccin_egui::MOCHA,
                UiTheme::Macchiato => catppuccin_egui::MACCHIATO,
                UiTheme::Frappe => catppuccin_egui::FRAPPE,
            },
        );

        ui.heading("Windows");

        // let style_clone = (*ctx.style()).clone();
        // ctx.style().text_styles = [(Button, FontId::new(24.0, Proportional))].into();
        // ui.style_mut().text_styles = ctx.style().text_styles;

        ui.vertical_centered_justified(|ui| {
            ui.style_mut().text_styles = [(Button, FontId::new(20.0, Proportional))].into();

            // Scoreboard window button
            if ui.button("Scoreboard").clicked() {
                ctf_app.current_window = CTFUiWindow::Scoreboard;
            }

            // Check if we're connected to the server
            if let ConnectionStateEnum::Opened = &ctf_app.connection_state.get_state() {
                // Check if we're authenticated
                match &ctf_app.authentication_state.state {
                    AuthenticationStateEnum::NotAuthenticated => {
                        // Login window button
                        if ui.button("Login").clicked() {
                            ctf_app.current_window = CTFUiWindow::Login;
                        }
                    }
                    AuthenticationStateEnum::Authenticated => {
                        // Team window button
                        if ui.button("Team").clicked() {
                            ctf_app.current_window = CTFUiWindow::Team;
                        }

                        // If we're on a team, show the challenge info
                        if let TeamData::OnTeam(..) = &ctf_app.client_state.ctf_state.team_data {
                            // Challenges window button
                            if ui.button("Challenges").clicked() {
                                ctf_app.current_window = CTFUiWindow::Challenge;
                            }
                        }
                    }
                }
            }
        });

        // // Undo the previous style change
        // ui.set_style(style_clone);

        ui.heading("Settings");
        egui::ComboBox::from_label("Theme")
            .selected_text(format!("{:?}", ctf_app.ui_theme))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Frappe, "Frappe");
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Macchiato, "Macchiato");
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Mocha, "Mocha");
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Latte, "Latte");
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        // Check if we're connected to the server
        if let ConnectionStateEnum::Opened = &ctf_app.connection_state.get_state() {
            // Display the current window
            match &ctf_app.current_window {
                CTFUiWindow::Login => {
                    // Show the login panel
                    ctf_app
                        .login_panel
                        .window(ctx, &mut ctf_app.connection_state);
                }
                CTFUiWindow::Team => {
                    // Show the team panel
                    ctf_app.team_panel.window(
                        ctx,
                        &ctf_app.client_state,
                        &mut ctf_app.connection_state,
                    );

                    // Show the hacker list
                    ctf_app.hacker_list.window(ctx, &ctf_app.client_state);
                }
                CTFUiWindow::Challenge => {
                    // Show the challenge list panel
                    ctf_app
                        .challenge_list_panel
                        .show(ctx, &ctf_app.client_state);

                    // Show the challenge panel
                    ctf_app.challenge_panel.window(
                        ctx,
                        &ctf_app.client_state,
                        &ctf_app.challenge_list_panel.visible_challenge,
                        &mut ctf_app.connection_state,
                    );
                }
                CTFUiWindow::Scoreboard => {
                    // Show the scoreboard
                    ctf_app.scoreboard_panel.ui(ui, &ctf_app.client_state);
                }
            }
        }
    });

    // Toasts
    ctf_app.toasts.show(ctx);
}
