use esp_hal::gpio::{Input, Level, Output};

#[derive(Debug)]
pub struct ExecuteBuffer<'a> {
    pub human_sensor: Input<'a>,
    pub relay: Output<'a>,
    pub open: Output<'a>,
    pub last_open: u64,
    pub last_relay_on: u64,
}

impl<'a> ExecuteBuffer<'a> {
    pub fn new(human_sensor: Input<'a>, relay: Output<'a>, open: Output<'a>) -> Self {
        Self {
            human_sensor,
            relay,
            open,
            last_open: 0,
            last_relay_on: 0,
        }
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
    pub fn execute(&mut self, time: u64) {
        if self.open.is_set_high() {
            if self.human_sensor.is_high() {
                self.last_open = time;
            }
            if time - self.last_open > 5000 {
                self.open.set_low();
            }
        }

        if self.relay.is_set_high() && time - self.last_relay_on > 1000 {
            self.relay.set_low();
        }

        if self.open.is_set_high() && self.relay.is_set_high() {
            // This is an illegal case, and we need to turn off the relay immediately.
            self.relay.set_low();
        }
    }
}
