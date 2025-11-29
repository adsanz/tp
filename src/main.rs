#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

mod cert;
mod gui;
mod logger;
mod server;

use clap::Parser;
use crossbeam_channel::unbounded;
use local_ip_address::local_ip;
use std::thread;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Use HTTP instead of HTTPS
    #[arg(long)]
    http: bool,
}

fn load_icon() -> eframe::egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon_bytes = include_bytes!("../icon.png");
        let image = image::load_from_memory(icon_bytes)
            .expect("Failed to load icon from memory")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

fn main() -> anyhow::Result<()> {
    // Setup logging
    let (log_tx, log_rx) = unbounded();
    let log_writer = logger::LogWriter::new(log_tx);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(log_writer)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    let ip = local_ip()?;

    let cert_key = if !args.http {
        let _ = rustls::crypto::ring::default_provider().install_default();
        tracing::info!("Generating self-signed certificates...");
        let certs =
            cert::generate_self_signed_certs(vec![ip.to_string(), "localhost".to_string()])?;
        Some((certs.cert, certs.key))
    } else {
        None
    };

    let (tx, rx) = unbounded();

    let port = args.port;
    let is_http = args.http;
    
    let pc_content = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let pc_content_server = pc_content.clone();
    let pc_content_gui = pc_content.clone();

    // Start server in a separate thread
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if !is_http {
                tracing::info!("Starting server with HTTPS on port {}", port);
                let res = server::start_server(port, tx, cert_key, pc_content_server).await;
                if let Err(e) = res {
                    tracing::error!("Server error: {}", e);
                }
            } else {
                tracing::info!("Starting server with HTTP on port {}", port);
                let res = server::start_server(port, tx, cert_key, pc_content_server).await;
                if let Err(e) = res {
                    tracing::error!("Server error: {}", e);
                }
            }
        });
    });

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([500.0, 700.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    let use_https_gui = !args.http;
    eframe::run_native(
        "TP - Teleport Platform",
        options,
        Box::new(move |cc| {
            Ok(Box::new(gui::TpApp::new(
                cc,
                rx,
                log_rx,
                ip,
                port,
                use_https_gui,
                pc_content_gui,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}
