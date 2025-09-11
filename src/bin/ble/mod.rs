pub(super) mod constants;
mod manager;
pub(crate) mod protocol;

use super::crypto::{Nonce, Proof};
use bleps::gatt;

#[derive(Debug, Clone)]
pub enum BleMessage {
    NonceRequest,
    NonceResponse(Nonce),
    ProofSubmission(Proof),
    UnlockResult(bool),
}

#[derive(Debug)]
pub enum BleError {
    SetupFailed,
    NotConnected,
    SendFailed,
    ReceiveFailed,
    ParseError,
    BufferFull,
}
