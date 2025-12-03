struct inMemoryDB {
    users: Vec<User>
    messages: Vec<Message>
    activities: Vec<Activity>
}

struct User {
    id: String,
    screen_name: String,
}

enum Activity {
    Message(String),
    Join(User),
    Leave(User),
}