/// API client module for FITS backend integration
/// 
/// This module provides reqwest-based HTTP client functions for interacting
/// with the FITS API backend. Currently supports the implemented endpoints:
/// 
/// - Health check (GET /health)
/// - Prometheus metrics (GET /metrics with Authorization)
/// 
/// # Example Usage
/// 
/// ```rust
/// use fits::api::handler::{FitsApiClient, ApiConfig};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client for development
///     let client = FitsApiClient::dev_client();
///     
///     // Check API health
///     let health = client.health_check().await?;
///     println!("API Status: {}", health.status);
///     
///     // For metrics, you need to configure the secret
///     let config = ApiConfig::new("http://localhost:8080".to_string())
///         .with_metrics_secret("your_secret".to_string());
///     let client = FitsApiClient::new(config);
///     let metrics = client.get_metrics().await?;
///     
///     Ok(())
/// }
/// ```

pub mod handler;

// Re-export main types for convenience
pub use handler::{FitsApiClient, ApiConfig, ApiError, HealthResponse};