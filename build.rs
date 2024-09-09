use std::io::Error;

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
	let out_dir = if let Some(completion_dir) = option_env!("COMPLETION_DIR") {
		std::fs::create_dir_all(completion_dir)?;
		completion_dir.to_string()
	} else if let Ok(out_dir) = std::env::var("OUT_DIR") {
		out_dir
	} else {
		println!("cargo:warning=Unable to resolve `COMPLETION_DIR` or `OUT_DIR` in environment. Completions will not be built");
		return Ok(());
	};

	let mut command = Cli::command();
	for &shell in Shell::value_variants() {
		generate_to(shell, &mut command, env!("CARGO_PKG_NAME"), &out_dir)?;
	}

	Ok(())
}
