#![allow(unused)]
use embassy_stm32::{
    gpio::{Output, OutputType},
    peripherals::TIM5,
    timer::{Channel, input_capture::InputCapture},
};
use embassy_time::Timer;

pub struct Ultrasonic<'a> {
    trigger: Output<'a>,
    echo: InputCapture<'a, TIM5>,
}

#[derive(Debug, defmt::Format)]
pub enum UltrasonicError {
    Timeout,
    OutOfRange,
    NoValidSamples,
}

impl<'a> Ultrasonic<'a> {
    pub fn new(trigger: Output<'a>, echo: InputCapture<'a, TIM5>) -> Self {
        Self { trigger, echo }
    }

    // Additional methods for ultrasonic functionality can be added here

    pub async fn measure(&mut self) -> Result<u32, UltrasonicError> {
        self.trigger.set_high();
        // wait for 10 microseconds
        Timer::after(embassy_time::Duration::from_micros(10)).await;
        self.trigger.set_low();

        let rising = embassy_time::with_timeout(
            embassy_time::Duration::from_millis(100),
            self.echo.wait_for_rising_edge(Channel::Ch3),
        )
        .await
        .map_err(|_| UltrasonicError::Timeout)?;

        let falling = embassy_time::with_timeout(
            embassy_time::Duration::from_millis(100),
            self.echo.wait_for_falling_edge(Channel::Ch3),
        )
        .await
        .map_err(|_| UltrasonicError::Timeout)?;

        let pulse_duration = falling.wrapping_sub(rising);

        // Convert to distance
        // Distance = (pulse_width * speed_of_sound) / 2
        // Speed of sound = 343 m/s = 0.0343 cm/us
        // Assuming 1MHz timer (1 tick = 1us)
        let distance_cm = (pulse_duration as f32 * 0.0343) / 2.0;

        Ok(distance_cm as u32)
    }

    pub fn init(&mut self) {
        self.echo.enable(Channel::Ch3);
    }

    pub fn deinit(&mut self) {
        self.echo.disable(Channel::Ch3);
    }
}
