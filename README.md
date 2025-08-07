# Orama Core Client for Rust

[![Crates.io](https://img.shields.io/crates/v/oramacore-client.svg)](https://crates.io/crates/oramacore-client)
[![Documentation](https://docs.rs/oramacore-client/badge.svg)](https://docs.rs/oramacore-client)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPLv3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

A comprehensive Rust client for [Orama Core](https://github.com/oramasearch/oramacore) and [Orama Cloud](https://app.orama.com).

## Features

- 🔍 **Full-Text and Vector Search** - Hybrid search combining traditional and semantic search
- 🤖 **AI-Powered Operations** - Natural language queries and streaming AI conversations
- 📄 **Document Management** - CRUD operations for documents with automatic indexing
- ☁️ **Cloud Integration** - Native support for Orama Cloud projects
- 🔧 **Collection Management** - Create, manage, and monitor collections
- ⚡ **Async/Await** - Built on Tokio for high-performance async operations
- 🛡️ **Type Safety** - Comprehensive type definitions with serde serialization
- 🔐 **Authentication** - Support for both API key and JWT authentication flows

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oramacore-client = "1.2.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

## Quick Start

```rust
use oramacore_client::{
    collection::{CollectionManager, CollectionManagerConfig},
    types::{SearchParams, SearchMode},
    error::Result,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client
    let config = CollectionManagerConfig::new("your-collection-id", "your-api-key");
    let client = CollectionManager::new(config).await?;

    // Perform a search
    let search_params = SearchParams::new("artificial intelligence")
        .with_mode(SearchMode::Hybrid)
        .with_limit(10);

    let results = client.search::<Document>(&search_params).await?;
    println!("Found {} documents", results.hits.len());

    for hit in results.hits {
        println!("- {} (score: {:.4})", hit.document.title, hit.score);
    }

    Ok(())
}
```

## Core Concepts

### Collections and Documents

Collections are containers for your documents. Each collection can have multiple indexes for different types of searches:

```rust
use oramacore_client::collection::{CollectionManager, CreateIndexParams};

// Create an index
let create_params = CreateIndexParams {
    id: Some("articles".to_string()),
    embeddings: Some(serde_json::json!("automatic")),
};

client.index.create(create_params).await?;

// Insert documents
let index = client.index.set("articles".to_string());
index.insert_documents(documents).await?;
```

### Search Modes

The client supports three search modes:

- **Fulltext**: Traditional keyword-based search
- **Vector**: Semantic search using embeddings
- **Hybrid**: Combines both approaches for best results

```rust
use oramacore_client::types::{SearchParams, SearchMode};

// Vector search for semantic similarity
let vector_search = SearchParams::new("machine learning concepts")
    .with_mode(SearchMode::Vector)
    .with_limit(5);

// Hybrid search for comprehensive results
let hybrid_search = SearchParams::new("deep learning")
    .with_mode(SearchMode::Hybrid)
    .with_threshold(0.8);
```

### AI-Powered Features

Create AI sessions for natural language interactions:

```rust
use oramacore_client::stream_manager::{AnswerConfig, CreateAiSessionConfig};
use oramacore_client::types::{LlmConfig, LlmProvider};

// Create an AI session
let session_config = CreateAiSessionConfig::new()
    .with_llm_config(LlmConfig {
        provider: LlmProvider::OpenAI,
        model: "gpt-4".to_string(),
    });

let ai_session = client.ai.create_ai_session().await?;

// Get an AI-powered answer
let answer_config = AnswerConfig::new("Explain quantum computing")
    .with_max_documents(5);

let answer = ai_session.answer(answer_config).await?;
println!("AI Response: {}", answer);
```

### Streaming Responses

For real-time AI interactions, use streaming:

```rust
use futures::StreamExt;

let stream_config = AnswerConfig::new("What is the future of AI?");
let mut answer_stream = ai_session.answer_stream(stream_config).await?;

while let Some(chunk_result) = answer_stream.next().await {
    match chunk_result {
        Ok(chunk) => print!("{}", chunk),
        Err(e) => eprintln!("Stream error: {}", e),
    }
}
```

## Authentication

The client supports two authentication methods:

### API Key Authentication

For regular API keys:

```rust
let config = CollectionManagerConfig::new("collection-id", "your-api-key");
```

### JWT Authentication

For private API keys (starting with "p_"):

```rust
let config = CollectionManagerConfig::new("collection-id", "p_your-private-key")
    .with_auth_jwt_url("https://your-auth-endpoint.com");
```

## Cloud Integration

For Orama Cloud projects:

```rust
use oramacore_client::cloud::{OramaCloud, ProjectManagerConfig, CloudSearchParams};

let cloud_config = ProjectManagerConfig::new("project-id", "api-key");
let cloud_client = OramaCloud::new(cloud_config).await?;

// Cloud-specific search with datasources
let search_params = CloudSearchParams::new(
    "search query",
    vec!["datasource1".to_string(), "datasource2".to_string()]
);

let results = cloud_client.search::<Document>(&search_params).await?;
```

## Collection Management

Create and manage collections programmatically:

```rust
use oramacore_client::manager::{OramaCoreManager, CreateCollectionParams};
use oramacore_client::types::{Language, EmbeddingsModel};

let manager_config = OramaCoreManagerConfig {
    url: "https://api.orama.com".to_string(),
    master_api_key: "your-master-key".to_string(),
};

let manager = OramaCoreManager::new(manager_config).await?;

// Create a new collection
let collection_params = CreateCollectionParams::new("my-collection")
    .with_description("My new collection")
    .with_language(Language::English)
    .with_embeddings_model(EmbeddingsModel::BgeBase);

let new_collection = manager.collection.create(collection_params).await?;
```

## Advanced Features

### System Prompts

Manage AI system prompts:

```rust
// Get all system prompts
let prompts = client.system_prompts.get_all().await?;

// Get a specific prompt
let prompt = client.system_prompts.get("prompt-id").await?;
```

### Tools Integration

Execute custom tools:

```rust
use oramacore_client::collection::ExecuteToolsBody;

let tools_body = ExecuteToolsBody {
    tool_ids: Some(vec!["tool1".to_string()]),
    messages: vec![/* your messages */],
    llm_config: None,
};

let result = client.tools.execute::<serde_json::Value>(tools_body).await?;
```

### Hooks Management

Manage collection hooks:

```rust
use oramacore_client::collection::AddHookConfig;
use oramacore_client::types::Hook;

let hook_config = AddHookConfig {
    name: Hook::BeforeInsert,
    code: "// Your hook code".to_string(),
};

let hook_response = client.hooks.insert(hook_config).await?;
```

## Examples

The repository includes comprehensive examples:

- **basic_search.rs** - Simple search operations
- **document_management.rs** - CRUD operations
- **ai_session.rs** - AI-powered features and streaming
- **server_app.rs** - Complete application example

Run an example:

```bash
cargo run --example basic_search
```

## Error Handling

The client provides detailed error types:

```rust
use oramacore_client::error::{OramaError, Result};

match client.search(&params).await {
    Ok(results) => println!("Found {} results", results.hits.len()),
    Err(OramaError::Auth(msg)) => eprintln!("Authentication error: {}", msg),
    Err(OramaError::Api { status, message }) => {
        eprintln!("API error {}: {}", status, message);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Configuration

### Environment Variables

You can configure the client using environment variables:

```bash
export ORAMA_MASTER_API_KEY="your-master-key"
export ORAMA_PROJECT_ID="your-project-id"
export ORAMA_COLLECTION_API_KEY="your-collection-key"
```

### Custom Cluster Configuration

For custom deployments:

```rust
use oramacore_client::collection::ClusterConfig;

let config = CollectionManagerConfig::new("collection-id", "api-key")
    .with_cluster(
        ClusterConfig::new()
            .with_read_url("https://your-reader.com")
            .with_writer_url("https://your-writer.com")
    );
```

## Performance

The client is designed for high-performance applications:

- **Connection Pooling**: Reuses HTTP connections
- **Async Operations**: Non-blocking I/O with Tokio
- **Concurrent Requests**: Execute multiple operations simultaneously
- **Streaming**: Real-time data processing
- **Type Safety**: Zero-cost abstractions with compile-time guarantees

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_search_operations
```

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to the main repository.

## Support

- **Documentation**: [docs.rs/oramacore-client](https://docs.rs/oramacore-client)
- **Issues**: [GitHub Issues](https://github.com/oramasearch/oramacore-client-rust/issues)
- **Orama Documentation**: [docs.orama.com](https://docs.orama.com)
