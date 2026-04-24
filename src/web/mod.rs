mod routes;
mod templates;

use crate::config::Config;
use crate::storage::Storage;
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub storage: Arc<RwLock<Storage>>,
    pub config: Arc<Config>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    routes::create_routes(state)
}

pub async fn serve(host: &str, port: u16, state: Arc<AppState>) -> anyhow::Result<()> {
    let app = create_router(state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Dashboard running at http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
