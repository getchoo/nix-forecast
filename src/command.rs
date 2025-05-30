use crate::{
	http::{self, Ext},
	nix,
};

use anyhow::Result;
use futures::{stream, StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::instrument;

pub trait Run {
	async fn run(&self) -> Result<()>;
}

impl Run for crate::Cli {
	#[instrument(skip(self))]
	async fn run(&self) -> Result<()> {
		let store_paths = if let Some(installables) = self.installables.clone() {
			resolve_installables(installables).await?
		} else if let Some(configuration) = &self.configuration {
			eprintln!("❓ Indexing requisites of configuration closure");
			nix::system_configuration_closure_paths(configuration)?
		} else if let Some(home) = &self.home {
			eprintln!("❓ Indexing requisites of home configuration closure");
			nix::home_configuration_closure_paths(home)?
		} else {
			eprintln!("❓ Indexing all installables of flake `{}`", self.flake);
			let installables = nix::all_flake_installables(&self.flake)?;
			resolve_installables(installables).await?
		};

		check_store_paths(&self.binary_caches, &store_paths, self.show_missing).await?;

		Ok(())
	}
}

#[instrument(skip(installables))]
async fn resolve_installables(installables: Vec<String>) -> Result<Vec<String>> {
	eprintln!(
		"🔃 Attempting to evaluate {} installable(s)",
		installables.len()
	);

	// Find our outputs concurrently
	let progress_bar = ProgressBar::new(installables.len() as u64).with_style(progress_style()?);
	let out_paths: Vec<String> = stream::iter(&installables)
		.map(|installable| {
			let progress_bar = &progress_bar;
			async move {
				progress_bar.inc(1);
				let out_path = nix::out_path(installable)?;

				anyhow::Ok(out_path)
			}
		})
		.buffer_unordered(num_cpus::get()) // try not to explode computers
		.try_collect()
		.await?;

	eprintln!("✅ Evaluated {} installable(s)!", out_paths.len());

	Ok(out_paths)
}

#[allow(clippy::cast_precision_loss)]
#[instrument(skip(store_paths))]
async fn check_store_paths(
	binary_caches: &Vec<String>,
	store_paths: &Vec<String>,
	show_missing: bool,
) -> Result<()> {
	let num_store_paths = store_paths.len();
	eprintln!("🌡️ Checking for {num_store_paths} store path(s) in: {binary_caches:?}");

	let http = <http::Client as http::Ext>::default();
	let progress_bar = ProgressBar::new(num_store_paths as u64).with_style(progress_style()?);
	let uncached_paths: Vec<&str> = stream::iter(store_paths)
		// Check the cache for all of our paths
		.map(|store_path| {
			let http = &http;
			let progress_bar = &progress_bar;
			async move {
				let mut has_store_path = false;
				for binary_cache in binary_caches {
					if http.has_store_path(binary_cache, store_path).await? {
						has_store_path = true;
					}
				}
				progress_bar.inc(1);

				anyhow::Ok((has_store_path, store_path.as_str()))
			}
		})
		.buffer_unordered(100)
		// Filter out misses
		.try_filter_map(|(has_store_path, store_path)| async move {
			Ok((!has_store_path).then_some(store_path))
		})
		.try_collect()
		.await?;

	let num_uncached = uncached_paths.len();
	let num_cached = num_store_paths - num_uncached;

	eprintln!(
		"☀️ {:.2}% of paths available ({} out of {})",
		(num_cached as f32 / num_store_paths as f32) * 100.0,
		num_cached,
		num_store_paths,
	);

	if show_missing {
		eprintln!("\n⛈️  Found {num_uncached} uncached paths:");
		println!("{}", uncached_paths.join("\n"));
	}

	Ok(())
}

pub fn progress_style() -> Result<ProgressStyle> {
	Ok(
		ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos:>7}/{len:7} {msg}")?
			.progress_chars("##-"),
	)
}
