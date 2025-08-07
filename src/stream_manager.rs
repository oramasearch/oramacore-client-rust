//! AI session streaming functionality.

use crate::auth::Target;
use crate::client::{ApiKeyPosition, ClientRequest, OramaClient};
use crate::error::{OramaError, Result};
use crate::types::*;
use crate::utils::{generate_uuid, parse_ai_response};
use futures::stream::{Stream, StreamExt};
use reqwest_eventsource::{Event, EventSource};
use serde::Serialize;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Streaming chunk types
#[derive(Debug, Clone, PartialEq)]
pub enum StreamChunk {
    /// Connection opened successfully
    ConnectionOpened,
    /// Content chunk from the AI response
    Content(String),
    /// Status update from the processing pipeline
    StatusUpdate(String),
    /// Raw data that couldn't be parsed
    RawData(String),
    /// Stream completed successfully
    Done,
    /// Connection retry attempt
    Retry { attempt: u32, delay_ms: u64 },
}

/// Configuration for streaming resilience
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial retry delay in milliseconds
    pub initial_retry_delay: u64,
    /// Maximum retry delay in milliseconds (for exponential backoff)
    pub max_retry_delay: u64,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Stream idle timeout in seconds
    pub stream_timeout: u64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay: 1000, // 1 second
            max_retry_delay: 30000,    // 30 seconds
            connection_timeout: 30,    // 30 seconds
            stream_timeout: 300,       // 5 minutes
        }
    }
}

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
    #[serde(rename = "LLMConfig", skip_serializing_if = "Option::is_none")]
    pub llm_config: Option<LlmConfig>,
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
    stream_config: StreamConfig,
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
            stream_config: StreamConfig::default(),
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
            stream_config: StreamConfig::default(),
        })
    }

    /// Create a new AI session stream with streaming configuration
    pub async fn with_stream_config(
        collection_id: String,
        client: OramaClient,
        config: CreateAiSessionConfig,
        stream_config: StreamConfig,
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
            stream_config,
        })
    }

    /// Get a complete answer (non-streaming)
    pub async fn answer(&self, data: AnswerConfig) -> Result<String> {
        info!("Starting AI answer request");
        let enriched_config = self.enrich_config(data).await;
        debug!("Enriched config: {:?}", enriched_config);

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

        let interaction = Interaction::new(interaction_id.clone(), enriched_config.query.clone());

        {
            let mut state = self.state.write().await;
            state.push(interaction);
        }

        // Make the actual API call
        let request = ClientRequest::post(
            format!("/v1/collections/{}/ai/answer", self.collection_id),
            Target::Reader,
            ApiKeyPosition::QueryParams,
            enriched_config,
        );

        let response: serde_json::Value = self.client.request(request).await.map_err(|e| {
            error!("API request failed: {}", e);
            e
        })?;

        // Extract the answer from the response
        let answer = response["answer"].as_str().unwrap_or_default().to_string();

        // Update the interaction and message
        {
            let mut state = self.state.write().await;
            if let Some(last_interaction) = state.last_mut() {
                last_interaction.response = answer.clone();
                last_interaction.loading = false;
                last_interaction.current_step = Some("completed".to_string());

                // Update with additional response data if available
                if let Some(sources) = response.get("sources") {
                    last_interaction.sources = Some(sources.clone());
                }
                if let Some(_related) = response.get("related") {
                    last_interaction.related = response["related"].as_str().map(String::from);
                }
            }
        }

        {
            let mut messages = self.messages.write().await;
            if let Some(last_message) = messages.last_mut() {
                last_message.content = answer.clone();
            }
        }

        info!("AI answer completed successfully, length: {}", answer.len());
        Ok(answer)
    }

    /// Create resilient SSE stream with retry logic
    async fn create_resilient_stream(
        &self,
        client: OramaClient,
        stream_url: String,
        auth_ref: crate::auth::AuthRef,
        enriched_config: AnswerConfig,
        messages: Arc<RwLock<Vec<Message>>>,
        state: Arc<RwLock<Vec<Interaction>>>,
    ) -> Result<impl Stream<Item = Result<StreamChunk>> + Send> {
        let stream_timeout = Duration::from_secs(self.stream_config.stream_timeout);
        let start_time = std::time::Instant::now();

        // Create request builder for EventSource
        let request_builder = client
            .inner()
            .post(&stream_url)
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("Authorization", format!("Bearer {}", auth_ref.bearer))
            .timeout(Duration::from_secs(self.stream_config.connection_timeout))
            .json(&enriched_config);

        // Create EventSource
        let event_source = EventSource::new(request_builder).map_err(|e| {
            error!("Failed to create EventSource: {}", e);
            OramaError::generic(&format!("EventSource creation failed: {}", e))
        })?;

        info!("Successfully created EventSource for streaming");

        // Convert EventSource to stream with comprehensive error handling
        let event_stream = event_source.map(move |event_result| {
            // Check for timeout
            if start_time.elapsed() >= stream_timeout {
                let timeout_secs = stream_timeout.as_secs();
                error!("Stream timeout after {} seconds", timeout_secs);
                let state_clone = state.clone();
                let timeout_msg = format!("Stream timeout after {} seconds", timeout_secs);
                tokio::spawn(async move {
                    Self::mark_interaction_error(state_clone, timeout_msg).await;
                });
                return Err(OramaError::generic(&format!(
                    "Stream timeout after {} seconds",
                    timeout_secs
                )));
            }

            match event_result {
                Ok(event) => match event {
                    Event::Open => {
                        debug!("Stream connection opened");
                        Ok(StreamChunk::ConnectionOpened)
                    }
                    Event::Message(message) => {
                        debug!("Received streaming message: {}", message.data);

                        match message.data.as_str() {
                            "[DONE]" => {
                                info!("Streaming completed successfully");
                                let state_clone = state.clone();
                                tokio::spawn(async move {
                                    let mut state = state_clone.write().await;
                                    if let Some(interaction) = state.last_mut() {
                                        interaction.loading = false;
                                        interaction.current_step = Some("completed".to_string());
                                    }
                                });
                                Ok(StreamChunk::Done)
                            }
                            data => {
                                Self::process_stream_data(data, messages.clone(), state.clone())
                            }
                        }
                    }
                },
                Err(event_error) => {
                    error!("Stream event error: {}", event_error);
                    let state_clone = state.clone();
                    let error_msg = event_error.to_string();
                    tokio::spawn(async move {
                        Self::mark_interaction_error(state_clone, error_msg).await;
                    });
                    Err(OramaError::generic(&format!(
                        "Stream event error: {}",
                        event_error
                    )))
                }
            }
        });

        Ok(event_stream)
    }

    /// Get streaming answer with server-sent events
    pub async fn answer_stream(
        &self,
        data: AnswerConfig,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        info!("Starting streaming AI answer request");
        let enriched_config = self.enrich_config(data).await;
        debug!("Enriched streaming config: {:?}", enriched_config);

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

        let interaction = Interaction::new(interaction_id.clone(), enriched_config.query.clone());

        {
            let mut state = self.state.write().await;
            state.push(interaction);
        }

        let client = self.client.clone();
        let collection_id = self.collection_id.clone();
        let messages = self.messages.clone();
        let state = self.state.clone();

        // Get auth reference for the streaming request
        let auth_ref = client.get_auth_ref(Target::Reader).await.map_err(|e| {
            error!("Failed to get auth reference: {}", e);
            e
        })?;

        let base_url = &auth_ref.base_url;
        let stream_url = format!(
            "{}/v1/collections/{}/ai/answer/stream",
            base_url, collection_id
        );

        debug!("Creating streaming request to: {}", stream_url);

        // Create SSE stream using reqwest-eventsource with retry
        let stream = self
            .create_resilient_stream(
                client.clone(),
                stream_url,
                auth_ref,
                enriched_config,
                messages.clone(),
                state.clone(),
            )
            .await?;

        Ok(Box::pin(stream))
    }

    /// Regenerate the last response
    pub async fn regenerate_last(&self, stream: bool) -> Result<String> {
        info!("Starting regenerate_last, stream: {}", stream);

        let state_len = {
            let state = self.state.read().await;
            state.len()
        };

        let messages_len = {
            let messages = self.messages.read().await;
            messages.len()
        };

        if state_len == 0 || messages_len == 0 {
            warn!("No messages to regenerate");
            return Err(OramaError::generic("No messages to regenerate"));
        }

        // Check if last message is from assistant
        {
            let messages = self.messages.read().await;
            if let Some(last_message) = messages.last() {
                if !matches!(last_message.role, Role::Assistant) {
                    warn!("Last message is not from assistant");
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

        let last_params = last_params.ok_or_else(|| {
            warn!("No last interaction parameters available");
            OramaError::generic("No last interaction parameters available")
        })?;

        // Remove last assistant message and state
        {
            let mut messages = self.messages.write().await;
            messages.pop();
        }

        {
            let mut state = self.state.write().await;
            state.pop();
        }

        // Regenerate based on stream preference
        if stream {
            info!("Regenerating with streaming");
            let mut stream_result = self.answer_stream(last_params).await?;
            let mut complete_response = String::new();

            // Collect the stream
            while let Some(chunk_result) = stream_result.next().await {
                match chunk_result? {
                    StreamChunk::Content(content) => {
                        complete_response.push_str(&content);
                    }
                    StreamChunk::Done => {
                        break;
                    }
                    StreamChunk::StatusUpdate(status) => {
                        debug!("Status update during regeneration: {}", status);
                    }
                    _ => {
                        // Ignore other chunk types for regeneration
                    }
                }
            }

            Ok(complete_response)
        } else {
            info!("Regenerating without streaming");
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

    /// Get current stream configuration
    pub fn get_stream_config(&self) -> &StreamConfig {
        &self.stream_config
    }

    /// Update stream configuration
    pub fn set_stream_config(&mut self, config: StreamConfig) {
        self.stream_config = config;
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

        // Use session's LLM config if none is provided in the request
        if config.llm_config.is_none() {
            config.llm_config = self.llm_config.clone();
        }

        config
    }

    /// Process streaming data chunk with robust JSON parsing
    fn process_stream_data(
        data: &str,
        messages: Arc<RwLock<Vec<Message>>>,
        state: Arc<RwLock<Vec<Interaction>>>,
    ) -> Result<StreamChunk> {
        // Use robust AI response parsing with automatic JSON fixing
        match parse_ai_response::<serde_json::Value>(data) {
            Ok(parsed) => {
                if let Some(content) = parsed.get("content").and_then(|c| c.as_str()) {
                    // Content chunk - update message and interaction
                    let content = content.to_string();
                    let content_for_update = content.clone();
                    let parsed_clone = parsed.clone();

                    tokio::spawn(async move {
                        // Update assistant message
                        {
                            let mut messages = messages.write().await;
                            if let Some(last_message) = messages.last_mut() {
                                if matches!(last_message.role, Role::Assistant) {
                                    last_message.content.push_str(&content_for_update);
                                }
                            }
                        }

                        // Update interaction state
                        {
                            let mut state = state.write().await;
                            if let Some(last_interaction) = state.last_mut() {
                                last_interaction.response.push_str(&content_for_update);

                                // Update step if provided
                                if let Some(step) =
                                    parsed_clone.get("step").and_then(|s| s.as_str())
                                {
                                    last_interaction.current_step = Some(step.to_string());
                                }

                                // Update verbose step if provided
                                if let Some(verbose) =
                                    parsed_clone.get("verbose_step").and_then(|s| s.as_str())
                                {
                                    last_interaction.current_step_verbose =
                                        Some(verbose.to_string());
                                }
                            }
                        }
                    });

                    Ok(StreamChunk::Content(content))
                } else if let Some(step) = parsed.get("step").and_then(|s| s.as_str()) {
                    // Status update
                    let step = step.to_string();
                    let step_for_update = step.clone();

                    tokio::spawn(async move {
                        let mut state = state.write().await;
                        if let Some(last_interaction) = state.last_mut() {
                            last_interaction.current_step = Some(step_for_update);
                        }
                    });

                    Ok(StreamChunk::StatusUpdate(step))
                } else if let Some(error_msg) = parsed.get("error").and_then(|e| e.as_str()) {
                    // Error in stream
                    warn!("Stream error received: {}", error_msg);

                    let state_clone = state.clone();
                    let error_message = error_msg.to_string();
                    tokio::spawn(async move {
                        Self::mark_interaction_error(state_clone, error_message).await;
                    });
                    Err(OramaError::generic(error_msg))
                } else {
                    // Unknown structured data
                    debug!("Unknown structured stream data: {}", data);
                    Ok(StreamChunk::RawData(data.to_string()))
                }
            }
            Err(parse_err) => {
                // Parsing failed even with JSON fixing - treat as raw data
                debug!(
                    "Failed to parse AI response as JSON ({}): {}",
                    parse_err, data
                );
                Ok(StreamChunk::RawData(data.to_string()))
            }
        }
    }

    /// Mark interaction as errored (async version)
    async fn mark_interaction_error(state: Arc<RwLock<Vec<Interaction>>>, error_message: String) {
        let mut state = state.write().await;
        if let Some(interaction) = state.last_mut() {
            interaction.error = true;
            interaction.error_message = Some(error_message);
            interaction.loading = false;
        }
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
            llm_config: None,
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

    /// Set LLM configuration
    pub fn with_llm_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = Some(config);
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
