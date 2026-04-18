use anyhow::Error;
use clap::Parser;
use intcalc::cli::Cli;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Cli::parse().run().await?;

    Ok(())
}
