//! Common test utilities and helpers.

use serde::{Deserialize, Serialize};
use std::env;

/// Test document structure used across tests
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub tags: Option<Vec<String>>,
}

/// Test user structure
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TestUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

/// Get test configuration from environment variables
pub fn get_test_config() -> TestConfig {
    TestConfig {
        collection_id: env::var("TEST_COLLECTION_ID")
            .unwrap_or_else(|_| "test-collection-rust".to_string()),
        api_key: env::var("TEST_API_KEY")
            .unwrap_or_else(|_| "test-api-key".to_string()),
        master_key: env::var("TEST_MASTER_KEY")
            .unwrap_or_else(|_| "test-master-key".to_string()),
        project_id: env::var("TEST_PROJECT_ID")
            .unwrap_or_else(|_| "test-project".to_string()),
        base_url: env::var("TEST_BASE_URL")
            .unwrap_or_else(|_| "https://api.orama.com".to_string()),
    }
}

/// Test configuration structure
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub collection_id: String,
    pub api_key: String,
    pub master_key: String,
    pub project_id: String,
    pub base_url: String,
}

/// Create sample test documents
pub fn create_test_documents() -> Vec<TestDocument> {
    vec![
        TestDocument {
            id: "doc1".to_string(),
            title: "Introduction to Machine Learning".to_string(),
            content: "Machine learning is a method of data analysis that automates analytical model building. It is a branch of artificial intelligence based on the idea that systems can learn from data, identify patterns and make decisions with minimal human intervention.".to_string(),
            category: "Technology".to_string(),
            tags: Some(vec!["AI".to_string(), "ML".to_string(), "Data Science".to_string()]),
        },
        TestDocument {
            id: "doc2".to_string(),
            title: "Deep Learning Networks".to_string(),
            content: "Deep learning is part of a broader family of machine learning methods based on artificial neural networks with representation learning. Learning can be supervised, semi-supervised or unsupervised.".to_string(),
            category: "AI".to_string(),
            tags: Some(vec!["Deep Learning".to_string(), "Neural Networks".to_string(), "AI".to_string()]),
        },
        TestDocument {
            id: "doc3".to_string(),
            title: "Natural Language Processing Fundamentals".to_string(),
            content: "Natural language processing (NLP) is a subfield of linguistics, computer science, and artificial intelligence concerned with the interactions between computers and human language.".to_string(),
            category: "AI".to_string(),
            tags: Some(vec!["NLP".to_string(), "Linguistics".to_string(), "AI".to_string()]),
        },
        TestDocument {
            id: "doc4".to_string(),
            title: "Computer Vision Applications".to_string(),
            content: "Computer vision is an interdisciplinary scientific field that deals with how computers can gain high-level understanding from digital images or videos.".to_string(),
            category: "Technology".to_string(),
            tags: Some(vec!["Computer Vision".to_string(), "Image Processing".to_string(), "AI".to_string()]),
        },
        TestDocument {
            id: "doc5".to_string(),
            title: "Robotics and Automation".to_string(),
            content: "Robotics is an interdisciplinary branch of computer science and engineering that involves the conception, design, manufacture, and operation of robots.".to_string(),
            category: "Engineering".to_string(),
            tags: Some(vec!["Robotics".to_string(), "Automation".to_string(), "Engineering".to_string()]),
        },
    ]
}

/// Create sample test users
pub fn create_test_users() -> Vec<TestUser> {
    vec![
        TestUser {
            id: "user1".to_string(),
            name: "John Doe".to_string(),
            email: "john.doe@example.com".to_string(),
        },
        TestUser {
            id: "user2".to_string(),
            name: "Jane Smith".to_string(),
            email: "jane.smith@example.com".to_string(),
        },
        TestUser {
            id: "user3".to_string(),
            name: "Bob Johnson".to_string(),
            email: "bob.johnson@example.com".to_string(),
        },
    ]
}

/// Check if integration tests should run
pub fn should_run_integration_tests() -> bool {
    env::var("TEST_INTEGRATION").is_ok()
}

/// Check if we have valid test credentials
pub fn has_valid_credentials() -> bool {
    env::var("TEST_API_KEY").is_ok() 
        && env::var("TEST_COLLECTION_ID").is_ok()
        && !env::var("TEST_API_KEY").unwrap_or_default().starts_with("test-")
}

/// Wait for a short period (useful in async tests)
pub async fn wait_short() {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

/// Wait for a medium period (useful for operations that need time)
pub async fn wait_medium() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

/// Generate a random test ID
pub fn generate_test_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("test_{}", timestamp)
}

/// Setup test logging (useful for debugging tests)
pub fn setup_test_logging() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_documents() {
        let docs = create_test_documents();
        assert_eq!(docs.len(), 5);
        assert_eq!(docs[0].id, "doc1");
        assert!(docs[0].content.contains("machine learning"));
    }

    #[test]
    fn test_create_test_users() {
        let users = create_test_users();
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].name, "John Doe");
    }

    #[test]
    fn test_generate_test_id() {
        let id1 = generate_test_id();
        let id2 = generate_test_id();
        
        assert!(id1.starts_with("test_"));
        assert!(id2.starts_with("test_"));
        assert_ne!(id1, id2); // Should be different due to timestamp
    }

    #[test]
    fn test_get_test_config() {
        let config = get_test_config();
        assert!(!config.collection_id.is_empty());
        assert!(!config.api_key.is_empty());
        assert!(!config.base_url.is_empty());
    }
}