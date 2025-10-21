/// Example: Basic health check API call
/// 
/// This example demonstrates how to make a simple health check call to the FITS API.
/// 
/// Configuration is loaded from environment variables or .env file:
/// - FITS_API_BASE_URL: The base URL of the FITS API (default: http://localhost:8080)
/// - API_LOG: Logging level (default: info)
/// 
/// Run with: `cargo run --example api_health_check`

use fits::api::FitsApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    let _ = dotenvy::dotenv();
    
    // Initialize logger to see request details
    env_logger::init();

    println!("🔍 FITS API Health Check Example");
    println!("================================");

    // Show current configuration
    let api_url = std::env::var("FITS_API_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    println!("🔧 Configuration:");
    println!("   API URL: {}", api_url);
    println!("   Log Level: {}", std::env::var("API_LOG").unwrap_or_else(|_| "info".to_string()));

    // Method 1: Using environment variables (recommended)
    println!("\n📋 Method 1: Using environment configuration (recommended)");
    let env_client = FitsApiClient::from_env();
    
    match env_client.health_check().await {
        Ok(health_response) => {
            println!("✅ Health check successful!");
            println!("   Status: {}", health_response.status);
            println!("   Time: {}", health_response.time);
        }
        Err(e) => {
            println!("❌ Health check failed: {}", e);
            println!("   Make sure the FITS API server is running on {}", api_url);
        }
    }

    // Method 2: Using dev_client convenience function  
    println!("\n📋 Method 2: Using dev_client() convenience function");
    let dev_client = FitsApiClient::dev_client();

    match dev_client.health_check().await {
        Ok(health_response) => {
            println!("✅ Health check with dev_client successful!");
            println!("   Status: {}", health_response.status);
            println!("   Time: {}", health_response.time);
        }
        Err(e) => {
            println!("❌ Health check with dev_client failed: {}", e);
        }
    }

    // Method 3: Using the convenience is_healthy() method
    println!("\n📋 Method 3: Using is_healthy() convenience method");
    let is_healthy = env_client.is_healthy().await;
    if is_healthy {
        println!("✅ API is healthy!");
    } else {
        println!("❌ API is not healthy or unreachable");
    }

    println!("\n🏁 Health check example completed!");
    
    Ok(())
}