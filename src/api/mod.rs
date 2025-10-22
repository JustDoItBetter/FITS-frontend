pub mod auth;
/// API client module for FITS backend integration
///
/// This module provides reqwest-based HTTP client functions for interacting
/// with the FITS API backend. Currently supports the implemented endpoints:
///
/// - Health check (GET /health)
/// - Authentication (POST /api/v1/auth/login)
///
/// # Example Usage
///
/// ## Health Check
/// ```rust
/// use fits::api::FitsApiClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client using environment configuration
///     let client = FitsApiClient::from_env();
///     
///     // Check API health
///     let health = client.health_check().await?;
///     println!("API Status: {}", health.status);
///     
///     Ok(())
/// }
/// ```
///
/// ## Authentication
/// ```rust,no_run
/// use fits::api::auth::AuthClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create authentication client
///     let auth_client = AuthClient::from_env();
///     
///     // Login with credentials (this would require valid credentials)
///     match auth_client.login("username", "password").await {
///         Ok(login_response) => {
///             if login_response.success {
///                 println!("Login successful! Token: {:?}", login_response.token);
///             }
///         }
///         Err(e) => {
///             println!("Login failed: {}", e);
///         }
///     }
///     
///     Ok(())
/// }
/// ```
pub mod handler;

// Re-export main types for convenience
pub use handler::{FitsApiClient, ApiConfig, ApiError, HealthResponse};
pub use auth::{AuthClient, LoginRequest, LoginResponse, LogoutResponse, UserInfo, AuthError};
