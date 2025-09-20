use crate::ble::protocol::BleProtocolHandler;
use crate::crypto::challenge::{Challenge, ChallengeManager};
use crate::crypto::proof::ProofManager;
use crate::crypto::Nonce;
use crate::shared::constants::{MAX_ATTEMPTS, MAX_PACKET_SIZE};
use crate::shared::{BleError, CryptoError};
use crate::storage::nonce_manager::NonceManager;
use esp_hal::gpio::{Input, Level, Output};
use esp_hal::rng::Rng;
use esp_println::println;

#[derive(Debug)]
pub struct BeaconState<'a> {
    pub human_sensor: Input<'a>,
    pub relay: Output<'a>,
    pub open: Output<'a>,
    pub unlock_attempts: u8,
    pub nonce_manager: NonceManager<32>,
    pub challenge_manager: ChallengeManager,
    pub buffer: BleProtocolHandler,
    pub proof_manager: ProofManager,
    pub last_open: u64,
    pub last_relay_on: u64,
    pub triggered: bool,
}

impl<'a> BeaconState<'a> {
    pub fn new(
        private_key: [u8; 32],
        human_sensor: Input<'a>,
        mut relay: Output<'a>,
        mut open: Output<'a>,
        rng: Rng,
    ) -> Self {
        relay.set_low();
        open.set_low();
        Self {
            human_sensor,
            relay,
            open,
            nonce_manager: NonceManager::<32>::new(),
            proof_manager: ProofManager::new(private_key),
            unlock_attempts: 0,
            buffer: BleProtocolHandler::new(),
            challenge_manager: ChallengeManager::new(rng),
            last_open: 0,
            last_relay_on: 0,
            triggered: false,
        }
    }

    pub fn set_server_public_key(&mut self, public_key: [u8; 64]) -> Result<(), ()> {
        self.proof_manager
            .set_server_public_key(public_key)
            .map_err(|_| ())
    }

    pub fn set_open(&mut self, state: bool, current: u64) {
        self.open.set_level(Level::from(state));
        if state {
            self.last_open = current;
        }
    }

    /// Once open, wait for 5 seconds before closing.
    /// During the window, if human sensor is triggered, keep it open and close it 3 seconds after last trigger.
    /// If human sensor is not triggered, close it after 5 seconds.
    /// This is used for running in a loop.
    pub fn check_executors(&mut self, time: u64) {
        if time % 20000 == 0 {
            println!("Checking executors at time: {}", time);
            println!("Open state: {}", self.open.is_set_high());
            println!("Human sensor state: {}", self.human_sensor.is_high());
            println!("Last open time: {}", self.last_open);
            println!("Relay state: {}", self.relay.is_set_high());
            println!("Last relay on time: {}", self.last_relay_on);
        }
        if self.open.is_set_high() && !self.triggered {
            if self.human_sensor.is_high() && self.relay.is_set_low() {
                self.relay.set_high();
                self.last_open = time;
                self.triggered = true;
            }
            if time - self.last_open > 10_000 {
                self.open.set_low();
            }
        }

        if self.relay.is_set_high() && time - self.last_relay_on > 5_000 {
            self.relay.set_low();
            self.open.set_low();
        }
    }

    pub fn validate_proof(
        &mut self,
        proof: &crate::crypto::proof::Proof,
        current_timestamp: u64,
    ) -> Result<(), CryptoError> {
        if self.unlock_attempts >= MAX_ATTEMPTS && current_timestamp - proof.timestamp < 300_000 {
            return Err(CryptoError::RateLimited);
        }

        if !self
            .nonce_manager
            .check_and_mark_challenge_hash(proof.challenge_hash, proof.timestamp)
        {
            self.unlock_attempts += 1;
            return Err(CryptoError::ReplayDetected);
        }

        match self.proof_manager.validate_proof(proof) {
            Ok(_) => {
                self.unlock_attempts = 0;
                Ok(())
            }
            Err(e) => {
                self.unlock_attempts += 1;
                Err(e)
            }
        }
    }

    pub fn generate_nonce(&mut self, rng: &mut Rng) -> Nonce {
        self.nonce_manager.generate_nonce(rng)
    }

    pub fn serialize_message(
        &mut self,
        message: &crate::ble::protocol::BleMessage,
    ) -> Result<[u8; MAX_PACKET_SIZE], BleError> {
        self.buffer.serialize_message(message)
    }

    pub fn deserialize_message(
        &mut self,
        data: Option<&[u8]>,
    ) -> Result<crate::ble::protocol::BleMessage, BleError> {
        self.buffer.deserialize_message(data)
    }

    pub fn read_human_sensor(&self) -> bool {
        self.human_sensor.is_high()
    }
}
