# Components

The Navign project is composed with different components, with a variety of technologies integrated to achieve autonomous indoor navigation and user interaction.

- **Server**: Centralized system for processing location data, path planning, and managing robot fleets.
- **Mobile**: User application for indoor navigation, providing real-time directions and assistance.
- **Beacon**: ESP32-C3 based BLE beacons for indoor positioning.
- **Robot**: Autonomous robots equipped with navigation and interaction capabilities.
   - **Upper**: Robot control and navigation logic, running on a Raspberry Pi.
   - **Lower**: Motor control and low-level operations, running on an STM32 microcontroller.
- **Vision**: Apple Vision Pro application for gesture recognition and spatial understanding.
- **Miniapp**: WeChat Mini Program for user interaction and navigation assistance.
- **Admin**: Administrative interface for managing beacons, robots, and system settings.
   - **Orchestra**: Fleet management and coordination system for robots; also interface server for Admin.
   - **Tower**: One-to-one communication with robot via Socket.IO.
   - **Vision**: Vision processing backend for map generation and spatial analysis.
- **Shared**: Shared libraries and utilities used across different components.
- **Docs**: Documentation and guides for the project.
