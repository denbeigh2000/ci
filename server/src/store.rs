use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use chrono::{DateTime, Utc};
use postgres_from_row::FromRow;
use serde::{Deserialize, Serialize};
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

#[derive(FromRow, Serialize, Deserialize)]
pub struct BuildRecord {
    hash: String,
    build_id: String,
    build_url: String,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    success: Option<bool>,
}

type PostgresPool = Pool<PostgresConnectionManager<NoTls>>;

pub struct Hash([char; 33]);

pub struct Query(Vec<Hash>);

#[derive(Clone)]
pub struct Store {
    pool: PostgresPool,
}

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("timed out waiting for DB connection")]
    ConnectionTimeout,
    #[error("error interacting with DB: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
    #[error("No matching entries were found to update")]
    UpdateMissingEntry,
}

impl From<bb8::RunError<tokio_postgres::Error>> for StoreError {
    fn from(value: bb8::RunError<tokio_postgres::Error>) -> Self {
        match value {
            bb8::RunError::User(e) => StoreError::DatabaseError(e),
            bb8::RunError::TimedOut => StoreError::ConnectionTimeout,
        }
    }
}

impl Store {
    pub fn new(pool: PostgresPool) -> Self {
        Self { pool }
    }

    // TODO: finish
    pub async fn query(&mut self, derivs: &[String]) -> Result<Vec<BuildRecord>, StoreError> {
        let conn = self.pool.get().await?;
        let rows = conn.query(FIND_DERIV_QUERY, &[&derivs]).await?;

        let records = rows.iter().map(BuildRecord::from_row).collect();
        Ok(records)
    }

    pub async fn insert_start(&mut self, record: &BuildRecord) -> Result<(), StoreError> {
        // TODO: insert en masse?
        let conn = self.pool.get().await?;
        conn.execute(
            INSERT_DERIV_QUERY,
            &[
                &record.hash,
                &record.build_url,
                &record.build_id,
                &record.started_at,
            ],
        )
        .await?;

        Ok(())
    }

    pub async fn mark_finish(&mut self, record: &BuildRecord) -> Result<(), StoreError> {
        let conn = self.pool.get().await?;
        let res = conn
            .execute(
                UPDATE_DERIV_FINISHED_QUERY,
                &[
                    &record.started_at,
                    &record.finished_at,
                    &record.success,
                    &record.hash,
                    &record.build_id,
                ],
            )
            .await?;

        match res {
            0 => Err(StoreError::UpdateMissingEntry),
            1.. => Ok(()),
        }
    }
}
