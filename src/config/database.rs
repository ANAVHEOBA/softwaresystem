use mongodb::{Client, Database};
use std::env;

pub async fn connect() -> Database {
    let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let db_name = env::var("MONGODB_DATABASE").unwrap_or_else(|_| "cleuly".to_string());

    let client = Client::with_uri_str(&uri)
        .await
        .expect("Failed to connect to MongoDB");

    client.database(&db_name)
}
