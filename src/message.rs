use colored::*;
use crate::inMemoryDB::{Activity, User};

pub fn format_activity(activity: &Activity) -> String {
    match activity {
        Activity::Join(user) => {
            format!("{} {}", "JOIN:".green().bold(), user.screen_name.green())
        }
        Activity::Leave(user) => {
            format!("{} {}", "LEAVE:".red().bold(), user.screen_name.red())
        }
        Activity::Message(msg) => {
            // Parse the message to color the username and content separately
            if let Some((username, content)) = msg.split_once(": ") {
                format!("{} {} {}", 
                    "MESSAGE:".cyan().bold(), 
                    username.cyan().bold(), 
                    content.white()
                )
            } else {
                format!("{} {}", "MESSAGE:".cyan().bold(), msg.white())
            }
        }
    }
}

pub fn format_user_list(users: &[User]) -> String {
    if users.is_empty() {
        format!("{} {}", "USERS:".yellow().bold(), "No other users connected".white())
    } else {
        let user_names: Vec<String> = users.iter()
            .map(|u| u.screen_name.clone())
            .collect();
        format!("{} {}", 
            "USERS:".yellow().bold(), 
            user_names.join(", ").white()
        )
    }
}

