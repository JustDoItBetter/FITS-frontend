use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};

/// Invitation information response
#[derive(Debug, Deserialize, Serialize)]
pub struct InvitationResponse {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub expires_at: String,
    pub teacher_uuid: Option<String>,
    pub department: Option<String>,
}

/// Complete invitation request
#[derive(Debug, Serialize)]
pub struct CompleteInvitationRequest {
    pub username: String,
    pub password: String,
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

/// Invitation-related errors
#[derive(Debug)]
pub enum InvitationError {
    Request(ReqwestError),
    BadRequest(ErrorResponse),
    Unauthorized(ErrorResponse),
    NotFound(ErrorResponse),
    Conflict(ErrorResponse),
    UnprocessableEntity(ErrorResponse),
    ServerError { status: u16, message: String },
    ParseError(String),
}

impl std::fmt::Display for InvitationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvitationError::Request(e) => write!(f, "Request error: {}", e),
            InvitationError::BadRequest(err) => write!(
                f,
                "Bad request: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            InvitationError::Unauthorized(err) => write!(
                f,
                "Unauthorized: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            InvitationError::NotFound(err) => write!(
                f,
                "Not found: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            InvitationError::Conflict(err) => write!(
                f,
                "Conflict: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            InvitationError::UnprocessableEntity(err) => write!(
                f,
                "Unprocessable entity: {}",
                err.details.as_deref().unwrap_or(&err.error)
            ),
            InvitationError::ServerError { status, message } => {
                write!(f, "Server error {}: {}", status, message)
            }
            InvitationError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for InvitationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InvitationError::Request(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ReqwestError> for InvitationError {
    fn from(error: ReqwestError) -> Self {
        InvitationError::Request(error)
    }
}

/// Invitation client for FITS API invitation operations
#[derive(Debug)]
pub struct InvitationClient {
    client: Client,
    base_url: String,
}

impl InvitationClient {
    /// Create a new invitation client
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Create invitation client from environment variables
    pub fn from_env() -> Self {
        let base_url = std::env::var("FITS_API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self::new(base_url)
    }

    /// Get invitation details by token
    /// GET /api/v1/invite/{token}
    pub async fn get_invitation(&self, token: &str) -> Result<InvitationResponse, InvitationError> {
        let url = format!("{}/api/v1/invite/{}", self.base_url, token);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if status.is_success() {
            let success_response = response
                .json::<SuccessResponse<InvitationResponse>>()
                .await
                .map_err(|e| {
                    InvitationError::ParseError(format!(
                        "Failed to parse invitation response: {}",
                        e
                    ))
                })?;

            success_response.data.ok_or_else(|| {
                InvitationError::ParseError("Invitation response missing data field".to_string())
            })
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                InvitationError::ParseError(format!("Failed to parse error response: {}", e))
            })?;

            match status.as_u16() {
                400 => Err(InvitationError::BadRequest(error_response)),
                401 => Err(InvitationError::Unauthorized(error_response)),
                404 => Err(InvitationError::NotFound(error_response)),
                _ => Err(InvitationError::ServerError {
                    status: status.as_u16(),
                    message: error_response.error,
                }),
            }
        }
    }

    /// Complete invitation with username and password
    /// POST /api/v1/invite/{token}/complete
    pub async fn complete_invitation(
        &self,
        token: &str,
        username: &str,
        password: &str,
    ) -> Result<(), InvitationError> {
        let url = format!("{}/api/v1/invite/{}/complete", self.base_url, token);

        let request_body = CompleteInvitationRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client.post(&url).json(&request_body).send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let error_response = response.json::<ErrorResponse>().await.map_err(|e| {
                InvitationError::ParseError(format!("Failed to parse error response: {}", e))
            })?;

            match status.as_u16() {
                400 => Err(InvitationError::BadRequest(error_response)),
                409 => Err(InvitationError::Conflict(error_response)),
                422 => Err(InvitationError::UnprocessableEntity(error_response)),
                _ => Err(InvitationError::ServerError {
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
    fn test_invitation_response_deserialization() {
        let data = json!({
            "email": "max@example.com",
            "first_name": "Max",
            "last_name": "Mustermann",
            "role": "student",
            "expires_at": "2025-10-25T12:00:00Z",
            "teacher_uuid": "550e8400-e29b-41d4-a716-446655440000",
            "department": null
        });
        let resp: InvitationResponse = serde_json::from_value(data).unwrap();
        assert_eq!(resp.email, "max@example.com");
        assert_eq!(resp.first_name, "Max");
        assert_eq!(resp.last_name, "Mustermann");
        assert_eq!(resp.role, "student");
        assert_eq!(resp.expires_at, "2025-10-25T12:00:00Z");
        assert_eq!(
            resp.teacher_uuid,
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
        assert_eq!(resp.department, None);
    }

    #[test]
    fn test_invitation_response_teacher_deserialization() {
        let data = json!({
            "email": "anna@example.com",
            "first_name": "Anna",
            "last_name": "Schmidt",
            "role": "teacher",
            "expires_at": "2025-10-25T12:00:00Z",
            "teacher_uuid": null,
            "department": "IT"
        });
        let resp: InvitationResponse = serde_json::from_value(data).unwrap();
        assert_eq!(resp.email, "anna@example.com");
        assert_eq!(resp.role, "teacher");
        assert_eq!(resp.teacher_uuid, None);
        assert_eq!(resp.department, Some("IT".to_string()));
    }

    #[test]
    fn test_complete_invitation_request_serialization() {
        let req = CompleteInvitationRequest {
            username: "max.mustermann".to_string(),
            password: "SecurePassword123!".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("max.mustermann"));
        assert!(json.contains("SecurePassword123!"));
    }

    #[test]
    fn test_success_response_deserialization() {
        let data = json!({
            "success": true,
            "message": "Invitation retrieved successfully",
            "data": {
                "email": "max@example.com",
                "first_name": "Max",
                "last_name": "Mustermann",
                "role": "student",
                "expires_at": "2025-10-25T12:00:00Z",
                "teacher_uuid": "550e8400-e29b-41d4-a716-446655440000",
                "department": null
            }
        });
        let resp: SuccessResponse<InvitationResponse> = serde_json::from_value(data).unwrap();
        assert!(resp.success);
        assert_eq!(
            resp.message,
            Some("Invitation retrieved successfully".to_string())
        );
        assert!(resp.data.is_some());
    }

    #[test]
    fn test_error_response_deserialization() {
        let data = json!({
            "success": false,
            "error": "invalid request",
            "details": "invitation token expired",
            "code": 400
        });
        let resp: ErrorResponse = serde_json::from_value(data).unwrap();
        assert!(!resp.success);
        assert_eq!(resp.error, "invalid request");
        assert_eq!(resp.details, Some("invitation token expired".to_string()));
        assert_eq!(resp.code, 400);
    }

    #[test]
    fn test_invitation_client_creation() {
        let client = InvitationClient::new("http://example.com".to_string());
        assert_eq!(client.base_url, "http://example.com");
    }

    #[test]
    fn test_invitation_client_from_env() {
        let client = InvitationClient::from_env();
        assert_eq!(client.base_url, "http://localhost:8080");
    }
}
