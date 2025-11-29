#![allow(unused)]
use embassy_stm32::{
    exti::ExtiInput,
    gpio::Input,
    i2c::{Config, I2c, Master},
    mode::Async,
};

pub struct Accelerometer<'a> {
    i2c: I2c<'a, Async, Master>,
    drdy: ExtiInput<'a>,
}

impl<'a> Accelerometer<'a> {
    pub const SLAVE_ADDRESS_AD0_0: u8 = 0b1101000 << 1; // align with 8-bit addressing
    pub const SLAVE_ADDRESS_AD0_1: u8 = 0b1101001 << 1; // align with 8-bit addressing

    pub const ACCEL_XOUT_H: u8 = 0x3B;
    pub const ACCEL_XOUT_L: u8 = 0x3C;
    pub const ACCEL_YOUT_H: u8 = 0x3D;
    pub const ACCEL_YOUT_L: u8 = 0x3E;
    pub const ACCEL_ZOUT_H: u8 = 0x3F;
    pub const ACCEL_ZOUT_L: u8 = 0x40;

    pub const TEMP_OUT_L: u8 = 0x41;
    pub const TEMP_OUT_H: u8 = 0x42;

    pub const GYRO_XOUT_H: u8 = 0x43;
    pub const GYRO_XOUT_L: u8 = 0x44;
    pub const GYRO_YOUT_H: u8 = 0x45;
    pub const GYRO_YOUT_L: u8 = 0x46;
    pub const GYRO_ZOUT_H: u8 = 0x47;
    pub const GYRO_ZOUT_L: u8 = 0x48;

    pub fn new(i2c: I2c<'a, Async, Master>, drdy: ExtiInput<'a>) -> Self {
        Self { i2c, drdy }
    }

    pub async fn init(&mut self) -> Result<(), ()> {
        const PWR_MGMT_1: u8 = 0x6B;

        // Wake up the device
        self.i2c
            .write(Self::SLAVE_ADDRESS_AD0_0, &[PWR_MGMT_1, 0x00])
            .await
            .map_err(|_| ())?;

        self.identity_check().await?;

        // Wait for drdy interrupt
        self.drdy.wait_for_rising_edge().await;

        Ok(())
    }

    pub async fn identity_check(&mut self) -> Result<bool, ()> {
        let mut buffer = [0u8; 1];
        // WHO_AM_I register
        const WHO_AM_I_REG: u8 = 0x75;
        const EXPECTED_WHO_AM_I: u8 = 0x68;

        self.i2c
            .write_read(Self::SLAVE_ADDRESS_AD0_0, &[WHO_AM_I_REG], &mut buffer)
            .await
            .map_err(|_| ())?;

        Ok(buffer[0] == EXPECTED_WHO_AM_I)
    }

    pub async fn read_acceleration(&mut self) -> Result<(i16, i16, i16), ()> {
        let mut buffer = [0u8; 6];

        self.i2c
            .write_read(
                Self::SLAVE_ADDRESS_AD0_0,
                &[Self::ACCEL_XOUT_H],
                &mut buffer,
            )
            .await
            .map_err(|_| ())?;

        let ax = i16::from_be_bytes([buffer[0], buffer[1]]);
        let ay = i16::from_be_bytes([buffer[2], buffer[3]]);
        let az = i16::from_be_bytes([buffer[4], buffer[5]]);

        Ok((ax, ay, az))
    }

    pub async fn read_gyroscope(&mut self) -> Result<(i16, i16, i16), ()> {
        let mut buffer = [0u8; 6];

        self.i2c
            .write_read(Self::SLAVE_ADDRESS_AD0_0, &[Self::GYRO_XOUT_H], &mut buffer)
            .await
            .map_err(|_| ())?;

        let gx = i16::from_be_bytes([buffer[0], buffer[1]]);
        let gy = i16::from_be_bytes([buffer[2], buffer[3]]);
        let gz = i16::from_be_bytes([buffer[4], buffer[5]]);

        Ok((gx, gy, gz))
    }

    pub async fn read_temperature(&mut self) -> Result<i16, ()> {
        let mut buffer = [0u8; 2];

        self.i2c
            .write_read(Self::SLAVE_ADDRESS_AD0_0, &[Self::TEMP_OUT_H], &mut buffer)
            .await
            .map_err(|_| ())?;

        Ok(i16::from_be_bytes([buffer[0], buffer[1]]))
    }
}
