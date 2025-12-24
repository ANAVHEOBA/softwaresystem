use mongodb::Database;
use redis::aio::ConnectionManager;

pub mod config;
pub mod modules;
pub mod services;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: ConnectionManager,
}
