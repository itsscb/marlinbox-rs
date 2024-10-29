use std::{fs, path::PathBuf, sync::Arc};

use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    routing::{get, post},
};
use crossbeam_channel::{Receiver, Sender};
use tokio::runtime::Runtime;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, error, info};

/// Starts the Manager and listens for incoming connections.
///
/// # Arguments
///
/// * `connection_string` - A string representing the connection address and port.
/// * `tx_pairing` - An Arc-wrapped Sender used for message passing.
/// * `rx_shutdown` - A Receiver used to receive shutdown signals.
///
/// # Returns
///
/// Returns `Ok(())` if the server started successfully, otherwise returns an error.
pub fn serve(
    connection_string: &str,
    tx_pairing: Arc<Sender<()>>,
    rx_shutdown: Arc<Receiver<()>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    let app = axum::Router::new()
        .nest_service("/assets", ServeDir::new(PathBuf::from("assets")))
        .route_service("/", ServeFile::new(PathBuf::from("assets/index.html")))
        .route("/pair", get(handler))
        .route("/failed", get(|| async { "Failed to send message" }))
        .route("/upload", post(upload_file))
        .with_state(tx_pairing);

    rt.block_on(async {
        let listener = match tokio::net::TcpListener::bind(connection_string).await {
            Ok(it) => {
                info!("Manager listening on: {connection_string}");
                it
            }
            Err(err) => {
                error!("Manager failed to bind to {connection_string}: {err}");
                return;
            }
        };
        match axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                match rx_shutdown.recv() {
                    Ok(()) => info!("Manager shutting down"),
                    Err(err) => error!("Manager failed to shut down: {err}"),
                }
            })
            .await
        {
            Ok(()) => info!("Manager listening on {connection_string}"),
            Err(err) => error!("Manager failed to start: {err}"),
        }
    });
    Ok(())
}

async fn handler(State(state): State<Arc<Sender<()>>>) -> impl IntoResponse {
    match state.send(()) {
        Ok(()) => {
            info!("Sent pairing request");
            "Pairing"
        }
        Err(err) => {
            error!("Failed to send pairing request: {err}");
            "Failed to start pairing"
        }
    }
}

async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    while let Ok(resp) = multipart.next_field().await {
        if let Some(field) = resp {
            let Some(filename) = field.file_name() else {
                return "No file name provided";
            };
            let filepath = PathBuf::from(format!("./uploads/{filename}"));

            if let Err(err) = fs::create_dir_all("uploads") {
                error!("Failed to create directory: {err}");
                return "Failed to create directory";
            }

            let data = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => {
                    error!("Failed to read file: {err}");
                    return "Failed to read file";
                }
            };
            if let Err(err) = fs::write(&filepath, data) {
                error!("Failed to write file: {err}");
                return "Failed to write file";
            }
            debug!("File uploaded: {}", filepath.to_string_lossy());
        } else {
            break;
        }
    }
    "File uploaded"
}
