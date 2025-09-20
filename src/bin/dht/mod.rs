use esp_hal::delay::Delay;
use esp_hal::gpio::interconnect::PeripheralSignal;
use esp_hal::gpio::Flex;
use esp_println::println;

#[derive(Debug, Clone, Copy)]
pub struct Dht11Data {
    pub temperature: f32,
    pub humidity: f32,
}

pub enum Dht11Error {
    Timeout,
    ChecksumMismatch,
    PinError,
}

impl Dht11Data {
    pub fn new(temperature: f32, humidity: f32) -> Self {
        Self {
            temperature,
            humidity,
        }
    }
}

impl Default for Dht11Data {
    fn default() -> Self {
        Self {
            temperature: -1.0,
            humidity: -1.0,
        }
    }
}

pub struct DhtReader<'a> {
    pin: Flex<'a>,
}

impl<'a> DhtReader<'a> {
    pub fn new(pin: Flex<'a>) -> Self {
        Self { pin }
    }

    pub fn read(&mut self) -> Result<Dht11Data, Dht11Error> {
        let mut delay = Delay::new();

        println!("Reading DHT11 Data...");

        // Start signal
        self.pin.set_input_enable(false);
        self.pin.set_output_enable(true);
        self.pin.set_low();
        delay.delay_millis(18u32);
        self.pin.set_high();
        delay.delay_micros(20u32);

        println!("Start signal sent.");

        // Switch to input
        self.pin.set_input_enable(true);
        self.pin.set_output_enable(false);
        delay.delay_micros(40u32);

        println!("Switched to input mode.");

        // Wait for response
        self.wait_for_level(false, 100)?;
        self.wait_for_level(true, 100)?;
        self.wait_for_level(false, 100)?;

        println!("Response received, reading data...");

        let mut data: [u8; 5] = [0; 5];

        for i in 0..40 {
            self.wait_for_level(true, 100)?;
            let mut counter = 0u32;
            while self.pin.is_high() && counter < 100 {
                // Small delay
                for _ in 0..10 {
                    unsafe { core::arch::asm!("nop") };
                }
                counter += 1;
            }
            if counter >= 100 {
                return Err(Dht11Error::Timeout);
            }
            // If high for more than ~40us, it's a 1
            if counter > 4 {
                data[i / 8] |= 1 << (7 - (i % 8));
            }
            self.wait_for_level(false, 100)?;
        }

        let checksum = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);
        if checksum != data[4] {
            return Err(Dht11Error::ChecksumMismatch);
        }

        let humidity = data[0] as f32 + data[1] as f32 * 0.1;
        let temperature = data[2] as f32 + data[3] as f32 * 0.1;

        Ok(Dht11Data::new(temperature, humidity))
    }

    fn wait_for_level(&mut self, level: bool, timeout: u32) -> Result<(), Dht11Error> {
        let mut counter = 0u32;
        while (self.pin.is_high() != level) && counter < timeout {
            // Small delay
            for _ in 0..10 {
                unsafe { core::arch::asm!("nop") };
            }
            counter += 1;
        }

        if counter >= timeout {
            Err(Dht11Error::Timeout)
        } else {
            Ok(())
        }
    }
}
