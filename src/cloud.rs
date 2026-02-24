//! Orama Cloud client functionality.

use serde::Serialize;

use crate::collection::{ClusterConfig, CollectionManager, CollectionManagerConfig};
use crate::error::Result;
use crate::types::*;

/// Configuration for OramaCloud
#[derive(Debug, Clone)]
pub struct ProjectManagerConfig {
    pub project_id: String,
    pub api_key: String,
    pub cluster: Option<ClusterConfig>,
    pub auth_jwt_url: Option<String>,
}

/// Cloud search parameters (uses datasources instead of indexes)
#[derive(Debug, Clone, Serialize, Default)]
pub struct CloudSearchParams {
    pub term: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<SearchMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<String>>,
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub where_clause: Option<AnyObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<AnyObject>,
    pub datasources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<u32>,
    #[serde(rename = "userID", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Data source operations namespace
#[derive(Debug, Clone)]
pub struct DataSourceNamespace {
    index: crate::collection::Index,
}

impl DataSourceNamespace {
    pub(crate) fn new(index: crate::collection::Index) -> Self {
        Self { index }
    }

    /// Reindex the data source
    pub async fn reindex(&self) -> Result<()> {
        self.index.reindex().await
    }

    /// Insert documents into the data source
    pub async fn insert_documents<T>(&self, documents: Vec<T>) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.index.insert_documents(documents).await
    }

    /// Delete documents from the data source
    pub async fn delete_documents(&self, document_ids: Vec<String>) -> Result<()> {
        self.index.delete_documents(document_ids).await
    }

    /// Upsert documents in the data source
    pub async fn upsert_documents<T>(&self, documents: Vec<T>) -> Result<()>
    where
        T: serde::Serialize,
    {
        self.index.upsert_documents(documents).await
    }
}

/// Main Orama Cloud client
#[derive(Debug, Clone)]
pub struct OramaCloud {
    client: CollectionManager,
}

impl OramaCloud {
    /// Create a new OramaCloud client
    pub async fn new(config: ProjectManagerConfig) -> Result<Self> {
        // Use CollectionManager internally with project_id as collection_id
        let mut collection_config = CollectionManagerConfig::new(config.project_id, config.api_key);
        if let Some(cluster) = config.cluster {
            collection_config = collection_config.with_cluster(cluster);
        }
        if let Some(auth_jwt_url) = config.auth_jwt_url {
            collection_config = collection_config.with_auth_jwt_url(auth_jwt_url);
        }

        let client = CollectionManager::new(collection_config).await?;

        Ok(Self { client })
    }

    /// Perform a search with datasources parameter
    pub async fn search<T>(&self, params: &CloudSearchParams) -> Result<SearchResult<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        // Convert CloudSearchParams to SearchParams
        let search_params = SearchParams {
            term: params.term.clone(),
            mode: params.mode.clone(),
            limit: params.limit,
            offset: params.offset,
            properties: params.properties.clone(),
            where_clause: params.where_clause.clone(),
            facets: params.facets.clone(),
            indexes: Some(params.datasources.clone()), // Map datasources to indexes
            datasource_ids: None,
            exact: params.exact,
            threshold: params.threshold,
            tolerance: params.tolerance,
            user_id: params.user_id.clone(),
        };

        self.client.search(&search_params).await
    }

    /// Get a data source namespace for operations
    pub fn data_source(&self, id: String) -> DataSourceNamespace {
        let index = self.client.index.set(id);
        DataSourceNamespace::new(index)
    }

    /// Access to AI operations
    pub fn ai(&self) -> &crate::collection::AiNamespace {
        &self.client.ai
    }

    /// Access to collections operations
    pub fn collections(&self) -> &crate::collection::CollectionsNamespace {
        &self.client.collections
    }

    /// Access to index operations
    pub fn index(&self) -> &crate::collection::IndexNamespace {
        &self.client.index
    }

    /// Access to hooks operations
    pub fn hooks(&self) -> &crate::collection::HooksNamespace {
        &self.client.hooks
    }

    /// Access to system prompts operations
    pub fn system_prompts(&self) -> &crate::collection::SystemPromptsNamespace {
        &self.client.system_prompts
    }

    /// Access to tools operations
    pub fn tools(&self) -> &crate::collection::ToolsNamespace {
        &self.client.tools
    }
}

// Builder implementations
impl ProjectManagerConfig {
    /// Create a new ProjectManagerConfig
    pub fn new<S: Into<String>>(project_id: S, api_key: S) -> Self {
        Self {
            project_id: project_id.into(),
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

impl CloudSearchParams {
    /// Create a new CloudSearchParams
    pub fn new<S: Into<String>>(term: S, datasources: Vec<String>) -> Self {
        Self {
            term: term.into(),
            datasources,
            ..Default::default()
        }
    }

    /// Set search mode
    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set properties to search in
    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set where clause
    pub fn with_where(mut self, where_clause: AnyObject) -> Self {
        self.where_clause = Some(where_clause);
        self
    }

    /// Set facets
    pub fn with_facets(mut self, facets: AnyObject) -> Self {
        self.facets = Some(facets);
        self
    }

    /// Set exact matching
    pub fn with_exact(mut self, exact: bool) -> Self {
        self.exact = Some(exact);
        self
    }

    /// Set similarity threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Set tolerance
    pub fn with_tolerance(mut self, tolerance: u32) -> Self {
        self.tolerance = Some(tolerance);
        self
    }

    /// Set user ID
    pub fn with_user_id<S: Into<String>>(mut self, user_id: S) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}
