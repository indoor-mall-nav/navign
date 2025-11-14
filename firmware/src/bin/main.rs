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
extern crate alloc;

pub(crate) mod ble;
pub(crate) mod crypto;
pub(crate) mod execute;
pub(crate) mod internet;
pub(crate) mod shared;
pub(crate) mod storage;

use crate::execute::{BeaconState, UnlockMethod};
use crate::shared::constants::*;
use crate::shared::{CryptoError, DeviceCapabilities, DeviceTypes};
use alloc::rc::Rc;
use bleps::att::Uuid;
use bleps::attribute_server::WorkResult;
use bleps::{
    Ble, HciConnector,
    ad_structure::{
        AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE, create_advertising_data,
    },
    attribute_server::{AttributeServer, NotificationData},
    gatt,
};
use core::cell::RefCell;
use embedded_dht_rs::dht11::Dht11;
use esp_alloc::heap_allocator;
use esp_bootloader_esp_idf::ota::OtaImageState;
use esp_hal::delay::Delay;
use esp_hal::efuse::{BLOCK_KEY0, Efuse};
use esp_hal::gpio::{Flex, Level};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::rng::{Trng, TrngSource};
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig},
    gpio::{Output, OutputConfig},
    main, time,
};
use esp_println::println;
use esp_radio::ble::Config;
use esp_radio::{ble::controller::BleConnector, init};
use esp_rtos as _;
use esp_storage::FlashStorage;
use heapless::Vec;
use navign_shared::BleMessage;

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info);
    loop {}
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize pins.
    let dht11 = Flex::new(peripherals.GPIO4);
    let button = Input::new(peripherals.GPIO3, InputConfig::default());
    let relay = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
    let led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    let human_body = Input::new(peripherals.GPIO1, InputConfig::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    heap_allocator!(size: 192 * 1024);

    #[allow(unused)]
    let mut flash = FlashStorage::new(peripherals.FLASH);

    let mut buffer = [0u8; esp_bootloader_esp_idf::partitions::PARTITION_TABLE_MAX_LEN];
    let pt = esp_bootloader_esp_idf::partitions::read_partition_table(&mut flash, &mut buffer)
        .expect("Failed to read partition table - flash may be corrupted");

    // List all partitions - this is just FYI
    for part in pt.iter() {
        println!("{:?}", part);
    }

    println!("Currently booted partition {:?}", pt.booted_partition());

    let mut ota = esp_bootloader_esp_idf::ota_updater::OtaUpdater::new(&mut flash, &mut buffer)
        .expect("Failed to initialize OTA updater - check partition configuration");

    let current = ota
        .selected_partition()
        .expect("Failed to get selected partition - OTA configuration error");
    println!(
        "current image state {:?} (only relevant if the bootloader was built with auto-rollback support)",
        ota.current_ota_state()
    );
    println!("currently selected partition {:?}", current);

    if let Ok(state) = ota.current_ota_state() {
        if matches!(state, OtaImageState::New | OtaImageState::PendingVerify) {
            ota.set_current_ota_state(esp_bootloader_esp_idf::ota::OtaImageState::Valid)
                .ok();
        }
    }

    let server_public_key = [
        4, 247, 145, 243, 155, 54, 15, 43, 52, 88, 198, 230, 245, 57, 127, 80, 180, 157, 227, 135,
        176, 94, 224, 236, 37, 54, 221, 105, 63, 80, 127, 21, 31, 197, 85, 159, 22, 13, 72, 233,
        62, 112, 201, 230, 232, 229, 154, 214, 241, 133, 209, 2, 54, 122, 111, 222, 23, 6, 77, 33,
        104, 142, 37, 110, 136,
    ];

    #[allow(unused)]
    let trng_source = TrngSource::new(peripherals.RNG, peripherals.ADC1);

    let mut rng = Trng::try_new()
        .expect("Failed to initialize TRNG - hardware random number generator unavailable");

    let private_key = Efuse::read_field_le::<[u8; 32]>(BLOCK_KEY0);

    // If private key is not set, panic
    if private_key == [0u8; 32] {
        println!("EFUSE BLOCK_KEY0 is not set. Please set it to a valid 32-byte private key.");
    }

    let method = UnlockMethod::Relay(relay);

    let executor = BeaconState::new(private_key, button, human_body, method, led);

    let delay = Delay::new();

    let mut dht = Dht11::new(dht11, delay);

    let executor = Rc::new(RefCell::new(executor));

    executor
        .borrow_mut()
        .set_server_public_key(server_public_key)
        .expect("Failed to set server public key - invalid key format");

    Delay::new().delay_millis(3_000u32);

    let esp_wifi_ctrl = init()
        .expect("Failed to initialize ESP WiFi/BLE radio - check hardware configuration");

    let device_id = b"68a84b6ebdfa76608b934b0a";
    println!("Device ID: {:?}", device_id);
    let device_type = DeviceTypes::MERCHANT;
    let capabilities = DeviceCapabilities::UNLOCK_GATE;

    let mut uuids = Vec::<Uuid, 4>::new();

    uuids
        .push(Uuid::Uuid16(0x1819))
        .expect("Failed to add Location and Navigation Service UUID - vector capacity exceeded");
    uuids
        .push(Uuid::Uuid16(0x1821))
        .expect("Failed to add Indoor Positioning Service UUID - vector capacity exceeded");

    if capabilities.contains(DeviceCapabilities::UNLOCK_GATE) {
        uuids
            .insert(0, Uuid::Uuid16(0x183D))
            .expect("Failed to add Authorization Control Service UUID - vector capacity exceeded");
    }

    if capabilities.contains(DeviceCapabilities::ENVIRONMENTAL_DATA) {
        uuids
            .push(Uuid::Uuid16(0x181A))
            .expect("Failed to add Environmental Sensing Service UUID - vector capacity exceeded");
    }

    let mut bluetooth = peripherals.BT;

    let now = || time::Instant::now().duration_since_epoch().as_millis();

    #[allow(clippy::never_loop)]
    loop {
        // TODO: Handle the OTA update process here, possibly restarting the device if an update was applied.
        // TODO: Implement BluFi functionality for connecting to Wi-Fi networks over BLE.
        Rc::clone(&executor).borrow_mut().check_executors(now());
        let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth.reborrow(), Config::default())
            .expect("Failed to create BLE connector");
        let hci = HciConnector::new(connector, now);
        let mut ble = Ble::new(&hci);

        ble.init()
            .expect("Failed to initialize BLE stack");
        ble.cmd_set_le_advertising_parameters()
            .expect("Failed to set BLE advertising parameters");
        ble.cmd_set_le_advertising_data(
            create_advertising_data(&[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(uuids.as_ref()),
                AdStructure::CompleteLocalName("NAVIGN-BEACON"),
            ])
            .expect("Failed to create advertising data - check service UUIDs and name length"),
        )
        .expect("Failed to set BLE advertising data");

        ble.cmd_set_le_advertise_enable(true)
            .expect("Failed to enable BLE advertising");

        println!("Started advertising.");

        #[allow(unused_mut)]
        let mut gatt_attributes: &[Attribute] = &[];
        // Here it would be defined in `gatt!` macro, but we need to inform the lsp to recognize them.
        let unlock_service_notify_enable_handle = 0x00u16;
        let unlock_service_handle = 0x00u16;
        println!("Attributes length: {}", gatt_attributes.len());
        println!(
            "unlock_service_notify_enable_handle: {:x}",
            unlock_service_notify_enable_handle
        );
        println!("unlock_service_handle: {:x}", unlock_service_handle);

        let mut wf = |offset: usize, data: &[u8]| {
            println!("Write at offset {}: {:x?}", offset, data);
            Rc::clone(&executor)
                .borrow_mut()
                .buffer
                .store_message(data, offset)
                .ok();
        };

        let mut rf = |offset: usize, buffer: &mut [u8]| -> usize {
            println!("Read request at offset {}", offset);
            let target = Rc::clone(&executor)
                .borrow_mut()
                .buffer
                .extract_message(offset);
            let length = match target[0] {
                DEVICE_RESPONSE => DEVICE_RESPONSE_LENGTH,
                NONCE_RESPONSE => NONCE_RESPONSE_LENGTH,
                UNLOCK_RESPONSE => UNLOCK_RESPONSE_LENGTH,
                _ => 0,
            };
            let terminus = if offset < length {
                length
            } else {
                return 0;
            };
            buffer[..terminus].copy_from_slice(&target[..terminus]);
            println!("Read at offset {}: {:x?}", offset, &buffer[..terminus]);
            terminus
        };

        gatt!([service {
            uuid: "134b1d88-cd91-8134-3e94-5c4052743845",
            characteristics: [characteristic {
                name: "unlock_service",
                description: "Unlock Service",
                uuid: "99d92823-9e38-72ff-6cf1-d2d593316af8",
                notify: true,
                read: rf,
                write: wf,
            },],
        },]);

        println!("GATT service registered.");

        let mut no_rng = bleps::no_rng::NoRng;
        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut no_rng);

        let time_loop_start = now();

        loop {
            if now() % 50_000 == 5_000 {
                println!("Reading DHT11 Data...");
                match dht.read().map(|res| {
                    println!(
                        "Temperature: {}°C, Humidity: {}%",
                        res.temperature, res.humidity
                    );
                }) {
                    Ok(_) => {}
                    Err(e) => println!("Failed to read DHT11 data: {:?}", e),
                }
            }
            let instance = Rc::clone(&executor);
            instance.borrow_mut().check_executors(now());

            let mut notification = None;
            let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
            let mut send_buffer = [0u8; MAX_PACKET_SIZE];

            if srv
                .get_characteristic_value(
                    unlock_service_notify_enable_handle,
                    0,
                    &mut receive_buffer,
                )
                .is_some()
                && instance.borrow().buffer.has_message()
            {
                let message = instance.borrow_mut().deserialize_message(None).ok();
                println!("Request received: {:?}", message);
                println!("Handling message");
                instance.borrow_mut().buffer.processing = true;
                let response: Option<BleMessage> = match message {
                    Some(BleMessage::DeviceRequest) => {
                        Some(BleMessage::DeviceResponse(device_type, capabilities, {
                            let mut id = [0u8; 24];
                            id.copy_from_slice(device_id.as_ref());
                            id
                        }))
                    }
                    Some(BleMessage::NonceRequest) => {
                        let nonce = instance.borrow_mut().generate_nonce(&mut rng);
                        let mut identifier = [0u8; 8];
                        if let Ok(sig) = instance.borrow().proof_manager.sign_data(nonce.as_bytes())
                        {
                            identifier.copy_from_slice(&sig[sig.len() - 8..]);
                        }
                        Some(((*nonce.as_bytes()), identifier).into())
                    }
                    Some(BleMessage::UnlockRequest(ref proof)) => {
                        let mut cell = instance.borrow_mut();
                        let unlock_response = match cell.validate_proof(proof, now()) {
                            Ok(_) => {
                                cell.set_open(true, now());
                                BleMessage::UnlockResponse(true, CryptoError::InvalidSignature)
                            }
                            Err(e) => BleMessage::UnlockResponse(false, e),
                        };
                        Some(unlock_response)
                    }
                    // Debug functionality not supported in shared BleMessage enum
                    // Some(BleMessage::DebugRequest(_)) => {
                    //     let length = rng.random().wrapping_rem(16) + 1;
                    //     let mut data = [0u8; 16];
                    //     for i in 0..length {
                    //         data[i as usize] = rng.random() as u8;
                    //     }
                    //     Some(BleMessage::DebugResponse(data.into()))
                    // }
                    _ => None,
                };
                println!("Response: {:?}", response);
                if let Some(resp) = response {
                    let result = instance.borrow_mut().serialize_message(&resp).ok();
                    println!("Should have response: {:?}", result);
                    if let Some(data) = result {
                        send_buffer.fill(0);
                        send_buffer[..data.len()].copy_from_slice(&data);
                        notification =
                            Some(NotificationData::new(unlock_service_handle, &send_buffer));
                    }
                    instance.borrow_mut().buffer.processing = false;
                    instance.borrow_mut().buffer.clear_receive_buffer();
                }
            }

            if notification.is_some() {
                println!("Notification is {:?}", notification);
            }

            match srv.do_work_with_notification(notification) {
                Ok(res) => {
                    if let WorkResult::GotDisconnected = res {
                        break;
                    }
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            }

            if now() - time_loop_start > 300_000 {
                println!("Restarting advertising to refresh connections.");
                break;
            }
        }
    }
}
