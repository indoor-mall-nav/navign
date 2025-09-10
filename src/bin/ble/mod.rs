mod protocol;

use super::crypto::{Nonce, Proof};

#[derive(Debug, Clone)]
pub enum BleMessage {
    NonceRequest,
    NonceResponse(Nonce),
    ProofSubmission(Proof),
    UnlockResult(bool),
}

#[derive(Debug)]
pub enum BleError {
    NotConnected,
    SendFailed,
    ReceiveFailed,
    ParseError,
    BufferFull,
}
