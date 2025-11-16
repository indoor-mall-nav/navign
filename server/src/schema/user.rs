#![allow(dead_code)]
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
        let hashed_password = hash(password, 12).expect("Failed to hash password");
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
        bcrypt::verify(password, self.hashed_password.as_str()).unwrap_or(false)
    }

    pub fn is_privileged(&self) -> bool {
        self.privileged
    }
}
