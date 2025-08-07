//! # Orama Rust Client
//!
//! A server-side Rust client for [Orama](https://orama.com), a search engine,
//! vector database, and LLM inference provider.
//!
//! This client is designed for use in server environments and Rust applications,
//! providing async/await support and full type safety.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use oramacore_client::{CollectionManager, SearchParams};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = CollectionManager::new(
//!         "your-collection-id",
//!         "your-api-key"
//!     ).await?;
//!
//!     let results = manager.search(&SearchParams {
//!         term: "rust programming".to_string(),
//!         limit: Some(10),
//!         ..Default::default()
//!     }).await?;
//!
//!     println!("Found {} results", results.count);
//!     
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod cloud;
pub mod collection;
pub mod error;
pub mod manager;
pub mod stream_manager;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use cloud::OramaCloud;
pub use collection::CollectionManager;
pub use error::{OramaError, Result};
pub use manager::OramaCoreManager;
pub use stream_manager::OramaCoreStream;
pub use types::*;
