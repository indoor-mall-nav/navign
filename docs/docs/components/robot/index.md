# Robot Upper Layer Components

See `/robot/README.md` for comprehensive documentation.

## Quick Links

- [Scheduler](scheduler.md) - Central coordinator
- [Serial](serial.md) - UART bridge to STM32
- [Network](network.md) - Server API communication
- [Vision](vision.md) - AprilTag & YOLO detection
- [Audio](audio.md) - Wake word & TTS
- [Protocol Buffers](protocol-buffers.md) - Message schemas

## Architecture

All components communicate via **Zenoh** pub/sub using **Protocol Buffers**.

See `/robot/README.md` for detailed architecture, message flows, and deployment instructions.
