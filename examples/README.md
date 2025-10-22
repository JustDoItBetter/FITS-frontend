# FITS API Examples

This directory contains example programs demonstrating how to use the FITS API client library. These examples show various aspects of API interaction, error handling, and best practices.

## Prerequisites

Before running these examples, make sure you have:

1. **FITS API Server Running**: The examples assume a FITS API server is running on `http://localhost:8080`
2. **Rust Environment**: Ensure you have Rust installed with Cargo

## Quick Start

```bash
# Run the health check example
cargo run --example api_health_check

# Run authentication examples  
cargo run --example api_auth              # Basic login test
cargo run --example api_auth_logout       # Complete login/logout flow

# Enable detailed logging for any example
RUST_LOG=debug cargo run --example api_health_check
```

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

### Authentication Examples

#### Basic Authentication (`api_auth.rs`)

Interactive example that prompts for username and password to test login functionality.

**Run with:**
```bash
cargo run --example api_auth
```

**Features demonstrated:**
- Login with username/password credentials
- Error handling for invalid credentials
- Server error responses
- Credential verification convenience method

#### Complete Authentication Flow (`api_auth_logout.rs`)

Interactive example demonstrating complete login and logout flow.

**Run with:**
```bash
cargo run --example api_auth_logout
```

**Features demonstrated:**
- Complete authentication workflow (login → logout)
- Interactive credential input
- Login response handling
- Logout response handling
- Error handling for both operations



## Usage Patterns

### Basic Health Check
```rust
use fits::api::FitsApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables or .env file
    let client = FitsApiClient::from_env();
    let health = client.health_check().await?;
    println!("API Status: {}", health.status);
    Ok(())
}
```



### Custom Configuration
```rust
use fits::api::{FitsApiClient, ApiConfig};

// Create client with custom configuration
let config = ApiConfig::new("https://api.yourdomain.com".to_string());
let client = FitsApiClient::new(config);
```

### Authentication
```rust
use fits::api::auth::AuthClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables or .env file
    let auth_client = AuthClient::from_env();
    
    // Login with credentials
    let login_response = auth_client.login("username", "password").await?;
    
    if login_response.success {
        println!("Login successful!");
        if let Some(token) = login_response.token {
            println!("Token: {}", token);
        }
    }
    
    Ok(())
}
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