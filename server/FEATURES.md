# Server Features Documentation

## Rate Limiting

The server now includes IP-based rate limiting to prevent abuse and ensure fair resource allocation.

### Configuration

Rate limiting can be configured using environment variables:

```bash
# Maximum requests per second per IP (default: 100)
RATE_LIMIT_PER_SECOND=100

# Maximum burst size - number of requests that can be made at once (default: 200)
RATE_LIMIT_BURST_SIZE=200
```

### Behavior

- Rate limiting is applied to all API endpoints
- Limits are enforced per client IP address
- Uses the `SmartIpKeyExtractor` which handles proxy headers (X-Forwarded-For, X-Real-IP)
- When rate limit is exceeded, clients receive HTTP 429 (Too Many Requests)

### Implementation Details

- Uses `tower-governor` library for efficient rate limiting
- Configured in `src/rate_limiting.rs`
- Applied as a middleware layer in the Axum router

## Persistent Private Key

The server now persists its P-256 ECDSA private key across restarts instead of generating a new one each time.

### Configuration

The private key location can be configured using an environment variable:

```bash
# Path to private key file (default: ./private_key.pem)
PRIVATE_KEY_FILE=/path/to/custom/private_key.pem
```

### Behavior

- On first startup, generates a new P-256 private key and saves it to disk
- On subsequent startups, loads the existing key from disk
- Key is stored in PEM format (PKCS#8)
- On Unix systems, file permissions are automatically set to 600 (read/write for owner only)

### Security Considerations

1. **Backup the key file**: Loss of the key file means clients will need to update their trusted public keys
2. **Protect the key file**: Ensure the file has appropriate permissions and is not committed to version control
3. **Key rotation**: To rotate keys, simply delete the existing key file and restart the server

### Implementation Details

- Key management logic is in `src/key_management.rs`
- Includes comprehensive tests for save/load functionality
- Uses `p256` crate for ECDSA operations
- Uses `pkcs8` encoding for interoperability

## Getting Started

### Default Configuration

With no environment variables set, the server will:
- Allow 100 requests per second per IP with a burst of 200
- Store the private key in `./private_key.pem`

### Production Deployment

For production, consider:

1. **Rate Limiting**: Adjust based on your traffic patterns
   ```bash
   RATE_LIMIT_PER_SECOND=50
   RATE_LIMIT_BURST_SIZE=100
   ```

2. **Private Key**: Use a secure location outside the project directory
   ```bash
   PRIVATE_KEY_FILE=/etc/navign/private_key.pem
   ```

3. **Key Backup**: Regularly backup the private key to secure storage

4. **Monitoring**: Monitor rate limit hits to detect potential abuse or need for adjustment

## Testing

### Rate Limiting Test

You can test rate limiting using tools like `ab` (Apache Bench) or `wrk`:

```bash
# Send 1000 requests with 10 concurrent connections
ab -n 1000 -c 10 http://localhost:3000/

# Check for 429 responses
```

### Private Key Test

```bash
# First startup - key will be generated
cargo run

# Check the key file exists
ls -la private_key.pem

# Second startup - same key will be loaded
cargo run

# Compare the public keys in logs - they should match
```

## Troubleshooting

### Rate Limiting Issues

**Problem**: Legitimate users getting rate limited
**Solution**: Increase `RATE_LIMIT_PER_SECOND` or `RATE_LIMIT_BURST_SIZE`

**Problem**: Rate limiting not working behind reverse proxy
**Solution**: Ensure your reverse proxy sets `X-Forwarded-For` or `X-Real-IP` headers

### Private Key Issues

**Problem**: Permission denied when reading/writing key file
**Solution**: Ensure the server has appropriate file system permissions

**Problem**: "Failed to parse private key from PEM format"
**Solution**: The key file may be corrupted. Delete it and let the server generate a new one

**Problem**: Want to use a specific private key
**Solution**: Generate a key externally and save it in PEM format (PKCS#8) to the configured location

## Migration Guide

### From Previous Version

1. **No breaking changes**: The server will automatically generate and save a key on first startup
2. **Optional**: Set `PRIVATE_KEY_FILE` environment variable if you want a custom location
3. **Optional**: Set rate limiting environment variables if defaults don't suit your needs

### Rollback

If you need to rollback these features:

1. Remove the `tower-governor` dependency from `Cargo.toml`
2. Remove `rate_limiting.rs` and `key_management.rs` modules
3. Revert changes to `main.rs`
4. Use the previous random key generation: `SigningKey::random(&mut OsRng)`
