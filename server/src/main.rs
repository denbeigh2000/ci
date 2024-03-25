use server::Server;

async fn real_main() -> Result<(), ()> {}

#[tokio::main]
async fn main() {
    if let Err(e) = real_main().await {
        eprintln!("error: {e}");
    }
}
