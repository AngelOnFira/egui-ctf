use common::{
    ctf_message::{CTFClientState, CTFClientStateComponent, CTFMessage, CTFState, ClientUpdate},
    NetworkMessage,
};
use core::fmt::Display;
use egui_notify::Toasts;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use log::info;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::panels::{
    hacker_list::HackerList, login::LoginPanel, submission::SubmissionPanel, team::TeamPanel,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct CTFApp {
    // Panels
    login_panel: LoginPanel,

    #[serde(skip)]
    hacker_list: HackerList,

    #[serde(skip)]
    submission_panel: SubmissionPanel,

    #[serde(skip)]
    team_panel: TeamPanel,

    // Other visuals
    #[serde(skip)]
    toasts: Toasts,

    // Other state
    #[serde(skip)]
    connection_state: ConnectionState,

    authentication_state: AuthenticationState,

    // #[serde(skip)]
    // websocket_connection:
    #[serde(skip)]
    websocket_thread_handle: Option<std::thread::JoinHandle<()>>,

    client_state: ClientState,
}

#[derive(Deserialize, Serialize)]
pub struct ClientState {
    // pub credentials: Option<Credentials>,
    pub ctf_state: CTFClientState,
}

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected {
        ws_sender: WsSender,
        ws_receiver: WsReceiver,
    },
}

impl ConnectionState {
    /// Send a message to the backend. Return an error if we're not connected to
    /// the server.
    pub fn send_message(&mut self, message: NetworkMessage) -> Result<(), ConnectionStateError> {
        match self {
            ConnectionState::Connected { ws_sender, .. } => {
                ws_sender.send(WsMessage::Text(serde_json::to_string(&message).unwrap()));
                Ok(())
            }
            _ => Err(ConnectionStateError::NotConnected),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AuthenticationState {
    NotAuthenticated,
    Authenticated { valid_token: String },
}

pub enum ConnectionStateError {
    NotConnected,
}

impl Display for ConnectionStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionStateError::NotConnected => write!(f, "Not connected to server"),
        }
    }
}

impl Default for CTFApp {
    fn default() -> Self {
        Self {
            // Panels
            login_panel: LoginPanel::default(),
            hacker_list: HackerList::default(),
            submission_panel: SubmissionPanel::default(),
            team_panel: TeamPanel::default(),
            // Other visuals
            toasts: Toasts::default(),
            // Other state
            websocket_thread_handle: None,
            connection_state: ConnectionState::Disconnected,
            authentication_state: AuthenticationState::NotAuthenticated,
            client_state: ClientState {
                ctf_state: CTFClientState::default(),
            },
        }
    }
}

impl CTFApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self {
            websocket_thread_handle: None,
            ..Default::default()
        }
    }

    fn connect(&mut self, ctx: egui::Context) {
        self.connection_state = ConnectionState::Connecting;
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
        match ewebsock::connect_with_wakeup("ws://127.0.0.1:4040/ws", wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.connection_state = ConnectionState::Connected {
                    ws_sender,
                    ws_receiver,
                };

                info!("Auth status: {:?}", &self.authentication_state);

                // If we already have a valid login token, send it to the
                // backend to auth this connection
                if let AuthenticationState::Authenticated { valid_token } =
                    &self.authentication_state
                {
                    if let Err(e) = self
                        .connection_state
                        .send_message(NetworkMessage::CTFMessage(CTFMessage::Login(
                            valid_token.to_owned(),
                        )))
                    {
                        info!("Failed to send login token: {}", e);
                    }
                }
            }
            Err(error) => {
                panic!("Failed to connect {}", error);
            }
        }
    }
}

impl eframe::App for CTFApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut save_flag = false;

        match &self.connection_state {
            ConnectionState::Disconnected => {
                self.connect(ctx.clone());
            }
            ConnectionState::Connected {
                ws_sender: _,
                ws_receiver,
            } => {
                while let Some(event) = ws_receiver.try_recv() {
                    if let WsEvent::Message(WsMessage::Text(ws_text)) = event {
                        // Deserialize the message
                        let message: NetworkMessage = serde_json::from_str(&ws_text).unwrap();

                        match message {
                            // All messages about the CTF game
                            NetworkMessage::CTFMessage(ctf_message) => {
                                match ctf_message {
                                    // If we get a state update from the server
                                    CTFMessage::CTFClientStateComponent(
                                        ctf_client_state_component,
                                    ) => match ctf_client_state_component {
                                        CTFClientStateComponent::GlobalData(global_data) => {
                                            self.client_state.ctf_state.global_data =
                                                Some(global_data);
                                        }
                                        CTFClientStateComponent::GameData(game_data) => {
                                            self.client_state.ctf_state.game_data = game_data;
                                        }
                                        CTFClientStateComponent::TeamData(team_data) => {
                                            self.client_state.ctf_state.team_data = team_data;
                                        }
                                        CTFClientStateComponent::ClientData(client_data) => {
                                            self.client_state.ctf_state.client_data = client_data;
                                        }
                                    },

                                    // The client can't receive submissions
                                    CTFMessage::SubmitFlag(_) => unreachable!(),

                                    // The client can't receive login requests
                                    CTFMessage::Login(_) => unreachable!(),

                                    // Events that the server sends and we
                                    // should display
                                    CTFMessage::ClientUpdate(event) => match event {
                                        ClientUpdate::ScoredPoint(string) => {
                                            self.toasts
                                                .info(string)
                                                .set_duration(Some(Duration::from_secs(5)));
                                        }
                                        ClientUpdate::TeamScoredPoint => todo!(),
                                        ClientUpdate::IncorrectFlag(string) => {
                                            self.toasts
                                                .error(string)
                                                .set_duration(Some(Duration::from_secs(5)));
                                        }
                                        ClientUpdate::Authenticated {
                                            discord_username,
                                            valid_token,
                                        } => {
                                            self.toasts
                                                .info(format!("Logged in as {}", discord_username))
                                                .set_duration(Some(Duration::from_secs(5)));

                                            // Set the authentication state
                                            self.authentication_state =
                                                AuthenticationState::Authenticated { valid_token };

                                            // Flag to save the app state
                                            save_flag = true;
                                        }
                                        ClientUpdate::IncorrectToken => {
                                            self.toasts
                                                .error("Incorrect token")
                                                .set_duration(Some(Duration::from_secs(5)));
                                        }
                                    },
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        };

        // I'm doing a save flag since I can't figure out how to get mutable
        // access to self in the match statement above.
        if save_flag {
            // Manually call save to persist the app
            // state
            if let Some(storage) = frame.storage_mut() {
                self.save(storage);
            }
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

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
            if let ConnectionState::Connected { .. } = &self.connection_state {
                // Check if we're authenticated
                match &self.authentication_state {
                    AuthenticationState::NotAuthenticated => {
                        // Show the login panel
                        self.login_panel.show(ctx, &mut self.connection_state);
                    }
                    AuthenticationState::Authenticated { valid_token } => {
                        // Show the hacker list
                        self.hacker_list.show(ctx, &self.client_state);

                        // Show the submission panel
                        self.submission_panel.show(ctx, &mut self.connection_state);

                        // Show the team panel
                        self.team_panel.show(ctx, &self.client_state);
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
        self.toasts.show(ctx);
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
