/// HTTP interfaces
use crate::Error;

use anyhow::Result;
use reqwest::{Method, StatusCode};
use tracing::{event, instrument, Level};

// https://nix.dev/manual/nix/2.24/store/store-path
const STORE_DIRECTORY: &str = "/nix/store";
const DIGEST_SIZE: usize = 32;

pub type Client = reqwest::Client;

pub trait Ext {
	fn default() -> Self;
	async fn has_store_path(&self, binary_cache_url: &str, store_path: &str) -> Result<bool>;
}

impl Ext for Client {
	/// Create a client with our user agent
	fn default() -> Self {
		Client::builder()
			.user_agent(concat!(
				env!("CARGO_PKG_NAME"),
				"/",
				env!("CARGO_PKG_VERSION")
			))
			.build()
			.unwrap()
	}

	/// Check if a store path is available in the given binary cache
	#[instrument(skip(self, binary_cache_url, store_path))]
	async fn has_store_path(&self, binary_cache_url: &str, store_path: &str) -> Result<bool> {
		let url = format!(
			"{binary_cache_url}/{}.narinfo",
			hash_from_store_path(store_path)
		);

		let request = self.request(Method::HEAD, url).build()?;
		event!(
			Level::TRACE,
			"Checking for store path {store_path} in binary cache at {}",
			request.url()
		);
		let response = self.execute(request).await?;

		match response.status() {
			StatusCode::OK => {
				event!(
					Level::TRACE,
					"Found store path `{store_path}` in binary cache"
				);
				Ok(true)
			}
			StatusCode::NOT_FOUND => {
				event!(
					Level::TRACE,
					"Did not find store path `{store_path}` in binary cache"
				);
				Ok(false)
			}
			status_code => Err(Error::HTTPFailed(status_code).into()),
		}
	}
}

/// Strip the <hash> from /nix/store/<hash>-<name>
fn hash_from_store_path(store_path: &str) -> &str {
	// Store paths will always start with the store directory, followed by a path separator. See
	// the above link
	let start_index = STORE_DIRECTORY.len() + 1;
	// The next DIGEST_SIZE characters will then be the digest
	let end_index = start_index + DIGEST_SIZE;

	&store_path[start_index..end_index]
}
