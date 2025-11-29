#![allow(unused)]

use embassy_stm32::peripherals::TIM2;
use embassy_stm32::timer::{
    Channel,
    simple_pwm::{PwmPin, SimplePwm},
};

pub struct Ptz<'a> {
    pwm: SimplePwm<'a, TIM2>,
}

impl<'a> Ptz<'a> {
    pub fn new(pwm: SimplePwm<'a, TIM2>) -> Self {
        Self { pwm }
    }

    pub fn set_angle(&mut self, angle: f32, channel: u8) {
        // Assuming angle is between 0 and 180 degrees
        let chan = match channel {
            1 => Channel::Ch1,
            2 => Channel::Ch2,
            _ => return, // Invalid channel
        };
        let duty_cycle =
            ((angle / 180.0) * (self.pwm.channel(chan).max_duty_cycle() as f32)) as u16;
        self.pwm.channel(chan).set_duty_cycle(duty_cycle);
    }

    pub fn init(&mut self) {
        self.pwm.channel(Channel::Ch1).enable();
        self.pwm.channel(Channel::Ch2).enable();
    }

    pub fn deinit(&mut self) {
        self.pwm.channel(Channel::Ch1).set_duty_cycle(0);
        self.pwm.channel(Channel::Ch2).set_duty_cycle(0);
        self.pwm.channel(Channel::Ch1).disable();
        self.pwm.channel(Channel::Ch2).disable();
    }

    pub fn reset(&mut self) {
        self.pwm.channel(Channel::Ch1).set_duty_cycle(0);
        self.pwm.channel(Channel::Ch2).set_duty_cycle(0);
    }
}
