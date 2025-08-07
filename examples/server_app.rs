//! Complete server application example using all Orama features.

use oramacore_client::{
    cloud::{CloudSearchParams, OramaCloud, ProjectManagerConfig},
    collection::{ClusterConfig, CollectionManager, CollectionManagerConfig},
    error::Result,
    manager::{CreateCollectionParams, OramaCoreManager, OramaCoreManagerConfig},
    types::{EmbeddingsModel, Language, LlmConfig, LlmProvider, SearchMode},
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Product {
    id: String,
    name: String,
    description: String,
    category: String,
    price: f64,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    preferences: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting Orama Server Application Demo");

    // Load configuration from environment
    let master_api_key =
        env::var("ORAMA_MASTER_API_KEY").unwrap_or_else(|_| "your-master-api-key".to_string());
    let project_id = env::var("ORAMA_PROJECT_ID").unwrap_or_else(|_| "your-project-id".to_string());
    let collection_api_key = env::var("ORAMA_COLLECTION_API_KEY")
        .unwrap_or_else(|_| "your-collection-api-key".to_string());

    // ==========================================================================
    // PART 1: Collection Management with OramaCoreManager
    // ==========================================================================
    println!("\n📋 Part 1: Collection Management");

    let manager_config = OramaCoreManagerConfig {
        url: "https://api.orama.com".to_string(),
        master_api_key: master_api_key.clone(),
    };

    let core_manager = OramaCoreManager::new(manager_config).await?;

    // Create a new collection
    println!("Creating new collection...");
    let collection_params = CreateCollectionParams::new("demo-products-collection")
        .with_description("Demo collection for products")
        .with_language(Language::English)
        .with_embeddings_model(EmbeddingsModel::BgeBase);

    let new_collection = core_manager.collection.create(collection_params).await?;
    println!("✅ Collection created: {}", new_collection.id);
    println!("   Write API Key: {}", new_collection.write_api_key);
    println!("   Read API Key: {}", new_collection.readonly_api_key);

    // List all collections
    println!("\nListing all collections...");
    let collections = core_manager.collection.list().await?;
    println!("✅ Found {} collections", collections.len());
    for collection in &collections {
        println!(
            "   - {} (docs: {})",
            collection.id, collection.document_count
        );
    }

    // ==========================================================================
    // PART 2: Document Management with CollectionManager
    // ==========================================================================
    println!("\n📄 Part 2: Document Management");

    // Use the newly created collection's write API key
    let collection_config =
        CollectionManagerConfig::new(&new_collection.id, &new_collection.write_api_key)
            .with_cluster(ClusterConfig::new().with_read_url("https://collections.orama.com"));

    let collection_client = CollectionManager::new(collection_config).await?;

    // Insert sample products
    println!("Inserting sample products...");
    let sample_products = vec![
        Product {
            id: "1".to_string(),
            name: "Smart Laptop Pro".to_string(),
            description: "High-performance laptop with AI capabilities and long battery life"
                .to_string(),
            category: "Electronics".to_string(),
            price: 1299.99,
            tags: vec![
                "laptop".to_string(),
                "AI".to_string(),
                "professional".to_string(),
            ],
        },
        Product {
            id: "2".to_string(),
            name: "Wireless Earbuds Ultra".to_string(),
            description: "Premium wireless earbuds with noise cancellation and spatial audio"
                .to_string(),
            category: "Audio".to_string(),
            price: 299.99,
            tags: vec![
                "earbuds".to_string(),
                "wireless".to_string(),
                "premium".to_string(),
            ],
        },
        Product {
            id: "3".to_string(),
            name: "Smart Home Hub".to_string(),
            description:
                "Central hub for controlling all your smart home devices with voice control"
                    .to_string(),
            category: "Smart Home".to_string(),
            price: 149.99,
            tags: vec![
                "smart home".to_string(),
                "hub".to_string(),
                "voice control".to_string(),
            ],
        },
    ];

    let main_index = collection_client.index.set("main".to_string());
    main_index.insert_documents(sample_products.clone()).await?;
    println!("✅ {} products inserted", sample_products.len());

    // Search products
    println!("\nSearching for products...");
    let search_params = oramacore_client::types::SearchParams::new("laptop AI professional")
        .with_mode(SearchMode::Hybrid)
        .with_limit(5);

    let search_results = collection_client.search::<Product>(&search_params).await?;
    println!("✅ Search found {} results", search_results.hits.len());
    for hit in &search_results.hits {
        println!(
            "   - {} (score: {:.4}) - ${}",
            hit.document.name, hit.score, hit.document.price
        );
    }

    // ==========================================================================
    // PART 3: AI-Powered Operations
    // ==========================================================================
    println!("\n🤖 Part 3: AI-Powered Operations");

    // Create AI session
    let ai_session = collection_client.ai.create_ai_session().await?;
    println!("✅ AI session created: {}", ai_session.session_id());

    // AI-powered product recommendation
    let recommendation_query = oramacore_client::stream_manager::AnswerConfig::new(
        "I'm looking for a high-tech device for productivity and entertainment. What would you recommend?"
    )
    .with_max_documents(3)
    .with_min_similarity(0.7);

    println!("\nGetting AI recommendation...");
    let recommendation = ai_session.answer(recommendation_query).await?;
    println!("🤖 AI Recommendation: {}", recommendation);

    // ==========================================================================
    // PART 4: Cloud Operations (if available)
    // ==========================================================================
    println!("\n☁️ Part 4: Cloud Operations");

    let cloud_config = ProjectManagerConfig::new(&project_id, &collection_api_key);
    let cloud_client = OramaCloud::new(cloud_config).await?;

    // Cloud-specific search with datasources
    println!("Performing cloud search...");
    let cloud_search = CloudSearchParams::new(
        "smart technology",
        vec!["main".to_string()], // datasource IDs
    )
    .with_mode(SearchMode::Vector)
    .with_limit(3);

    let cloud_results = cloud_client.search::<Product>(&cloud_search).await?;
    println!("✅ Cloud search found {} results", cloud_results.hits.len());
    for hit in &cloud_results.hits {
        println!(
            "   - {} (${}) in category: {}",
            hit.document.name, hit.document.price, hit.document.category
        );
    }

    // ==========================================================================
    // PART 5: Advanced Features
    // ==========================================================================
    println!("\n⚡ Part 5: Advanced Features");

    // System prompts management
    println!("Managing system prompts...");
    let system_prompts = collection_client.system_prompts.get_all().await?;
    println!("✅ Found {} system prompts", system_prompts.len());

    // Tools management
    println!("Managing tools...");
    let tools = collection_client.tools.get_all().await?;
    println!("✅ Found {} tools", tools.len());

    // Hooks management
    println!("Managing hooks...");
    let hooks = collection_client.hooks.list().await?;
    println!("✅ Found {} hooks", hooks.len());

    // ==========================================================================
    // PART 6: Analytics and Monitoring
    // ==========================================================================
    println!("\n📊 Part 6: Analytics and Monitoring");

    // Get collection statistics
    let stats = collection_client
        .collections
        .get_stats(&new_collection.id)
        .await?;
    println!("📊 Collection Statistics:");
    println!("{}", serde_json::to_string_pretty(&stats)?);

    // Performance testing
    println!("\nPerformance testing - Multiple concurrent searches...");
    let mut tasks = Vec::new();

    for i in 0..5 {
        let client_clone = collection_client.clone();
        let query = format!("search query {}", i);

        let task = tokio::spawn(async move {
            let params = oramacore_client::types::SearchParams::new(&query).with_limit(3);
            client_clone.search::<Product>(&params).await
        });

        tasks.push(task);
    }

    // Wait for all searches to complete
    let mut successful_searches = 0;
    for task in tasks {
        match task.await {
            Ok(Ok(result)) => {
                successful_searches += 1;
                println!("   ✅ Search completed: {} results", result.hits.len());
            }
            Ok(Err(e)) => println!("   ❌ Search failed: {}", e),
            Err(e) => println!("   ❌ Task failed: {}", e),
        }
    }

    println!(
        "📊 Performance test completed: {}/5 searches successful",
        successful_searches
    );

    // ==========================================================================
    // PART 7: Cleanup
    // ==========================================================================
    println!("\n🧹 Part 7: Cleanup");

    // Clear AI session
    ai_session.clear_session().await;
    println!("✅ AI session cleared");

    // Note: In a real application, you might want to keep the collection
    // For demo purposes, we're showing how to delete it
    println!("Demo completed successfully! 🎉");
    println!("\n💡 Tips for production:");
    println!("   - Store API keys securely (environment variables, secrets manager)");
    println!("   - Implement proper error handling and retries");
    println!("   - Use connection pooling for high-throughput applications");
    println!("   - Monitor API usage and performance metrics");
    println!("   - Implement caching for frequently accessed data");

    Ok(())
}
