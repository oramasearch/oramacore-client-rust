//! Document management example showing CRUD operations.

use oramacore_client::collection::{CollectionManager, CollectionManagerConfig, CreateIndexParams};
use oramacore_client::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Article {
    id: String,
    title: String,
    content: String,
    author: String,
    tags: Vec<String>,
    published_at: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the collection manager
    let config = CollectionManagerConfig::new("your-collection-id", "your-api-key");
    let client = CollectionManager::new(config).await?;

    // Example 1: Create an index
    println!("=== Creating Index ===");
    let create_index_params = CreateIndexParams {
        id: Some("articles".to_string()),
        embeddings: Some(serde_json::json!("automatic")),
    };

    client.index.create(create_index_params).await?;
    println!("Index 'articles' created successfully");

    // Example 2: Insert documents
    println!("\n=== Inserting Documents ===");
    let sample_articles = vec![
        Article {
            id: "1".to_string(),
            title: "Introduction to Machine Learning".to_string(),
            content: "Machine learning is a subset of artificial intelligence...".to_string(),
            author: "John Doe".to_string(),
            tags: vec!["AI".to_string(), "ML".to_string(), "Technology".to_string()],
            published_at: "2024-01-15".to_string(),
        },
        Article {
            id: "2".to_string(),
            title: "Deep Learning Fundamentals".to_string(),
            content: "Deep learning uses neural networks with multiple layers...".to_string(),
            author: "Jane Smith".to_string(),
            tags: vec![
                "AI".to_string(),
                "Deep Learning".to_string(),
                "Neural Networks".to_string(),
            ],
            published_at: "2024-02-01".to_string(),
        },
        Article {
            id: "3".to_string(),
            title: "Natural Language Processing".to_string(),
            content: "NLP combines computational linguistics with statistical models..."
                .to_string(),
            author: "Bob Johnson".to_string(),
            tags: vec!["NLP".to_string(), "AI".to_string(), "Language".to_string()],
            published_at: "2024-02-15".to_string(),
        },
    ];

    let articles_index = client.index.set("articles".to_string());
    articles_index.insert_documents(sample_articles).await?;
    println!("Documents inserted successfully");

    // Example 3: Update documents (upsert)
    println!("\n=== Updating Documents ===");
    let updated_article = Article {
        id: "1".to_string(),
        title: "Introduction to Machine Learning - Updated".to_string(),
        content: "Machine learning is a powerful subset of artificial intelligence that enables computers to learn without explicit programming...".to_string(),
        author: "John Doe".to_string(),
        tags: vec!["AI".to_string(), "ML".to_string(), "Technology".to_string(), "Updated".to_string()],
        published_at: "2024-01-15".to_string(),
    };

    articles_index
        .upsert_documents(vec![updated_article])
        .await?;
    println!("Document updated successfully");

    // Example 4: Delete specific documents
    println!("\n=== Deleting Documents ===");
    let documents_to_delete = vec!["3".to_string()];
    articles_index.delete_documents(documents_to_delete).await?;
    println!("Documents deleted successfully");

    // Example 5: Reindex the collection
    println!("\n=== Reindexing Collection ===");
    articles_index.reindex().await?;
    println!("Collection reindexed successfully");

    // Example 6: Get collection stats
    println!("\n=== Collection Statistics ===");
    let stats = client.collections.get_stats().await?;
    println!(
        "Collection stats: {}",
        serde_json::to_string_pretty(&stats)?
    );

    // Example 7: Clean up - delete the index
    println!("\n=== Cleaning Up ===");
    client.index.delete("articles").await?;
    println!("Index deleted successfully");

    Ok(())
}
