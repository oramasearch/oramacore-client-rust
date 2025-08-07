//! AI session streaming functionality.

use crate::client::OramaClient;
use crate::error::{OramaError, Result};
use crate::types::*;
use crate::utils::generate_uuid;
use futures::stream::{Stream, StreamExt};
use serde::Serialize;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for creating an AI session
#[derive(Debug, Clone)]
pub struct CreateAiSessionConfig {
    pub llm_config: Option<LlmConfig>,
    pub initial_messages: Option<Vec<Message>>,
}

/// Answer configuration for AI requests
#[derive(Debug, Clone, Serialize)]
pub struct AnswerConfig {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visitor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<RelatedQuestionsConfig>,
    #[serde(rename = "datasourceIDs", skip_serializing_if = "Option::is_none")]
    pub datasource_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_documents: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ragat_notation: Option<String>,
}

/// Interaction state for conversations
#[derive(Debug, Clone)]
pub struct Interaction {
    pub id: String,
    pub query: String,
    pub response: String,
    pub sources: Option<AnyObject>,
    pub loading: bool,
    pub error: bool,
    pub error_message: Option<String>,
    pub aborted: bool,
    pub related: Option<String>,
    pub current_step: Option<String>,
    pub current_step_verbose: Option<String>,
    pub selected_llm: Option<LlmConfig>,
    pub optimized_query: Option<SearchParams>,
    pub advanced_autoquery: Option<serde_json::Value>,
}

impl Interaction {
    /// Create a new interaction
    pub fn new(id: String, query: String) -> Self {
        Self {
            id,
            query,
            response: String::new(),
            sources: None,
            loading: true,
            error: false,
            error_message: None,
            aborted: false,
            related: None,
            current_step: Some("starting".to_string()),
            current_step_verbose: None,
            selected_llm: None,
            optimized_query: None,
            advanced_autoquery: None,
        }
    }
}

/// AI session stream manager
#[derive(Debug)]
pub struct OramaCoreStream {
    collection_id: String,
    client: OramaClient,
    session_id: String,
    llm_config: Option<LlmConfig>,
    messages: Arc<RwLock<Vec<Message>>>,
    state: Arc<RwLock<Vec<Interaction>>>,
    last_interaction_params: Arc<RwLock<Option<AnswerConfig>>>,
}

impl OramaCoreStream {
    /// Create a new AI session stream
    pub async fn new(collection_id: String, client: OramaClient) -> Result<Self> {
        Ok(Self {
            collection_id,
            client,
            session_id: generate_uuid(),
            llm_config: None,
            messages: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(Vec::new())),
            last_interaction_params: Arc::new(RwLock::new(None)),
        })
    }

    /// Create a new AI session stream with configuration
    pub async fn with_config(
        collection_id: String,
        client: OramaClient,
        config: CreateAiSessionConfig,
    ) -> Result<Self> {
        let messages = config.initial_messages.unwrap_or_default();

        Ok(Self {
            collection_id,
            client,
            session_id: generate_uuid(),
            llm_config: config.llm_config,
            messages: Arc::new(RwLock::new(messages)),
            state: Arc::new(RwLock::new(Vec::new())),
            last_interaction_params: Arc::new(RwLock::new(None)),
        })
    }

    /// Get a complete answer (non-streaming)
    pub async fn answer(&self, data: AnswerConfig) -> Result<String> {
        // For simplicity, we'll implement a basic version
        // In a full implementation, this would handle streaming and return the complete result
        let enriched_config = self.enrich_config(data).await;

        // Store the interaction parameters
        {
            let mut last_params = self.last_interaction_params.write().await;
            *last_params = Some(enriched_config.clone());
        }

        // Add user message
        {
            let mut messages = self.messages.write().await;
            messages.push(Message {
                role: Role::User,
                content: enriched_config.query.clone(),
            });
            messages.push(Message {
                role: Role::Assistant,
                content: String::new(),
            });
        }

        // Create interaction
        let interaction_id = enriched_config
            .interaction_id
            .clone()
            .unwrap_or_else(generate_uuid);

        let interaction = Interaction::new(interaction_id, enriched_config.query.clone());

        {
            let mut state = self.state.write().await;
            state.push(interaction);
        }

        // For this simplified implementation, return a placeholder response
        // In a full implementation, this would make the API call and process the stream
        let response = "This is a simplified response. Full streaming implementation would be needed for production.".to_string();

        // Update the interaction and message
        {
            let mut state = self.state.write().await;
            if let Some(last_interaction) = state.last_mut() {
                last_interaction.response = response.clone();
                last_interaction.loading = false;
            }
        }

        {
            let mut messages = self.messages.write().await;
            if let Some(last_message) = messages.last_mut() {
                last_message.content = response.clone();
            }
        }

        Ok(response)
    }

    /// Get streaming answer (simplified implementation)
    pub async fn answer_stream(
        &self,
        data: AnswerConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        // For now, we'll create a simple stream that returns the complete answer
        // A full implementation would use server-sent events
        let answer = self.answer(data).await?;

        // Split the answer into chunks for streaming simulation
        let chunks: Vec<String> = answer
            .chars()
            .collect::<Vec<_>>()
            .chunks(10)
            .map(|chunk| chunk.iter().collect())
            .collect();

        let stream = futures::stream::iter(chunks.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    /// Regenerate the last response
    pub async fn regenerate_last(&self, stream: bool) -> Result<String> {
        let state_len = {
            let state = self.state.read().await;
            state.len()
        };

        let messages_len = {
            let messages = self.messages.read().await;
            messages.len()
        };

        if state_len == 0 || messages_len == 0 {
            return Err(OramaError::generic("No messages to regenerate"));
        }

        // Check if last message is from assistant
        {
            let messages = self.messages.read().await;
            if let Some(last_message) = messages.last() {
                if !matches!(last_message.role, Role::Assistant) {
                    return Err(OramaError::generic(
                        "Last message is not an assistant message",
                    ));
                }
            }
        }

        // Get the last interaction parameters
        let last_params = {
            let params = self.last_interaction_params.read().await;
            params.clone()
        };

        let last_params = last_params
            .ok_or_else(|| OramaError::generic("No last interaction parameters available"))?;

        // Remove last assistant message and state
        {
            let mut messages = self.messages.write().await;
            messages.pop();
        }

        {
            let mut state = self.state.write().await;
            state.pop();
        }

        // Regenerate
        if stream {
            let _stream_result = self.answer_stream(last_params.clone()).await?;
            // For streaming regeneration, we'll just return the answer for now
            // In a full implementation, you would collect the stream
            self.answer(last_params).await
        } else {
            self.answer(last_params).await
        }
    }

    /// Clear the session
    pub async fn clear_session(&self) {
        {
            let mut messages = self.messages.write().await;
            messages.clear();
        }

        {
            let mut state = self.state.write().await;
            state.clear();
        }
    }

    /// Get current messages
    pub async fn get_messages(&self) -> Vec<Message> {
        let messages = self.messages.read().await;
        messages.clone()
    }

    /// Get current state
    pub async fn get_state(&self) -> Vec<Interaction> {
        let state = self.state.read().await;
        state.clone()
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Enrich config with default values
    async fn enrich_config(&self, mut config: AnswerConfig) -> AnswerConfig {
        if config.visitor_id.is_none() {
            config.visitor_id = Some(DEFAULT_SERVER_USER_ID.to_string());
        }

        if config.interaction_id.is_none() {
            config.interaction_id = Some(generate_uuid());
        }

        if config.session_id.is_none() {
            config.session_id = Some(self.session_id.clone());
        }

        config
    }
}

// Builder implementations
impl AnswerConfig {
    /// Create a new AnswerConfig
    pub fn new<S: Into<String>>(query: S) -> Self {
        Self {
            query: query.into(),
            interaction_id: None,
            visitor_id: None,
            session_id: None,
            messages: None,
            related: None,
            datasource_ids: None,
            min_similarity: None,
            max_documents: None,
            ragat_notation: None,
        }
    }

    /// Set interaction ID
    pub fn with_interaction_id<S: Into<String>>(mut self, id: S) -> Self {
        self.interaction_id = Some(id.into());
        self
    }

    /// Set visitor ID
    pub fn with_visitor_id<S: Into<String>>(mut self, id: S) -> Self {
        self.visitor_id = Some(id.into());
        self
    }

    /// Set session ID
    pub fn with_session_id<S: Into<String>>(mut self, id: S) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Set messages
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = Some(messages);
        self
    }

    /// Set related questions config
    pub fn with_related(mut self, related: RelatedQuestionsConfig) -> Self {
        self.related = Some(related);
        self
    }

    /// Set datasource IDs
    pub fn with_datasource_ids(mut self, ids: Vec<String>) -> Self {
        self.datasource_ids = Some(ids);
        self
    }

    /// Set minimum similarity
    pub fn with_min_similarity(mut self, similarity: f64) -> Self {
        self.min_similarity = Some(similarity);
        self
    }

    /// Set maximum documents
    pub fn with_max_documents(mut self, max_docs: u32) -> Self {
        self.max_documents = Some(max_docs);
        self
    }

    /// Set RAGAT notation
    pub fn with_ragat_notation<S: Into<String>>(mut self, notation: S) -> Self {
        self.ragat_notation = Some(notation.into());
        self
    }
}

impl CreateAiSessionConfig {
    /// Create a new CreateAiSessionConfig
    pub fn new() -> Self {
        Self {
            llm_config: None,
            initial_messages: None,
        }
    }

    /// Set LLM configuration
    pub fn with_llm_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// Set initial messages
    pub fn with_initial_messages(mut self, messages: Vec<Message>) -> Self {
        self.initial_messages = Some(messages);
        self
    }
}

impl Default for CreateAiSessionConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_answer_config_builder() {
        let config = AnswerConfig::new("test query")
            .with_visitor_id("user-123")
            .with_min_similarity(0.8)
            .with_max_documents(10);

        assert_eq!(config.query, "test query");
        assert_eq!(config.visitor_id, Some("user-123".to_string()));
        assert_eq!(config.min_similarity, Some(0.8));
        assert_eq!(config.max_documents, Some(10));
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
        let interaction = Interaction::new("id-123".to_string(), "test query".to_string());

        assert_eq!(interaction.id, "id-123");
        assert_eq!(interaction.query, "test query");
        assert!(interaction.loading);
        assert!(!interaction.error);
        assert!(!interaction.aborted);
        assert_eq!(interaction.current_step, Some("starting".to_string()));
    }
}
