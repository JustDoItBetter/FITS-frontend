/// Refresh token request structure
#[derive(Debug, Serialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Refresh token response structure (same as login)
pub type RefreshTokenResponse = LoginResponse;

use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};

/// Login request structure
#[derive(Debug, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response structure for successful authentication
#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u32>,
    pub role: Option<String>,
    pub token_type: Option<String>,
    pub user_id: Option<String>,
    pub user: Option<UserInfo>,
}

#[derive(Debug, Deserialize)]
pub struct LogoutResponse {
    pub data: Option<String>,
    pub message: Option<String>,
    pub success: bool,
}

/// User information returned in login response
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
}

/// Error response structure
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Option<String>,
    pub code: u16,
}

/// Authentication-related errors
#[derive(Debug)]
pub enum AuthError {
    Request(ReqwestError),
    InvalidCredentials(String),
    BadRequest(ErrorResponse),
    Unauthorized(ErrorResponse),
    UnprocessableEntity(ErrorResponse),
    ServerError { status: u16, message: String },
    ParseError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::Request(e) => write!(f, "Request error: {}", e),
            AuthError::InvalidCredentials(msg) => write!(f, "Invalid credentials: {}", msg),
            AuthError::BadRequest(err) => write!(f, "Bad request: {}", err.details.as_deref().unwrap_or(&err.error)),
            AuthError::Unauthorized(err) => write!(f, "Unauthorized: {}", err.details.as_deref().unwrap_or(&err.error)),
            AuthError::UnprocessableEntity(err) => write!(f, "Unprocessable entity: {}", err.details.as_deref().unwrap_or(&err.error)),
            AuthError::ServerError { status, message } => {
                write!(f, "Server error {}: {}", status, message)
            }
            AuthError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuthError::Request(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ReqwestError> for AuthError {
    fn from(error: ReqwestError) -> Self {
        AuthError::Request(error)
    }
}

/// Authentication client for FITS API
#[derive(Debug)]
pub struct AuthClient {
    client: Client,
    base_url: String,
}

impl AuthClient {
    /// Refresh access token using a refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshTokenResponse, AuthError> {
        let req = RefreshTokenRequest {
            refresh_token: refresh_token.to_string(),
        };
        let url = format!("{}/api/v1/auth/refresh", self.base_url);
        let response = self.client.post(&url).json(&req).send().await?;
        let status = response.status();
        if status.is_success() {
            let refresh_response = response.json::<RefreshTokenResponse>().await.map_err(|e| {
                AuthError::ParseError(format!("Failed to parse refresh response: {}", e))
            })?;
            Ok(refresh_response)
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                AuthError::ParseError(format!("Failed to parse error response: {}", e))
            })?;
            match status.as_u16() {
                400 => Err(AuthError::BadRequest(error_response)),
                401 => Err(AuthError::Unauthorized(error_response)),
                422 => Err(AuthError::UnprocessableEntity(error_response)),
                _ => Err(AuthError::ServerError {
                    status: status.as_u16(),
                    message: error_response.error,
                }),
            }
        }
    }
    /// Create a new authentication client
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Create authentication client from environment variables
    pub fn from_env() -> Self {
        let base_url = std::env::var("FITS_API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self::new(base_url)
    }

    /// Login with username and password
    ///
    /// Returns the login response with token and user information on success
    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResponse, AuthError> {
        let login_request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let url = format!("{}/api/v1/auth/login", self.base_url);

        let response = self.client.post(&url).json(&login_request).send().await?;

        let status = response.status();

        if status.is_success() {
            // Parse successful login response
            let login_response = response.json::<LoginResponse>().await.map_err(|e| {
                AuthError::ParseError(format!("Failed to parse login response: {}", e))
            })?;

            Ok(login_response)
        } else {
            // Parse error response
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                AuthError::ParseError(format!("Failed to parse error response: {}", e))
            })?;

            match status.as_u16() {
                401 => Err(AuthError::InvalidCredentials(
                    error_response.details.unwrap_or(error_response.error),
                )),
                _ => Err(AuthError::ServerError {
                    status: status.as_u16(),
                    message: error_response.error,
                }),
            }
        }
    }
    pub async fn logout(&self) -> Result<LogoutResponse, AuthError> {
        let url = format!("{}/api/v1/auth/logout", self.base_url);

        let response = self.client.post(url).send().await?;

        let status = response.status();

        if status.is_success() {
            let logut_response = response.json::<LogoutResponse>().await.map_err(|e| {
                AuthError::ParseError(format!("Failed to parse login reespnse: {}", e))
            })?;
            Ok(logut_response)
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                AuthError::ParseError(format!("Failed to parse login reespnse: {}", e))
            })?;
            Err(AuthError::ServerError {
                status: status.as_u16(),
                message: error_response.error,
            })
        }
    }

    /// Convenience method to check if credentials are valid
    pub async fn verify_credentials(&self, username: &str, password: &str) -> bool {
        self.login(username, password).await.is_ok()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_refresh_token_request_serialization() {
        let req = RefreshTokenRequest {
            refresh_token: "my_refresh_token".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("my_refresh_token"));
    }

    #[test]
    fn test_refresh_token_response_deserialization() {
        let data = json!({
            "success": true,
            "message": "operation successful",
            "access_token": "eyJhbGc...",
            "refresh_token": "eyJhbGc...",
            "expires_in": 3600,
            "role": "student",
            "token_type": "Bearer",
            "user_id": "550e8400-e29b-41d4-a716-446655440000"
        });
        let resp: RefreshTokenResponse = serde_json::from_value(data).unwrap();
        assert_eq!(resp.success, true);
        assert_eq!(resp.access_token.as_deref(), Some("eyJhbGc..."));
        assert_eq!(resp.refresh_token.as_deref(), Some("eyJhbGc..."));
        assert_eq!(resp.expires_in, Some(3600));
        assert_eq!(resp.role.as_deref(), Some("student"));
        assert_eq!(resp.token_type.as_deref(), Some("Bearer"));
        assert_eq!(resp.user_id.as_deref(), Some("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_refresh_token_error_response_deserialization() {
        let data = json!({
            "code": 400,
            "details": "field validation failed",
            "error": "invalid request",
            "success": false
        });
        let resp: ErrorResponse = serde_json::from_value(data).unwrap();
        assert_eq!(resp.code, 400);
        assert_eq!(resp.details.as_deref(), Some("field validation failed"));
        assert_eq!(resp.error, "invalid request");
        assert_eq!(resp.success, false);
    }

    #[test]
    fn test_login_request_serialization() {
        let login_request = LoginRequest {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        let json = serde_json::to_string(&login_request).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("testpass"));
    }

    #[test]
    fn test_auth_client_creation() {
        let client = AuthClient::new("http://example.com".to_string());
        assert_eq!(client.base_url, "http://example.com");
    }

    #[test]
    fn test_auth_client_from_env() {
        // Test creating client from environment (will use default if not set)
        let client = AuthClient::from_env();
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"success":false,"error":"Unauthorized","details":"invalid credentials","code":401}"#;
        let error_response: ErrorResponse = serde_json::from_str(json).unwrap();

        assert!(!error_response.success);
        assert_eq!(error_response.error, "Unauthorized");
        assert_eq!(
            error_response.details,
            Some("invalid credentials".to_string())
        );
        assert_eq!(error_response.code, 401);
    }

    #[test]
    fn test_logout_response_deserialization() {
        let json = r#"{"success":true,"message":"Logged out successfully","data":"session_cleared"}"#;
        let logout_response: LogoutResponse = serde_json::from_str(json).unwrap();

        assert!(logout_response.success);
        assert_eq!(logout_response.message, Some("Logged out successfully".to_string()));
        assert_eq!(logout_response.data, Some("session_cleared".to_string()));
    }

    #[test]
    fn test_logout_response_minimal() {
        let json = r#"{"success":true}"#;
        let logout_response: LogoutResponse = serde_json::from_str(json).unwrap();

        assert!(logout_response.success);
        assert_eq!(logout_response.message, None);
        assert_eq!(logout_response.data, None);
    }

    #[test]
    fn test_logout_response_with_error() {
        let json = r#"{"success":false,"message":"Session already expired"}"#;
        let logout_response: LogoutResponse = serde_json::from_str(json).unwrap();

        assert!(!logout_response.success);
        assert_eq!(logout_response.message, Some("Session already expired".to_string()));
        assert_eq!(logout_response.data, None);
    }
}
