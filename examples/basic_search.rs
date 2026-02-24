//! Basic search example showing how to perform searches with the Orama client.

use oramacore_client::collection::{CollectionManager, CollectionManagerConfig};
use oramacore_client::error::Result;
use oramacore_client::types::{SearchMode, SearchParams};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
    category: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the collection manager
    let config = CollectionManagerConfig::new("your-collection-id", "your-api-key");
    let client = CollectionManager::new(config).await?;

    // Example 1: Basic text search
    println!("=== Basic Text Search ===");
    let search_params = SearchParams::new("artificial intelligence")
        .with_mode(SearchMode::Fulltext)
        .with_limit(5);

    let results = client.search::<Document>(&search_params).await?;
    println!("Found {} documents", results.hits.len());

    for hit in &results.hits {
        println!("- {} (score: {:.4})", hit.document.title, hit.score);
    }

    // Example 2: Vector search
    println!("\n=== Vector Search ===");
    let vector_search = SearchParams::new("machine learning algorithms")
        .with_mode(SearchMode::Vector)
        .with_limit(3)
        .with_properties(vec!["title".to_string(), "content".to_string()]);

    let vector_results = client.search::<Document>(&vector_search).await?;
    println!(
        "Vector search found {} documents",
        vector_results.hits.len()
    );

    for hit in &vector_results.hits {
        println!("- {} (similarity: {:.4})", hit.document.title, hit.score);
    }

    // Example 3: Hybrid search
    println!("\n=== Hybrid Search ===");
    let hybrid_search = SearchParams::new("deep learning")
        .with_mode(SearchMode::Hybrid)
        .with_limit(10)
        .with_threshold(0.7);

    let hybrid_results = client.search::<Document>(&hybrid_search).await?;
    println!(
        "Hybrid search found {} documents",
        hybrid_results.hits.len()
    );

    if let Some(elapsed) = &hybrid_results.elapsed {
        println!("Search completed in {}", elapsed.formatted);
    }

    // Example 4: Search with filters
    println!("\n=== Filtered Search ===");
    let mut where_clause = serde_json::Map::new();
    where_clause.insert("category".to_string(), serde_json::json!("technology"));

    let filtered_search = SearchParams::new("neural networks")
        .with_mode(SearchMode::Fulltext)
        .with_where(serde_json::Value::Object(where_clause))
        .with_limit(5);

    let filtered_results = client.search::<Document>(&filtered_search).await?;
    println!(
        "Filtered search found {} documents",
        filtered_results.hits.len()
    );

    Ok(())
}
