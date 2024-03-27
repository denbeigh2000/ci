// API:
//  (NOTE: This is not a particularly RESTful API, but for these purposes it
//  doesn't need to be.)
//  - GET /derivation-builds
//    Return previous builds of the given derivations
//    (JSON body in GET request defines hashes to query for)
//  - POST /derivation-builds
//    Record new derivation builds in the database

use axum::body::Body;
use axum::extract::{Json, State};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;

use crate::store::{BuildRecord, Store, StoreError};

#[derive(Clone)]
pub struct AppState {
    store: Store,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HTTPHandlingError {
    #[error("db error: {0}")]
    StoreError(#[from] StoreError),
}

impl IntoResponse for HTTPHandlingError {
    fn into_response(self) -> axum::response::Response {
        let b = match self {
            Self::StoreError(e) => Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR),
        };

        b.body(Body::empty()).unwrap()
    }
}

// TODO: retrieve body from request
pub async fn handle_get(
    State(mut state): State<AppState>,
    Json(body): Json<Vec<String>>,
) -> Result<Json<Vec<BuildRecord>>, HTTPHandlingError> {
    let results = state.store.query(&body).await?;

    Ok(Json(results))
}

pub async fn handle_post(
    State(mut state): State<AppState>,
    Json(body): Json<BuildRecord>,
) -> Result<(), HTTPHandlingError> {
    state.store.insert_start(&body).await?;

    Ok(())
}

pub async fn handle_put(
    State(mut state): State<AppState>,
    Json(body): Json<BuildRecord>,
) -> Result<(), HTTPHandlingError> {
    state.store.mark_finish(&body).await?;

    Ok(())
}

// TODO: handle_post, handle_put
