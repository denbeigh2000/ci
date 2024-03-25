// API:
//  (NOTE: This is not a particularly RESTful API, but for these purposes it
//  doesn't need to be.)
//  - GET /derivation-builds
//    Return previous builds of the given derivations
//    (JSON body in GET request defines hashes to query for)
//  - POST /derivation-builds
//    Record new derivation builds in the database

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{Response, StatusCode};

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct AppState {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl AppState {
    pub fn new(pool: Pool<PostgresConnectionManager<NoTls>>) -> Self {
        Self { pool }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HTTPHandlingError {
    #[error("connection timeout")]
    ConnectionTimeout,
    #[error("db error: {0}")]
    DBConnectionError(tokio_postgres::Error),
}

impl Into<Response<Bytes>> for HTTPHandlingError {
    fn into(self) -> Response<Bytes> {
        let b = match self {
            Self::ConnectionTimeout => {
                Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR)
            }
            Self::DBConnectionError(e) => {
                Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR)
            }
        };

        b.body(Bytes::new()).unwrap()
    }
}

// TODO: retrieve body from request
async fn handle_get(State(state): State<AppState>) -> Result<Response<Bytes>, HTTPHandlingError> {
    let conn = match state.pool.get().await {
        Ok(conn) => Ok(conn),
        Err(bb8::RunError::TimedOut) => Err(HTTPHandlingError::ConnectionTimeout),
        Err(bb8::RunError::User(e)) => Err(HTTPHandlingError::DBConnectionError(e)),
    }?;

    unimplemented!()
}

// TODO: handle_post, handle_put
