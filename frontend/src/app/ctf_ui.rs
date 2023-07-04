use common::{
    ctf_message::{CTFClientState, CTFClientStateComponent, CTFMessage, ClientUpdate, TeamData},
    NetworkMessage,
};
use core::fmt::Display;
use egui_notify::Toasts;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::info;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    time::Duration,
};

use panels::{
    challenge_list::ChallengeList, challenge_panel::ChallengePanel, hacker_list::HackerList,
    login::LoginPanel, scoreboard::ScoreboardPanel, team::TeamPanel,
};

use crate::CTFApp;

use super::{panels, UiTheme, AuthenticationStateEnum, ConnectionStateEnum};

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

        ui.heading("Side Panel");
        ui.heading("Settings");
        egui::ComboBox::from_label("Theme")
            .selected_text(format!("{:?}", ctf_app.ui_theme))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Frappe, "Frappe");
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Macchiato, "Macchiato");
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Mocha, "Mocha");
                ui.selectable_value(&mut ctf_app.ui_theme, UiTheme::Latte, "Latte");
            });

        // TODO: put stuff here to switch windows?

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to(
                    "eframe",
                    "https://github.com/emilk/egui/tree/master/crates/eframe",
                );
                ui.label(".");
            });
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        // The central panel the region left after adding TopPanel's and SidePanel's

        ui.heading("eframe template");
        ui.hyperlink("https://github.com/emilk/eframe_template");
        ui.add(egui::github_link_file!(
            "https://github.com/emilk/eframe_template/blob/master/",
            "Source code."
        ));
        egui::warn_if_debug_build(ui);

        // Check if we're connected to the server
        if let ConnectionStateEnum::Opened = &ctf_app.connection_state.get_state() {
            // Check if we're authenticated
            match &ctf_app.authentication_state.state {
                AuthenticationStateEnum::NotAuthenticated => {
                    // Show the login panel
                    ctf_app.login_panel.show(ctx, &mut ctf_app.connection_state);

                    // Show the scoreboard
                    ctf_app.scoreboard_panel.show(ctx, &ctf_app.client_state);
                }
                AuthenticationStateEnum::Authenticated => {
                    // Show the hacker list
                    ctf_app.hacker_list.show(ctx, &ctf_app.client_state);

                    // Show the team panel
                    ctf_app.team_panel.show(
                        ctx,
                        &ctf_app.client_state,
                        &mut ctf_app.connection_state,
                    );

                    // If we're on a team, show the challenge info
                    if let TeamData::OnTeam(..) = &ctf_app.client_state.ctf_state.team_data {
                        // Show the challenge list panel
                        ctf_app
                            .challenge_list_panel
                            .show(ctx, &ctf_app.client_state);

                        // Show the challenge panel
                        ctf_app.challenge_panel.show(
                            ctx,
                            &ctf_app.client_state,
                            &ctf_app.challenge_list_panel.visible_challenge,
                            &mut ctf_app.connection_state,
                        );
                    }
                }
            }
        }
    });

    if false {
        egui::Window::new("Window").show(ctx, |ui| {
            ui.label("Windows can be moved by dragging them.");
            ui.label("They are automatically sized based on contents.");
            ui.label("You can turn on resizing and scrolling if you like.");
            ui.label("You would normally choose either panels OR windows.");
        });
    }

    // Toasts
    ctf_app.toasts.show(ctx);
}
