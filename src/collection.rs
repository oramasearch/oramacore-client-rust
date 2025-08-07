//! Collection management and search functionality.

use crate::auth::{ApiKeyAuth, Auth, AuthConfig, JwtAuth, Target};
use crate::client::{ApiKeyPosition, ClientRequest, OramaClient};
use crate::error::Result;
use crate::stream_manager::OramaCoreStream;
use crate::types::*;
use crate::utils::{current_time_millis, format_duration};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

const DEFAULT_READER_URL: &str = "https://collections.orama.com";
const DEFAULT_JWT_URL: &str = "https://app.orama.com/api/user/jwt";

/// Configuration for CollectionManager
#[derive(Debug, Clone)]
pub struct CollectionManagerConfig {
    pub collection_id: String,
    pub api_key: String,
    pub cluster: Option<ClusterConfig>,
    pub auth_jwt_url: Option<String>,
}

/// Cluster configuration
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub writer_url: Option<String>,
    pub read_url: Option<String>,
}

/// NLP search parameters
#[derive(Debug, Clone, Serialize)]
pub struct NlpSearchParams {
    pub query: String,
    #[serde(rename = "LLMConfig", skip_serializing_if = "Option::is_none")]
    pub llm_config: Option<LlmConfig>,
    #[serde(rename = "userID", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Index creation parameters
#[derive(Debug, Clone, Serialize)]
pub struct CreateIndexParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeddings: Option<serde_json::Value>, // Can be "automatic", "all_properties", or array of strings
}

/// Hook configuration for insertion
#[derive(Debug, Clone, Serialize)]
pub struct AddHookConfig {
    pub name: Hook,
    pub code: String,
}

/// Response from hook insertion
#[derive(Debug, Clone, Deserialize)]
pub struct NewHookResponse {
    #[serde(rename = "hookID")]
    pub hook_id: String,
    pub code: String,
}

/// Execute tools request body
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteToolsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_ids: Option<Vec<String>>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_config: Option<LlmConfig>,
}

/// AI operations namespace
#[derive(Debug, Clone)]
pub struct AiNamespace {
    client: OramaClient,
    collection_id: String,
}

impl AiNamespace {
    pub(crate) fn new(client: OramaClient, collection_id: String) -> Self {
        Self {
            client,
            collection_id,
        }
    }

    /// Perform NLP-based search
    pub async fn nlp_search<T>(&self, params: NlpSearchParams) -> Result<Vec<NlpSearchResult<T>>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let request = ClientRequest::post(
            format!("/v1/collections/{}/nlp_search", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
            params,
        );

        self.client.request(request).await
    }

    /// Create an AI session for streaming conversations
    pub async fn create_ai_session(&self) -> Result<OramaCoreStream> {
        OramaCoreStream::new(self.collection_id.clone(), self.client.clone()).await
    }
}

/// Collections operations namespace
#[derive(Debug, Clone)]
pub struct CollectionsNamespace {
    client: OramaClient,
    collection_id: String,
}

impl CollectionsNamespace {
    pub(crate) fn new(client: OramaClient, collection_id: String) -> Self {
        Self {
            client,
            collection_id,
        }
    }

    /// Get collection statistics
    pub async fn get_stats(&self, collection_id: &str) -> Result<serde_json::Value> {
        let request = ClientRequest::<()>::get(
            format!("/v1/collections/{}/stats", collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
        );

        self.client.request(request).await
    }

    /// Get all documents in collection
    pub async fn get_all_docs<T>(&self, id: &str) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let body = serde_json::json!({ "id": id });
        let request = ClientRequest::post(
            "/v1/collections/list".to_string(),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        self.client.request(request).await
    }
}

/// Index operations namespace
#[derive(Debug, Clone)]
pub struct IndexNamespace {
    client: OramaClient,
    collection_id: String,
}

impl IndexNamespace {
    pub(crate) fn new(client: OramaClient, collection_id: String) -> Self {
        Self {
            client,
            collection_id,
        }
    }

    /// Create a new index
    pub async fn create(&self, config: CreateIndexParams) -> Result<()> {
        let body = serde_json::json!({
            "id": config.id,
            "embedding": config.embeddings
        });

        let request = ClientRequest::post(
            format!("/v1/collections/{}/indexes/create", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }

    /// Delete an index
    pub async fn delete(&self, index_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "index_id_to_delete": index_id
        });

        let request = ClientRequest::post(
            format!("/v1/collections/{}/indexes/delete", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }

    /// Get an Index instance for operations
    pub fn set(&self, id: String) -> Index {
        Index::new(self.client.clone(), self.collection_id.clone(), id)
    }
}

/// Hooks operations namespace
#[derive(Debug, Clone)]
pub struct HooksNamespace {
    client: OramaClient,
    collection_id: String,
}

impl HooksNamespace {
    pub(crate) fn new(client: OramaClient, collection_id: String) -> Self {
        Self {
            client,
            collection_id,
        }
    }

    /// Insert a new hook
    pub async fn insert(&self, config: AddHookConfig) -> Result<NewHookResponse> {
        let body = serde_json::json!({
            "name": config.name,
            "code": config.code
        });

        let request = ClientRequest::post(
            format!("/v1/collections/{}/hooks/set", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;

        Ok(NewHookResponse {
            hook_id: serde_json::to_string(&config.name)?,
            code: config.code,
        })
    }

    /// List all hooks
    pub async fn list(&self) -> Result<HashMap<String, Option<String>>> {
        let request = ClientRequest::<()>::get(
            format!("/v1/collections/{}/hooks/list", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
        );

        let response: serde_json::Value = self.client.request(request).await?;
        let empty_map = serde_json::Map::new();
        let hooks = response["hooks"].as_object().unwrap_or(&empty_map);

        let mut result = HashMap::new();
        for (key, value) in hooks {
            let val = value.as_str().map(|s| s.to_string());
            result.insert(key.clone(), val);
        }

        Ok(result)
    }

    /// Delete a hook
    pub async fn delete(&self, hook: Hook) -> Result<()> {
        let body = serde_json::json!({
            "name_to_delete": hook
        });

        let request = ClientRequest::post(
            format!("/v1/collections/{}/hooks/delete", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }
}

/// System prompts operations namespace
#[derive(Debug, Clone)]
pub struct SystemPromptsNamespace {
    client: OramaClient,
    collection_id: String,
}

impl SystemPromptsNamespace {
    pub(crate) fn new(client: OramaClient, collection_id: String) -> Self {
        Self {
            client,
            collection_id,
        }
    }

    /// Insert a system prompt
    pub async fn insert(&self, system_prompt: InsertSystemPromptBody) -> Result<serde_json::Value> {
        let request = ClientRequest::post(
            format!(
                "/v1/collections/{}/system_prompts/insert",
                self.collection_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            system_prompt,
        );

        self.client.request(request).await
    }

    /// Get a system prompt
    pub async fn get(&self, id: &str) -> Result<SystemPrompt> {
        let request = ClientRequest::<()>::get(
            format!("/v1/collections/{}/system_prompts/get", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
        )
        .with_param("system_prompt_id", id);

        let response: serde_json::Value = self.client.request(request).await?;
        let prompt = response["system_prompt"].clone();
        Ok(serde_json::from_value(prompt)?)
    }

    /// Get all system prompts
    pub async fn get_all(&self) -> Result<Vec<SystemPrompt>> {
        let request = ClientRequest::<()>::get(
            format!("/v1/collections/{}/system_prompts/all", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
        );

        let response: serde_json::Value = self.client.request(request).await?;
        let prompts = response["system_prompts"].clone();
        Ok(serde_json::from_value(prompts)?)
    }

    /// Delete a system prompt
    pub async fn delete(&self, id: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({ "id": id });
        let request = ClientRequest::post(
            format!(
                "/v1/collections/{}/system_prompts/delete",
                self.collection_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        self.client.request(request).await
    }

    /// Update a system prompt
    pub async fn update(&self, system_prompt: SystemPrompt) -> Result<serde_json::Value> {
        let request = ClientRequest::post(
            format!(
                "/v1/collections/{}/system_prompts/update",
                self.collection_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            system_prompt,
        );

        self.client.request(request).await
    }

    /// Validate a system prompt
    pub async fn validate(
        &self,
        system_prompt: SystemPrompt,
    ) -> Result<SystemPromptValidationResponse> {
        let request = ClientRequest::post(
            format!(
                "/v1/collections/{}/system_prompts/validate",
                self.collection_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            system_prompt,
        );

        let response: serde_json::Value = self.client.request(request).await?;
        let result = response["result"].clone();
        Ok(serde_json::from_value(result)?)
    }
}

/// Tools operations namespace
#[derive(Debug, Clone)]
pub struct ToolsNamespace {
    client: OramaClient,
    collection_id: String,
}

impl ToolsNamespace {
    pub(crate) fn new(client: OramaClient, collection_id: String) -> Self {
        Self {
            client,
            collection_id,
        }
    }

    /// Insert a tool
    pub async fn insert(&self, tool: InsertToolBody) -> Result<()> {
        let request = ClientRequest::post(
            format!("/v1/collections/{}/tools/insert", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
            tool,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }

    /// Get a tool
    pub async fn get(&self, id: &str) -> Result<Tool> {
        let request = ClientRequest::<()>::get(
            format!("/v1/collections/{}/tools/get", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
        )
        .with_param("tool_id", id);

        let response: serde_json::Value = self.client.request(request).await?;
        let tool = response["tool"].clone();
        Ok(serde_json::from_value(tool)?)
    }

    /// Get all tools
    pub async fn get_all(&self) -> Result<Vec<Tool>> {
        let request = ClientRequest::<()>::get(
            format!("/v1/collections/{}/tools/all", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
        );

        let response: serde_json::Value = self.client.request(request).await?;
        let tools = response["tools"].clone();
        Ok(serde_json::from_value(tools)?)
    }

    /// Delete a tool
    pub async fn delete(&self, id: &str) -> Result<serde_json::Value> {
        let body = serde_json::json!({ "id": id });
        let request = ClientRequest::post(
            format!("/v1/collections/{}/tools/delete", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        self.client.request(request).await
    }

    /// Update a tool
    pub async fn update(&self, tool: UpdateToolBody) -> Result<serde_json::Value> {
        let request = ClientRequest::post(
            format!("/v1/collections/{}/tools/update", self.collection_id),
            Target::Writer,
            ApiKeyPosition::Header,
            tool,
        );

        self.client.request(request).await
    }

    /// Execute tools
    pub async fn execute<T>(&self, tools: ExecuteToolsBody) -> Result<ExecuteToolsParsedResponse<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let request = ClientRequest::post(
            format!("/v1/collections/{}/tools/run", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
            tools,
        );

        self.client.request(request).await
    }
}

/// Index operations for document management
#[derive(Debug, Clone)]
pub struct Index {
    client: OramaClient,
    collection_id: String,
    index_id: String,
}

impl Index {
    pub(crate) fn new(client: OramaClient, collection_id: String, index_id: String) -> Self {
        Self {
            client,
            collection_id,
            index_id,
        }
    }

    /// Reindex the collection
    pub async fn reindex(&self) -> Result<()> {
        let request = ClientRequest::<()>::post(
            format!(
                "/v1/collections/{}/indexes/{}/reindex",
                self.collection_id, self.index_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            (),
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }

    /// Insert documents
    pub async fn insert_documents<T>(&self, documents: Vec<T>) -> Result<()>
    where
        T: Serialize,
    {
        let body = serde_json::json!({
            "documents": documents
        });

        let request = ClientRequest::post(
            format!(
                "/v1/collections/{}/indexes/{}/documents/insert",
                self.collection_id, self.index_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }

    /// Delete documents
    pub async fn delete_documents(&self, document_ids: Vec<String>) -> Result<()> {
        let body = serde_json::json!({
            "document_ids": document_ids
        });

        let request = ClientRequest::post(
            format!(
                "/v1/collections/{}/indexes/{}/documents/delete",
                self.collection_id, self.index_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }

    /// Upsert documents
    pub async fn upsert_documents<T>(&self, documents: Vec<T>) -> Result<()>
    where
        T: Serialize,
    {
        let body = serde_json::json!({
            "documents": documents
        });

        let request = ClientRequest::post(
            format!(
                "/v1/collections/{}/indexes/{}/documents/upsert",
                self.collection_id, self.index_id
            ),
            Target::Writer,
            ApiKeyPosition::Header,
            body,
        );

        let _: serde_json::Value = self.client.request(request).await?;
        Ok(())
    }
}

/// Main collection manager
#[derive(Debug, Clone)]
pub struct CollectionManager {
    client: OramaClient,
    collection_id: String,
    pub ai: AiNamespace,
    pub collections: CollectionsNamespace,
    pub index: IndexNamespace,
    pub hooks: HooksNamespace,
    pub system_prompts: SystemPromptsNamespace,
    pub tools: ToolsNamespace,
}

impl CollectionManager {
    /// Create a new CollectionManager
    pub async fn new(config: CollectionManagerConfig) -> Result<Self> {
        let auth_config = if config.api_key.starts_with("p_") {
            // Private API Key (JWT flow)
            AuthConfig::Jwt(
                JwtAuth::new(
                    config.auth_jwt_url.as_deref().unwrap_or(DEFAULT_JWT_URL),
                    &config.collection_id,
                    &config.api_key,
                )
                .with_reader_url(
                    config
                        .cluster
                        .as_ref()
                        .and_then(|c| c.read_url.as_deref())
                        .unwrap_or(DEFAULT_READER_URL),
                )
                .with_writer_url(
                    config
                        .cluster
                        .as_ref()
                        .and_then(|c| c.writer_url.as_deref())
                        .unwrap_or(""),
                ),
            )
        } else {
            // Regular API Key
            AuthConfig::ApiKey(
                ApiKeyAuth::new(&config.api_key)
                    .with_reader_url(
                        config
                            .cluster
                            .as_ref()
                            .and_then(|c| c.read_url.as_deref())
                            .unwrap_or(DEFAULT_READER_URL),
                    )
                    .with_writer_url(
                        config
                            .cluster
                            .as_ref()
                            .and_then(|c| c.writer_url.as_deref())
                            .unwrap_or(""),
                    ),
            )
        };

        let client = Client::new();
        let auth = Auth::new(auth_config, Arc::new(client));
        let orama_client = OramaClient::new(auth)?;

        let collection_id = config.collection_id.clone();

        Ok(Self {
            ai: AiNamespace::new(orama_client.clone(), collection_id.clone()),
            collections: CollectionsNamespace::new(orama_client.clone(), collection_id.clone()),
            index: IndexNamespace::new(orama_client.clone(), collection_id.clone()),
            hooks: HooksNamespace::new(orama_client.clone(), collection_id.clone()),
            system_prompts: SystemPromptsNamespace::new(
                orama_client.clone(),
                collection_id.clone(),
            ),
            tools: ToolsNamespace::new(orama_client.clone(), collection_id.clone()),
            client: orama_client,
            collection_id,
        })
    }

    /// Perform a search
    pub async fn search<T>(&self, query: &SearchParams) -> Result<SearchResult<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let start_time = current_time_millis();

        let request = ClientRequest::post(
            format!("/v1/collections/{}/search", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
            query,
        );

        let mut result: SearchResult<T> = self.client.request(request).await?;

        let elapsed_time = current_time_millis() - start_time;
        result.elapsed = Some(Elapsed {
            raw: elapsed_time,
            formatted: format_duration(elapsed_time),
        });

        Ok(result)
    }
}

// Builder implementations
impl CollectionManagerConfig {
    /// Create a new CollectionManagerConfig
    pub fn new<S: Into<String>>(collection_id: S, api_key: S) -> Self {
        Self {
            collection_id: collection_id.into(),
            api_key: api_key.into(),
            cluster: None,
            auth_jwt_url: None,
        }
    }

    /// Set cluster configuration
    pub fn with_cluster(mut self, cluster: ClusterConfig) -> Self {
        self.cluster = Some(cluster);
        self
    }

    /// Set auth JWT URL
    pub fn with_auth_jwt_url<S: Into<String>>(mut self, url: S) -> Self {
        self.auth_jwt_url = Some(url.into());
        self
    }
}

impl ClusterConfig {
    /// Create a new ClusterConfig
    pub fn new() -> Self {
        Self {
            writer_url: None,
            read_url: None,
        }
    }

    /// Set writer URL
    pub fn with_writer_url<S: Into<String>>(mut self, url: S) -> Self {
        self.writer_url = Some(url.into());
        self
    }

    /// Set read URL
    pub fn with_read_url<S: Into<String>>(mut self, url: S) -> Self {
        self.read_url = Some(url.into());
        self
    }
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_manager_config_builder() {
        let config = CollectionManagerConfig::new("test-collection", "test-key")
            .with_cluster(
                ClusterConfig::new()
                    .with_read_url("https://reader.example.com")
                    .with_writer_url("https://writer.example.com"),
            )
            .with_auth_jwt_url("https://auth.example.com");

        assert_eq!(config.collection_id, "test-collection");
        assert_eq!(config.api_key, "test-key");
        assert!(config.cluster.is_some());
        assert_eq!(
            config.auth_jwt_url,
            Some("https://auth.example.com".to_string())
        );
    }

    #[test]
    fn test_nlp_search_params() {
        let params = NlpSearchParams {
            query: "test query".to_string(),
            llm_config: Some(LlmConfig {
                provider: LlmProvider::OpenAI,
                model: "gpt-4".to_string(),
            }),
            user_id: Some("user-123".to_string()),
        };

        assert_eq!(params.query, "test query");
        assert!(params.llm_config.is_some());
        assert_eq!(params.user_id, Some("user-123".to_string()));
    }
}
