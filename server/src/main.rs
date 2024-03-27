use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use server::{store::Store, Server};

use clap::Parser;
use tokio_postgres::{Config, NoTls};

#[derive(Parser)]
struct Args {
    #[arg(short = 'u', long, env = "CI_SERVER_DB_USER")]
    db_user: String,

    #[arg(short = 'p', env = "CI_SERVER_DB_PASSWORD")]
    db_password: String,

    #[arg(short = 'n', long, env = "CI_SERVER_DB_NAME")]
    db_name: String,

    #[arg(short = 'a', long, env = "CI_SERVER_DB_ADDRESS")]
    db_addr: String,
}

#[derive(thiserror::Error, Debug)]
enum MainError {
    #[error("Creating pool: {0}")]
    CreatingPool(#[from] tokio_postgres::Error),
    #[error("Serving HTTP: {0}")]
    Serving(#[from] server::HTTPServeError),
}

async fn real_main() -> Result<(), MainError> {
    let args = Args::parse();
    let mut config = Config::new();
    config
        .user(&args.db_user)
        .password(&args.db_password)
        .dbname(&args.db_name);

    let mgr = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::builder().build(mgr).await?;
    let store = Store::new(pool);
    let server = Server::new(1234, store);
    server.run_http_server().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = real_main().await {
        eprintln!("error: {e}");
    }
}
