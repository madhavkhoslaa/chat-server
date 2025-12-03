use tokio::net::TcpListener;
use tracing::{error, info};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:5000";
    let listener = TcpListener::bind(addr).await?;
    info!("Server running on {}", addr);

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                continue;
            }
        };

        // Generate a UUID for this client
        let id = Uuid::new_v4();

        // Log the event
        info!("Client connected: {}  UUID={}", peer, id);

        // Spawn a task for the client (even if we do nothing with it yet)
        tokio::spawn(async move {
            // Keep the connection open until the client disconnects
            // (We simply wait for stream to close)
            let _ = stream.readable().await;
            info!("Client {} disconnected", id);
        });
    }
}
