#![allow(unused)]
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::TIM8;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::complementary_pwm::IdlePolarity;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::{
    complementary_pwm::{ComplementaryPwm, ComplementaryPwmPin},
    simple_pwm::PwmPin,
};

pub struct MotorControl<'a> {
    pwm: ComplementaryPwm<'a, TIM8>,

    inputs: [(Output<'a>, Output<'a>); 4],

    stbys: [Output<'a>; 2],
}

impl<'a> MotorControl<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pwm: ComplementaryPwm<'a, TIM8>,
        ain1: Output<'a>,
        ain2: Output<'a>,
        bin1: Output<'a>,
        bin2: Output<'a>,
        cin1: Output<'a>,
        cin2: Output<'a>,
        din1: Output<'a>,
        din2: Output<'a>,
        stby1: Output<'a>,
        stby2: Output<'a>,
    ) -> Self {
        Self {
            pwm,
            inputs: [(ain1, ain2), (bin1, bin2), (cin1, cin2), (din1, din2)],
            stbys: [stby1, stby2],
        }
    }
}

impl<'a> MotorControl<'a> {
    pub fn set_run(&mut self, motor: u8, dir: Option<bool>) {
        if motor >= 4 {
            return;
        }
        let Some(motor_instance) = self.inputs.get_mut(motor as usize) else {
            return;
        };
        match dir {
            Some(true) => {
                motor_instance.0.set_high();
                motor_instance.1.set_low();
            }
            Some(false) => {
                motor_instance.0.set_low();
                motor_instance.1.set_high();
            }
            None => {
                motor_instance.0.set_high();
                motor_instance.1.set_high();
            }
        }
    }

    pub fn set_straight(&mut self, duty_cycle: u16, forward: bool) {
        self.pwm
            .set_output_idle_state(&[Channel::Ch2, Channel::Ch3], IdlePolarity::OisActive);
        self.pwm.set_duty(Channel::Ch2, duty_cycle);
        self.pwm.set_duty(Channel::Ch3, duty_cycle);
        self.stbys.iter_mut().for_each(|s| s.set_high());
        self.set_run(0, Some(forward));
        self.set_run(1, Some(forward));
        self.set_run(2, Some(forward));
        self.set_run(3, Some(forward));
    }

    pub fn set_stop(&mut self) {
        self.pwm
            .set_output_idle_state(&[Channel::Ch2, Channel::Ch3], IdlePolarity::OisnActive);
        self.pwm.set_duty(Channel::Ch2, 0);
        self.pwm.set_duty(Channel::Ch3, 0);
        self.stbys.iter_mut().for_each(|s| s.set_low());
        for motor in 0..4 {
            self.set_run(motor, None);
        }
    }

    pub fn set_turn(&mut self, duty_cycle: u16, left: bool) {
        self.pwm
            .set_output_idle_state(&[Channel::Ch2, Channel::Ch3], IdlePolarity::OisActive);
        self.pwm.set_duty(Channel::Ch2, duty_cycle);
        self.pwm.set_duty(Channel::Ch3, duty_cycle);
        self.stbys.iter_mut().for_each(|s| s.set_high());
        if left {
            self.set_run(0, Some(false));
            self.set_run(1, Some(false));
            self.set_run(2, Some(true));
            self.set_run(3, Some(true));
        } else {
            self.set_run(0, Some(true));
            self.set_run(1, Some(true));
            self.set_run(2, Some(false));
            self.set_run(3, Some(false));
        }
    }

    pub fn set_terminate(&mut self) {
        self.pwm
            .set_output_idle_state(&[Channel::Ch2, Channel::Ch3], IdlePolarity::OisnActive);
        self.pwm.set_duty(Channel::Ch2, 0);
        self.pwm.set_duty(Channel::Ch3, 0);
        self.stbys.iter_mut().for_each(|s| s.set_low());
        for motor in 0..4 {
            self.inputs[motor as usize].0.set_low();
            self.inputs[motor as usize].1.set_low();
        }
    }

    pub fn init(&mut self) {
        self.pwm.enable(Channel::Ch2);
        self.pwm.enable(Channel::Ch3);

        self.set_straight(0, true);
    }

    pub fn deinit(&mut self) {
        self.set_terminate();

        self.pwm.disable(Channel::Ch2);
        self.pwm.disable(Channel::Ch3);
    }
}
