//! Orama Core Manager for collection management operations.

use crate::auth::{ApiKeyAuth, Auth, AuthConfig, Target};
use crate::client::{ApiKeyPosition, ClientRequest, OramaClient};
use crate::error::Result;
use crate::types::*;
use crate::utils::create_random_string;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for OramaCoreManager
#[derive(Debug, Clone)]
pub struct OramaCoreManagerConfig {
    pub url: String,
    pub master_api_key: String,
}

/// Parameters for creating a collection
#[derive(Debug, Clone, Serialize)]
pub struct CreateCollectionParams {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings_model: Option<EmbeddingsModel>,
}

/// Response from creating a new collection
#[derive(Debug, Clone, Deserialize)]
pub struct NewCollectionResponse {
    pub id: String,
    pub description: Option<String>,
    #[serde(rename = "writeAPIKey")]
    pub write_api_key: String,
    #[serde(rename = "readonlyAPIKey")]
    pub readonly_api_key: String,
}

/// Collection index field information
#[derive(Debug, Clone, Deserialize)]
pub struct CollectionIndexField {
    pub field_id: String,
    pub field_path: String,
    pub is_array: bool,
    pub field_type: serde_json::Value,
}

/// Collection index information
#[derive(Debug, Clone, Deserialize)]
pub struct CollectionIndex {
    pub id: String,
    pub document_count: u32,
    pub fields: Vec<CollectionIndexField>,
    pub automatically_chosen_properties: serde_json::Value,
}

/// Response from getting collections
#[derive(Debug, Clone, Deserialize)]
pub struct GetCollectionsResponse {
    pub id: String,
    pub description: Option<String>,
    pub document_count: u32,
    pub indexes: Vec<CollectionIndex>,
}

/// Collection management namespace
#[derive(Debug, Clone)]
pub struct CollectionNamespace {
    client: OramaClient,
}

impl CollectionNamespace {
    /// Create a new collection namespace
    pub(crate) fn new(client: OramaClient) -> Self {
        Self { client }
    }

    /// Create a new collection
    pub async fn create(&self, config: CreateCollectionParams) -> Result<NewCollectionResponse> {
        let mut body = serde_json::json!({
            "id": config.id,
            "description": config.description,
            "write_api_key": config.write_api_key.unwrap_or_else(|| create_random_string(32)),
            "read_api_key": config.read_api_key.unwrap_or_else(|| create_random_string(32)),
        });

        if let Some(embeddings_model) = config.embeddings_model {
            body["embeddings_model"] = serde_json::to_value(embeddings_model)?;
        }

        let request = ClientRequest::post(
            "/v1/collections/create".to_string(),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let response: serde_json::Value = self.client.request(request).await?;

        // Convert response to NewCollectionResponse
        Ok(NewCollectionResponse {
            id: response["id"].as_str().unwrap_or_default().to_string(),
            description: response["description"].as_str().map(|s| s.to_string()),
            write_api_key: response["write_api_key"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            readonly_api_key: response["read_api_key"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
    }

    /// List all collections
    pub async fn list(&self) -> Result<Vec<GetCollectionsResponse>> {
        let request = ClientRequest::<()>::get(
            "/v1/collections".to_string(),
            Target::Writer,
            ApiKeyPosition::Header,
        );

        self.client.request(request).await
    }

    /// Get a specific collection
    pub async fn get(&self, collection_id: &str) -> Result<GetCollectionsResponse> {
        let request = ClientRequest::<()>::get(
            format!("/v1/collections/{}", collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
        );

        self.client.request(request).await
    }

    /// Delete a collection
    pub async fn delete(&self, collection_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "collection_id_to_delete": collection_id
        });

        let request = ClientRequest::post(
            "/v1/collections/delete".to_string(),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }
}

/// Main manager class for Orama Core operations
#[derive(Debug, Clone)]
pub struct OramaCoreManager {
    pub collection: CollectionNamespace,
}

impl OramaCoreManager {
    /// Create a new OramaCoreManager
    pub async fn new(config: OramaCoreManagerConfig) -> Result<Self> {
        let auth_config =
            AuthConfig::ApiKey(ApiKeyAuth::new(config.master_api_key).with_writer_url(config.url));

        let client = Client::new();
        let auth = Auth::new(auth_config, Arc::new(client));
        let orama_client = OramaClient::new(auth)?;

        Ok(Self {
            collection: CollectionNamespace::new(orama_client),
        })
    }
}

impl CreateCollectionParams {
    /// Create a new CreateCollectionParams
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            description: None,
            write_api_key: None,
            read_api_key: None,
            language: None,
            embeddings_model: None,
        }
    }

    /// Set the description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the write API key
    pub fn with_write_api_key<S: Into<String>>(mut self, key: S) -> Self {
        self.write_api_key = Some(key.into());
        self
    }

    /// Set the read API key
    pub fn with_read_api_key<S: Into<String>>(mut self, key: S) -> Self {
        self.read_api_key = Some(key.into());
        self
    }

    /// Set the language
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Set the embeddings model
    pub fn with_embeddings_model(mut self, model: EmbeddingsModel) -> Self {
        self.embeddings_model = Some(model);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_collection_params_builder() {
        let params = CreateCollectionParams::new("test-collection")
            .with_description("Test collection")
            .with_language(Language::English)
            .with_embeddings_model(EmbeddingsModel::BgeBase);

        assert_eq!(params.id, "test-collection");
        assert_eq!(params.description, Some("Test collection".to_string()));
        assert!(matches!(params.language, Some(Language::English)));
        assert!(matches!(
            params.embeddings_model,
            Some(EmbeddingsModel::BgeBase)
        ));
    }
}
