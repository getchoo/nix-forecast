use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
	/// A list of Nix installables to look for. If not given, all paths in nixpkgs are checked
	#[arg(required_unless_present("configuration"))]
	pub installables: Option<Vec<String>>,

	/// Flake reference pointing to a NixOS or nix-darwin configuration
	#[allow(clippy::doc_markdown)] // Why does "NixOS" trigger this???
	#[arg(short, long, conflicts_with("installables"))]
	pub configuration: Option<String>,

	/// URL of the substituter to check
	#[arg(short, long, default_value = "https://cache.nixos.org")]
	pub binary_cache: String,

	/// Flake reference of nixpkgs (or other package repository)
	#[arg(short, long, default_value = "nixpkgs")]
	pub flake: String,

	/// Show a list of store paths not found in the substituter
	#[arg(short, long)]
	pub show_missing: bool,
}
