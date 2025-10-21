# FITS API Examples

This directory contains example programs demonstrating how to use the FITS API client library. These examples show various aspects of API interaction, error handling, and best practices.

## Prerequisites

Before running these examples, make sure you have:

1. **FITS API Server Running**: The examples assume a FITS API server is running on `http://localhost:8080`
2. **Rust Environment**: Ensure you have Rust installed with Cargo


## Available Examples

### Health Check Example (`api_health_check.rs`)

Demonstrates basic health check API calls with different client configurations.

**Run with:**
```bash
cargo run --example api_health_check
```

**Features demonstrated:**
- Creating API clients with different configurations
- Making health check requests
- Basic error handling
- Using convenience methods like `is_healthy()`



## Usage Patterns

### Basic Health Check
```rust
use fits::api::FitsApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FitsApiClient::dev_client();
    let health = client.health_check().await?;
    println!("API Status: {}", health.status);
    Ok(())
}
```



### Production Configuration
```rust
use fits::api::FitsApiClient;

let client = FitsApiClient::prod_client("https://api.production.example.com".to_string());
```

## Error Handling

The example demonstrates proper error handling for common scenarios:

- **Connection Errors**: Server not running or unreachable
- **HTTP Errors**: Various HTTP status codes
- **Timeout Errors**: Request timeouts and network issues

## Logging

Most examples use `env_logger` for detailed request/response logging. Enable debug logging:

```bash
RUST_LOG=debug cargo run --example api_health_check
```

## Testing Against Different Servers

You can test against different server configurations by modifying the base URL in the examples:

```rust
let config = ApiConfig::new("http://your-server:port".to_string());
```

## Common Issues

1. **Connection Refused**: Make sure the FITS API server is running
2. **Timeout**: Check network connectivity and server load
3. **Invalid Response**: Ensure server is returning expected JSON format

## Building for Production

When integrating these patterns into your production code:

1. Implement proper retry logic with backoff
2. Add comprehensive logging and monitoring
3. Handle all error cases gracefully
4. Consider using connection pooling for high-volume usage

## Additional Resources

- [FITS API Documentation](../README.md)
- [Reqwest Documentation](https://docs.rs/reqwest/)
- [Tokio Documentation](https://docs.rs/tokio/)