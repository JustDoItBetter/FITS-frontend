use reqwest::{Client, Error as ReqwestError};
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration for the API client
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub metrics_secret: Option<String>,
}

impl ApiConfig {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            metrics_secret: None,
        }
    }

    pub fn with_metrics_secret(mut self, secret: String) -> Self {
        self.metrics_secret = Some(secret);
        self
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

    /// Prometheus metrics endpoint - GET /metrics
    /// 
    /// Requires metrics_secret to be configured in ApiConfig
    /// Returns raw Prometheus metrics as string
    pub async fn get_metrics(&self) -> Result<String, ApiError> {
        let metrics_secret = self.config.metrics_secret
            .as_ref()
            .ok_or(ApiError::MissingCredentials("metrics_secret not configured".to_string()))?;

        let url = format!("{}/metrics", self.config.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", metrics_secret))
            .send()
            .await
            .map_err(ApiError::Request)?;

        if !response.status().is_success() {
            return Err(ApiError::Http {
                status: response.status().as_u16(),
                message: format!("Failed to fetch metrics: {}", response.status()),
            });
        }

        let metrics_text = response
            .text()
            .await
            .map_err(ApiError::Request)?;
            
        Ok(metrics_text)
    }
}

/// Custom error types for the API client
#[derive(Debug)]
pub enum ApiError {
    Request(ReqwestError),
    MissingCredentials(String),
    Http { status: u16, message: String },
    Serialization(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Request(e) => write!(f, "Request error: {}", e),
            ApiError::MissingCredentials(msg) => write!(f, "Missing credentials: {}", msg),
            ApiError::Http { status, message } => write!(f, "HTTP error {}: {}", status, message),
            ApiError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
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

    /// Get metrics as a parsed HashMap for easier processing
    /// Note: This is a basic parser - for production use consider a proper Prometheus client
    pub async fn get_metrics_parsed(&self) -> Result<HashMap<String, String>, ApiError> {
        let metrics_text = self.get_metrics().await?;
        let mut metrics = HashMap::new();
        
        for line in metrics_text.lines() {
            // Skip comments and empty lines
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            // Basic parsing: metric_name value
            if let Some((key, value)) = line.split_once(' ') {
                metrics.insert(key.to_string(), value.to_string());
            }
        }
        
        Ok(metrics)
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
    pub fn prod_client(base_url: String, metrics_secret: Option<String>) -> Self {
        let mut config = ApiConfig::new(base_url);
        if let Some(secret) = metrics_secret {
            config = config.with_metrics_secret(secret);
        }
        Self::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        // This is a basic test structure - you'll need a running server to test properly
        let config = ApiConfig::new("http://localhost:8080".to_string());
        let client = FitsApiClient::new(config);
        
        // Uncomment when you have a test server running
        // let result = client.health_check().await;
        // assert!(result.is_ok());
    }

    #[tokio::test] 
    async fn test_metrics_without_secret() {
        let config = ApiConfig::new("http://localhost:8080".to_string());
        let client = FitsApiClient::new(config);
        
        let result = client.get_metrics().await;
        assert!(matches!(result, Err(ApiError::MissingCredentials(_))));
    }

    #[test]
    fn test_config_creation() {
        let config = ApiConfig::new("http://example.com".to_string())
            .with_metrics_secret("secret123".to_string());
        
        assert_eq!(config.base_url, "http://example.com");
        assert_eq!(config.metrics_secret, Some("secret123".to_string()));
    }

    #[test]
    fn test_client_creation() {
        let client = FitsApiClient::dev_client();
        assert_eq!(client.config.base_url, "http://localhost:8080");
        
        let prod_client = FitsApiClient::prod_client(
            "https://api.example.com".to_string(),
            Some("prod_secret".to_string())
        );
        assert_eq!(prod_client.config.base_url, "https://api.example.com");
        assert_eq!(prod_client.config.metrics_secret, Some("prod_secret".to_string()));
    }
}