use crate::app::ctf_ui::ctf_ui;
use common::{
    ctf_message::{CTFClientState, CTFClientStateComponent, CTFMessage, ClientUpdate},
    NetworkMessage,
};
use core::fmt::Display;
use egui_notify::Toasts;
use ewebsock::{WsEvent, WsMessage};
use log::info;
use panels::{
    challenge_list::ChallengeList, challenge_panel::ChallengePanel, hacker_list::HackerList,
    login::LoginPanel, scoreboard::ScoreboardPanel, team::TeamPanel,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    time::Duration,
};
use wasm_timer::Instant;

use self::{
    connection_state::{ConnectionState, ConnectionStateEnum},
    panels::connecting::ConnectingPanel,
};

mod connection_state;
mod ctf_ui;
mod panels;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct CTFApp {
    // Panels
    login_panel: LoginPanel,

    #[serde(skip)]
    hacker_list: HackerList,

    #[serde(skip)]
    team_panel: TeamPanel,

    challenge_list_panel: ChallengeList,

    challenge_panel: ChallengePanel,

    #[serde(skip)]
    scoreboard_panel: ScoreboardPanel,

    connecting_panel: ConnectingPanel,

    // Other visuals
    #[serde(skip)]
    toasts: Toasts,

    // Other state

    // This will be an arc mutex so we can asynchronously set it when we connect
    // or disconnect from the server.
    #[serde(skip)]
    connection_state: ConnectionState,

    //
    #[serde(skip)]
    ws_state_queue: Arc<Mutex<WSStateQueue>>,

    authentication_state: AuthenticationState,

    // #[serde(skip)]
    // websocket_connection:
    #[serde(skip)]
    websocket_thread_handle: Option<std::thread::JoinHandle<()>>,

    client_state: ClientState,

    ui_theme: UiTheme,

    current_window: CTFUIWindow,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum CTFUIWindow {
    Login,
    Team,
    Challenge,
    Scoreboard,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum UiTheme {
    Latte,
    Mocha,
    Macchiato,
    Frappe,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientState {
    // pub credentials: Option<Credentials>,
    pub ctf_state: CTFClientState,
}

pub struct WSStateQueue {
    pub queue: Vec<WsEvent>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AuthenticationState {
    valid_token: Option<String>,

    #[serde(skip)]
    state: AuthenticationStateEnum,
}

impl Default for AuthenticationState {
    fn default() -> Self {
        Self {
            valid_token: None,
            state: AuthenticationStateEnum::NotAuthenticated,
        }
    }
}

impl AuthenticationState {
    pub fn logout(&mut self, connection_state: &mut ConnectionState) {
        self.valid_token = None;
        self.state = AuthenticationStateEnum::NotAuthenticated;

        // Send a logout message to the backend
        connection_state.send_message(NetworkMessage::CTFMessage(CTFMessage::Logout));
    }
}

#[derive(Debug)]
pub enum AuthenticationStateEnum {
    NotAuthenticated,
    Authenticated,
}

impl Default for AuthenticationStateEnum {
    fn default() -> Self {
        Self::NotAuthenticated
    }
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
            team_panel: TeamPanel::default(),
            challenge_list_panel: ChallengeList::default(),
            challenge_panel: ChallengePanel::default(),
            scoreboard_panel: ScoreboardPanel::default(),
            connecting_panel: ConnectingPanel::default(),
            // Other visuals
            toasts: Toasts::default(),
            // Other state
            websocket_thread_handle: None,
            connection_state: ConnectionState::default(),
            ws_state_queue: Arc::new(Mutex::new(WSStateQueue { queue: Vec::new() })),
            authentication_state: AuthenticationState::default(),
            client_state: ClientState {
                ctf_state: CTFClientState::default(),
            },
            ui_theme: UiTheme::Frappe,
            current_window: CTFUIWindow::Scoreboard,
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
        // If we tried connecting less than 5 seconds ago, don't try again. This
        // is doing something to prevent spamming the server with connections
        // from an error that is coming up when a connection can't be made.
        // TODO: Fix this error

        let mut connection_state = self.connection_state.inner.lock().unwrap();

        if let Some(last_connection_attempt) = connection_state.last_connection_attempt {
            if last_connection_attempt.elapsed() < Duration::from_secs(5) {
                return;
            }
            // If 5 seconds have passed, update the last connection attempt
            else {
                connection_state.last_connection_attempt = Some(Instant::now());
            }
        } else {
            connection_state.last_connection_attempt = Some(Instant::now());
        }

        drop(connection_state);

        self.connection_state.set_state_connecting();

        let ws_state_queue_clone = self.ws_state_queue.clone();

        let wakeup = move |event: WsEvent| {
            ws_state_queue_clone.lock().unwrap().queue.push(event);
            ctx.request_repaint(); // wake up UI thread on new message}
        };

        // TODO: I don't know if there's a better way to hardcode the domain in,
        // since I can't ship an envfile with the frontend I think
        match ewebsock::connect_with_wakeup(include_str!("backend_domain.txt"), wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                self.connection_state
                    .set_state_connected(ws_sender, ws_receiver);

                info!("Auth status: {:?}", &self.authentication_state);

                // If we already have a valid login token, send it to the
                // backend to auth this connection
                if let Some(token) = &self.authentication_state.valid_token {
                    self.connection_state
                        .send_message(NetworkMessage::CTFMessage(CTFMessage::Login(
                            token.to_owned(),
                        )));
                }
                // Otherwise, send a unauthenticated connect message
                else {
                    self.connection_state
                        .send_message(NetworkMessage::CTFMessage(CTFMessage::Connect));
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
        // Set the egui theme
        catppuccin_egui::set_theme(
            ctx,
            match self.ui_theme {
                UiTheme::Latte => catppuccin_egui::LATTE,
                UiTheme::Mocha => catppuccin_egui::MOCHA,
                UiTheme::Macchiato => catppuccin_egui::MACCHIATO,
                UiTheme::Frappe => catppuccin_egui::FRAPPE,
            },
        );

        // Start by seeing if the websocket has any new messages
        for event in self.ws_state_queue.lock().unwrap().queue.drain(..) {
            match event {
                WsEvent::Opened => {
                    self.connection_state.set_state_opened();
                }
                WsEvent::Closed => {
                    log::info!("Connection closed");
                    self.connection_state.set_state_disconnected();
                }
                _ => {}
            }
        }

        // Now, if we're in an open connection, process our message queue
        self.connection_state.process_message_queue();

        let mut save_flag = false;

        match &self.connection_state.get_state() {
            ConnectionStateEnum::Disconnected => {
                self.connect(ctx.clone());
            }
            ConnectionStateEnum::Opened => {
                // Check for any messages from the server
                while let Some(event) = self
                    .connection_state
                    .inner
                    .lock()
                    .unwrap()
                    .ws_receiver
                    .as_ref()
                    .unwrap()
                    .try_recv()
                {
                    if let WsEvent::Message(WsMessage::Text(ws_text)) = event {
                        // Deserialize the message
                        let message: NetworkMessage = serde_json::from_str(&ws_text).unwrap();

                        // // Debug the state of the app
                        // info!("App state: {:?}", self.client_state);
                        // // info!("Connection state: {:?}", self.connection_state);
                        // info!("Auth state: {:?}", self.authentication_state);
                        // info!("Message: {:?}", message);

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
                                            self.authentication_state = AuthenticationState {
                                                valid_token: Some(valid_token),
                                                state: AuthenticationStateEnum::Authenticated,
                                            };

                                            // Change the screen back to the
                                            // scoreboard
                                            self.current_window = CTFUIWindow::Scoreboard;

                                            // Flag to save the app state
                                            save_flag = true;
                                        }
                                        ClientUpdate::IncorrectToken => {
                                            self.toasts
                                                .error("Incorrect token")
                                                .set_duration(Some(Duration::from_secs(5)));
                                        }
                                        ClientUpdate::Notification(notification) => {
                                            self.toasts
                                                .info(notification)
                                                .set_duration(Some(Duration::from_secs(5)));
                                        }
                                    },

                                    // The client can't receive any of these
                                    // messages

                                    // TODO: Redo the enum so that only messages
                                    // that the client can receive are in this
                                    // or something
                                    CTFMessage::SubmitFlag { .. }
                                    | CTFMessage::JoinTeam(_)
                                    | CTFMessage::CreateTeam(_)
                                    | CTFMessage::Login(_)
                                    | CTFMessage::Logout
                                    | CTFMessage::Connect
                                    | CTFMessage::SpawnTeams
                                    | CTFMessage::CloneRepo
                                    | CTFMessage::ResetDB
                                    | CTFMessage::LeaveTeam => unreachable!(),
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

        ctf_ui(self, ctx, frame);
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
