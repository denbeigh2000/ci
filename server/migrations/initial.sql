CREATE TABLE build_records (
    hash CHARACTER(33) NOT NULL,
    build_id CHARACTER(37) NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    finished_at TIMESTAMP WITH TIME ZONE,
    success BOOLEAN,
    build_url TEXT NOT NULL
);

CREATE INDEX idx_build_records_hash
    ON build_records (hash);

CREATE INDEX idx_build_records_hash_build_id
    ON build_records(hash, build_id);
