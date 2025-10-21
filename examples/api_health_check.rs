/// Example: Basic health check API call
/// 
/// This example demonstrates how to make a simple health check call to the FITS API.
/// Run with: `cargo run --example api_health_check`

use fits::api::{FitsApiClient, ApiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger to see request details
    env_logger::init();

    println!("🔍 FITS API Health Check Example");
    println!("================================");

    // Method 1: Using the dev_client convenience function
    println!("\n📋 Method 1: Using dev_client()");
    let dev_client = FitsApiClient::dev_client();
    
    match dev_client.health_check().await {
        Ok(health_response) => {
            println!("✅ Health check successful!");
            println!("   Status: {}", health_response.status);
            println!("   Time: {}", health_response.time);
        }
        Err(e) => {
            println!("❌ Health check failed: {}", e);
            println!("   Make sure the FITS API server is running on http://localhost:8080");
        }
    }

    // Method 2: Custom configuration
    println!("\n📋 Method 2: Using custom configuration");
    let custom_config = ApiConfig::new("http://127.0.0.1:8080".to_string());
    let custom_client = FitsApiClient::new(custom_config);

    match custom_client.health_check().await {
        Ok(health_response) => {
            println!("✅ Health check with custom config successful!");
            println!("   Status: {}", health_response.status);
            println!("   Time: {}", health_response.time);
        }
        Err(e) => {
            println!("❌ Health check with custom config failed: {}", e);
        }
    }

    // Method 3: Using the convenience is_healthy() method
    println!("\n📋 Method 3: Using is_healthy() convenience method");
    let is_healthy = dev_client.is_healthy().await;
    if is_healthy {
        println!("✅ API is healthy!");
    } else {
        println!("❌ API is not healthy or unreachable");
    }

    println!("\n🏁 Health check example completed!");
    
    Ok(())
}