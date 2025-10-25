use crate::ble::protocol::BleProtocolHandler;
use crate::crypto::Nonce;
use crate::crypto::proof::ProofManager;
use crate::shared::constants::{MAX_ATTEMPTS, MAX_PACKET_SIZE};
use crate::shared::{BleError, CryptoError};
use crate::storage::nonce_manager::NonceManager;
use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, Level, Output};
use esp_hal::ledc::LowSpeed;
use esp_hal::ledc::channel::{Channel as PwmChannel, ChannelIFace};
use esp_hal::ledc::timer::Timer as PwmTimer;
use esp_hal::rmt::{Channel, Rmt, Tx, TxChannelConfig, TxChannelCreator};
use esp_hal::rng::Trng;
use esp_println::println;

#[allow(dead_code)]
pub enum UnlockMethod<'a> {
    Relay(Output<'a>),
    Remote {
        channel: Channel<'a, Blocking, Tx>,
        addr: u8,
        cmd: u8,
    },
    Servo {
        channel: PwmChannel<'a, LowSpeed>,
        timer: PwmTimer<'a, LowSpeed>,
    },
}

impl<'a> UnlockMethod<'a> {
    #[allow(unused)]
    pub fn is_relay(&self) -> bool {
        matches!(self, UnlockMethod::Relay(_))
    }

    #[allow(unused)]
    pub fn relay(output: Output<'a>) -> Self {
        UnlockMethod::Relay(output)
    }

    #[allow(unused)]
    pub fn remote(
        rmt: Rmt<'a, Blocking>,
        output: Output<'a>,
        addr: u8,
        packet: u8,
    ) -> Result<Self, esp_hal::rmt::Error> {
        let rmt_channel: Channel<Blocking, Tx> = rmt
            .channel0
            .configure_tx(output, TxChannelConfig::default())?;
        Ok(UnlockMethod::Remote {
            channel: rmt_channel,
            addr,
            cmd: packet,
        })
    }

    #[allow(unused)]
    pub fn servo(channel: PwmChannel<'a, LowSpeed>, timer: PwmTimer<'a, LowSpeed>) -> Self {
        UnlockMethod::Servo { channel, timer }
    }
}

pub struct BeaconState<'a> {
    pub button: Input<'a>,
    pub human_sensor: Input<'a>,
    pub unlock_method: UnlockMethod<'a>,
    pub open: Output<'a>,
    pub unlock_attempts: u8,
    pub nonce_manager: NonceManager<32>,
    pub buffer: BleProtocolHandler,
    pub proof_manager: ProofManager,
    pub last_open: u64,
    pub last_relay_on: u64,
}

impl<'a> BeaconState<'a> {
    pub fn new(
        private_key: [u8; 32],
        human_sensor: Input<'a>,
        button: Input<'a>,
        unlock_method: UnlockMethod<'a>,
        mut open: Output<'a>,
    ) -> Self {
        open.set_low();
        Self {
            button,
            human_sensor,
            unlock_method,
            open,
            nonce_manager: NonceManager::<32>::new(),
            proof_manager: ProofManager::new(private_key),
            unlock_attempts: 0,
            buffer: BleProtocolHandler::new(),
            last_open: 0,
            last_relay_on: 0,
        }
    }

    pub fn set_server_public_key(&mut self, public_key: [u8; 65]) -> Result<(), ()> {
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

    fn check_relay_is_high(&self) -> bool {
        if let UnlockMethod::Relay(output) = &self.unlock_method {
            output.is_set_high()
        } else {
            false
        }
    }

    fn set_relay_high(&mut self) {
        if let UnlockMethod::Relay(output) = &mut self.unlock_method {
            output.set_high();
        }
    }

    fn set_relay_low(&mut self) {
        if let UnlockMethod::Relay(output) = &mut self.unlock_method {
            output.set_low();
        }
    }

    /// Once open, wait for 5 seconds before closing.
    /// During the window, if human sensor is triggered, keep it open and close it 3 seconds after last trigger.
    /// If human sensor is not triggered, close it after 5 seconds.
    /// This is used for running in a loop.
    pub fn check_executors(&mut self, time: u64) {
        if time % 2000 == 0 && self.open.is_set_high() {
            println!("Checking executors at time: {}", time);
            println!("Button state: {}", self.button.is_low());
            println!("Open state: {}", self.open.is_set_high());
            println!("Human sensor state: {}", self.human_sensor.is_high());
            println!("Last open time: {}", self.last_open);
            println!("Last relay on time: {}", self.last_relay_on);
        }

        if self.open.is_set_high() {
            if self.human_sensor.is_high() {
                match &mut self.unlock_method {
                    UnlockMethod::Relay(rel) => {
                        if rel.is_set_low() {
                            self.set_relay_high();
                            self.last_relay_on = time;
                        }
                    }
                    UnlockMethod::Remote { .. } => {
                        // Send the RMT signal to open the gate
                        // example packet
                        // let mut data = [PulseCode::new(Level::High, 200, Level::Low, 50); 20];
                        // data[data.len() - 2] = PulseCode::new(Level::High, 3000, Level::Low, 500);
                        // data[data.len() - 1] = PulseCode::end_marker();
                        // let transaction = channel.transmit(&data).expect("Failed to transmit RMT data");
                        // *channel = transaction.wait().expect("Failed to complete RMT transmission");
                        todo!("Ownership problem")
                    }
                    UnlockMethod::Servo { channel, .. } => {
                        channel.set_duty(10).ok();
                        Delay::new().delay_millis(50u32);
                        channel.set_duty(0).ok();
                    }
                }
            }
            if time - self.last_open > 10_000 {
                self.open.set_low();
            }
        }

        if self.check_relay_is_high() && time - self.last_relay_on > 5_000 {
            self.set_relay_low();
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
            .check_and_mark_nonce(Nonce::from(proof.nonce), proof.timestamp)
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

    pub fn generate_nonce(&mut self, rng: &mut Trng) -> Nonce {
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
}
