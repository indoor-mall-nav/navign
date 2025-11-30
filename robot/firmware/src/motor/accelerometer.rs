#![allow(unused)]
use defmt::{info, trace};
use embassy_stm32::{
    exti::ExtiInput,
    gpio::Input,
    i2c::{Config, Error, I2c, Master},
    mode::Async,
};

#[derive(Debug, defmt::Format)]
pub enum MotionError {
    I2cError(Error),
    IdentityMismatch,
    NoCalibrationData,
}

#[derive(Debug, defmt::Format, Clone, Copy)]
pub struct BMP180CalibrationData {
    // Calibration coefficients
    ac1: i16,
    ac2: i16,
    ac3: i16,
    ac4: u16,
    ac5: u16,
    ac6: u16,
    b1: i16,
    b2: i16,
    mb: i16,
    mc: i16,
    md: i16,
}

pub struct Motion<'a> {
    i2c: I2c<'a, Async, Master>,
    drdy: ExtiInput<'a>,

    calib: Option<BMP180CalibrationData>,
}

impl<'a> Motion<'a> {
    // Sensor I2C addresses
    pub const MPU_ADDRESS: u8 = 0x68;
    pub const HMC_ADDRESS: u8 = 0xE8;
    pub const BMP_ADDRESS: u8 = 0x77;

    // MPU6050 register addresses
    pub const ACCEL_START: u8 = 0x3B;
    pub const MPU_TEMP_START: u8 = 0x41;
    pub const GYRO_START: u8 = 0x43;
    const PWR_MGMT_1: u8 = 0x6B;
    const WHO_AM_I_REG: u8 = 0x75;
    const EXPECTED_WHO_AM_I: u8 = 0x68;

    // MPU6050 scale factors (±2g, ±250°/s)
    const ACCEL_SCALE: f32 = 16384.0; // LSB/g
    const GYRO_SCALE: f32 = 131.0; // LSB/(°/s)
    const G: f32 = 9.80665; // m/s²

    // HMC5883L register addresses
    const HMC_CONFIG_A: u8 = 0x00;
    const HMC_CONFIG_B: u8 = 0x01;
    const HMC_MODE: u8 = 0x02;
    const HMC_DATA_START: u8 = 0x03;

    // HMC5883L configuration values
    const HMC_8_SAMPLES_15HZ: u8 = 0b01110000;
    const HMC_GAIN_1_3GA: u8 = 0b00100000;
    const HMC_CONTINUOUS_MODE: u8 = 0x00;

    // HMC5883L scale factors (±1.3 Ga range)
    const MAG_SCALE: f32 = 1090.0; // LSB/Gauss
    const GAUSS_TO_UT: f32 = 100.0; // µT per Gauss

    // BMP180 register addresses
    const BMP_CONTROL: u8 = 0xF4;
    const BMP_DATA_START: u8 = 0xF6;
    const BMP_CAL_START: u8 = 0xAA;

    // BMP180 commands
    const BMP_CMD_TEMP: u8 = 0x2E;
    const BMP_CMD_PRESSURE: u8 = 0x34; // OSS=0 (ultra low power)

    // BMP180 timing (ms)
    const BMP_TEMP_DELAY: u64 = 5;
    const BMP_PRESSURE_DELAY: u64 = 5; // For OSS=0

    pub fn new(i2c: I2c<'a, Async, Master>, drdy: ExtiInput<'a>) -> Self {
        Self {
            i2c,
            drdy,
            calib: None,
        }
    }

    pub async fn init(&mut self) -> Result<(), MotionError> {
        trace!("Initializing MPU6050...");

        // Wake up the device
        self.i2c
            .write(Self::MPU_ADDRESS, &[Self::PWR_MGMT_1, 0x00])
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        trace!("MPU6050 woken up from sleep.");

        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;

        // Configuration Register A: 8 samples averaged, 15Hz output rate, normal measurement
        self.i2c
            .write(
                Self::HMC_ADDRESS,
                &[Self::HMC_CONFIG_A, Self::HMC_8_SAMPLES_15HZ],
            )
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        // Configuration Register B: Gain = ±1.3 Ga (1090 LSB/Gauss)
        self.i2c
            .write(
                Self::HMC_ADDRESS,
                &[Self::HMC_CONFIG_B, Self::HMC_GAIN_1_3GA],
            )
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        // Mode Register: Continuous measurement mode
        self.i2c
            .write(
                Self::HMC_ADDRESS,
                &[Self::HMC_MODE, Self::HMC_CONTINUOUS_MODE],
            )
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        trace!("HMC5883L configured.");

        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;

        let mut bmp_cal_data = [0u8; 22];

        // Read BMP180 calibration data
        self.i2c
            .write_read(Self::BMP_ADDRESS, &[Self::BMP_CAL_START], &mut bmp_cal_data)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        self.calib = Some(BMP180CalibrationData {
            ac1: i16::from_be_bytes([bmp_cal_data[0], bmp_cal_data[1]]),
            ac2: i16::from_be_bytes([bmp_cal_data[2], bmp_cal_data[3]]),
            ac3: i16::from_be_bytes([bmp_cal_data[4], bmp_cal_data[5]]),
            ac4: u16::from_be_bytes([bmp_cal_data[6], bmp_cal_data[7]]),
            ac5: u16::from_be_bytes([bmp_cal_data[8], bmp_cal_data[9]]),
            ac6: u16::from_be_bytes([bmp_cal_data[10], bmp_cal_data[11]]),
            b1: i16::from_be_bytes([bmp_cal_data[12], bmp_cal_data[13]]),
            b2: i16::from_be_bytes([bmp_cal_data[14], bmp_cal_data[15]]),
            mb: i16::from_be_bytes([bmp_cal_data[16], bmp_cal_data[17]]),
            mc: i16::from_be_bytes([bmp_cal_data[18], bmp_cal_data[19]]),
            md: i16::from_be_bytes([bmp_cal_data[20], bmp_cal_data[21]]),
        });

        trace!("BMP180 calibration data read: {:?}", self.calib);

        self.identity_check().await?;

        trace!("Identity verified.");

        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;

        Ok(())
    }

    pub async fn identity_check(&mut self) -> Result<bool, MotionError> {
        let mut buffer = [0u8; 1];

        self.i2c
            .write_read(Self::MPU_ADDRESS, &[Self::WHO_AM_I_REG], &mut buffer)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        info!("WHO_AM_I: 0x{:X}", buffer[0]);

        if buffer[0] == Self::EXPECTED_WHO_AM_I {
            Ok(true)
        } else {
            Err(MotionError::IdentityMismatch)
        }
    }

    pub async fn read_acceleration(&mut self) -> Result<(f32, f32, f32), MotionError> {
        let mut buffer = [0u8; 6];

        self.i2c
            .write_read(Self::MPU_ADDRESS, &[Self::ACCEL_START], &mut buffer)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        let ax = i16::from_be_bytes([buffer[0], buffer[1]]);
        let ay = i16::from_be_bytes([buffer[2], buffer[3]]);
        let az = i16::from_be_bytes([buffer[4], buffer[5]]);

        const ACCEL_SCALE: f32 = 16384.0; // LSB/g
        const G: f32 = 9.80665; // standard gravity in m/s²

        let ax_ms2 = (ax as f32 / ACCEL_SCALE) * G; // m/s²
        let ay_ms2 = (ay as f32 / ACCEL_SCALE) * G;
        let az_ms2 = (az as f32 / ACCEL_SCALE) * G;

        Ok((ax_ms2, ay_ms2, az_ms2))
    }

    pub async fn read_gyroscope(&mut self) -> Result<(f32, f32, f32), MotionError> {
        let mut buffer = [0u8; 6];

        self.i2c
            .write_read(Self::MPU_ADDRESS, &[Self::GYRO_START], &mut buffer)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        let gx = i16::from_be_bytes([buffer[0], buffer[1]]);
        let gy = i16::from_be_bytes([buffer[2], buffer[3]]);
        let gz = i16::from_be_bytes([buffer[4], buffer[5]]);

        const GYRO_SCALE: f32 = 131.0; // LSB/(°/s)

        let gx_dps = gx as f32 / GYRO_SCALE; // °/s
        let gy_dps = gy as f32 / GYRO_SCALE;
        let gz_dps = gz as f32 / GYRO_SCALE;

        Ok((gx_dps, gy_dps, gz_dps))
    }

    pub async fn read_temperature_mpu(&mut self) -> Result<i16, MotionError> {
        let mut buffer = [0u8; 2];

        self.i2c
            .write_read(Self::MPU_ADDRESS, &[Self::MPU_TEMP_START], &mut buffer)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        Ok(i16::from_be_bytes([buffer[0], buffer[1]]))
    }

    /// Read raw temperature from BMP180
    async fn read_raw_temperature(&mut self) -> Result<i32, MotionError> {
        // Start temperature measurement
        self.i2c
            .write(Self::BMP_ADDRESS, &[Self::BMP_CONTROL, Self::BMP_CMD_TEMP])
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        embassy_time::Timer::after(embassy_time::Duration::from_millis(Self::BMP_TEMP_DELAY)).await;

        // Read result
        let mut buffer = [0u8; 2];
        self.i2c
            .write_read(Self::BMP_ADDRESS, &[Self::BMP_DATA_START], &mut buffer)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        Ok(i16::from_be_bytes([buffer[0], buffer[1]]) as i32)
    }

    /// Read raw pressure from BMP180
    async fn read_raw_pressure(&mut self) -> Result<i32, MotionError> {
        // Start pressure measurement (OSS=0)
        self.i2c
            .write(
                Self::BMP_ADDRESS,
                &[Self::BMP_CONTROL, Self::BMP_CMD_PRESSURE],
            )
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        embassy_time::Timer::after(embassy_time::Duration::from_millis(
            Self::BMP_PRESSURE_DELAY,
        ))
        .await;

        // Read result (3 bytes)
        let mut buffer = [0u8; 3];
        self.i2c
            .write_read(Self::BMP_ADDRESS, &[Self::BMP_DATA_START], &mut buffer)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        let up = ((buffer[0] as i32) << 16 | (buffer[1] as i32) << 8 | buffer[2] as i32) >> (8 - 0); // OSS=0

        Ok(up)
    }

    /// Read temperature and pressure in °C and Pa
    pub async fn read_barometer(&mut self) -> Result<(f32, f32), MotionError> {
        let ut = self.read_raw_temperature().await?;
        let up = self.read_raw_pressure().await?;

        let cal = match self.calib {
            Some(c) => c,
            None => return Err(MotionError::NoCalibrationData), // Calibration data not available
        };

        // Calculate true temperature
        let x1 = ((ut - cal.ac6 as i32) * cal.ac5 as i32) >> 15;
        let x2 = ((cal.mc as i32) << 11) / (x1 + cal.md as i32);
        let b5 = x1 + x2;
        let temp_c = ((b5 + 8) >> 4) as f32 / 10.0;

        // Calculate true pressure (OSS=0)
        let b6 = b5 - 4000;
        let x1 = (cal.b2 as i32 * ((b6 * b6) >> 12)) >> 11;
        let x2 = (cal.ac2 as i32 * b6) >> 11;
        let x3 = x1 + x2;
        let b3 = (((cal.ac1 as i32 * 4 + x3) << 0) + 2) / 4; // OSS=0
        let x1 = (cal.ac3 as i32 * b6) >> 13;
        let x2 = (cal.b1 as i32 * ((b6 * b6) >> 12)) >> 16;
        let x3 = ((x1 + x2) + 2) >> 2;
        let b4 = (cal.ac4 as u32 * (x3 + 32768) as u32) >> 15;
        let b7 = (up as u32 - b3 as u32) * (50000 >> 0); // OSS=0

        let p = if b7 < 0x80000000 {
            (b7 * 2) / b4
        } else {
            (b7 / b4) * 2
        };

        let x1 = (p >> 8) * (p >> 8);
        let x1 = ((x1 * 3038) >> 16) as i32;
        let x2 = (-7357 * p as i32) >> 16;
        let pressure_pa = (p as i32 + ((x1 + x2 + 3791) >> 4)) as f32;

        Ok((temp_c, pressure_pa))
    }

    /// Calculate altitude in meters from pressure
    /// sea_level_pa: sea level pressure in Pa (typically 101325 Pa)
    pub fn calculate_altitude(pressure_pa: f32, sea_level_pa: f32) -> f32 {
        use micromath::F32Ext;
        44330.0 * (1.0 - (pressure_pa / sea_level_pa).powf(1.0 / 5.255))
    }

    /// Read magnetic field in microteslas (µT)
    /// Earth's magnetic field is typically 25-65 µT depending on location
    pub async fn read_magnetometer(&mut self) -> Result<(f32, f32, f32), MotionError> {
        let mut buffer = [0u8; 6];

        // Data starts at register 0x03
        // Note: HMC5883L has unusual register order: X, Z, Y
        self.i2c
            .write_read(Self::HMC_ADDRESS, &[Self::HMC_DATA_START], &mut buffer)
            .await
            .map_err(|err| MotionError::I2cError(err))?;

        // Register order: X_MSB, X_LSB, Z_MSB, Z_LSB, Y_MSB, Y_LSB
        let mx = i16::from_be_bytes([buffer[0], buffer[1]]);
        let mz = i16::from_be_bytes([buffer[2], buffer[3]]);
        let my = i16::from_be_bytes([buffer[4], buffer[5]]);

        let mx_ut = (mx as f32 / Self::MAG_SCALE) * Self::GAUSS_TO_UT;
        let my_ut = (my as f32 / Self::MAG_SCALE) * Self::GAUSS_TO_UT;
        let mz_ut = (mz as f32 / Self::MAG_SCALE) * Self::GAUSS_TO_UT;

        Ok((mx_ut, my_ut, mz_ut))
    }

    /// Calculate heading in degrees (0-360°)
    /// Note: This assumes the sensor is level. For tilted sensors, you need tilt compensation.
    pub fn calculate_heading(mx: f32, my: f32) -> f32 {
        use micromath::F32Ext;

        let mut heading = my.atan2(mx) * (180.0 / core::f32::consts::PI);

        // Normalize to 0-360°
        if heading < 0.0 {
            heading += 360.0;
        }

        heading
    }
}
