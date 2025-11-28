#![no_std]
#![no_main]

mod motor;

use {defmt_rtt as _, panic_probe as _};

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::i2c::{Config, I2c};
use embassy_stm32::timer::complementary_pwm::ComplementaryPwm;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, OutputType, Speed},
    time::Hertz,
    timer::low_level::CountingMode,
    timer::{complementary_pwm::ComplementaryPwmPin, simple_pwm::PwmPin},
};
use embassy_time::TICK_HZ;
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs {
    I2C1_EV => embassy_stm32::i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => embassy_stm32::i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
});

fn get_stm_config() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    #[cfg(debug_assertions)]
    let dbgmcu = embassy_stm32::pac::DBGMCU;
    dbgmcu.cr().modify(|w| {
        w.set_dbgsleep_d1(true);
        w.set_dbgstby_d1(true);
        w.set_dbgstop_d1(true);
    });
    {
        info!("The tick frequency is {} Hz", TICK_HZ);
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV4);
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(400),
            mode: HseMode::Oscillator,
        });
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV1),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    config
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = get_stm_config();
    info!("Starting up!");
    let p = embassy_stm32::init(config);
    info!("Initialized peripherals.");
    let mut led = Output::new(p.PB8, Level::High, Speed::Low);
    info!("Starting main loop.");

    let mcpwm_a = PwmPin::new(p.PC7, OutputType::PushPull);
    let mcpwm_b = ComplementaryPwmPin::new(p.PB0, OutputType::PushPull);
    let mcpwm_c = PwmPin::new(p.PC8, OutputType::PushPull);
    let mcpwm_d = ComplementaryPwmPin::new(p.PB1, OutputType::PushPull);

    let motor_pwm = ComplementaryPwm::new(
        p.TIM8,
        None,
        None,
        Some(mcpwm_a),
        Some(mcpwm_b),
        Some(mcpwm_c),
        Some(mcpwm_d),
        None,
        None,
        Hertz::mhz(24),
        CountingMode::CenterAlignedBothInterrupts,
    );

    let mcina1 = Output::new(p.PE0, Level::Low, Speed::Low);
    let mcina2 = Output::new(p.PE1, Level::Low, Speed::Low);
    let mcinb1 = Output::new(p.PE2, Level::Low, Speed::Low);
    let mcinb2 = Output::new(p.PE3, Level::Low, Speed::Low);
    let mcinc1 = Output::new(p.PE4, Level::Low, Speed::Low);
    let mcinc2 = Output::new(p.PE5, Level::Low, Speed::Low);
    let mcind1 = Output::new(p.PE6, Level::Low, Speed::Low);
    let mcind2 = Output::new(p.PE7, Level::Low, Speed::Low);

    let mcstby1 = Output::new(p.PE8, Level::High, Speed::Low);
    let mcstby2 = Output::new(p.PE9, Level::High, Speed::Low);

    let mut motor_control = motor::MotorControl::new(
        motor_pwm, mcina1, mcina2, mcinb1, mcinb2, mcinc1, mcinc2, mcind1, mcind2, mcstby1, mcstby2,
    );

    let i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        p.DMA1_CH0,
        p.DMA1_CH1,
        Config::default(),
    );

    let mut accelerometer = motor::Accelerometer::new(i2c);

    accelerometer.init().await;

    motor_control.init();

    loop {
        info!("Hello, World!");
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
