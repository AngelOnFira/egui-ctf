use common::ctf_message::CTFMessage;
use common::ctf_message::TeamData;
use common::NetworkMessage;
use eframe::App;
use egui::FontFamily::Proportional;
use egui::FontId;
use egui::TextStyle::*;
use std::time::Duration;

use crate::CTFApp;

use super::{connection_state::ConnectionStateEnum, AuthenticationStateEnum, CTFUIWindow, UiTheme};

pub fn ctf_ui(ctf_app: &mut CTFApp, ctx: &egui::Context, frame: &mut eframe::Frame) {
    // Check if we're connected to the server
    if let ConnectionStateEnum::Opened = &ctf_app.connection_state.get_state() {
        // The left panel is the windows select panel and settings
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Windows");

            

            ui.vertical_centered_justified(|ui| {
                ui.style_mut().text_styles = [(Button, FontId::new(18.0, Proportional))].into();

                // Scoreboard window button
                if ui.button("Scoreboard").clicked() {
                    ctf_app.current_window = CTFUIWindow::Scoreboard;

                    // Save to storage
                    if let Some(storage) = frame.storage_mut() {
                        ctf_app.save(storage);
                    }
                }
                // Check if we're authenticated
                match &ctf_app.authentication_state.state {
                    AuthenticationStateEnum::NotAuthenticated => {
                        // Login window button
                        if ui.button("Login").clicked() {
                            ctf_app.current_window = CTFUIWindow::Login;

                            // Save to storage
                            if let Some(storage) = frame.storage_mut() {
                                ctf_app.save(storage);
                            }
                        }
                    }
                    AuthenticationStateEnum::Authenticated => {
                        // Team window button
                        if ui.button("Team").clicked() {
                            ctf_app.current_window = CTFUIWindow::Team;

                            // Save to storage
                            if let Some(storage) = frame.storage_mut() {
                                ctf_app.save(storage);
                            }
                        }

                        // If we're on a team, show the challenge info
                        if let TeamData::OnTeam(..) = &ctf_app.client_state.ctf_state.team_data {
                            // Challenges window button
                            if ui.button("Challenges").clicked() {
                                ctf_app.current_window = CTFUIWindow::Challenge;

                                // Save to storage
                                if let Some(storage) = frame.storage_mut() {
                                    ctf_app.save(storage);
                                }
                            }
                        }

                        // Logout button
                        if ui.button("Logout").clicked() {
                            ctf_app
                                .authentication_state
                                .logout(&mut ctf_app.connection_state);

                            // Remove the token from the login screen
                            ctf_app.login_panel.token = String::new();

                            // Change the screen to the scoreboard
                            ctf_app.current_window = CTFUIWindow::Scoreboard;

                            // Save to storage
                            if let Some(storage) = frame.storage_mut() {
                                ctf_app.save(storage);
                            }

                            // Add a toast to say we logged out
                            ctf_app
                                .toasts
                                .info("Logged out successfully")
                                .set_duration(Some(Duration::from_secs(5)));

                            // Force the storage to save
                            if let Some(storage) = frame.storage_mut() {
                                ctf_app.save(storage);
                            }
                        }
                    }
                }
            });

            ui.separator();

            ui.heading("Settings");
            egui::ComboBox::from_label("Theme")
                .selected_text(format!("{:?}", ctf_app.ui_theme))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Frappe, "Frappe");
                    // ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Macchiato, "Macchiato");
                    ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Mocha, "Mocha");
                    ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Latte, "Latte");
                });

            // TODO: Add these settings for admin/debug only
            ui.separator();

            // Reset database
            if ui.button("Reset DB and spawn teams").clicked() {
                // Send a message to the backend to wipe the db and rerun
                // migrations
                ctf_app
                    .connection_state
                    .send_message(NetworkMessage::CTFMessage(CTFMessage::ResetDB));
            }

            // // Clone the repo
            // if ui.button("Clone Repo").clicked() {
            //     // Send a message to the backend to clone the repo
            //     ctf_app
            //         .connection_state
            //         .send_message(NetworkMessage::CTFMessage(CTFMessage::CloneRepo));
            // }

            // // Spawn 1000 teams
            // if ui.button("Spawn 1000 teams").clicked() {
            //     // Send a message to the backend to spawn 1000 teams
            //     ctf_app
            //         .connection_state
            //         .send_message(NetworkMessage::CTFMessage(CTFMessage::SpawnTeams));
            // }

            ui.separator();

            // Show the hacker list
            ctf_app.hacker_list.ui(ui, &ctf_app.client_state);
        });

        // The central panel will have most of the content that will be used
        egui::CentralPanel::default().show(ctx, |ui| {
            // Check if we're connected to the server
            if let ConnectionStateEnum::Opened = &ctf_app.connection_state.get_state() {
                // Display the current window
                match &ctf_app.current_window {
                    CTFUIWindow::Login => {
                        // Show the login panel
                        ctf_app
                            .login_panel
                            .window(ctx, &mut ctf_app.connection_state);
                    }
                    CTFUIWindow::Team => {
                        // Show the team panel
                        ctf_app.team_panel.window(
                            ctx,
                            &ctf_app.client_state,
                            &mut ctf_app.connection_state,
                        );
                    }
                    CTFUIWindow::Challenge => {
                        ui.columns(2, |columns| {
                            // Show the challenge list panel
                            ctf_app
                                .challenge_list_panel
                                .ui(&mut columns[0], &ctf_app.client_state);

                            // Show the challenge panel
                            ctf_app.challenge_panel.ui(
                                &mut columns[1],
                                &ctf_app.client_state,
                                &ctf_app.challenge_list_panel.visible_challenge,
                                &mut ctf_app.connection_state,
                            );
                        });
                    }
                    CTFUIWindow::Scoreboard => {
                        // Show the scoreboard
                        ctf_app.scoreboard_panel.ui(ui, &ctf_app.client_state);
                    }
                }
            }
        });
    } else {
        // Display the connecting screen
        ctf_app.connecting_panel.window(ctx);
    }

    // Toasts
    ctf_app.toasts.show(ctx);
}
