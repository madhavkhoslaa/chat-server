use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader, split};
use tracing::{error, info};
use uuid::Uuid;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::inMemoryDB::{InMemoryDB, User, Activity};
use crate::message::{format_activity, format_user_list};

pub async fn handle_client(
    stream: TcpStream,
    client_uuid: Uuid,
    user: User,
    activity_rx: &mut tokio::sync::broadcast::Receiver<Activity>,
    db: Arc<InMemoryDB>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = split(stream);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    
    // Send list of currently connected users (excluding the new user)
    let all_users = db.get_all_users();
    let other_users: Vec<User> = all_users.into_iter()
        .filter(|u| u.uuid != user.uuid)
        .collect();
    
    let user_list_msg = format_user_list(&other_users);
    if let Err(e) = writer.write_all(user_list_msg.as_bytes()).await {
        info!("Client {} disconnected (write error sending user list: {})", client_uuid, e);
        return Ok(());
    }
    if let Err(e) = writer.write_all(b"\n").await {
        info!("Client {} disconnected (write error sending newline: {})", client_uuid, e);
        return Ok(());
    }
    if let Err(e) = writer.flush().await {
        info!("Client {} disconnected (flush error: {})", client_uuid, e);
        return Ok(());
    }
    
    loop {
        tokio::select! {
            // Listen for activities to broadcast
            result = activity_rx.recv() => {
                match result {
                    Ok(activity) => {
                        let message = format_activity(&activity);
                        info!("Broadcasting to client {}: {}", client_uuid, message);
                        if let Err(e) = writer.write_all(message.as_bytes()).await {
                            info!("Client {} disconnected (write error: {})", client_uuid, e);
                            break;
                        }
                        if let Err(e) = writer.write_all(b"\n").await {
                            info!("Client {} disconnected (write error: {})", client_uuid, e);
                            break;
                        }
                        if let Err(e) = writer.flush().await {
                            info!("Client {} disconnected (flush error: {})", client_uuid, e);
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        error!("Client {} lagged behind by {} messages", client_uuid, skipped);
                        // Continue receiving
                    }
                    Err(e) => {
                        error!("Error receiving activity for client {}: {}", client_uuid, e);
                        break;
                    }
                }
            }
            // Read messages from the client
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => {
                        // EOF - client closed the connection
                        info!("Client {} disconnected (EOF)", client_uuid);
                        break;
                    }
                    Ok(_n) => {
                        // Client sent a message
                        let content = line.trim().to_string();
                        if !content.is_empty() {
                            // Get current timestamp
                            let timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            
                            info!("Client {} ({}) sent message: '{}'", client_uuid, user.screen_name, content);
                            
                            // Send the message through the database (which will broadcast it)
                            db.send_message(user.clone(), content, timestamp);
                            
                            info!("Message broadcasted for client {}", client_uuid);
                        }
                        line.clear(); // Clear the buffer for the next message
                    }
                    Err(e) => {
                        info!("Client {} disconnected (read error: {})", client_uuid, e);
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
