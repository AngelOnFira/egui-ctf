use common::{
    ctf_message::{CTFClientState, CTFClientStateComponent, CTFMessage, ClientUpdate},
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
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientState {
    // pub credentials: Option<Credentials>,
    pub ctf_state: CTFClientState,
}

#[derive(Clone)]
pub struct ConnectionState {
    inner: Arc<Mutex<ConnectionStateInner>>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ConnectionStateInner {
                connection_state: ConnectionStateEnum::Disconnected,
                message_queue: Vec::new(),
                ws_sender: None,
                ws_receiver: None,
            })),
        }
    }
}

pub struct ConnectionStateInner {
    connection_state: ConnectionStateEnum,
    message_queue: Vec<NetworkMessage>,
    ws_sender: Option<WsSender>,
    ws_receiver: Option<WsReceiver>,
}

impl ConnectionState {
    pub fn send_message(&mut self, message: NetworkMessage) {
        // If we're connected to the backend, send the message right away. If
        // we're connecting or disconnected, queue the message to be sent when
        // we connect.

        // Get access to the inner
        let mut inner = self.inner.lock().unwrap();

        // Add it to the message queue
        inner.message_queue.push(message);

        // Drop the lock so that we can have unique access to self again
        drop(inner);

        // Call the empty queue function in case we're connected
        self.process_message_queue();
    }

    // Try to empty the queue of messages to send to the backend. This may or
    // may not send messages.
    fn process_message_queue(&mut self) {
        // Get access to the inner
        let mut inner = self.inner.lock().unwrap();

        // If we're connected to the backend, send the message right away. If
        // we're connecting or disconnected, queue the message to be sent when
        // we connect.
        match inner.connection_state {
            ConnectionStateEnum::Opened => {
                // TODO: figure out how to not need to clone since we're just
                // taking ownership of the queue
                let messages = inner.message_queue.clone();
                inner.message_queue.clear();
                for message in messages {
                    inner
                        .ws_sender
                        .as_mut()
                        .unwrap()
                        .send(WsMessage::Text(serde_json::to_string(&message).unwrap()));
                }
            }
            _ => {}
        }
    }

    fn set_state_connecting(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Connecting;
    }

    fn set_state_connected(&mut self, ws_sender: WsSender, ws_receiver: WsReceiver) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Connected;
        inner.ws_sender = Some(ws_sender);
        inner.ws_receiver = Some(ws_receiver);
    }

    fn set_state_opened(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Opened;
    }

    fn set_state_disconnected(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.connection_state = ConnectionStateEnum::Disconnected;
    }

    fn get_state(&self) -> ConnectionStateEnum {
        let inner = self.inner.lock().unwrap();
        inner.connection_state.clone()
    }
}

#[derive(Clone, Debug)]
pub enum ConnectionStateEnum {
    Disconnected,
    Connecting,
    Connected,
    Opened,
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
            submission_panel: SubmissionPanel::default(),
            team_panel: TeamPanel::default(),
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
        self.connection_state.set_state_connecting();

        let ws_state_queue_clone = self.ws_state_queue.clone();

        let wakeup = move |event: WsEvent| {
            ws_state_queue_clone.lock().unwrap().queue.push(event);
            ctx.request_repaint(); // wake up UI thread on new message}
        };
        match ewebsock::connect_with_wakeup("ws://127.0.0.1:4040/ws", wakeup) {
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
        // Start by seeing if the websocket has any new messages
        for event in self.ws_state_queue.lock().unwrap().queue.drain(..) {
            match event {
                WsEvent::Opened => {
                    self.connection_state.set_state_opened();
                }
                WsEvent::Closed => {
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

                        // Debug the state of the app
                        info!("App state: {:?}", self.client_state);
                        // info!("Connection state: {:?}", self.connection_state);
                        info!("Auth state: {:?}", self.authentication_state);
                        info!("Message: {:?}", message);

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
                                    CTFMessage::SubmitFlag(_)
                                    | CTFMessage::JoinTeam(_)
                                    | CTFMessage::CreateTeam(_)
                                    | CTFMessage::Login(_) => unreachable!(),
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
            if let ConnectionStateEnum::Opened = &self.connection_state.get_state() {
                // Check if we're authenticated
                match &self.authentication_state.state {
                    AuthenticationStateEnum::NotAuthenticated => {
                        // Show the login panel
                        self.login_panel.show(ctx, &mut self.connection_state);
                    }
                    AuthenticationStateEnum::Authenticated => {
                        // Show the hacker list
                        self.hacker_list.show(ctx, &self.client_state);

                        // Show the submission panel
                        self.submission_panel.show(ctx, &mut self.connection_state);

                        // Show the team panel
                        self.team_panel
                            .show(ctx, &self.client_state, &mut self.connection_state);
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
