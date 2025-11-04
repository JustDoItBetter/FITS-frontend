/// Example: FITS API Signing Operations
///
/// This example demonstrates how to use the signing endpoints:
/// 1. Upload a parquet file
/// 2. Get pending sign requests
/// 3. Upload signed requests
///
/// Configuration is loaded from environment variables or .env file:
/// - FITS_API_BASE_URL: The base URL of the FITS API (default: http://localhost:8080)
/// - FITS_ACCESS_TOKEN: Access token for authenticated requests
/// - RUST_LOG: Logging level (default: info)
///
/// Run with: `cargo run --example api_signing`
use fits::api::signing::SigningClient;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if it exists
    let _ = dotenvy::dotenv();

    // Initialize logger
    env_logger::init();

    println!("📦 FITS API Signing Operations Example");
    println!("======================================");

    // Show current configuration
    let api_url =
        std::env::var("FITS_API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    println!("🔧 Configuration:");
    println!("   API URL: {}", api_url);
    println!(
        "   Log Level: {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    );

    // Create signing client
    println!("\n📋 Creating signing client...");
    let mut signing_client = SigningClient::from_env();

    // Get access token
    print!("\n🔑 Enter access token: ");
    io::stdout().flush()?;
    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim();

    if !token.is_empty() {
        signing_client.set_token(token.to_string());
        println!("✅ Token set");
    } else {
        println!("⚠️  No token provided, authenticated endpoints will fail");
    }

    // Show menu
    loop {
        println!("\n📋 Available Operations:");
        println!("   1. Upload parquet file");
        println!("   2. Get pending sign requests");
        println!("   3. Upload signed requests");
        println!("   4. Exit");

        print!("\n👉 Select operation (1-4): ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        match choice {
            "1" => {
                // Upload parquet file
                print!("\n📂 Enter path to parquet file: ");
                io::stdout().flush()?;
                let mut file_path = String::new();
                io::stdin().read_line(&mut file_path)?;
                let file_path = file_path.trim();

                println!("\n🔄 Uploading file...");
                match signing_client.upload_parquet(file_path).await {
                    Ok(upload_record) => {
                        println!("✅ Upload successful!");
                        println!("   Upload ID: {}", upload_record.upload_id);
                        println!("   Student UUID: {}", upload_record.student_uuid);
                        println!("   File Name: {}", upload_record.file_name);
                        println!("   File Size: {} bytes", upload_record.file_size);
                        println!("   Content Hash: {}", upload_record.content_hash);
                        println!("   Uploaded At: {}", upload_record.uploaded_at);
                    }
                    Err(e) => {
                        println!("❌ Upload failed: {}", e);
                        match e {
                            fits::api::signing::SigningError::Unauthorized(_) => {
                                println!("   💡 Check your access token");
                            }
                            fits::api::signing::SigningError::BadRequest(_) => {
                                println!("   💡 Check the file format and content");
                            }
                            fits::api::signing::SigningError::NotImplemented(_) => {
                                println!(
                                    "   💡 This endpoint is not yet implemented on the server"
                                );
                            }
                            fits::api::signing::SigningError::IoError(_) => {
                                println!("   💡 Check the file path");
                            }
                            _ => {
                                println!("   💡 Check server status and network connectivity");
                            }
                        }
                    }
                }
            }
            "2" => {
                // Get pending sign requests
                print!("\n📂 Enter path to save sign requests file: ");
                io::stdout().flush()?;
                let mut save_path = String::new();
                io::stdin().read_line(&mut save_path)?;
                let save_path = save_path.trim();

                println!("\n🔄 Fetching pending sign requests...");
                match signing_client.get_sign_requests().await {
                    Ok(data) => {
                        println!("✅ Sign requests retrieved!");
                        println!("   Data size: {} bytes", data.len());

                        // Save to file
                        match tokio::fs::write(save_path, &data).await {
                            Ok(_) => {
                                println!("   Saved to: {}", save_path);
                            }
                            Err(e) => {
                                println!("   ⚠️  Failed to save file: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to get sign requests: {}", e);
                        match e {
                            fits::api::signing::SigningError::Unauthorized(_) => {
                                println!("   💡 Check your access token");
                            }
                            fits::api::signing::SigningError::NotImplemented(_) => {
                                println!(
                                    "   💡 This endpoint is not yet implemented on the server"
                                );
                            }
                            _ => {
                                println!("   💡 Check server status and network connectivity");
                            }
                        }
                    }
                }
            }
            "3" => {
                // Upload signed requests
                print!("\n📂 Enter path to signed requests parquet file: ");
                io::stdout().flush()?;
                let mut file_path = String::new();
                io::stdin().read_line(&mut file_path)?;
                let file_path = file_path.trim();

                println!("\n🔄 Uploading signed requests...");
                match signing_client.upload_signed_requests(file_path).await {
                    Ok(_) => {
                        println!("✅ Signed requests uploaded successfully!");
                    }
                    Err(e) => {
                        println!("❌ Upload failed: {}", e);
                        match e {
                            fits::api::signing::SigningError::Unauthorized(_) => {
                                println!("   💡 Check your access token");
                            }
                            fits::api::signing::SigningError::BadRequest(_) => {
                                println!("   💡 Check the file format and content");
                            }
                            fits::api::signing::SigningError::NotImplemented(_) => {
                                println!(
                                    "   💡 This endpoint is not yet implemented on the server"
                                );
                            }
                            fits::api::signing::SigningError::IoError(_) => {
                                println!("   💡 Check the file path");
                            }
                            _ => {
                                println!("   💡 Check server status and network connectivity");
                            }
                        }
                    }
                }
            }
            "4" => {
                println!("\n👋 Exiting...");
                break;
            }
            _ => {
                println!("⚠️  Invalid choice, please select 1-4");
            }
        }
    }

    println!("\n🏁 Signing operations example completed!");

    Ok(())
}
