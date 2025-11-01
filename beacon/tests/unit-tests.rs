#![no_std]
#![no_main]
#[cfg(test)]
#[embedded_test::tests]
mod tests {
    #[init]
    fn init() -> Peripherals {
        Peripherals::take().unwrap()
    }

    // Tests can be async (needs feature `embassy`)
    // Tests can take the state returned by the init function (optional)
    #[test]
    fn takes_state(_state: Peripherals) {
        assert!(true)
    }
}
