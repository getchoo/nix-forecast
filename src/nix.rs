/// Abstractions over Nix's CLI
use crate::Error;

use std::{collections::HashMap, process::Command};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{event, instrument, Level};

/// JSON output of `nix build`
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Build {
	drv_path: String,
	/// Derivation output names and their path
	outputs: HashMap<String, String>,
}

#[instrument(skip(installable))]
pub fn dry_build_output(installable: &str) -> Result<Vec<u8>> {
	event!(Level::TRACE, "Running command `nix build --extra-experimental-features 'nix-command flakes' --dry-run --json {installable}`");
	let output = Command::new("nix")
		.args([
			"--extra-experimental-features",
			"nix-command flakes",
			"build",
			"--dry-run",
			"--json",
			installable,
		])
		.output()?;

	if output.status.success() {
		Ok(output.stdout)
	} else {
		let code = output.status.code().unwrap_or(1);
		let stderr = String::from_utf8(output.stderr.clone()).unwrap_or_default();

		Err(Error::Nix { code, stderr }.into())
	}
}

/// Get the `outPath` (store path) of an installable
#[instrument(skip(installable))]
pub fn out_path(installable: &str) -> Result<String> {
	let dry_build_output = dry_build_output(installable)?;
	let data: Vec<Build> = serde_json::from_slice(&dry_build_output)?;

	let out_path = data
		.first()
		.context("Unable to parse `nix build` output!")?
		.outputs
		.get("out")
		.with_context(|| format!("Unable to find output `out` for installable {installable}"))?;

	Ok(out_path.to_string())
}

/// Get the `drvPath` (derivation path) of an installable
#[instrument(skip(installable))]
pub fn drv_path(installable: &str) -> Result<String> {
	let dry_build_output = dry_build_output(installable)?;
	let data: Vec<Build> = serde_json::from_slice(&dry_build_output)?;

	let drv_path = &data
		.first()
		.context("Unable to parse `nix build` output!")?
		.drv_path;

	Ok(drv_path.to_string())
}

/// Get all paths in a closure at the given store path
#[instrument(skip(store_path))]
pub fn closure_paths(store_path: &str) -> Result<Vec<String>> {
	event!(Level::TRACE, "Running command `nix --extra-experimental-features 'nix-command flakes' path-info --json --recursive {store_path}`");
	let output = Command::new("nix")
		.args([
			"--extra-experimental-features",
			"nix-command flakes",
			"path-info",
			"--json",
			"--recursive",
			store_path,
		])
		.output()?;

	if output.status.success() {
		let path_infos: HashMap<String, Value> = serde_json::from_slice(&output.stdout)?;
		let paths = path_infos.into_keys().collect();
		Ok(paths)
	} else {
		let code = output.status.code().unwrap_or(1);
		let stderr = String::from_utf8(output.stderr.clone()).unwrap_or_default();

		Err(Error::Nix { code, stderr }.into())
	}
}

/// Get all paths in a NixOS or nix-darwin configuration's closure
#[instrument(skip(configuration_ref))]
pub fn configuration_closure_paths(configuration_ref: &str) -> Result<Vec<String>> {
	let installable = format!("{configuration_ref}.config.system.build.toplevel");
	let store_path = drv_path(&installable)?;
	let paths = closure_paths(&store_path)?;

	Ok(paths)
}

/// Get all installables available in a given Flake
#[instrument(skip(flake_ref))]
pub fn all_flake_installables(flake_ref: &str) -> Result<Vec<String>> {
	event!(
		Level::TRACE,
		"Running command `nix --extra-experimental-features 'nix-command flakes' search --json {flake_ref} .`"
	);
	let output = Command::new("nix")
		.args([
			"--extra-experimental-features",
			"nix-command flakes",
			"search",
			"--json",
			flake_ref,
			".",
		])
		.output()?;

	if output.status.success() {
		let search_results: HashMap<String, Value> = serde_json::from_slice(&output.stdout)?;
		let package_names = search_results
			.keys()
			.map(|name| format!("{flake_ref}#{name}"))
			.collect::<Vec<_>>();

		Ok(package_names)
	} else {
		let code = output.status.code().unwrap_or(1);
		let stderr = String::from_utf8(output.stderr.clone()).unwrap_or_default();

		Err(Error::Nix { code, stderr }.into())
	}
}
