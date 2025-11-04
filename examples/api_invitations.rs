/// Example: FITS API Invitation Acceptance
///
/// This example demonstrates how to accept an invitation:
/// 1. Get invitation details using the token
/// 2. Complete the invitation with username and password
///
/// Configuration is loaded from environment variables or .env file:
/// - FITS_API_BASE_URL: The base URL of the FITS API (default: http://localhost:8080)
/// - RUST_LOG: Logging level (default: info)
///
/// Run with: `cargo run --example api_invitations`
use fits::api::invitations::InvitationClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    let _ = dotenvy::dotenv();

    // Initialize logger
    env_logger::init();

    println!("📨 FITS API Invitation Acceptance Example");
    println!("==========================================");

    // Show current configuration
    let api_url =
        std::env::var("FITS_API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    println!("🔧 Configuration:");
    println!("   API URL: {}", api_url);
    println!(
        "   Log Level: {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    );

    // Create invitation client
    println!("\n📋 Creating invitation client...");
    let invitation_client = InvitationClient::from_env();

    // Get invitation token
    print!("\n🎫 Enter invitation token: ");
    io::stdout().flush()?;
    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim();

    if token.is_empty() {
        println!("❌ No token provided, exiting...");
        return Ok(());
    }

    // Get invitation details
    println!("\n🔄 Fetching invitation details...");
    let invitation = match invitation_client.get_invitation(token).await {
        Ok(inv) => {
            println!("✅ Invitation found!");
            println!("   Email: {}", inv.email);
            println!("   Name: {} {}", inv.first_name, inv.last_name);
            println!("   Role: {}", inv.role);
            println!("   Expires: {}", inv.expires_at);
            if let Some(teacher_uuid) = &inv.teacher_uuid {
                println!("   Teacher UUID: {}", teacher_uuid);
            }
            if let Some(department) = &inv.department {
                println!("   Department: {}", department);
            }
            inv
        }
        Err(e) => {
            println!("❌ Failed to get invitation: {}", e);
            match e {
                fits::api::invitations::InvitationError::NotFound(_) => {
                    println!("   💡 The invitation token may be invalid or expired");
                }
                fits::api::invitations::InvitationError::BadRequest(_) => {
                    println!("   💡 Check the invitation token format");
                }
                fits::api::invitations::InvitationError::Request(_) => {
                    println!(
                        "   💡 Check if the FITS API server is running on {}",
                        api_url
                    );
                }
                _ => {
                    println!("   💡 Check server status and network connectivity");
                }
            }
            return Ok(());
        }
    };

    // Ask user if they want to complete the invitation
    print!("\n❓ Do you want to complete this invitation? (yes/no): ");
    io::stdout().flush()?;
    let mut proceed = String::new();
    io::stdin().read_line(&mut proceed)?;

    if proceed.trim().to_lowercase() != "yes" {
        println!("⚠️  Invitation not completed");
        return Ok(());
    }

    // Get username and password
    print!("\n👤 Enter username (min 3 characters): ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    if username.len() < 3 {
        println!("❌ Username must be at least 3 characters");
        return Ok(());
    }

    print!("🔑 Enter password (min 8 characters): ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    if password.len() < 8 {
        println!("❌ Password must be at least 8 characters");
        return Ok(());
    }

    print!("🔑 Confirm password: ");
    io::stdout().flush()?;
    let mut password_confirm = String::new();
    io::stdin().read_line(&mut password_confirm)?;
    let password_confirm = password_confirm.trim();

    if password != password_confirm {
        println!("❌ Passwords do not match");
        return Ok(());
    }

    // Complete the invitation
    println!("\n🔄 Completing invitation...");
    match invitation_client
        .complete_invitation(token, username, password)
        .await
    {
        Ok(_) => {
            println!("✅ Invitation completed successfully!");
            println!("   You can now login with:");
            println!("   Username: {}", username);
            println!("   Email: {}", invitation.email);
            println!("   Role: {}", invitation.role);
        }
        Err(e) => {
            println!("❌ Failed to complete invitation: {}", e);
            match e {
                fits::api::invitations::InvitationError::BadRequest(_) => {
                    println!("   💡 Check the username and password format");
                }
                fits::api::invitations::InvitationError::Conflict(_) => {
                    println!("   💡 Username may already be taken");
                }
                fits::api::invitations::InvitationError::UnprocessableEntity(_) => {
                    println!("   💡 Validation error - check username/password requirements");
                }
                fits::api::invitations::InvitationError::Request(_) => {
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

    println!("\n🏁 Invitation acceptance example completed!");

    Ok(())
}
