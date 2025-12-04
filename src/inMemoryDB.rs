use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub struct InMemoryDB {
    users: Arc<Mutex<Vec<User>>>,
    messages: Arc<Mutex<Vec<Message>>>,
    activities: Arc<Mutex<Vec<Activity>>>,
    activity_tx: broadcast::Sender<Activity>,
}

impl InMemoryDB {
    pub fn new(activity_tx: broadcast::Sender<Activity>) -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
            activities: Arc::new(Mutex::new(Vec::new())),
            activity_tx,
        }
    }

    pub fn add_user(&self, user: User) {
        let mut users = self.users.lock().unwrap();
        users.push(user.clone());
    }

    pub fn add_activity(&self, activity: Activity) {
        let mut activities = self.activities.lock().unwrap();
        activities.push(activity.clone());
        // Broadcast the activity to all connected clients
        match self.activity_tx.send(activity) {
            Ok(_count) => {
                // count is the number of receivers that received the message
                // Note: this might be 0 if no receivers are subscribed yet
            }
            Err(e) => {
                eprintln!("Failed to broadcast activity: {:?}", e);
            }
        }
    }

    pub fn on_user_join(&self, user: User) {
        self.add_user(user.clone());
        self.add_activity(Activity::Join(user));
    }

    pub fn on_user_leave(&self, user: User) {
        // Remove user from the list
        let mut users = self.users.lock().unwrap();
        users.retain(|u| u.uuid != user.uuid);
        // Broadcast leave event
        self.add_activity(Activity::Leave(user));
    }

    pub fn send_message(&self, user: User, content: String, timestamp: u64) {
        // Store the message
        let message = Message {
            user: user.clone(),
            content: content.clone(),
            timestamp,
        };
        let mut messages = self.messages.lock().unwrap();
        messages.push(message);
        
        // Broadcast the message as an activity
        self.add_activity(Activity::Message(format!("{}: {}", user.screen_name, content)));
    }

    pub fn get_all_users(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.clone()
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub screen_name: String,
    pub uuid: String,
}

#[derive(Clone, Debug)]
pub enum Activity {
    Message(String),
    Join(User),
    Leave(User),
}

pub struct Message {
    pub user: User,
    pub content: String,
    pub timestamp: u64, //unix timestamp
}
