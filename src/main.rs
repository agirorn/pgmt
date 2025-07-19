use clap::Parser;
use pgmt_cli::{run, Cli};

#[tokio::main]
async fn main() {
    // TODO add dotenv so it can pick up .en files like a grounup
    // dotenv().ok();
    let cli = Cli::parse();

    run(cli).await;
}
