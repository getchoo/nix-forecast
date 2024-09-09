#![allow(clippy::multiple_crate_versions)]
use anyhow::Result;
use clap::Parser;
use reqwest::StatusCode;
use tracing::instrument;

mod cli;
mod command;
mod http;
mod nix;

use cli::Cli;
use command::Run;

#[derive(Clone, Debug, thiserror::Error)]
enum Error {
	#[error("Unstable to complete HTTP request: {0}")]
	HTTPFailed(StatusCode),
	#[error("Nix exited with code {code}: {stderr}")]
	Nix { code: i32, stderr: String },
}

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();

	let cli = Cli::parse();
	if let Err(why) = cli.run().await {
		eprintln!("{why}");
	}

	Ok(())
}
