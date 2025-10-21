use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;

/// Configuration for the API client
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub base_url: String,
}

impl ApiConfig {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
        }
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
        
        let response = self.client
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

/// Example usage and helper functions
impl FitsApiClient {
    /// Create a client with common development configuration
    pub fn dev_client() -> Self {
        let config = ApiConfig::new("http://localhost:8080".to_string());
        Self::new(config)
    }

    /// Create a client with production configuration
    pub fn prod_client(base_url: String) -> Self {
        let config = ApiConfig::new(base_url);
        Self::new(config)
    }

    /// Create a client using environment variable configuration
    /// 
    /// Loads configuration from FITS_API_BASE_URL environment variable.
    /// This is the recommended way to create clients in applications.
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
        let client = FitsApiClient::dev_client();
        assert_eq!(client.config.base_url, "http://localhost:8080");
        
        let prod_client = FitsApiClient::prod_client("https://api.example.com".to_string());
        assert_eq!(prod_client.config.base_url, "https://api.example.com");
    }
}