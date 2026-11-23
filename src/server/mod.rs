//! Server core.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use axum::{Router, http::StatusCode, routing::get, serve};

use crate::{app::store::Store, config::Config, prelude::*};

/// `/api/days` and `/api/briefings`.
mod briefings;

/// State shared by every route.
#[derive(Clone)]
pub(crate) struct AppState {
    /// The database. The lock makes requests touch it one at a time.
    store: Arc<Mutex<Store>>,
}

impl AppState {
    /// Wraps an open store.
    pub(crate) fn new(store: Store) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }

    /// Locks the store.
    fn store(&self) -> MutexGuard<'_, Store> {
        // Each change is one statement, so a panic can't leave one half done.
        self.store.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Liveness probe: answers as long as the server is up.
async fn health() -> StatusCode {
    StatusCode::OK
}

/// Global server routes.
fn routes(state: AppState) -> Router {
    let router = Router::new().merge(briefings::routes());
    router.route("/health", get(health)).with_state(state)
}

/// Start web services.
///
/// # Errors
///
/// Returns an [`Error`] if the database cannot be opened or the address
/// cannot be bound.
pub async fn start(config: &Config) -> Result<()> {
    let state = AppState::new(Store::open(config.database.clone())?);
    let (address, port) = (&config.server.address, config.server.port);
    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    tracing::info!("Server running on {}", listener.local_addr()?);
    Ok(serve(listener, routes(state)).await?)
}
