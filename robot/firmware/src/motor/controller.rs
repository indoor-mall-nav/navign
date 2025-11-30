#![allow(unused)]
use defmt::info;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals::TIM8;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::complementary_pwm::IdlePolarity;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::simple_pwm::PwmPin;
use embassy_stm32::timer::simple_pwm::SimplePwm;

pub struct MotorControl<'a> {
    pwm: SimplePwm<'a, TIM8>,

    inputs: [(Output<'a>, Output<'a>); 4],

    stbys: [Output<'a>; 2],
}

impl<'a> MotorControl<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pwm: SimplePwm<'a, TIM8>,
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

    pub fn set_move(&mut self, duty_cycle_l: u16, duty_cycle_r: u16, forw_l: bool, forw_r: bool) {
        self.stbys.iter_mut().for_each(|s| s.set_high());

        let max_duty = self.pwm.max_duty_cycle();

        info!("Max duty: {}", max_duty);

        let duty_cycle_l = duty_cycle_l.min(max_duty);
        let duty_cycle_r = duty_cycle_r.min(max_duty);

        self.pwm.ch1().set_duty_cycle(duty_cycle_l);
        self.pwm.ch2().set_duty_cycle(duty_cycle_r);
        self.pwm.ch3().set_duty_cycle(duty_cycle_l);
        self.pwm.ch4().set_duty_cycle(duty_cycle_r);

        self.set_run(0, Some(forw_l));
        self.set_run(1, Some(forw_r));
        self.set_run(2, Some(forw_l));
        self.set_run(3, Some(forw_r));
    }

    pub fn set_terminate(&mut self) {
        let channels = [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4];
        channels
            .iter()
            .for_each(|ch| self.pwm.channel(*ch).set_duty_cycle(0));
        self.stbys.iter_mut().for_each(|s| s.set_low());
        for motor in 0..4 {
            self.inputs[motor as usize].0.set_low();
            self.inputs[motor as usize].1.set_low();
        }
    }

    pub fn init(&mut self) {
        let channels = [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4];
        channels
            .iter()
            .for_each(|ch| self.pwm.channel(*ch).enable());

        self.set_move(0, 0, true, true);
    }

    pub fn deinit(&mut self) {
        self.set_terminate();

        let channels = [Channel::Ch1, Channel::Ch2, Channel::Ch3, Channel::Ch4];
        channels
            .iter()
            .for_each(|ch| self.pwm.channel(*ch).disable());
    }
}
