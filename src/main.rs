use tokio::net::TcpListener;
use tracing::{error, info};
use std::sync::Arc;
use tcp_server::inMemoryDB::{InMemoryDB, Activity};
use tcp_server::client;
use tcp_server::user;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:5000";
    let listener = TcpListener::bind(addr).await?;
    info!("Server running on {}", addr);

    // Create a broadcast channel for activities
    let (activity_tx, _) = tokio::sync::broadcast::channel::<Activity>(1000);

    // Initialize the in-memory database with the broadcast sender
    let db = Arc::new(InMemoryDB::new(activity_tx.clone()));

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                continue;
            }
        };

        // Clone the database reference and broadcast receiver for this connection
        let db = db.clone();
        let mut activity_rx = activity_tx.subscribe();

        // Create a new user
        let (uuid, user) = user::create_user();

        // Add user to database and create join event
        db.on_user_join(user.clone());

        // Log the event
        info!("Client connected: {}  UUID={}  Name={}", peer, uuid, user.screen_name);

        // Spawn a task for the client
        let user_for_leave = user.clone();
        tokio::spawn(async move {
            if let Err(e) = client::handle_client(stream, uuid, user, &mut activity_rx, db.clone()).await {
                error!("Error handling client {}: {}", uuid, e);
            }
            // Broadcast leave event when client disconnects
            db.on_user_leave(user_for_leave);
            info!("Client {} disconnected", uuid);
        });
    }
}
