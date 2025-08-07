//! Core types for the Orama client.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type alias for generic object data
pub type AnyObject = serde_json::Value;

/// Supported languages for search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Arabic,
    Bulgarian,
    Chinese,
    Danish,
    Dutch,
    German,
    Greek,
    English,
    Estonian,
    Spanish,
    Finnish,
    French,
    Irish,
    Hindi,
    Hungarian,
    Armenian,
    Indonesian,
    Italian,
    Japanese,
    Korean,
    Lithuanian,
    Nepali,
    Norwegian,
    Portuguese,
    Romanian,
    Russian,
    Sanskrit,
    Slovenian,
    Serbian,
    Swedish,
    Tamil,
    Turkish,
    Ukrainian,
}

/// Supported embeddings models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingsModel {
    #[serde(rename = "E5MultilangualSmall")]
    E5MultilangualSmall,
    #[serde(rename = "E5MultilangualBase")]
    E5MultilangualBase,
    #[serde(rename = "E5MultilangualLarge")]
    E5MultilangualLarge,
    #[serde(rename = "BGESmall")]
    BgeSmall,
    #[serde(rename = "BGEBase")]
    BgeBase,
    #[serde(rename = "BGELarge")]
    BgeLarge,
}

/// Embeddings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    pub model: Option<EmbeddingsModel>,
    pub document_fields: Option<Vec<String>>,
}

/// Hook types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Hook {
    #[serde(rename = "BeforeAnswer")]
    BeforeAnswer,
    #[serde(rename = "BeforeRetrieval")]
    BeforeRetrieval,
}

/// Search modes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    Fulltext,
    Vector,
    Hybrid,
    Auto,
}

/// Search parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchParams {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexes: Option<Vec<String>>,
    #[serde(rename = "datasourceIDs", skip_serializing_if = "Option::is_none")]
    pub datasource_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<u32>,
    #[serde(rename = "userID", skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Cloud search parameters (omits indexes field)
pub type CloudSearchParams = SearchParams;

/// Search hit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hit<T = AnyObject> {
    pub id: String,
    pub score: f64,
    pub document: T,
    #[serde(rename = "datasource_id", skip_serializing_if = "Option::is_none")]
    pub datasource_id: Option<String>,
}

/// Elapsed time information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Elapsed {
    pub raw: u64,
    pub formatted: String,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<T = AnyObject> {
    pub count: u32,
    pub hits: Vec<Hit<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<AnyObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed: Option<Elapsed>,
}

/// Trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub name: String,
    pub description: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_id: Option<String>,
}

/// Segment definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
}

/// Request body for inserting a segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSegmentBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
}

/// Request body for inserting a trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTriggerBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub response: String,
    pub segment_id: String,
}

/// Response for segment insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSegmentResponse {
    pub success: bool,
    pub id: String,
    pub segment: Segment,
}

/// Response for trigger insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertTriggerResponse {
    pub success: bool,
    pub id: String,
    pub trigger: Trigger,
}

/// Response for trigger update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTriggerResponse {
    pub success: bool,
    pub trigger: Trigger,
}

/// System prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPrompt {
    pub id: String,
    pub name: String,
    pub prompt: String,
    pub usage_mode: SystemPromptUsageMode,
}

/// System prompt usage modes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemPromptUsageMode {
    Automatic,
    Manual,
}

/// Request body for inserting a system prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertSystemPromptBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub prompt: String,
    pub usage_mode: SystemPromptUsageMode,
}

/// System prompt validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptValidationResponse {
    pub security: SecurityValidation,
    pub technical: TechnicalValidation,
    pub overall_assessment: OverallAssessment,
}

/// Security validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityValidation {
    pub valid: bool,
    pub reason: String,
    pub violations: Vec<String>,
}

/// Technical validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalValidation {
    pub valid: bool,
    pub reason: String,
    pub instruction_count: u32,
}

/// Overall assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallAssessment {
    pub valid: bool,
    pub summary: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

/// Request body for inserting a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertToolBody {
    pub id: String,
    pub description: String,
    pub parameters: serde_json::Value, // Can be string, object, or schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

/// Request body for updating a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateToolBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Function call definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

/// Parsed function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallParsed {
    pub name: String,
    pub arguments: AnyObject,
}

/// Execute tools response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteToolsResponse {
    pub results: Option<Vec<FunctionCall>>,
}

/// Function result for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteToolsFunctionResult<T = AnyObject> {
    #[serde(rename = "functionResult")]
    pub function_result: FunctionResultData<T>,
}

/// Function result data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResultData<T = AnyObject> {
    pub tool_id: String,
    pub result: T,
}

/// Function parameters result for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteToolsParametersResult<T = AnyObject> {
    #[serde(rename = "functionParameters")]
    pub function_parameters: FunctionParametersData<T>,
}

/// Function parameters data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParametersData<T = AnyObject> {
    pub tool_id: String,
    pub result: T,
}

/// Execute tools result (union type)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExecuteToolsResult<T = AnyObject> {
    FunctionResult(ExecuteToolsFunctionResult<T>),
    ParametersResult(ExecuteToolsParametersResult<T>),
}

/// Parsed execute tools response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteToolsParsedResponse<T = AnyObject> {
    pub results: Option<Vec<ExecuteToolsResult<T>>>,
}

/// NLP search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpSearchResult<T> {
    pub original_query: String,
    pub generated_query: SearchParams,
    pub results: Vec<HashMap<String, serde_json::Value>>,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T> NlpSearchResult<T> {
    pub fn new(
        original_query: String,
        generated_query: SearchParams,
        results: Vec<HashMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            original_query,
            generated_query,
            results,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// NLP search stream status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NlpSearchStreamStatus {
    Init,
    OptimizingQuery,
    QueryOptimized,
    SelectingProps,
    SelectedProps,
    CombiningQueriesAndProperties,
    CombinedQueriesAndProperties,
    GeneratingQueries,
    GeneratedQueries,
    Searching,
    SearchResults,
    #[serde(untagged)]
    Other(String),
}

/// Generated query information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedQuery {
    pub index: u32,
    pub original_query: String,
    pub generated_query: HashMap<String, serde_json::Value>,
}

/// Selected properties information
pub type SelectedProperties = HashMap<String, HashMap<String, Vec<serde_json::Value>>>;

/// Combined query and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedQueryAndProperties {
    pub query: String,
    pub properties: SelectedProperties,
    pub filter_properties: HashMap<String, serde_json::Value>,
}

/// NLP search stream result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NlpSearchStreamResult<T> {
    pub status: NlpSearchStreamStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip)]
    _phantom: std::marker::PhantomData<T>,
}

impl<T> NlpSearchStreamResult<T> {
    pub fn new(status: NlpSearchStreamStatus, data: Option<serde_json::Value>) -> Self {
        Self {
            status,
            data,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
}

/// LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    OpenAI,
    Fireworks,
    Together,
    Google,
    Claude,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Assistant,
    User,
}

/// Message for conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Related questions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedQuestionsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<RelatedQuestionsFormat>,
}

/// Related questions format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelatedQuestionsFormat {
    Question,
    Query,
}

// Implementation methods for SearchParams
impl SearchParams {
    /// Create a new SearchParams with a term
    pub fn new<S: Into<String>>(term: S) -> Self {
        Self {
            term: term.into(),
            mode: None,
            limit: None,
            offset: None,
            properties: None,
            where_clause: None,
            facets: None,
            indexes: None,
            datasource_ids: None,
            exact: None,
            threshold: None,
            tolerance: None,
            user_id: None,
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

    /// Set the where clause for filtering
    pub fn with_where(mut self, where_clause: AnyObject) -> Self {
        self.where_clause = Some(where_clause);
        self
    }

    /// Set facets
    pub fn with_facets(mut self, facets: AnyObject) -> Self {
        self.facets = Some(facets);
        self
    }

    /// Set properties to search in
    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = Some(properties);
        self
    }
}

/// Default server user ID for server-side operations
pub const DEFAULT_SERVER_USER_ID: &str = "server-user-default";
