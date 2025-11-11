use crate::schema::Service;
use uuid::Uuid;
use sqlx::FromRow;
use p256::ecdsa::VerifyingKey;
use p256::pkcs8::DecodePublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPublicKeys {
    pub id: Uuid,
    pub user: String,
    pub model: String,
    pub arch: String,
    pub identifier: String,
    pub public_key: String,
}

impl Service for UserPublicKeys {
    type Id = Uuid;

    fn get_id(&self) -> Uuid {
        self.id
    }
    fn get_name(&self) -> String {
        self.identifier.clone()
    }
    fn set_name(&mut self, name: String) {
        self.identifier = name;
    }
    fn get_description(&self) -> Option<String> {
        None
    }
    fn set_description(&mut self, _description: Option<String>) {}
    fn get_table_name() -> &'static str {
        "user_public_keys"
    }
    fn require_unique_name() -> bool {
        true
    }
}

impl UserPublicKeys {
    pub fn public_key(&self) -> Option<VerifyingKey> {
        VerifyingKey::from_public_key_pem(self.public_key.as_str()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_public_keys_creation() {
        let keys = UserPublicKeys {
            id: Uuid::new_v4(),
            user: "user123".to_string(),
            model: "iPhone 14".to_string(),
            arch: "arm64".to_string(),
            identifier: "device-001".to_string(),
            public_key: "test_key".to_string(),
        };

        assert_eq!(keys.user, "user123");
        assert_eq!(keys.model, "iPhone 14");
        assert_eq!(keys.arch, "arm64");
        assert_eq!(keys.identifier, "device-001");
    }

    #[test]
    fn test_service_trait_implementation() {
        let keys = UserPublicKeys {
            id: Uuid::new_v4(),
            user: "user456".to_string(),
            model: "Android".to_string(),
            arch: "x86_64".to_string(),
            identifier: "device-002".to_string(),
            public_key: "test_key".to_string(),
        };

        assert_eq!(keys.get_name(), "device-002");
        assert_eq!(keys.get_description(), None);
        assert_eq!(UserPublicKeys::get_table_name(), "user_public_keys");
        assert!(UserPublicKeys::require_unique_name());
    }

    #[test]
    fn test_service_set_name() {
        let mut keys = UserPublicKeys {
            id: Uuid::new_v4(),
            user: "user789".to_string(),
            model: "Desktop".to_string(),
            arch: "x86_64".to_string(),
            identifier: "old-identifier".to_string(),
            public_key: "test_key".to_string(),
        };

        keys.set_name("new-identifier".to_string());
        assert_eq!(keys.identifier, "new-identifier");
        assert_eq!(keys.get_name(), "new-identifier");
    }

    #[test]
    fn test_serialization() {
        let keys = UserPublicKeys {
            id: Uuid::new_v4(),
            user: "testuser".to_string(),
            model: "Test Model".to_string(),
            arch: "test_arch".to_string(),
            identifier: "test-device".to_string(),
            public_key: "pem_content".to_string(),
        };

        let json = serde_json::to_string(&keys).unwrap();
        let deserialized: UserPublicKeys = serde_json::from_str(&json).unwrap();

        assert_eq!(keys.user, deserialized.user);
        assert_eq!(keys.model, deserialized.model);
        assert_eq!(keys.arch, deserialized.arch);
        assert_eq!(keys.identifier, deserialized.identifier);
    }

    #[test]
    fn test_public_key_invalid_pem() {
        let keys = UserPublicKeys {
            id: Uuid::new_v4(),
            user: "user".to_string(),
            model: "model".to_string(),
            arch: "arch".to_string(),
            identifier: "id".to_string(),
            public_key: "invalid_pem_data".to_string(),
        };

        // Invalid PEM should return None
        assert!(keys.public_key().is_none());
    }

    #[test]
    fn test_service_set_description() {
        let mut keys = UserPublicKeys {
            id: Uuid::new_v4(),
            user: "user".to_string(),
            model: "model".to_string(),
            arch: "arch".to_string(),
            identifier: "id".to_string(),
            public_key: "key".to_string(),
        };

        // set_description should be a no-op
        keys.set_description(Some("description".to_string()));
        assert_eq!(keys.get_description(), None);
    }

    #[test]
    fn test_get_id_returns_uuid() {
        let uuid = Uuid::new_v4();
        let keys = UserPublicKeys {
            id: uuid,
            user: "user".to_string(),
            model: "model".to_string(),
            arch: "arch".to_string(),
            identifier: "id".to_string(),
            public_key: "key".to_string(),
        };

        let id = keys.get_id();
        assert_eq!(id, uuid);
        assert_eq!(id.to_string().len(), 36); // UUID string is 36 characters
    }
}
