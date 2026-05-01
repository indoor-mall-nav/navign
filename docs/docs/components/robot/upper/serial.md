# Serial Bridge

The serial component provides UART communication between the robot upper layer (Raspberry Pi) and the lower controller (STM32).

## Overview

**Language:** Rust
**Location:** `robot/serial/`
**Protocol:** Postcard binary serialization
**Baud Rate:** 115200

## Architecture

```
   Scheduler
       |
       | Zenoh: robot/serial/command
       v
  Serial Bridge
    (Postcard
     Encoder)
       |
       | UART (115200 baud)
       v
  STM32 Lower
  Controller
```

## Key Features

- **Bidirectional Communication:** Commands to STM32, sensor data from STM32
- **Postcard Serialization:** Efficient binary format
- **Automatic Reconnection:** Handles USB disconnections
- **Async I/O:** Non-blocking with tokio_serial
- **Zenoh Integration:** Publishes sensor data, subscribes to commands

## Zenoh Topics

### Published

- `robot/serial/sensors` - Sensor data from STM32 (IMU, encoders, ultrasonic)
- `robot/serial/status` - Lower controller health

### Subscribed

- `robot/serial/command` - Motor commands from scheduler

## Running

```bash
cd robot/serial
SERIAL_PORT=/dev/ttyUSB0 cargo run
```

## Environment Variables

- `SERIAL_PORT` - Serial device path (default: `/dev/ttyUSB0`)
- `SERIAL_BAUD` - Baud rate (default: `115200`)
- `ZENOH_CONFIG` - Zenoh configuration (optional)

## Troubleshooting

**Port not found:**

```bash
ls /dev/ttyUSB*
sudo chmod 666 /dev/ttyUSB0
```

**Permission denied:**

```bash
sudo usermod -a -G dialout $USER
# Logout and login
```

## See Also

- [Scheduler](scheduler.md)
- [Robot Lower Layer](../lower.md)
- [Protocol Buffers](/robot/proto/serial.proto)
