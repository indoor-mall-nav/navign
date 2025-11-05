# Mobile

The Mobile component is a user application designed for indoor navigation, providing real-time directions and assistance to users within indoor environments. It leverages BLE beacons for accurate positioning and offers an intuitive interface for seamless navigation.

Language: Rust (via Tauri), TypeScript (Vue 3), some native mobile code (Swift/Kotlin)

# Features

- **Real-time Indoor Navigation**: Provides users with turn-by-turn directions within indoor spaces such as malls, airports, and office buildings.
- **BLE Beacon Integration**: Utilizes signals from ESP32-C3 based BLE beacons to determine user location with high accuracy.
- **User-friendly Interface**: Offers an intuitive and accessible UI for easy navigation and interaction.
- **Multi-platform Support**: Available on both iOS and Android platforms, ensuring broad accessibility.

# Functionality

- **Positioning**: Continuously receives and processes BLE signals to determine the user's location within the indoor environment.
- **Path Planning**: Calculates optimal routes to the user's desired destination using advanced algorithms.
- **User Interaction**: Provides visual and auditory cues to guide users along their path, including notifications for turns and points of interest.
- **Offline Mode**: Allows users to download maps and navigation data for use without an internet connection.
- **Contactless Access Control**: Enables users to unlock doors and access restricted areas through secure authentication with BLE beacons.

# Security

- **Secure Communication**: Implements encryption and authentication protocols to ensure secure data exchange between the mobile app and BLE beacons.
- **Access Control**: Utilizes TOTP and nonce-based challenge-response mechanisms for secure access to doors and merchant spaces.
- **Data Privacy**: Adheres to best practices for user data privacy and protection.

# Libraries

- **Tauri**: Framework for building cross-platform desktop and mobile applications using web technologies.
- **Vue 3**: JavaScript framework for building user interfaces.
- **Shadcn-Vue**: Component library for Vue.js applications.
- **Axios**: Promise-based HTTP client for making API requests.
- **Pinia**: State management library for Vue.js applications.
- **Vue Router**: Official router for Vue.js applications.
- **Btleplug**: Cross-platform BLE library for Rust.
