# Server Testing Documentation

This document describes the test suite for the Navign server, focusing on the rate limiting and key management features.

## Test Organization

The server tests are organized into:
- **Unit tests**: Located within source files (`src/**/*.rs`) in `#[cfg(test)] mod tests` blocks
- **Integration tests**: Located in the `tests/` directory

## Running Tests

### Run All Tests

```bash
cd server
cargo test
```

### Run Specific Test Module

```bash
# Run only key management tests
cargo test key_management

# Run only rate limiting tests
cargo test rate_limiting
```

### Run Tests with Output

```bash
# Show println! output from tests
cargo test -- --nocapture

# Show test names as they run
cargo test -- --show-output
```

### Run Tests in Parallel or Serial

```bash
# Run tests in parallel (default)
cargo test

# Run tests serially (useful for tests that modify environment)
cargo test -- --test-threads=1
```

## Test Coverage

### Key Management Tests (`src/key_management.rs`)

**Total Tests**: 14

#### Basic Functionality Tests

1. **`test_save_and_load_key`**
   - Tests basic save and load cycle
   - Verifies public keys match
   - Verifies signatures work correctly

2. **`test_load_or_generate_creates_new_key`**
   - Tests key generation when no key exists
   - Verifies file creation
   - Verifies subsequent loads return same key

3. **`test_load_or_generate_loads_existing_key`**
   - Tests loading existing key file
   - Verifies correct key is loaded

#### File System Tests

4. **`test_save_key_creates_parent_directories`**
   - Tests directory creation for nested paths
   - Verifies parent directories are created automatically

5. **`test_saved_key_has_correct_permissions`** (Unix only)
   - Tests file permissions are set to 0o600
   - Ensures key file is readable/writable only by owner

#### Error Handling Tests

6. **`test_load_key_with_invalid_pem`**
   - Tests error handling for invalid PEM format
   - Verifies appropriate error message

7. **`test_load_key_with_nonexistent_file`**
   - Tests error handling for missing files
   - Verifies appropriate error message

#### Configuration Tests

8. **`test_get_key_file_path_default`**
   - Tests default key file path
   - Verifies `./private_key.pem` is used by default

9. **`test_get_key_file_path_from_env`**
   - Tests custom path from environment variable
   - Verifies `PRIVATE_KEY_FILE` is respected

#### Format Validation Tests

10. **`test_key_pem_format_is_valid`**
    - Tests PEM format structure
    - Verifies headers and footers
    - Verifies base64 content

#### Reliability Tests

11. **`test_multiple_save_and_load_cycles`**
    - Tests 5 save/load cycles
    - Verifies key integrity across cycles
    - Ensures no data corruption

12. **`test_concurrent_key_operations`**
    - Tests concurrent save/load from multiple threads
    - Verifies thread safety
    - Tests with 5 concurrent operations

### Rate Limiting Tests

#### Unit Tests (`src/rate_limiting.rs`)

**Total Unit Tests**: 8

1. **`test_create_rate_limit_layer_with_custom_values`**
   - Tests layer creation with custom parameters

2. **`test_create_default_rate_limit_layer`**
   - Tests layer creation with default parameters

3. **`test_create_default_rate_limit_layer_with_env`**
   - Tests layer creation with environment variables

4. **`test_rate_limit_error_response`**
   - Tests error response status code (429)

5. **`test_rate_limit_config_parsing`**
   - Tests parsing of valid configuration values

6. **`test_rate_limit_config_defaults`**
   - Tests default values when env vars not set

7. **`test_rate_limit_config_invalid_values`**
   - Tests fallback to defaults with invalid env vars

8. **`test_rate_limit_config_edge_cases`**
   - Tests with 0 and very large values

#### Integration Tests (`tests/rate_limiting_tests.rs`)

**Total Integration Tests**: 11

1. **`test_rate_limit_allows_within_limit`**
   - Tests requests within rate limit succeed
   - Makes 5 requests with limit of 10/sec

2. **`test_rate_limit_blocks_over_limit`**
   - Tests requests over limit are blocked
   - Verifies HTTP 429 response

3. **`test_rate_limit_per_ip`**
   - Tests rate limiting is per IP address
   - Verifies different IPs have separate limits

4. **`test_rate_limit_respects_x_real_ip`**
   - Tests X-Real-IP header handling
   - Verifies proxy header support

5. **`test_rate_limit_burst_size`**
   - Tests burst functionality
   - Allows 5 rapid requests, blocks 6th

6. **`test_rate_limit_recovery_after_wait`**
   - Tests rate limit recovery over time
   - Waits 200ms for tokens to replenish

7. **`test_rate_limit_with_no_forwarded_headers`**
   - Tests fallback to connection IP
   - Verifies behavior without proxy headers

8. **`test_high_throughput_rate_limiting`**
   - Tests with 250 rapid requests
   - Verifies exactly 200 succeed (burst size)
   - Verifies 50 are rate limited

9. **`test_rate_limit_configuration_from_env`**
   - Tests environment variable configuration
   - Tests with and without env vars

10. **`test_rate_limit_configuration_invalid_env`**
    - Tests handling of invalid environment values
    - Verifies fallback to defaults

## Test Best Practices

### Environment Variable Handling

Tests that modify environment variables should:
1. Save original values (if any)
2. Set test values
3. Run test logic
4. Restore original values or remove test values

Example:
```rust
#[test]
fn test_env_handling() {
    // Setup
    std::env::set_var("MY_VAR", "test_value");

    // Test logic
    // ...

    // Cleanup
    std::env::remove_var("MY_VAR");
}
```

### Temporary Files

Tests that create files should use `tempfile::TempDir`:
```rust
use tempfile::TempDir;

#[test]
fn test_with_files() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_file.txt");

    // Test logic...

    // TempDir automatically cleans up when dropped
}
```

### Async Tests

Integration tests that test Axum endpoints use `#[tokio::test]`:
```rust
#[tokio::test]
async fn test_async_endpoint() {
    let app = create_test_router();
    let response = app.oneshot(request).await.unwrap();
    // assertions...
}
```

## Known Limitations

### Rate Limiting Integration Tests

The rate limiting integration tests make assumptions about timing that may be flaky in slow CI environments. If tests fail intermittently:

1. Increase wait durations in `test_rate_limit_recovery_after_wait`
2. Run tests serially: `cargo test -- --test-threads=1`
3. Consider adjusting rate limits for more predictable behavior

### Key Management Tests

The `test_saved_key_has_correct_permissions` test only runs on Unix systems. Windows file permissions are not currently tested.

## Adding New Tests

### When to Add Tests

Add tests when:
- Adding new public functions
- Fixing bugs (add regression test)
- Refactoring (ensure behavior unchanged)
- Adding configuration options

### Test Naming Convention

Use descriptive test names that clearly state what is being tested:
- `test_[function]_[scenario]_[expected_result]`
- Examples:
  - `test_save_key_creates_parent_directories`
  - `test_rate_limit_blocks_over_limit`
  - `test_load_key_with_invalid_pem`

### Test Structure

Follow the Arrange-Act-Assert pattern:
```rust
#[test]
fn test_example() {
    // Arrange: Set up test data and preconditions
    let input = setup_test_data();

    // Act: Execute the code under test
    let result = function_under_test(input);

    // Assert: Verify the results
    assert_eq!(result, expected_value);
}
```

## Continuous Integration

Tests are automatically run in CI on:
- Pull requests
- Pushes to main branch
- Release tags

CI configuration is in `.github/workflows/ci.yml`.

### CI-Specific Considerations

1. **MongoDB Requirement**: Server tests require MongoDB to be running
2. **Network Access**: Some tests may require network access (currently blocked)
3. **Environment**: Tests run on Ubuntu Linux in CI

## Troubleshooting

### Test Failures

**Problem**: Tests fail with "Too many open files"
**Solution**: Increase ulimit or reduce concurrent test threads

**Problem**: Tests timeout
**Solution**: Increase timeout with `#[timeout(60)]` or run fewer tests in parallel

**Problem**: Environment variable conflicts
**Solution**: Run tests serially: `cargo test -- --test-threads=1`

### Test Coverage

To generate coverage reports:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# View coverage report
open coverage/index.html
```

## Performance Considerations

### Test Speed

- Unit tests: ~50ms total
- Integration tests: ~500ms total (due to async runtime startup)

### Optimization Tips

1. Use `cargo test --lib` to run only unit tests (faster)
2. Use `cargo test --test rate_limiting_tests` to run specific integration tests
3. Consider using `cargo nextest` for faster test execution

## Future Improvements

Potential test additions:
1. Load testing for rate limiter under extreme conditions
2. Fuzz testing for key management PEM parsing
3. Property-based testing for cryptographic operations
4. Integration tests with actual MongoDB database
5. End-to-end tests with full server startup
6. Chaos testing for failure scenarios
7. Performance benchmarks for rate limiting overhead

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [Axum Testing Examples](https://github.com/tokio-rs/axum/tree/main/examples/testing)
- [Tower Governor Documentation](https://docs.rs/tower-governor/)
