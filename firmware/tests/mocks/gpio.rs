use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// Mock GPIO pin for testing
pub struct MockPin {
    level: AtomicBool,
    mode: AtomicU8, // 0=input, 1=output
}

impl MockPin {
    pub fn new(initial_level: bool, mode: u8) -> Self {
        Self {
            level: AtomicBool::new(initial_level),
            mode: AtomicU8::new(mode),
        }
    }

    pub fn set_high(&self) {
        self.level.store(true, Ordering::SeqCst);
    }

    pub fn set_low(&self) {
        self.level.store(false, Ordering::SeqCst);
    }

    pub fn is_high(&self) -> bool {
        self.level.load(Ordering::SeqCst)
    }

    pub fn is_low(&self) -> bool {
        !self.is_high()
    }

    pub fn toggle(&self) {
        let current = self.level.load(Ordering::SeqCst);
        self.level.store(!current, Ordering::SeqCst);
    }
}

/// Mock relay for unlock testing
pub struct MockRelay {
    pin: MockPin,
    activation_count: AtomicU8,
}

impl MockRelay {
    pub fn new() -> Self {
        Self {
            pin: MockPin::new(false, 1), // Output, initially low
            activation_count: AtomicU8::new(0),
        }
    }

    pub fn activate(&self) {
        self.pin.set_high();
        self.activation_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn deactivate(&self) {
        self.pin.set_low();
    }

    pub fn is_active(&self) -> bool {
        self.pin.is_high()
    }

    pub fn activation_count(&self) -> u8 {
        self.activation_count.load(Ordering::SeqCst)
    }

    pub fn reset_count(&self) {
        self.activation_count.store(0, Ordering::SeqCst);
    }
}

impl Default for MockRelay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_basic_operations() {
        let pin = MockPin::new(false, 1);

        assert!(pin.is_low());
        pin.set_high();
        assert!(pin.is_high());
        pin.set_low();
        assert!(pin.is_low());
    }

    #[test]
    fn test_pin_toggle() {
        let pin = MockPin::new(false, 1);

        pin.toggle();
        assert!(pin.is_high());
        pin.toggle();
        assert!(pin.is_low());
    }

    #[test]
    fn test_relay_activation() {
        let relay = MockRelay::new();

        assert!(!relay.is_active());
        assert_eq!(relay.activation_count(), 0);

        relay.activate();
        assert!(relay.is_active());
        assert_eq!(relay.activation_count(), 1);

        relay.activate();
        assert_eq!(relay.activation_count(), 2);

        relay.deactivate();
        assert!(!relay.is_active());
        assert_eq!(relay.activation_count(), 2); // Count doesn't reset on deactivate

        relay.reset_count();
        assert_eq!(relay.activation_count(), 0);
    }
}
