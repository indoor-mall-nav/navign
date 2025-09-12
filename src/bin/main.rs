//! BLE Example
//!
//! - starts Bluetooth advertising
//! - offers one service with three characteristics (one is read/write, one is write only, one is
//!   read/write/notify)
//! - pressing the boot-button on a dev-board will send a notification if it is subscribed

//% FEATURES: esp-wifi esp-wifi/ble esp-hal/unstable
//% CHIPS: esp32 esp32s3 esp32c2 esp32c3 esp32c6 esp32h2

#![no_std]
#![no_main]

pub(crate) mod ble;
pub(crate) mod crypto;
pub(crate) mod execute;
pub(crate) mod shared;
pub(crate) mod storage;

use crate::execute::BeaconState;
use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::{AttributeServer, NotificationData, WorkResult},
    gatt, Ble, HciConnector,
};
use esp_alloc as _;
use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::gpio::Level;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Pull},
    gpio::{Output, OutputConfig},
    main,
    rng::Rng,
    time,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_wifi::{ble::controller::BleConnector, init};
use crate::ble::BleMessage;
use crate::ble::protocol::BleProtocolHandler;
use crate::shared::constants::{NONCE_REQUEST_LENGTH, PROOF_SUBMISSION_LENGTH};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    let trigger = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());

    let human_body = Input::new(peripherals.GPIO6, InputConfig::default());

    let mut executor = BeaconState::new([0u8; 32], human_body, trigger, led);

    heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);

    let mut rng = Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = init(timg0.timer0, rng).unwrap();

    let config = InputConfig::default().with_pull(Pull::Down);

    let button = Input::new(peripherals.GPIO9, config);

    let mut debounce_cnt = 500;

    let mut bluetooth = peripherals.BT;

    let now = || time::Instant::now().duration_since_epoch().as_millis();
    loop {
        executor.check_executors(now());
        let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth.reborrow());
        let hci = HciConnector::new(connector, now);
        let mut ble = Ble::new(&hci);

        ble.init().unwrap();
        ble.cmd_set_le_advertising_parameters().unwrap();
        ble.cmd_set_le_advertising_data(
            create_advertising_data(&[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                AdStructure::CompleteLocalName("NAVIGN-BEACON"),
            ])
            .unwrap(),
        )
        .unwrap();

        ble.cmd_set_le_advertise_enable(true).unwrap();

        println!("Started advertising.");

        let mut gatt_attributes: &[Attribute] = &[];
        // Here it would be defined in `gatt!` macro, but we need to inform the lsp to recognize them.
        let nonce_characteristic_notify_enable_handle = 0x00u16;
        let nonce_characteristic_handle = 0x00u16;
        let proof_characteristic_handle = 0x00u16;
        let unlock_characteristic_handle = 0x00u16;
        let mut protocol = BleProtocolHandler::new();

        gatt!([service {
            uuid: "ab1ffeae-127c-422f-8e8d-1590229f67c0",
            characteristics: [
                characteristic {
                    name: "nonce_characteristic",
                    uuid: "49e595a0-3e9a-4831-8a3d-c63818783144",
                    notify: true,
                    // read: rf_nonce_response,
                    // write: wf_nonce_request,
                },
                characteristic {
                    name: "proof_characteristic",
                    uuid: "9f3e943e-153e-441e-9d5e-3f0da83edc6f",
                    notify: false,
                    // write: wf_proof_submission,
                },
                characteristic {
                    name: "unlock_characteristic",
                    uuid: "d2b0f2e4-3c3a-4e5f-8e1d-7f4b6c8e9a0b",
                    notify: false,
                    // read: rf_unlock_result,
                },
            ],
        },]);

        let mut no_rng = bleps::no_rng::NoRng;
        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut no_rng);

        loop {
            executor.check_executors(now());

            // Handle nonce requests
            let mut nonce_request = [0u8; NONCE_REQUEST_LENGTH];
            if let Some(1) = srv.get_characteristic_value(nonce_characteristic_handle, 0, &mut nonce_request) {
                let message = protocol.deserialize_message(&nonce_request).ok();
                if let Some(ble::protocol::BleMessage::NonceRequest) = message {
                    let nonce = executor.generate_nonce(&mut rng);
                    let response = ble::protocol::BleMessage::NonceResponse(nonce);
                    let result = protocol.serialize_message(&response).ok();
                    if let Some(data) = result {
                        let notification = NotificationData::new(nonce_characteristic_handle, &data);
                        match srv.do_work_with_notification(Some(notification)) {
                            Ok(_) => {}
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                }
            }

            let mut proof_submission = [0u8; PROOF_SUBMISSION_LENGTH];
            if let Some(1) = srv.get_characteristic_value(proof_characteristic_handle, 0, &mut proof_submission) {
                let message = protocol.deserialize_message(&proof_submission).ok();
                if let Some(ble::protocol::BleMessage::ProofSubmission(proof)) = message {
                    let current_timestamp = now();
                    let unlock_result = match executor.validate_proof(&proof, current_timestamp) {
                        Ok(_) => {
                            executor.open.set_high();
                            (true, None)
                        }
                        Err(e) => (false, Some(e)),
                    };
                    let response = ble::protocol::BleMessage::UnlockResult(unlock_result.0, unlock_result.1);
                    let result = protocol.serialize_message(&response).ok();
                    if let Some(data) = result {
                        let notification = NotificationData::new(proof_characteristic_handle, &data);
                        match srv.do_work_with_notification(Some(notification)) {
                            Ok(_) => {}
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                }
            }
        }
    }
}
