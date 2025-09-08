// user_service.rs - A typical service module that needs various edits

use std::collections::HashMap;
use std::sync::Arc;

pub struct UserService {
    users: HashMap<u64, User>,
    next_id: u64,
}

#[derive(Clone, Debug)]
pub struct User {
    id: u64,
    name: String,
    email: String,
}

impl UserService {
    pub fn new() -> Self {
        UserService {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_user(&mut self, name: String, email: String) -> User {
        let user = User {
            id: self.next_id,
            name,
            email,
        };
        self.users.insert(user.id, user.clone());
        self.next_id += 1;
        user
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }
}

// More functionality needed here...