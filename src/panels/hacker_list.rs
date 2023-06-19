use eframe::egui;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};

pub struct HackerList {
}

impl HackerList {
    pub fn new(ws_sender: WsSender, ws_receiver: WsReceiver) -> Self {
        Self {
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        // while let Some(event) = self.ws_receiver.try_recv() {
        //     self.events.push(event);
        // }

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.horizontal(|ui| {
        //         ui.label("Message to send:");
        //         if ui.text_edit_singleline(&mut self.text_to_send).lost_focus()
        //             && ui.input(|i| i.key_pressed(egui::Key::Enter))
        //         {
        //             self.ws_sender
        //                 .send(WsMessage::Text(std::mem::take(&mut self.text_to_send)));
        //         }
        //     });

        //     ui.separator();
        //     ui.heading("Received events:");
        //     for event in &self.events {
        //         ui.label(format!("{:?}", event));
        //     }
        // });
    }
}
