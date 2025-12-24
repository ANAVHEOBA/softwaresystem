use redis::aio::ConnectionManager;
use std::env;

pub async fn connect() -> ConnectionManager {
    let uri = env::var("REDIS_URI").expect("REDIS_URI must be set");

    let client = redis::Client::open(uri).expect("Failed to create Redis client");

    ConnectionManager::new(client)
        .await
        .expect("Failed to connect to Redis")
}
