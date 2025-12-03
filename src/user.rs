use uuid::Uuid;
use rnglib::{RNG, Language};
use crate::inMemoryDB::User;

pub fn create_user() -> (Uuid, User) {
    let uuid = Uuid::new_v4();
    let uuid_string = uuid.to_string();
    
    // Generate a random name for the user
    let rng = RNG::try_from(&Language::Fantasy).unwrap();
    let screen_name = rng.generate_name_by_count(3);
    
    let user = User {
        id: uuid_string.clone(),
        screen_name: screen_name.clone(),
        uuid: uuid_string.clone(),
    };
    
    (uuid, user)
}

