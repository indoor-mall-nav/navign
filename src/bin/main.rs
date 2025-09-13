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
mod internet;
pub(crate) mod shared;
pub(crate) mod storage;

use crate::ble::BleMessage;
use crate::execute::BeaconState;
use crate::shared::constants::{NONCE_REQUEST_LENGTH, NONCE_RESPONSE_LENGTH, PROOF_SUBMISSION_LENGTH, UNLOCK_RESULT_LENGTH};
use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::{AttributeServer, NotificationData},
    gatt, Ble, HciConnector,
    attribute::Attribute
};
use esp_alloc as _;
use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::gpio::Level;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Pull},
    gpio::{Output, OutputConfig},
    main,
    rng::Rng,
    time,
    timer::timg::TimerGroup,
};
use esp_hal::efuse::{Efuse, BLOCK_KEY0};
use esp_println::println;
use esp_wifi::wifi::{AuthMethod, Configuration};
use esp_wifi::{ble::controller::BleConnector, init};
use smoltcp::iface::{SocketSet, SocketStorage};
use blocking_network_stack::Stack;
// use reqwless::client::

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    let trigger = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());

    let human_body = Input::new(peripherals.GPIO6, InputConfig::default());

    heap_allocator!(size: 128 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);

    let mut rng = Rng::new(peripherals.RNG);

    let private_key = Efuse::read_field_le::<[u8; 32]>(BLOCK_KEY0);

    let mut executor = BeaconState::new(private_key, human_body, trigger, led, rng.clone());

    let esp_wifi_ctrl = init(timg0.timer0, rng).unwrap();

    let config = InputConfig::default().with_pull(Pull::Down);

    let button = Input::new(peripherals.GPIO9, config);

    let debounce_cnt = 500;

    let device_id = b"68a47c2a7f3f39855509523f";

    let mut bluetooth = peripherals.BT;

    let now = || time::Instant::now().duration_since_epoch().as_millis();

    let (mut wifi_controller, interfaces) =
        esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();
    let mut device = interfaces.sta;
    let iface = smoltcp::iface::Interface::new(
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress::from_bytes(&device.mac_address()),
        )),
        &mut device,
        smoltcp::time::Instant::from_micros(
            time::Instant::now().duration_since_epoch().as_micros() as i64,
        ),
    );

    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();

    let wifi_config = Configuration::Client(esp_wifi::wifi::ClientConfiguration {
        ssid: "ssid".into(),
        password: "password".into(),
        auth_method: AuthMethod::WPAWPA2Personal,
        ..Default::default()
    });

    let wifi_res = wifi_controller.set_configuration(&wifi_config).ok();

    if let Err(e) = wifi_controller.start() {
        println!("Failed to start WiFi: {:?}", e);
    } else {
        println!("WiFi started");
    }

    wifi_controller.connect().ok();

    let stack = Stack::new(iface, device, socket_set, now, rng.random());

    #[allow(clippy::never_loop)]
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

        #[allow(unused_mut)]
        let mut gatt_attributes: &[Attribute] = &[];
        // Here it would be defined in `gatt!` macro, but we need to inform the lsp to recognize them.
        let nonce_characteristic_notify_enable_handle = 0x00u16;
        let nonce_characteristic_handle = 0x00u16;
        let proof_characteristic_handle = 0x00u16;
        let unlock_characteristic_handle = 0x00u16;

        let mut wf_nonce_request = |offset: usize, data: &[u8]| {};

        let mut rf_nonce_response = |offset: usize, buffer: &mut [u8]| -> usize {
            // Check the length of the buffer
            if buffer.len() != NONCE_RESPONSE_LENGTH {
                0
            } else {
                NONCE_RESPONSE_LENGTH
            }
        };

        let mut wf_unlock_request = |offset: usize, data: &[u8]| {};

        let mut rf_unlock_response = |offset: usize, buffer: &mut [u8]| -> usize {
            // Check the length of the buffer
            if buffer.len() != UNLOCK_RESULT_LENGTH {
                0
            } else {
                UNLOCK_RESULT_LENGTH
            }
        };

        gatt!([service {
            uuid: "134b1d88-cd91-8134-3e94-5c4052743845",
            characteristics: [
                characteristic {
                    name: "device_characteristic",
                    uuid: "99d92823-9e38-72ff-6cf1-d2d593316af8",
                    notify: true,
                    value: device_id,
                },
                characteristic {
                    name: "nonce_characteristic",
                    uuid: "49e595a0-3e9a-4831-8a3d-c63818783144",
                    notify: true,
                    read: rf_nonce_response,
                    write: wf_nonce_request,
                },
                characteristic {
                    name: "unlock_characteristic",
                    uuid: "d2b0f2e4-6c3a-4e5f-8e1d-7f4b6c8e9a0b",
                    notify: true,
                    read: rf_unlock_response,
                    write: wf_unlock_request,
                },
            ],
        },]);

        println!("GATT service registered.");

        let mut no_rng = bleps::no_rng::NoRng;
        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut no_rng);

        loop {
            executor.check_executors(now());
            // Handle nonce requests
            let mut nonce_request = [0u8; NONCE_REQUEST_LENGTH];
            if let Some(1) =
                srv.get_characteristic_value(nonce_characteristic_handle, 0, &mut nonce_request)
            {
                let message = executor.deserialize_message(&nonce_request).ok();
                if let Some(BleMessage::NonceRequest) = message {
                    let challenge = executor.generate_nonce(&mut rng);
                    let response = BleMessage::NonceResponse(challenge);
                    let result = executor.serialize_message(&response).ok();
                    if let Some(data) = result {
                        let notification = NotificationData::new(nonce_characteristic_handle, data);
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
            if let Some(1) =
                srv.get_characteristic_value(proof_characteristic_handle, 0, &mut proof_submission)
            {
                let message = executor.deserialize_message(&proof_submission).ok();
                if let Some(BleMessage::ProofSubmission(proof)) = message {
                    let current_timestamp = now();
                    let unlock_result = match executor.validate_proof(&proof, current_timestamp) {
                        Ok(_) => {
                            executor.set_open(true, now());
                            (true, None)
                        }
                        Err(e) => (false, Some(e)),
                    };
                    let response =
                        ble::protocol::BleMessage::UnlockResult(unlock_result.0, unlock_result.1);
                    let result = executor.serialize_message(&response).ok();
                    if let Some(data) = result {
                        let notification = NotificationData::new(proof_characteristic_handle, data);
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
