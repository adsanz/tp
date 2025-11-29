use arboard::Clipboard;
use crossbeam_channel::Receiver;
use eframe::egui;
use qr_code::QrCode;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

pub struct TpApp {
    rx: Receiver<String>,
    log_rx: Receiver<String>,
    clipboard: Option<Clipboard>,
    last_content: String,
    history: Vec<String>,
    logs: Vec<String>,
    ip: IpAddr,
    port: u16,
    qr_texture: Option<egui::TextureHandle>,
    use_https: bool,
    show_logs: bool,
    pc_content: Arc<Mutex<String>>,
    input_content: String,
}

impl TpApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        rx: Receiver<String>,
        log_rx: Receiver<String>,
        ip: IpAddr,
        port: u16,
        use_https: bool,
        pc_content: Arc<Mutex<String>>,
    ) -> Self {
        let clipboard = Clipboard::new().ok();
        Self {
            rx,
            log_rx,
            clipboard,
            last_content: "Waiting for content...".to_string(),
            history: Vec::new(),
            logs: Vec::new(),
            ip,
            port,
            qr_texture: None,
            use_https,
            show_logs: false,
            pc_content,
            input_content: String::new(),
        }
    }

    fn update_clipboard(&mut self, text: &str) {
        if let Some(cb) = &mut self.clipboard {
            if let Err(e) = cb.set_text(text.to_owned()) {
                tracing::error!("Failed to set clipboard: {}", e);
            } else {
                tracing::info!("Clipboard updated!");
            }
        } else {
            tracing::warn!("Clipboard not available");
        }
    }
}

impl eframe::App for TpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint to ensure we poll channels frequently
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Check for new messages
        while let Ok(msg) = self.rx.try_recv() {
            self.last_content = msg.clone();
            self.history.insert(0, msg.clone());
            // Keep history reasonable
            if self.history.len() > 50 {
                self.history.pop();
            }
            self.update_clipboard(&msg);
        }

        // Check for new logs
        while let Ok(log) = self.log_rx.try_recv() {
            self.logs.push(log);
            // Keep log size reasonable
            if self.logs.len() > 1000 {
                self.logs.remove(0);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("TP - Teleport Platform");

                ui.add_space(10.0);

                let protocol = if self.use_https { "https" } else { "http" };
                let url = format!("{}://{}:{}", protocol, self.ip, self.port);
                ui.label(format!("Listening on: {}", url));

                ui.add_space(10.0);

                // Generate QR code if not already done
                if self.qr_texture.is_none() {
                    if let Ok(code) = QrCode::new(url.as_bytes()) {
                        let width = code.width();
                        let height = width;
                        let colors = code.into_colors();

                        let mut pixels = Vec::with_capacity(width * height * 4);
                        for color in colors {
                            let val = match color {
                                qr_code::Color::Light => 255,
                                qr_code::Color::Dark => 0,
                            };
                            pixels.extend_from_slice(&[val, val, val, 255]);
                        }

                        let image =
                            egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels);

                        // Scale it up for display
                        self.qr_texture =
                            Some(ctx.load_texture("qr_code", image, egui::TextureOptions::NEAREST));
                    }
                }

                if let Some(texture) = &self.qr_texture {
                    // Display it larger
                    ui.image((texture.id(), texture.size_vec2() * 4.0));
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(20.0);

                ui.heading("Received Content:");

                // Limit height of received content
                egui::ScrollArea::vertical()
                    .id_salt("content_scroll")
                    .max_height(100.0)
                    .show(ui, |ui| {
                        ui.label(&self.last_content);
                    });

                if ui.button("Copy to Clipboard (Manual)").clicked() {
                    let text = self.last_content.clone();
                    self.update_clipboard(&text);
                }

                ui.add_space(20.0);
                ui.separator();
                ui.heading("Send to Phone:");

                ui.add(
                    egui::TextEdit::multiline(&mut self.input_content)
                        .hint_text("Type here to send to phone..."),
                );

                ui.horizontal(|ui| {
                    if ui.button("Paste from Clipboard").clicked() {
                        if let Some(cb) = &mut self.clipboard {
                            if let Ok(text) = cb.get_text() {
                                self.input_content = text;
                            }
                        }
                    }
                    if ui.button("Send / Update").clicked() {
                        if let Ok(mut content) = self.pc_content.lock() {
                            *content = self.input_content.clone();
                            tracing::info!("Updated PC content to be available for phone");
                        }
                    }
                });

                ui.add_space(20.0);
                ui.separator();
                ui.heading("History:");

                let mut text_to_copy = None;
                egui::ScrollArea::vertical()
                    .id_salt("history_scroll")
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for msg in &self.history {
                            ui.horizontal(|ui| {
                                if ui.button("Copy").clicked() {
                                    text_to_copy = Some(msg.clone());
                                }
                                // Show first line or truncated content
                                let display_text = if msg.len() > 50 {
                                    format!("{}...", &msg[0..50].replace('\n', " "))
                                } else {
                                    msg.replace('\n', " ")
                                };
                                ui.label(display_text);
                            });
                        }
                    });

                if let Some(text) = text_to_copy {
                    self.update_clipboard(&text);
                }

                ui.add_space(20.0);
                ui.separator();
                ui.checkbox(&mut self.show_logs, "Show Logs");
            });
        });

        if self.show_logs {
            egui::TopBottomPanel::bottom("logs_panel")
                .resizable(true)
                .min_height(100.0)
                .show(ctx, |ui| {
                    ui.heading("Logs");
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for log in &self.logs {
                                ui.label(log.trim());
                            }
                        });
                });
        }
    }
}
