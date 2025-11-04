/// Example: Authentication with FITS API
///
/// This example demonstrates how to authenticate with the FITS API using
/// username and password credentials.
///
/// Configuration is loaded from environment variables or .env file:
/// - FITS_API_BASE_URL: The base URL of the FITS API (default: http://localhost:8080)
/// - RUST_LOG: Logging level (default: info)
///
/// Run with: `cargo run --example api_auth`
use fits::api::auth::AuthClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    let _ = dotenvy::dotenv();

    // Initialize logger
    env_logger::init();

    println!("🔐 FITS API Authentication Example");
    println!("==================================");

    // Show current configuration
    let api_url =
        std::env::var("FITS_API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    println!("🔧 Configuration:");
    println!("   API URL: {}", api_url);
    println!(
        "   Log Level: {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    );

    // Create authentication client
    println!("\n📋 Creating authentication client...");
    let auth_client = AuthClient::from_env();

    // Get credentials from user input
    print!("\n👤 Enter username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    print!("🔑 Enter password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    println!("\n🔄 Attempting to login...");

    // Attempt login
    match auth_client.login(username, password).await {
        Ok(login_response) => {
            println!("✅ Login successful!");
            println!("   Success: {}", login_response.success);
            if let Some(msg) = &login_response.message {
                println!("   Message: {}", msg);
            }
            if let Some(token) = &login_response.access_token {
                println!("   Access Token: {}", token);
            }
            if let Some(refresh) = &login_response.refresh_token {
                println!("   Refresh Token: {}", refresh);
            }
            if let Some(exp) = &login_response.expires_in {
                println!("   Expires In: {} seconds", exp);
            }
            if let Some(role) = &login_response.role {
                println!("   Role: {}", role);
            }
            if let Some(ttype) = &login_response.token_type {
                println!("   Token Type: {}", ttype);
            }
            if let Some(uid) = &login_response.user_id {
                println!("   User ID: {}", uid);
            }
            if let Some(user) = &login_response.user {
                println!("   User Info:");
                println!("      ID: {}", user.id);
                println!("      Username: {}", user.username);
                if let Some(email) = &user.email {
                    println!("      Email: {}", email);
                }
            }
        }
        Err(e) => {
            println!("❌ Login failed: {}", e);
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
                fits::api::auth::AuthError::InvalidCredentials(_) => {
                    println!("   💡 Please check your username and password");
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

    // Test the convenience method
    println!("\n📋 Testing credential verification...");
    let is_valid = auth_client.verify_credentials(username, password).await;
    println!("   Credentials valid: {}", is_valid);

    println!("\n🏁 Authentication example completed!");

    Ok(())
}
