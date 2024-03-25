use axum::Router;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

mod http;
mod store;

use crate::http::AppState;

pub struct Server {
    port: u32,
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Server {
    pub async fn run_http_server(&self) -> Result<(), ()> {
        let state = AppState::new(self.pool.clone());
        // TODO: actually route, and (probably) take bodies out of requests
        // here?
        let _app = Router::new()
            .route("/", axum::routing::get(|| async { "Hello, World!" }))
            .with_state(state);

        Ok(())
    }
}
