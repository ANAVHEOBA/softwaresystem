//! Run with: cargo run --bin cleanup

use mongodb::{bson::doc, Client};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("MONGODB_DATABASE").unwrap_or_else(|_| "cleuly".to_string());

    println!("Connecting to MongoDB...");
    let client = Client::with_uri_str(&uri).await?;
    let db = client.database(&db_name);

    // Drop sessions collection (has old datetime format)
    println!("Dropping sessions collection...");
    db.collection::<mongodb::bson::Document>("sessions")
        .drop()
        .await?;
    println!("✓ Sessions collection dropped");

    // Optionally clean other collections
    println!("\nCollections remaining:");
    let collections = db.list_collection_names().await?;
    for name in collections {
        println!("  - {}", name);
    }

    println!("\n✓ Cleanup complete!");
    Ok(())
}
