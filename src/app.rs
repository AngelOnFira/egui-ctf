use common::{
    ctf_message::{CTFClientState, CTFMessage},
    NetworkMessage,
};
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};

use crate::panels::{frontend::FrontEnd, hacker_list::HackerList, login::LoginPanel};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    login_panel: Option<LoginPanel>,

    #[serde(skip)]
    hacker_list: HackerList,

    #[serde(skip)]
    frontend: Option<FrontEnd>,

    #[serde(skip)]
    connection_state: ConnectionState,

    // #[serde(skip)]
    // websocket_connection:
    #[serde(skip)]
    websocket_thread_handle: Option<std::thread::JoinHandle<()>>,

    client_state: ClientState,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ClientState {
    // pub credentials: Option<Credentials>,
    pub ctf_state: Option<CTFClientState>,
}

pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected {
        ws_sender: WsSender,
        ws_receiver: WsReceiver,
    },
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Panels
            login_panel: None,
            frontend: None,
            hacker_list: HackerList::default(),
            // Other state
            websocket_thread_handle: None,
            connection_state: ConnectionState::Disconnected,
            client_state: ClientState { ctf_state: None },
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Even if we can load state, we still need to get the websocket thread
        // handle

        // // Start a thread to handle websocket events
        // // let (ws_tx, ws_rx) = std::sync::mpsc::channel();
        // let (mut sender, receiver) = ewebsock::connect("wss://ws.postman-echo.com/raw").unwrap();

        // sender.send(ewebsock::WsMessage::Text("Hellioooo!".into()));
        // let ws_thread = wasm_bindgen_futures::spawn_local(move || {
        //     while let Some(event) = receiver.try_recv() {
        //         panic!("Received {:?}", event);
        //     }
        // });

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Self {
            websocket_thread_handle: None,
            // websocket_thread_handle: Some(ws_thread),
            ..Default::default()
        }
    }

    fn connect(&mut self, ctx: egui::Context) {
        self.connection_state = ConnectionState::Connecting;
        let wakeup = move || ctx.request_repaint(); // wake up UI thread on new message
        match ewebsock::connect_with_wakeup("ws://127.0.0.1:4040/ws", wakeup) {
            Ok((ws_sender, ws_receiver)) => {
                // self.frontend = Some(FrontEnd::new(ws_sender, ws_receiver));
                self.connection_state = ConnectionState::Connected {
                    ws_sender,
                    ws_receiver,
                };
                // self.error.clear();
            }
            Err(error) => {
                panic!("Failed to connect {}", error);
            }
        }
    }
}

impl eframe::App for TemplateApp {
    // /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let Self {
        //     label,
        //     value,
        //     login_panel,
        //     connection_state,
        //     ..
        // } = self;

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
                                    CTFMessage::CTFClientState(ctf_client_state) => {
                                        self.client_state.ctf_state = Some(ctf_client_state);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        };

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        //     egui::menu::bar(ui, |ui| {
        //         ui.menu_button("File", |ui| {
        //             if ui.button("Quit").clicked() {
        //                 _frame.close();
        //             }
        //         });
        //     });
        // });
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(&mut self.label);
            // });

            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     self.value += 1.0;
            // }

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
                // Show the hacker list
                self.hacker_list.show(ctx, &self.client_state)
            }

            // // Add the login panel
            // if let Some(login_panel) = &mut self.login_panel {
            //     login_panel.show(ctx, &mut true);
            // }

            // if let Some(frontend) = &mut self.frontend {
            //     frontend.ui(ctx);
            // }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }

    // fn setup(
    //     &mut self,
    //     _ctx: &egui::Context,
    //     frame: &Frame,
    //     _storage: Option<&dyn Storage>,
    // ) {
    //     if let Some(web_info) = &frame.info().web_info {
    //         // allow `?url=` query param
    //         if let Some(url) = web_info.location.query_map.get("url") {
    //             self.url = url.clone();
    //         }
    //     }
    //     if self.url.is_empty() {
    //         self.url = "wss://echo.websocket.events/.ws".into(); // echo server
    //     }

    //     self.connect(frame.clone());
    // }

    // fn name(&self) -> &str {
    //     todo!()
    // }
}
