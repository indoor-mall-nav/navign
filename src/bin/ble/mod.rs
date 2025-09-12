mod manager;
pub(crate) mod protocol;

use super::crypto::{Nonce, Proof};
use crate::shared::CryptoError;

#[derive(Debug, Clone)]
pub enum BleMessage {
    NonceRequest,
    NonceResponse(Nonce),
    ProofSubmission(Proof),
    UnlockResult(bool, Option<CryptoError>),
}
