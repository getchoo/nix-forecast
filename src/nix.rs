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

/// JSON output of `nix path-info` pre Nix 2.19
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PathInfo {
	path: String,
}

/// Strip derivations from a list of store paths
fn strip_drvs(paths: Vec<String>) -> Vec<String> {
	paths
		.into_iter()
		.filter(|path| {
			std::path::Path::new(path)
				.extension()
				.is_some_and(|ext| !ext.eq_ignore_ascii_case("drv"))
		})
		.collect()
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
pub fn closure_paths(store_path: &str, with_outputs: bool) -> Result<Vec<String>> {
	event!(Level::TRACE, "Querying closure paths");

	let mut args = vec!["--query", "--requisites", store_path];

	if with_outputs {
		args.push("--include-outputs");
	}

	let output = Command::new("nix-store").args(args).output()?;

	if output.status.success() {
		// Capture paths from command output, strip drvs
		let stdout = String::from_utf8(output.stdout)?;
		let paths = stdout.lines().map(ToString::to_string).collect();

		Ok(paths)
	} else {
		let code = output.status.code().unwrap_or(1);
		let stderr = String::from_utf8(output.stderr.clone()).unwrap_or_default();

		Err(Error::Nix { code, stderr }.into())
	}
}

/// Get all paths in an installable's closure
#[instrument(skip(installable))]
fn installable_closure_paths(installable: &str) -> Result<Vec<String>> {
	let store_path = drv_path(installable)?;
	let paths = closure_paths(&store_path, true)?;
	let out_paths = strip_drvs(paths);

	Ok(out_paths)
}

/// Get all paths in a NixOS or nix-darwin configuration's closure
#[instrument(skip(configuration_ref))]
pub fn system_configuration_closure_paths(configuration_ref: &str) -> Result<Vec<String>> {
	let installable = format!("{configuration_ref}.config.system.build.toplevel");
	let out_paths = installable_closure_paths(&installable)?;

	Ok(out_paths)
}

/// Get all paths in a home-manager configuration's closure
#[instrument(skip(configuration_ref))]
pub fn home_configuration_closure_paths(configuration_ref: &str) -> Result<Vec<String>> {
	let installable = format!("{configuration_ref}.activationPackage");
	let out_paths = installable_closure_paths(&installable)?;

	Ok(out_paths)
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
