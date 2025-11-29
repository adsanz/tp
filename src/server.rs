use axum::{
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};

use axum_server::tls_rustls::RustlsConfig;
use crossbeam_channel::Sender;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerState {
    pub tx: Sender<String>,
}

#[derive(Deserialize)]
struct MsgForm {
    content: String,
}

pub async fn start_server(
    port: u16,
    tx: Sender<String>,
    cert_key: Option<(Vec<u8>, Vec<u8>)>,
) -> anyhow::Result<()> {
    let state = Arc::new(ServerState { tx });

    let app = Router::new()
        .route("/", get(index))
        .route("/send", post(receive_msg))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    if let Some((cert, key)) = cert_key {
        let config = RustlsConfig::from_der(vec![cert], key).await?;
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await?;
        return Ok(());
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TP - Transfer</title>
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; 
            padding: 20px; 
            max-width: 600px; 
            margin: 0 auto; 
            background-color: #ffffff; 
            color: #333333; 
        }
        h2 { color: #110b00ff; }
        textarea { 
            width: 100%; 
            height: 150px; 
            margin-bottom: 15px; 
            padding: 12px; 
            box-sizing: border-box; 
            border: 1px solid #ccc; 
            border-radius: 8px; 
            font-size: 16px; /* Prevent zoom on iOS */
        }
        button { 
            padding: 15px 20px; 
            font-size: 1.2em; 
            width: 100%; 
            background-color: #001c3aff; 
            color: white; 
            border: none; 
            border-radius: 8px; 
            cursor: pointer; 
            transition: background-color 0.2s;
        }
        button:hover { background-color: #00244bff; }
        button:active { background-color: #000000ff; }
        #status { margin-top: 15px; font-weight: bold; min-height: 1.2em; text-align: center; }
    </style>
</head>
<body>
    <h2>Send to PC</h2>
    <noscript>
        <p style="color: red;">JavaScript is required to use this app.</p>
    </noscript>
    <form id="msgForm">
        <textarea id="content" name="content" placeholder="Paste text here..." autofocus></textarea>
        <br>
        <button type="submit">Send</button>
    </form>
    <div id="status"></div>

    <script>
        document.getElementById('msgForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const content = document.getElementById('content').value;
            const statusDiv = document.getElementById('status');
            
            if (!content) {
                statusDiv.innerHTML = '<p style="color: orange;">Please enter some text.</p>';
                return;
            }

            statusDiv.innerHTML = '<p style="color: blue;">Sending...</p>';
            
            try {
                const response = await fetch('/send', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/x-www-form-urlencoded',
                    },
                    body: 'content=' + encodeURIComponent(content)
                });
                
                if (response.ok) {
                    statusDiv.innerHTML = '<p style="color: green;">Sent successfully!</p>';
                    document.getElementById('content').value = '';
                    setTimeout(() => { statusDiv.innerHTML = ''; }, 3000);
                } else {
                    statusDiv.innerHTML = '<p style="color: red;">Error sending.</p>';
                }
            } catch (error) {
                console.error('Error:', error);
                statusDiv.innerHTML = '<p style="color: red;">Network error.</p>';
            }
        });
    </script>
</body>
</html>
"#,
    )
}
async fn receive_msg(
    axum::extract::State(state): axum::extract::State<Arc<ServerState>>,
    Form(form): Form<MsgForm>,
) -> impl IntoResponse {
    tracing::info!(
        "Received content (len={}): '{}'",
        form.content.len(),
        form.content
    );
    let _ = state.tx.send(form.content.clone());
    // Return 200 OK
    axum::http::StatusCode::OK
}
