//! HTTP client for Orama API operations.

use crate::auth::{Auth, Target};
use crate::error::{OramaError, Result};
use reqwest::{Client as ReqwestClient, Method, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

/// API key position in the request
#[derive(Debug, Clone, PartialEq)]
pub enum ApiKeyPosition {
    Header,
    QueryParams,
}

/// Client request configuration
#[derive(Debug)]
pub struct ClientRequest<T> {
    pub target: Target,
    pub method: Method,
    pub path: String,
    pub api_key_position: ApiKeyPosition,
    pub body: Option<T>,
    pub params: Option<HashMap<String, String>>,
}

impl<T> ClientRequest<T> {
    /// Create a new GET request
    pub fn get(path: String, target: Target, api_key_position: ApiKeyPosition) -> Self {
        Self {
            target,
            method: Method::GET,
            path,
            api_key_position,
            body: None,
            params: None,
        }
    }

    /// Create a new POST request
    pub fn post(path: String, target: Target, api_key_position: ApiKeyPosition, body: T) -> Self {
        Self {
            target,
            method: Method::POST,
            path,
            api_key_position,
            body: Some(body),
            params: None,
        }
    }

    /// Add query parameters
    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = Some(params);
        self
    }

    /// Add a single query parameter
    pub fn with_param<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        let mut params = self.params.unwrap_or_default();
        params.insert(key.into(), value.into());
        self.params = Some(params);
        self
    }
}

/// HTTP client for Orama API
#[derive(Debug, Clone)]
pub struct OramaClient {
    client: Arc<ReqwestClient>,
    auth: Auth,
}

impl OramaClient {
    /// Create a new Orama client
    pub fn new(auth: Auth) -> Result<Self> {
        let client = ReqwestClient::builder()
            .user_agent("oramacore-client-rust/1.2.0")
            .build()?;

        Ok(Self {
            client: Arc::new(client),
            auth,
        })
    }

    /// Make a request and return the deserialized response
    pub async fn request<T, R>(&self, req: ClientRequest<T>) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        let response = self.get_response(req).await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();

            return Err(match status {
                401 => OramaError::auth("Unauthorized: are you using the correct API Key?"),
                400 => OramaError::api(status, format!("Bad Request: {}", text)),
                _ => OramaError::api(status, text),
            });
        }

        // Use robust JSON parsing for API responses
        let text = response.text().await?;
        let result = crate::utils::safe_json_parse::<R>(&text)
            .map_err(|e| OramaError::generic(&format!("Failed to parse API response: {}", e)))?;
        Ok(result)
    }

    /// Make a request and return the raw response
    pub async fn get_response<T>(&self, req: ClientRequest<T>) -> Result<Response>
    where
        T: Serialize,
    {
        let auth_ref = self.auth.get_ref(req.target).await?;
        let base_url = Url::parse(&auth_ref.base_url)?;
        let url = base_url.join(&req.path)?;

        let mut request_builder = self.client.request(req.method, url);

        // Set headers
        request_builder = request_builder.header("Content-Type", "application/json");

        if req.api_key_position == ApiKeyPosition::Header {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", auth_ref.bearer));
        }

        // Set query parameters
        let mut query_params = req.params.unwrap_or_default();
        if req.api_key_position == ApiKeyPosition::QueryParams {
            query_params.insert("api-key".to_string(), auth_ref.bearer);
        }

        if !query_params.is_empty() {
            request_builder = request_builder.query(&query_params);
        }

        // Set body for POST requests
        if let Some(body) = req.body {
            request_builder = request_builder.json(&body);
        }

        let response = request_builder.send().await?;
        Ok(response)
    }

    /// Get the underlying reqwest client
    pub fn inner(&self) -> &ReqwestClient {
        &self.client
    }

    /// Get authentication reference for a target
    pub async fn get_auth_ref(&self, target: Target) -> Result<crate::auth::AuthRef> {
        self.auth.get_ref(target).await
    }
}
