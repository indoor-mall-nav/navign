use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct CustomizedObjectId {
    #[serde(rename = "$oid")]
    pub oid: String,
}

impl<'a> PartialEq<&'a str> for CustomizedObjectId {
    fn eq(&self, other: &&'a str) -> bool {
        &self.oid == other
    }
}

impl Deref for CustomizedObjectId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.oid
    }
}

impl Display for CustomizedObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.oid)
    }
}
