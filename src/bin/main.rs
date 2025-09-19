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
mod internet;
pub(crate) mod shared;
pub(crate) mod storage;
mod dht;

use alloc::rc::Rc;
use core::cell::RefCell;
use crate::ble::BleMessage;
use crate::execute::BeaconState;
use crate::shared::constants::*;
use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::{AttributeServer, NotificationData},
    gatt, Ble, HciConnector,
    attribute::Attribute
};
use bleps::attribute_server::WorkResult;
use esp_alloc as _;
use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::gpio::{Flex, Level};
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
use embedded_dht_rs::dht11::Dht11;
use esp_hal::delay::Delay;
use heapless::Vec;
use crate::ble::protocol::BleProtocolHandler;
use crate::dht::DhtReader;
use crate::shared::{DeviceCapability, DeviceType};
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

    let dht11 = Flex::new(peripherals.GPIO4);

    heap_allocator!(size: 192 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);

    let mut rng = Rng::new(peripherals.RNG);

    let private_key = Efuse::read_field_le::<[u8; 32]>(BLOCK_KEY0);

    let mut executor = BeaconState::new(private_key, human_body, trigger, led, rng);

    let delay = Delay::new();

    let mut dht = Dht11::new(dht11, delay);
    
    let mut executor = Rc::new(RefCell::new(executor));

    let esp_wifi_ctrl = init(timg0.timer0, rng).unwrap();

    let config = InputConfig::default().with_pull(Pull::Down);

    let button = Input::new(peripherals.GPIO9, config);

    let debounce_cnt = 500;

    let device_id = b"68a84b6ebdfa76608b934b0a";
    let device_type = DeviceType::Merchant;
    let mut capabilities = Vec::<DeviceCapability, 3>::new();
    capabilities.push(DeviceCapability::UnlockGate).unwrap();

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
        Rc::clone(&executor).borrow_mut().check_executors(now());
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
        let unlock_service_notify_enable_handle = 0x00u16;
        let unlock_service_handle = 0x00u16;

        let mut wf = |offset: usize, data: &[u8]| {
            println!("Write at offset {}: {:x?}", offset, data);
            Rc::clone(&executor).borrow_mut().buffer.store_message(data).ok();
        };

        let mut rf = |offset: usize, buffer: &mut [u8]| -> usize {
            let target = Rc::clone(&executor).borrow().buffer.extract_message();
            let length = match target[0] {
                DEVICE_RESPONSE => DEVICE_RESPONSE_LENGTH,
                NONCE_RESPONSE => NONCE_RESPONSE_LENGTH,
                UNLOCK_RESPONSE => UNLOCK_RESPONSE_LENGTH,
                _ => 0
            };
            buffer[..length].copy_from_slice(&target[..length]);
            println!("Read at offset {}: {:x?}", offset, &buffer[..length]);
            length
        };

        gatt!([service {
            uuid: "134b1d88-cd91-8134-3e94-5c4052743845",
            characteristics: [
                characteristic {
                    name: "unlock_service",
                    description: "Unlock Service",
                    uuid: "99d92823-9e38-72ff-6cf1-d2d593316af8",
                    notify: true,
                    read: rf,
                    write: wf,
                },
            ],
        },]);

        println!("GATT service registered.");

        let mut no_rng = bleps::no_rng::NoRng;
        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut no_rng);

        loop {
            if now() % 50_000 == 5_000 {
                println!("Reading DHT11 Data...");
                match dht.read().map(|res| {
                    println!("Temperature: {}°C, Humidity: {}%", res.temperature, res.humidity);
                }) {
                    Ok(_) => {},
                    Err(e) => println!("Failed to read DHT11 data: {:?}", e),
                }
            }
            let mut instance = Rc::clone(&executor);
            instance.borrow_mut().check_executors(now());

            let mut notification = None;
            let mut receive_buffer = [0u8; MAX_PACKET_SIZE];
            let mut send_buffer = [0u8; MAX_PACKET_SIZE];

            if let Some(1) =
                srv.get_characteristic_value(unlock_service_notify_enable_handle, 0, &mut receive_buffer)
            {
                println!("Device request notify enabled: {:x?}", receive_buffer);
                if receive_buffer[0] != 0x00 {
                    println!("Device request received raw: {:x?}", receive_buffer);
                }
                let message = instance.borrow_mut().deserialize_message(None).ok();
                println!("Request received: {:?}", message);
                let response: Option<BleMessage> = match message {
                    Some(BleMessage::DeviceRequest) => {
                        let result = (device_type, capabilities.clone(), device_id.clone());
                        Some(result.into())
                    }
                    Some(BleMessage::NonceRequest) => {
                        let nonce = instance.borrow_mut().generate_nonce(&mut rng);
                        Some(nonce.into())
                    }
                    Some(BleMessage::UnlockRequest(ref proof)) => {
                        let unlock_result = match instance.borrow_mut().validate_proof(&proof, now()) {
                            Ok(_) => {
                                instance.borrow_mut().set_open(true, now());
                                (true, None)
                            }
                            Err(e) => (false, Some(e)),
                        };
                        Some(unlock_result.into())
                    }
                    _ => None
                };
                if let Some(resp) = response {
                    let result = instance.borrow_mut().serialize_message(&resp).ok();
                    if let Some(data) = result {
                        send_buffer.fill(0);
                        send_buffer[..data.len()].copy_from_slice(&data);
                        notification = Some(NotificationData::new(unlock_service_handle, &send_buffer));
                    }
                }
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
        }
    }
}
