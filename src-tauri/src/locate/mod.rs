mod locator;
mod beacon;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconInquiry {
    mac: String,
    id: String
}

impl BeaconInquiry {

}