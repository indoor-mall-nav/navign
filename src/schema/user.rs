use crate::schema::Service;
use bcrypt::hash;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub email: String,
    phone: Option<String>,
    google: Option<String>,
    wechat: Option<String>,
    hashed_password: String,
    pub activated: bool,
    privileged: bool,
}

impl User {
    pub fn new(
        username: String,
        email: String,
        phone: Option<String>,
        google: Option<String>,
        wechat: Option<String>,
        password: String,
    ) -> Self {
        let hashed_password = hash(password, 4).expect("Failed to hash password");
        Self {
            id: ObjectId::new(),
            username,
            email,
            phone,
            google,
            wechat,
            hashed_password,
            activated: false,
            privileged: false,
        }
    }

    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.hashed_password).unwrap_or(false)
    }

    pub fn is_privileged(&self) -> bool {
        self.privileged
    }
}

impl Service for User {
    fn get_id(&self) -> String {
        self.id.to_hex()
    }

    fn get_name(&self) -> String {
        self.username.clone()
    }

    fn set_name(&mut self, name: String) {
        self.username = name;
    }

    fn get_description(&self) -> Option<String> {
        Some(self.email.clone())
    }

    fn set_description(&mut self, description: Option<String>) {
        if let Some(email) = description {
            self.email = email;
        }
    }

    fn get_collection_name() -> &'static str {
        "users"
    }

    fn require_unique_name() -> bool {
        false
    }
}
