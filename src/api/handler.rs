use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;

/// Configuration for the API client
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub base_url: String,
}

impl ApiConfig {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    /// Create configuration from environment variables
    ///
    /// Looks for FITS_API_BASE_URL environment variable.
    /// Falls back to http://localhost:8080 if not set.
    pub fn from_env() -> Self {
        let base_url = std::env::var("FITS_API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        Self::new(base_url)
    }
}

/// Health check response structure
#[derive(Deserialize, Debug, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub time: String,
}

/// API client for FITS backend
#[derive(Debug)]
pub struct FitsApiClient {
    client: Client,
    config: ApiConfig,
}

impl FitsApiClient {
    /// Create a new API client instance
    pub fn new(config: ApiConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Health check endpoint - GET /health
    ///
    /// Returns the API health status and current time
    pub async fn health_check(&self) -> Result<HealthResponse, ReqwestError> {
        let url = format!("{}/health", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<HealthResponse>()
            .await?;

        Ok(response)
    }
}

/// Custom error types for the API client
#[derive(Debug)]
pub enum ApiError {
    Request(ReqwestError),
    Http { status: u16, message: String },
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Request(e) => write!(f, "Request error: {}", e),
            ApiError::Http { status, message } => write!(f, "HTTP error {}: {}", status, message),
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApiError::Request(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ReqwestError> for ApiError {
    fn from(error: ReqwestError) -> Self {
        ApiError::Request(error)
    }
}

/// Convenience functions for common API operations
impl FitsApiClient {
    /// Check if the API is healthy and reachable
    pub async fn is_healthy(&self) -> bool {
        match self.health_check().await {
            Ok(response) => response.status == "ok",
            Err(_) => false,
        }
    }
}

/// Additional constructors
impl FitsApiClient {
    /// Create a client using environment variable configuration
    ///
    /// Loads configuration from FITS_API_BASE_URL environment variable.
    /// Falls back to http://localhost:8080 if not set.
    pub fn from_env() -> Self {
        let config = ApiConfig::from_env();
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ApiConfig::new("http://example.com".to_string());
        assert_eq!(config.base_url, "http://example.com");
    }

    #[test]
    fn test_client_creation() {
        // Test creating client with explicit config
        let config = ApiConfig::new("https://api.example.com".to_string());
        let client = FitsApiClient::new(config);
        assert_eq!(client.config.base_url, "https://api.example.com");
    }

    #[test]
    fn test_from_env() {
        // Test creating client from environment (will use default if not set)
        let env_client = FitsApiClient::from_env();
        // Should use default if FITS_API_BASE_URL is not set
        assert_eq!(env_client.config.base_url, "http://localhost:8080");
    }
}

