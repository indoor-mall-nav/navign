#![cfg(test)]

/// Rate limiter state for testing
struct RateLimiter {
    attempts: heapless::Vec<u64, 5>,
    max_attempts: usize,
    window_ms: u64,
}

impl RateLimiter {
    fn new(max_attempts: usize, window_ms: u64) -> Self {
        Self {
            attempts: heapless::Vec::new(),
            max_attempts,
            window_ms,
        }
    }

    fn check_and_record(&mut self, current_time: u64) -> bool {
        // Remove old attempts
        self.attempts.retain(|&t| current_time - t < self.window_ms);

        if self.attempts.len() >= self.max_attempts {
            return false; // Rate limited
        }

        self.attempts.push(current_time).ok();
        true
    }
}

#[test]
fn test_rate_limit_allows_within_limit() {
    let mut limiter = RateLimiter::new(5, 5000);

    // 5 attempts within window should succeed
    for i in 0..5 {
        assert!(limiter.check_and_record(1000 + i * 100));
    }
}

#[test]
fn test_rate_limit_blocks_excess() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Fill up the limit
    for i in 0..5 {
        limiter.check_and_record(1000 + i);
    }

    // 6th attempt should be blocked
    assert!(!limiter.check_and_record(1005));
}

#[test]
fn test_rate_limit_resets_after_window() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Fill limit
    for i in 0..5 {
        limiter.check_and_record(1000 + i);
    }

    // Wait beyond window
    assert!(limiter.check_and_record(7000));
}

#[test]
fn test_rate_limit_sliding_window() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Attempts at: 1000, 2000, 3000, 4000, 5000
    for i in 0..5 {
        limiter.check_and_record(1000 + i * 1000);
    }

    // At time 6100, oldest (1000) should be expired
    assert!(limiter.check_and_record(6100));
}

#[test]
fn test_rate_limit_burst_protection() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Rapid burst at same timestamp
    for _ in 0..5 {
        assert!(limiter.check_and_record(1000));
    }

    // Next attempt should be blocked
    assert!(!limiter.check_and_record(1000));
}

#[test]
fn test_rate_limit_gradual_recovery() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Fill limit
    for i in 0..5 {
        limiter.check_and_record(1000 + i * 1000);
    }

    // As time progresses, old attempts expire one by one
    assert!(limiter.check_and_record(6100)); // 1000 expired
    assert!(limiter.check_and_record(7100)); // 2000 expired
    assert!(limiter.check_and_record(8100)); // 3000 expired
}

#[test]
fn test_rate_limit_empty_state() {
    let limiter = RateLimiter::new(5, 5000);

    // No attempts yet, all allowed
    assert_eq!(limiter.attempts.len(), 0);
}

#[test]
fn test_rate_limit_boundary_conditions() {
    let mut limiter = RateLimiter::new(5, 5000);

    // Exactly at window boundary
    limiter.check_and_record(1000);
    limiter.check_and_record(2000);
    limiter.check_and_record(3000);
    limiter.check_and_record(4000);
    limiter.check_and_record(5000);

    // At exactly 6000 (5000 window from earliest), first should expire
    assert!(limiter.check_and_record(6000));
}
