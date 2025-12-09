# Robot Lower Layer

Low-level motor control and sensor management for autonomous delivery robots.

## Overview

**Microcontroller:** STM32H743ZG (ARM Cortex-M7, 480 MHz)
**Language:** Rust
**Runtime:** Embassy async executor
**Location:** `robot/lower/`

## Hardware

- Motor drivers for differential drive
- Sensor interfaces (encoders, IMU, ultrasonic)
- Serial communication with upper controller (UART)
- Power management

## Software Architecture

- **Runtime:** Embassy async executor (async embedded Rust)
- **HAL:** embassy-stm32 0.4.0
- **Communication:** UART with Postcard serialization
- **Logging:** defmt for debugging

## Key Features

- Async task scheduling
- Real-time motor control
- Sensor data acquisition
- Inter-processor communication (UART)
- PWM motor control
- Encoder-based odometry

## Communication Protocol

### UART Configuration

- **Baud Rate:** 115200
- **Data Bits:** 8
- **Stop Bits:** 1
- **Parity:** None
- **Serialization:** Postcard (binary)

### Message Types

**Incoming (from upper layer):**
- `MotorCommand` - Motor speed and direction
- `SensorDataRequest` - Request specific sensor readings
- `ConfigUpdate` - Update motor PID parameters

**Outgoing (to upper layer):**
- `SensorDataResponse` - IMU, encoders, ultrasonic
- `StatusUpdate` - Battery, temperature, errors

## Build & Flash

```bash
cd robot/lower
cargo build --release

# Flash using probe-rs
probe-rs run --chip STM32H743ZGTx

# Or using OpenOCD
openocd -f interface/stlink.cfg -f target/stm32h7x.cfg \
  -c "program target/thumbv7em-none-eabihf/release/robot-lower verify reset exit"
```

## Development Status

**Current:**
- ✅ Basic structure implemented
- ✅ UART communication working
- ✅ Embassy async runtime configured
- ✅ Motor control logic in development
- ✅ Sensor integration ongoing

**Planned:**
- PID motor control
- Kalman filter for IMU
- Collision avoidance
- Battery monitoring
- Watchdog timer

## Dependencies

```toml
embassy-executor = { version = "0.9.1", features = ["arch-cortex-m"] }
embassy-stm32 = { version = "0.4.0", features = ["stm32f407zg"] }
embassy-time = "0.5.0"
defmt = "1.0.1"
cortex-m-rt = "0.7.0"
```

## See Also

- [Serial Bridge](upper/serial.md) - UART communication with upper layer
- [Scheduler](upper/scheduler.md) - Task coordination
- [Protocol Buffers](/robot/proto/serial.proto)
