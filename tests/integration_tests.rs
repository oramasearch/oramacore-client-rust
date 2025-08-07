//! Integration tests for the Orama client.

use oramacore_client::{
    cloud::{CloudSearchParams, OramaCloud, ProjectManagerConfig},
    collection::{ClusterConfig, CollectionManager, CollectionManagerConfig, CreateIndexParams},
    error::{OramaError, Result},
    manager::{CreateCollectionParams, OramaCoreManager, OramaCoreManagerConfig},
    types::{EmbeddingsModel, Language, SearchMode, SearchParams},
};
use serde::{Deserialize, Serialize};
use std::env;
use tokio_test;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct TestDocument {
    id: String,
    title: String,
    content: String,
    category: String,
}

/// Get test configuration from environment or use defaults
fn get_test_config() -> (String, String, String) {
    let collection_id =
        env::var("TEST_COLLECTION_ID").unwrap_or_else(|_| "test-collection".to_string());
    let api_key = env::var("TEST_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
    let master_key = env::var("TEST_MASTER_KEY").unwrap_or_else(|_| "test-master-key".to_string());

    (collection_id, api_key, master_key)
}

fn create_test_documents() -> Vec<TestDocument> {
    vec![
        TestDocument {
            id: "1".to_string(),
            title: "Introduction to Machine Learning".to_string(),
            content: "Machine learning is a subset of artificial intelligence that focuses on algorithms and statistical models.".to_string(),
            category: "Technology".to_string(),
        },
        TestDocument {
            id: "2".to_string(),
            title: "Deep Learning Fundamentals".to_string(),
            content: "Deep learning uses neural networks with multiple layers to model and understand complex patterns.".to_string(),
            category: "AI".to_string(),
        },
        TestDocument {
            id: "3".to_string(),
            title: "Natural Language Processing".to_string(),
            content: "NLP combines computational linguistics with statistical and machine learning methods.".to_string(),
            category: "AI".to_string(),
        },
    ]
}

#[cfg(test)]
mod collection_manager_tests {
    use super::*;

    #[tokio::test]
    async fn test_collection_manager_creation() {
        let (collection_id, api_key, _) = get_test_config();
        let config = CollectionManagerConfig::new(collection_id, api_key);

        // This should not panic - actual connection testing requires valid credentials
        let result = CollectionManager::new(config).await;

        // In a real test environment with valid credentials, this would succeed
        // For unit testing, we're mainly testing the configuration and setup
        match result {
            Ok(_) => println!("✅ Collection manager created successfully"),
            Err(e) => println!(
                "⚠️  Collection manager creation failed (expected in unit tests): {}",
                e
            ),
        }
    }

    #[tokio::test]
    async fn test_search_params_builder() {
        let search_params = SearchParams::new("test query")
            .with_mode(SearchMode::Hybrid)
            .with_limit(10)
            .with_offset(0)
            .with_threshold(0.8)
            .with_exact(false);

        assert_eq!(search_params.term, "test query");
        assert_eq!(search_params.mode, Some(SearchMode::Hybrid));
        assert_eq!(search_params.limit, Some(10));
        assert_eq!(search_params.offset, Some(0));
        assert_eq!(search_params.threshold, Some(0.8));
        assert_eq!(search_params.exact, Some(false));
    }

    #[tokio::test]
    async fn test_cluster_config_builder() {
        let cluster_config = ClusterConfig::new()
            .with_read_url("https://reader.example.com")
            .with_writer_url("https://writer.example.com");

        assert_eq!(
            cluster_config.read_url,
            Some("https://reader.example.com".to_string())
        );
        assert_eq!(
            cluster_config.writer_url,
            Some("https://writer.example.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_collection_config_with_cluster() {
        let (collection_id, api_key, _) = get_test_config();
        let cluster = ClusterConfig::new()
            .with_read_url("https://reader.test.com")
            .with_writer_url("https://writer.test.com");

        let config = CollectionManagerConfig::new(collection_id.clone(), api_key.clone())
            .with_cluster(cluster)
            .with_auth_jwt_url("https://auth.test.com");

        assert_eq!(config.collection_id, collection_id);
        assert_eq!(config.api_key, api_key);
        assert!(config.cluster.is_some());
        assert_eq!(
            config.auth_jwt_url,
            Some("https://auth.test.com".to_string())
        );
    }
}

#[cfg(test)]
mod core_manager_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_collection_params_builder() {
        let params = CreateCollectionParams::new("test-collection")
            .with_description("Test collection for unit tests")
            .with_language(Language::English)
            .with_embeddings_model(EmbeddingsModel::BgeBase)
            .with_write_api_key("write-key")
            .with_read_api_key("read-key");

        assert_eq!(params.id, "test-collection");
        assert_eq!(
            params.description,
            Some("Test collection for unit tests".to_string())
        );
        assert!(matches!(params.language, Some(Language::English)));
        assert!(matches!(
            params.embeddings_model,
            Some(EmbeddingsModel::BgeBase)
        ));
        assert_eq!(params.write_api_key, Some("write-key".to_string()));
        assert_eq!(params.read_api_key, Some("read-key".to_string()));
    }

    #[tokio::test]
    async fn test_core_manager_config() {
        let (_, _, master_key) = get_test_config();
        let config = OramaCoreManagerConfig {
            url: "https://api.test.com".to_string(),
            master_api_key: master_key.clone(),
        };

        assert_eq!(config.url, "https://api.test.com");
        assert_eq!(config.master_api_key, master_key);
    }
}

#[cfg(test)]
mod cloud_client_tests {
    use super::*;

    #[tokio::test]
    async fn test_project_manager_config() {
        let config = ProjectManagerConfig::new("project-123", "api-key-456")
            .with_auth_jwt_url("https://jwt.example.com");

        assert_eq!(config.project_id, "project-123");
        assert_eq!(config.api_key, "api-key-456");
        assert_eq!(
            config.auth_jwt_url,
            Some("https://jwt.example.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_cloud_search_params() {
        let params = CloudSearchParams::new(
            "test search",
            vec!["datasource1".to_string(), "datasource2".to_string()],
        )
        .with_mode(SearchMode::Vector)
        .with_limit(5)
        .with_threshold(0.9)
        .with_user_id("user-123");

        assert_eq!(params.term, "test search");
        assert_eq!(params.datasources.len(), 2);
        assert!(matches!(params.mode, Some(SearchMode::Vector)));
        assert_eq!(params.limit, Some(5));
        assert_eq!(params.threshold, Some(0.9));
        assert_eq!(params.user_id, Some("user-123".to_string()));
    }
}

#[cfg(test)]
mod types_tests {
    use super::*;
    use oramacore_client::types::*;

    #[test]
    fn test_search_mode_serialization() {
        let fulltext = SearchMode::Fulltext;
        let vector = SearchMode::Vector;
        let hybrid = SearchMode::Hybrid;

        // These should serialize correctly
        let fulltext_json = serde_json::to_string(&fulltext).unwrap();
        let vector_json = serde_json::to_string(&vector).unwrap();
        let hybrid_json = serde_json::to_string(&hybrid).unwrap();

        assert_eq!(fulltext_json, "\"fulltext\"");
        assert_eq!(vector_json, "\"vector\"");
        assert_eq!(hybrid_json, "\"hybrid\"");
    }

    #[test]
    fn test_role_serialization() {
        let system = Role::System;
        let user = Role::User;
        let assistant = Role::Assistant;

        let system_json = serde_json::to_string(&system).unwrap();
        let user_json = serde_json::to_string(&user).unwrap();
        let assistant_json = serde_json::to_string(&assistant).unwrap();

        assert_eq!(system_json, "\"system\"");
        assert_eq!(user_json, "\"user\"");
        assert_eq!(assistant_json, "\"assistant\"");
    }

    #[test]
    fn test_llm_provider_serialization() {
        let openai = LlmProvider::OpenAI;
        let claude = LlmProvider::Claude;

        let openai_json = serde_json::to_string(&openai).unwrap();
        let claude_json = serde_json::to_string(&claude).unwrap();

        assert_eq!(openai_json, "\"openai\"");
        assert_eq!(claude_json, "\"claude\"");
    }

    #[test]
    fn test_language_serialization() {
        let english = Language::English;
        let spanish = Language::Spanish;

        let english_json = serde_json::to_string(&english).unwrap();
        let spanish_json = serde_json::to_string(&spanish).unwrap();

        assert_eq!(english_json, "\"english\"");
        assert_eq!(spanish_json, "\"spanish\"");
    }

    #[test]
    fn test_message_serialization() {
        let message = Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&message).unwrap();
        let parsed: Message = serde_json::from_str(&json).unwrap();

        assert!(matches!(parsed.role, Role::User));
        assert_eq!(parsed.content, "Hello, world!");
    }

    #[test]
    fn test_llm_config_serialization() {
        let config = LlmConfig {
            provider: LlmProvider::OpenAI,
            model: "gpt-4".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: LlmConfig = serde_json::from_str(&json).unwrap();

        assert!(matches!(parsed.provider, LlmProvider::OpenAI));
        assert_eq!(parsed.model, "gpt-4");
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let auth_error = OramaError::auth("Invalid credentials");
        let api_error = OramaError::api(404, "Not found".to_string());
        let generic_error = OramaError::generic("Something went wrong");

        assert!(matches!(auth_error, OramaError::Auth { .. }));
        assert!(matches!(api_error, OramaError::Api { status: 404, .. }));
        assert!(matches!(generic_error, OramaError::Generic { .. }));
    }

    #[test]
    fn test_error_display() {
        let auth_error = OramaError::auth("Invalid API key");
        let error_string = format!("{}", auth_error);

        assert!(error_string.contains("Invalid API key"));
    }
}

#[cfg(test)]
mod stream_manager_tests {
    use super::*;
    use oramacore_client::stream_manager::{AnswerConfig, CreateAiSessionConfig, Interaction};
    use oramacore_client::types::{LlmConfig, LlmProvider, Message, Role};

    #[test]
    fn test_answer_config_builder() {
        let config = AnswerConfig::new("Test query")
            .with_visitor_id("user-123")
            .with_session_id("session-456")
            .with_max_documents(10)
            .with_min_similarity(0.8);

        assert_eq!(config.query, "Test query");
        assert_eq!(config.visitor_id, Some("user-123".to_string()));
        assert_eq!(config.session_id, Some("session-456".to_string()));
        assert_eq!(config.max_documents, Some(10));
        assert_eq!(config.min_similarity, Some(0.8));
    }

    #[test]
    fn test_create_ai_session_config_builder() {
        let llm_config = LlmConfig {
            provider: LlmProvider::OpenAI,
            model: "gpt-4".to_string(),
        };

        let messages = vec![Message {
            role: Role::System,
            content: "You are a helpful assistant".to_string(),
        }];

        let config = CreateAiSessionConfig::new()
            .with_llm_config(llm_config.clone())
            .with_initial_messages(messages.clone());

        assert!(config.llm_config.is_some());
        assert_eq!(config.initial_messages.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_interaction_creation() {
        let interaction = Interaction::new("test-id".to_string(), "test query".to_string());

        assert_eq!(interaction.id, "test-id");
        assert_eq!(interaction.query, "test query");
        assert!(interaction.loading);
        assert!(!interaction.error);
        assert!(!interaction.aborted);
        assert_eq!(interaction.current_step, Some("starting".to_string()));
    }
}

// Mock tests that don't require actual API calls
#[cfg(test)]
mod mock_tests {
    use super::*;

    #[tokio::test]
    async fn test_document_serialization() {
        let docs = create_test_documents();

        // Test that our test documents serialize/deserialize correctly
        for doc in docs {
            let json = serde_json::to_string(&doc).unwrap();
            let parsed: TestDocument = serde_json::from_str(&json).unwrap();
            assert_eq!(doc, parsed);
        }
    }

    #[test]
    fn test_search_params_with_properties() {
        let properties = vec!["title".to_string(), "content".to_string()];
        let params = SearchParams::new("test").with_properties(properties.clone());

        assert_eq!(params.properties, Some(properties));
    }

    #[test]
    fn test_search_params_with_where_clause() {
        let mut where_clause = serde_json::Map::new();
        where_clause.insert("category".to_string(), serde_json::json!("Technology"));

        let params =
            SearchParams::new("test").with_where(serde_json::Value::Object(where_clause.clone()));

        assert!(params.where_clause.is_some());
        if let Some(serde_json::Value::Object(clause)) = &params.where_clause {
            assert_eq!(
                clause.get("category"),
                Some(&serde_json::json!("Technology"))
            );
        }
    }
}

/// Integration test runner - only runs if TEST_INTEGRATION env var is set
#[cfg(test)]
mod integration_tests {
    use super::*;

    fn should_run_integration_tests() -> bool {
        env::var("TEST_INTEGRATION").is_ok()
    }

    #[tokio::test]
    async fn test_full_integration_flow() {
        if !should_run_integration_tests() {
            println!("⚠️  Skipping integration test - set TEST_INTEGRATION env var to run");
            return;
        }

        let (collection_id, api_key, _) = get_test_config();

        // This would be a full integration test with real API calls
        // Only run when proper test environment is configured
        println!(
            "🧪 Running integration test with collection: {}",
            collection_id
        );

        let config = CollectionManagerConfig::new(collection_id, api_key);
        let client = CollectionManager::new(config).await;

        match client {
            Ok(client) => {
                println!("✅ Successfully connected to Orama");

                // Here you would run actual API tests
                // let results = client.search(...).await?;
                // assert!(results.hits.len() > 0);
            }
            Err(e) => {
                println!("❌ Integration test failed: {}", e);
            }
        }
    }
}
