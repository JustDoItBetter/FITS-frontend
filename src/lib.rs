// FITS - Library exports for examples and external usage
// SPDX-License-Identifier: GPL-3.0-only

pub mod api;

// Re-export commonly used types for convenience
pub use api::{FitsApiClient, ApiConfig, ApiError, HealthResponse, AuthClient, LoginRequest, LoginResponse, LogoutResponse, UserInfo, AuthError};