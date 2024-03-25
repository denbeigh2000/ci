use std::{convert::Infallible, ops::Deref, task::Wake};

use axum::http::Uri;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use chrono::{DateTime, Utc};
use tokio_postgres::NoTls;

const FIND_DERIV_QUERY: &str = r#"
SELECT (
    hash,
    started_at,
    finished_at,
    success,
    build_url
)
FROM
    build_records
WHERE
    hash = ANY($1::CHAR(33)[]);
"#;

const INSERT_DERIV_QUERY: &str = r#"
INSERT INTO build_records (
    hash,
    build_id,
    started_at,
    build_url
)
VALUES (
    $1::CHAR(33),
    $2::CHAR(37),
    $3::TIMESTAMP WITH TIME ZONE,
    $4::TEXT
);
"#;

const UPDATE_DERIV_FINISHED_QUERY: &str = r#"
UPDATE build_records
SET
    started_at = $1::TIMESTAMP WITH TIME ZONE,
    finished_at = $2::TIMESTAMP WITH TIME ZONE,
    success = $3::BOOLEAN
WHERE
    hash = $4::CHAR(33);
    build_id = $5::CHAR(37);
;
"#;

pub struct BuildRecord {
    hash: Hash,
    build_url: Uri,
    started_at: DateTime<Utc>,
    finshed_at: Option<DateTime<Utc>>,
    success: Option<bool>,
}

type PostgresPool = Pool<PostgresConnectionManager<NoTls>>;

pub struct Hash([char; 33]);

pub struct Query(Vec<Hash>);

pub struct Store {
    pool: PostgresPool,
}

impl Store {
    pub fn new(pool: PostgresPool) -> Self {
        Self { pool }
    }

    // TODO: finish
    pub async fn query(&mut self, derivs: &[Hash]) -> Result<(), Infallible> {
        let conn = self.pool.get().await.unwrap();
        let v: Vec<String> = derivs.iter().map(|i| String::from_iter(i.0)).collect();
        conn.query(FIND_DERIV_QUERY, &[&v]);

        unimplemented!()
    }

    // TODO
    // async fn create(&mut self, records: &[InitialRecord]) -> Result<...> {

    // TODO
    // async fn finish(&mut self, records: &[FinishedRecord]) -> Result<...> {
}
