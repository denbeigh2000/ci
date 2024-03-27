use std::net::SocketAddr;

use crate::http::AppState;
use crate::store::Store;

use axum::Router;
use http::{handle_get, handle_post, handle_put};

mod http;
pub mod store;

pub struct Server {
    port: u16,
    store: Store,
}

#[derive(thiserror::Error, Debug)]
pub enum HTTPServeError {
    #[error("Error creating TCP socket: {0}")]
    CreatingTCPSocket(std::io::Error),
    #[error("Error serving HTTP: {0}")]
    Serving(std::io::Error),
}

impl Server {
    pub fn new(port: u16, store: Store) -> Self {
        Self { port, store }
    }

    pub async fn run_http_server(&self) -> Result<(), HTTPServeError> {
        let state = AppState::new(self.store.clone());
        // TODO: actually route, and (probably) take bodies out of requests
        // here?
        let app = Router::new()
            .route("/", axum::routing::get(handle_get))
            .route("/", axum::routing::post(handle_post))
            .route("/", axum::routing::put(handle_put))
            .with_state(state);

        let addr: SocketAddr = ([127, 0, 0, 1], self.port).into();
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(HTTPServeError::CreatingTCPSocket)?;

        axum::serve(listener, app)
            .await
            .map_err(HTTPServeError::Serving)
    }
}
