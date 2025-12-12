mod app;
mod cli;
mod model;
mod presentation;
mod services;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .with_level(true)
        .try_init()
        .ok();

    let args = cli::Cli::parse();
    app::run(args)
}
