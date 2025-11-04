/// Example: Refresh access token with FITS API
///
/// This example demonstrates how to refresh an access token using a refresh token.
///
/// Run with: `cargo run --example api_auth_refresh`
use fits::api::auth::AuthClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    env_logger::init();

    println!("🔄 FITS API Token Refresh Example");
    println!("==================================");

    let api_url =
        std::env::var("FITS_API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    println!("🔧 API URL: {}", api_url);

    let auth_client = AuthClient::from_env();

    print!("\n🔑 Enter refresh token: ");
    io::stdout().flush()?;
    let mut refresh_token = String::new();
    io::stdin().read_line(&mut refresh_token)?;
    let refresh_token = refresh_token.trim();

    println!("\n🔄 Attempting to refresh token...");
    match auth_client.refresh_token(refresh_token).await {
        Ok(resp) => {
            println!("✅ Token refresh successful!");
            println!("   Success: {}", resp.success);
            if let Some(msg) = &resp.message {
                println!("   Message: {}", msg);
            }
            if let Some(token) = &resp.access_token {
                println!("   Access Token: {}", token);
            }
            if let Some(refresh) = &resp.refresh_token {
                println!("   Refresh Token: {}", refresh);
            }
            if let Some(exp) = &resp.expires_in {
                println!("   Expires In: {} seconds", exp);
            }
            if let Some(role) = &resp.role {
                println!("   Role: {}", role);
            }
            if let Some(ttype) = &resp.token_type {
                println!("   Token Type: {}", ttype);
            }
            if let Some(uid) = &resp.user_id {
                println!("   User ID: {}", uid);
            }
        }
        Err(e) => {
            println!("❌ Token refresh failed: {}", e);
            match e {
                fits::api::auth::AuthError::BadRequest(ref err) => {
                    println!(
                        "   💡 Bad request: {}",
                        err.details.as_deref().unwrap_or(&err.error)
                    );
                }
                fits::api::auth::AuthError::Unauthorized(ref err) => {
                    println!(
                        "   💡 Unauthorized: {}",
                        err.details.as_deref().unwrap_or(&err.error)
                    );
                }
                fits::api::auth::AuthError::UnprocessableEntity(ref err) => {
                    println!(
                        "   💡 Unprocessable entity: {}",
                        err.details.as_deref().unwrap_or(&err.error)
                    );
                }
                fits::api::auth::AuthError::ServerError { status, .. } => {
                    println!("   💡 Server returned HTTP {}", status);
                }
                fits::api::auth::AuthError::Request(_) => {
                    println!(
                        "   💡 Check if the FITS API server is running on {}",
                        api_url
                    );
                }
                _ => {
                    println!("   💡 Check server status and network connectivity");
                }
            }
        }
    }

    println!("\n🏁 Token refresh example completed!");
    Ok(())
}
