#![no_std]
#![no_main]

mod measure;
mod motor;

use crate::measure::ptz::Ptz;

use {defmt_rtt as _, panic_probe as _};

use defmt::{error, info, panic};
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::i2c::{Config, I2c};
use embassy_stm32::timer::complementary_pwm::ComplementaryPwm;
use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, OutputType, Pull, Speed},
    time::Hertz,
    timer::low_level::CountingMode,
    timer::{complementary_pwm::ComplementaryPwmPin, simple_pwm::PwmPin},
};
use embassy_time::TICK_HZ;
use embassy_time::{Duration, Timer};

bind_interrupts!(struct Irqs {
    I2C1_EV => embassy_stm32::i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_ER => embassy_stm32::i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;

    TIM5 => embassy_stm32::timer::CaptureCompareInterruptHandler<embassy_stm32::peripherals::TIM5>;
});

fn get_stm_config() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    #[cfg(debug_assertions)]
    let dbgmcu = embassy_stm32::pac::DBGMCU;
    #[cfg(debug_assertions)]
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

    let mcpwm_a = PwmPin::new(p.PC7, OutputType::PushPull);
    let mcpwm_b = ComplementaryPwmPin::new(p.PB0, OutputType::PushPull);
    let mcpwm_c = PwmPin::new(p.PC8, OutputType::PushPull);
    let mcpwm_d = ComplementaryPwmPin::new(p.PB1, OutputType::PushPull);

    info!(
        "Configured PWM pins. PWM Frequency: {} MHz",
        Hertz::mhz(24).0
    );

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

    info!("Initialized motor PWM. Using TIM8.");

    let mcina1 = Output::new(p.PE0, Level::Low, Speed::Low);
    let mcina2 = Output::new(p.PE1, Level::Low, Speed::Low);
    let mcinb1 = Output::new(p.PE2, Level::Low, Speed::Low);
    let mcinb2 = Output::new(p.PE3, Level::Low, Speed::Low);
    let mcinc1 = Output::new(p.PE4, Level::Low, Speed::Low);
    let mcinc2 = Output::new(p.PE5, Level::Low, Speed::Low);
    let mcind1 = Output::new(p.PE6, Level::Low, Speed::Low);
    let mcind2 = Output::new(p.PE7, Level::Low, Speed::Low);

    info!("Initialized motor control inputs.");

    let mcstby1 = Output::new(p.PE8, Level::High, Speed::Low);
    let mcstby2 = Output::new(p.PE9, Level::High, Speed::Low);

    info!("Initialized motor standby outputs.");

    let mut motor_control = motor::MotorControl::new(
        motor_pwm, mcina1, mcina2, mcinb1, mcinb2, mcinc1, mcinc2, mcind1, mcind2, mcstby1, mcstby2,
    );

    info!("Motor control initialized.");

    let drdy = ExtiInput::new(p.PB5, p.EXTI5, Pull::Down);

    info!("Configured DRDY pin for accelerometer.");

    let i2c = I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        Irqs,
        p.DMA1_CH0,
        p.DMA1_CH1,
        Config::default(),
    );

    info!("Initialized I2C1 for accelerometer.");

    let mut accelerometer = motor::Accelerometer::new(i2c, drdy);

    info!("Created accelerometer instance.");

    let ptz_pwm_pina = PwmPin::new(p.PA0, OutputType::PushPull);
    let ptz_pwm_pinb = PwmPin::new(p.PA1, OutputType::PushPull);

    let ptz_pwm = SimplePwm::new(
        p.TIM2,
        Some(ptz_pwm_pina),
        Some(ptz_pwm_pinb),
        None,
        None,
        Hertz::hz(50),
        CountingMode::EdgeAlignedUp,
    );

    let mut ptz = Ptz::new(ptz_pwm);

    let ultrasonic_echo_pin =
        embassy_stm32::timer::input_capture::CapturePin::new(p.PA2, Pull::None);

    let ultrasonic_trigger = Output::new(p.PA4, Level::Low, Speed::Low);
    let ultrasonic_echo = embassy_stm32::timer::input_capture::InputCapture::new(
        p.TIM5,
        None,
        None,
        Some(ultrasonic_echo_pin),
        None,
        Irqs,
        Hertz::mhz(1),
        CountingMode::CenterAlignedUpInterrupts,
    );

    let mut ultrasonic = measure::ultrasonic::Ultrasonic::new(ultrasonic_trigger, ultrasonic_echo);

    match accelerometer.init().await {
        Ok(()) => {
            info!("Accelerometer identity check passed.");
        }
        Err(()) => {
            error!("Accelerometer identity check failed.");
            panic!("Cannot continue without accelerometer.");
        }
    }

    info!("Initialized accelerometer.");

    motor_control.init();

    info!("Motor control ready.");

    ptz.init();

    info!("PTZ initialized and enabled.");

    ultrasonic.init();

    info!("Ultrasonic sensor initialized.");

    loop {
        info!("Hello, World!");
        let accel = match accelerometer.read_acceleration().await {
            Ok(data) => data,
            Err(()) => {
                error!("Failed to read acceleration data.");
                continue;
            }
        };
        let gyro = match accelerometer.read_gyroscope().await {
            Ok(data) => data,
            Err(()) => {
                error!("Failed to read gyroscope data.");
                continue;
            }
        };

        info!("Acceleration: ax={} ay={} az={}", accel.0, accel.1, accel.2);
        info!("Gyroscope: gx={} gy={} gz={}", gyro.0, gyro.1, gyro.2);

        Timer::after(Duration::from_millis(500)).await;
    }
}
