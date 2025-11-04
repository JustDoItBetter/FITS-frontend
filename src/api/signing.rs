use reqwest::{Client, Error as ReqwestError, multipart};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Upload record response structure
#[derive(Debug, Deserialize, Serialize)]
pub struct UploadRecord {
    pub upload_id: String,
    pub student_uuid: String,
    pub file_name: String,
    pub file_size: i64,
    pub content_hash: String,
    pub uploaded_at: i64,
}

/// Success response wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

/// Error response structure
#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Option<String>,
    pub code: u16,
}

/// Signing-related errors
#[derive(Debug)]
pub enum SigningError {
    Request(ReqwestError),
    Unauthorized(ErrorResponse),
    BadRequest(ErrorResponse),
    NotImplemented(ErrorResponse),
    ServerError { status: u16, message: String },
    ParseError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for SigningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SigningError::Request(e) => write!(f, "Request error: {}", e),
            SigningError::Unauthorized(err) => write!(
                f,
                "Unauthorized: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            SigningError::BadRequest(err) => write!(
                f,
                "Bad request: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            SigningError::NotImplemented(err) => write!(
                f,
                "Not implemented: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            SigningError::ServerError { status, message } => {
                write!(f, "Server error {}: {}", status, message)
            }
            SigningError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SigningError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for SigningError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SigningError::Request(e) => Some(e),
            SigningError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ReqwestError> for SigningError {
    fn from(error: ReqwestError) -> Self {
        SigningError::Request(error)
    }
}

impl From<std::io::Error> for SigningError {
    fn from(error: std::io::Error) -> Self {
        SigningError::IoError(error)
    }
}

/// Signing client for FITS API signing operations
#[derive(Debug)]
pub struct SigningClient {
    client: Client,
    base_url: String,
    access_token: Option<String>,
}

impl SigningClient {
    /// Create a new signing client
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            access_token: None,
        }
    }

    /// Create signing client from environment variables
    pub fn from_env() -> Self {
        let base_url = std::env::var("FITS_API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self::new(base_url)
    }

    /// Set the access token for authenticated requests
    pub fn with_token(mut self, token: String) -> Self {
        self.access_token = Some(token);
        self
    }

    /// Set the access token for authenticated requests (mutable)
    pub fn set_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    /// Upload a parquet file containing student data
    /// POST /api/v1/signing/upload
    pub async fn upload_parquet<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<UploadRecord, SigningError> {
        let url = format!("{}/api/v1/signing/upload", self.base_url);

        // Read file content
        let file_content = tokio::fs::read(file_path.as_ref()).await?;
        let file_name = file_path
            .as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload.parquet")
            .to_string();

        // Create multipart form
        let part = multipart::Part::bytes(file_content)
            .file_name(file_name)
            .mime_str("application/octet-stream")
            .map_err(|e| SigningError::ParseError(format!("Failed to create multipart: {}", e)))?;

        let form = multipart::Form::new().part("file", part);

        // Build request with authorization
        let mut request = self.client.post(&url).multipart(form);

        if let Some(token) = &self.access_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let success_response = response
                .json::<SuccessResponse<UploadRecord>>()
                .await
                .map_err(|e| {
                    SigningError::ParseError(format!("Failed to parse upload response: {}", e))
                })?;

            success_response.data.ok_or_else(|| {
                SigningError::ParseError("Upload response missing data field".to_string())
            })
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                SigningError::ParseError(format!("Failed to parse error response: {}", e))
            })?;

            match status.as_u16() {
                400 => Err(SigningError::BadRequest(error_response)),
                401 => Err(SigningError::Unauthorized(error_response)),
                501 => Err(SigningError::NotImplemented(error_response)),
                _ => Err(SigningError::ServerError {
                    status: status.as_u16(),
                    message: error_response.error,
                }),
            }
        }
    }

    /// Get pending sign requests as a parquet file
    /// GET /api/v1/signing/sign_requests
    pub async fn get_sign_requests(&self) -> Result<Vec<u8>, SigningError> {
        let url = format!("{}/api/v1/signing/sign_requests", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.access_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                SigningError::ParseError(format!("Failed to parse error response: {}", e))
            })?;

            match status.as_u16() {
                401 => Err(SigningError::Unauthorized(error_response)),
                501 => Err(SigningError::NotImplemented(error_response)),
                _ => Err(SigningError::ServerError {
                    status: status.as_u16(),
                    message: error_response.error,
                }),
            }
        }
    }

    /// Upload signed requests as a parquet file
    /// POST /api/v1/signing/sign_uploads
    pub async fn upload_signed_requests<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<(), SigningError> {
        let url = format!("{}/api/v1/signing/sign_uploads", self.base_url);

        // Read file content
        let file_content = tokio::fs::read(file_path.as_ref()).await?;
        let file_name = file_path
            .as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("signed.parquet")
            .to_string();

        // Create multipart form
        let part = multipart::Part::bytes(file_content)
            .file_name(file_name)
            .mime_str("application/octet-stream")
            .map_err(|e| SigningError::ParseError(format!("Failed to create multipart: {}", e)))?;

        let form = multipart::Form::new().part("file", part);

        // Build request with authorization
        let mut request = self.client.post(&url).multipart(form);

        if let Some(token) = &self.access_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                SigningError::ParseError(format!("Failed to parse error response: {}", e))
            })?;

            match status.as_u16() {
                400 => Err(SigningError::BadRequest(error_response)),
                401 => Err(SigningError::Unauthorized(error_response)),
                501 => Err(SigningError::NotImplemented(error_response)),
                _ => Err(SigningError::ServerError {
                    status: status.as_u16(),
                    message: error_response.error,
                }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_upload_record_deserialization() {
        let data = json!({
            "upload_id": "upload-123-456",
            "student_uuid": "550e8400-e29b-41d4-a716-446655440000",
            "file_name": "report.parquet",
            "file_size": 1024000,
            "content_hash": "sha256:abc123...",
            "uploaded_at": 1727697600000i64
        });
        let record: UploadRecord = serde_json::from_value(data).unwrap();
        assert_eq!(record.upload_id, "upload-123-456");
        assert_eq!(record.student_uuid, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(record.file_name, "report.parquet");
        assert_eq!(record.file_size, 1024000);
        assert_eq!(record.content_hash, "sha256:abc123...");
        assert_eq!(record.uploaded_at, 1727697600000i64);
    }

    #[test]
    fn test_success_response_deserialization() {
        let data = json!({
            "success": true,
            "message": "operation successful",
            "data": {
                "upload_id": "upload-123-456",
                "student_uuid": "550e8400-e29b-41d4-a716-446655440000",
                "file_name": "report.parquet",
                "file_size": 1024000,
                "content_hash": "sha256:abc123...",
                "uploaded_at": 1727697600000i64
            }
        });
        let resp: SuccessResponse<UploadRecord> = serde_json::from_value(data).unwrap();
        assert!(resp.success);
        assert_eq!(resp.message, Some("operation successful".to_string()));
        assert!(resp.data.is_some());
    }

    #[test]
    fn test_error_response_deserialization() {
        let data = json!({
            "success": false,
            "error": "invalid request",
            "details": "file validation failed",
            "code": 400
        });
        let resp: ErrorResponse = serde_json::from_value(data).unwrap();
        assert!(!resp.success);
        assert_eq!(resp.error, "invalid request");
        assert_eq!(resp.details, Some("file validation failed".to_string()));
        assert_eq!(resp.code, 400);
    }

    #[test]
    fn test_signing_client_creation() {
        let client = SigningClient::new("http://example.com".to_string());
        assert_eq!(client.base_url, "http://example.com");
        assert!(client.access_token.is_none());
    }

    #[test]
    fn test_signing_client_with_token() {
        let client = SigningClient::new("http://example.com".to_string())
            .with_token("test_token".to_string());
        assert_eq!(client.access_token, Some("test_token".to_string()));
    }

    #[test]
    fn test_signing_client_from_env() {
        let client = SigningClient::from_env();
        assert_eq!(client.base_url, "http://localhost:8080");
    }
}
