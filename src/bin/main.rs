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

mod ble;
mod crypto;
mod execute;

use crate::execute::ExecuteBuffer;
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
    chip,
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
use ble::constants::{SERVICE_UUID, NONCE_CHAR_UUID, PROOF_CHAR_UUID, UNLOCK_CHAR_UUID};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let mut peripherals = esp_hal::init(config);

    let led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    let trigger = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());

    let human_body = Input::new(peripherals.GPIO6, InputConfig::default());

    let mut executor = ExecuteBuffer::new(human_body, led, trigger);

    heap_allocator!(size: 72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let rtc = esp_hal::rtc_cntl::Rtc::new(peripherals.LPWR);

    let rng = Rng::new(peripherals.RNG);

    let esp_wifi_ctrl = init(timg0.timer0, rng).unwrap();

    let config = InputConfig::default().with_pull(Pull::Down);

    let button = Input::new(peripherals.GPIO9, config);

    let mut debounce_cnt = 500;

    let mut bluetooth = peripherals.BT;

    let now = || time::Instant::now().duration_since_epoch().as_millis();
    loop {
        executor.execute(now());
        let connector = BleConnector::new(&esp_wifi_ctrl, bluetooth.reborrow());
        let hci = HciConnector::new(connector, now);
        let mut ble = Ble::new(&hci);

        ble.init().unwrap_or_else(|e| {
            println!("Failed to init BLE: {:?}", e);
        });
        ble.cmd_set_le_advertising_parameters().unwrap_or_else(|e| {
            println!("Failed to set advertising parameters: {:?}", e);
        });
        ble.cmd_set_le_advertising_data(
            create_advertising_data(&[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
                AdStructure::CompleteLocalName("BEACON-02-000"),
            ])
            .unwrap(),
        ).unwrap_or_else(|e| {
            println!("Failed to set advertising data: {:?}", e);
        });

        ble.cmd_set_le_advertise_enable(true).unwrap_or_else(|e| {
            println!("Failed to start advertising: {:?}", e);
        });

        println!("Started advertising.");

        let mut rf = |_offset: usize, data: &mut [u8]| {
            data[..36].copy_from_slice(&b"Hello Bare-Metal BLE"[..]);
            17
        };
        let mut wf = |offset: usize, data: &[u8]| {
            println!("RECEIVED: {} {:?}", offset, data);
        };

        let mut wf2 = |offset: usize, data: &[u8]| {
            println!("RECEIVED: {} {:?}", offset, data);
        };

        let mut rf3 = |_offset: usize, data: &mut [u8]| {
            data[..5].copy_from_slice(&b"Hola!"[..]);
            5
        };
        let mut wf3 = |offset: usize, data: &[u8]| {
            println!("RECEIVED: Offset {}, data {:?}", offset, data);
        };

        gatt!([service {
            uuid: "ab1ffeae-127c-422f-8e8d-1590229f67c0",
            characteristics: [
                characteristic {
                    name: "nonce_characteristic",
                    uuid: "49e595a0-3e9a-4831-8a3d-c63818783144",
                    notify: true,
                    read: rf,
                    write: wf,
                },
                characteristic {
                    name: "proof_characteristic",
                    uuid: "9f3e943e-153e-441e-9d5e-3f0da83edc6f",
                    notify: false,
                    write: wf2,
                },
                characteristic {
                    name: "unlock_characteristic",
                    uuid: "d2b0f2e4-3c3a-4e5f-8e1d-7f4b6c8e9a0b",
                    notify: false,
                    read: rf3,
                    write: wf3,
                },
            ],
        },]);

        let mut rng = bleps::no_rng::NoRng;
        let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut rng);

        loop {
            executor.execute(now());
            let mut notification = None;

            if button.is_low() && debounce_cnt > 0 {
                debounce_cnt -= 1;
                if debounce_cnt == 0 {
                    let mut cccd = [0u8; 1];
                    if let Some(1) = srv.get_characteristic_value(
                        nonce_characteristic_notify_enable_handle,
                        0,
                        &mut cccd,
                    ) {
                        // if notifications enabled
                        if cccd[0] == 1 {
                            notification = Some(NotificationData::new(
                                nonce_characteristic_handle,
                                &b"Notification"[..],
                            ));
                        }
                    }
                }
            };

            if button.is_high() {
                debounce_cnt = 500;
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
