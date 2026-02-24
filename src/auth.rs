//! Authentication handling for Orama client.

use std::sync::Arc;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{OramaError, Result};

/// JWT response from authentication endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtRequestResponse {
    jwt: String,
    #[serde(rename = "writerURL")]
    writer_url: String,
    #[serde(rename = "readerApiKey")]
    reader_api_key: String,
    #[serde(rename = "readerURL")]
    reader_url: String,
    #[serde(rename = "expiresIn")]
    expires_in: u64,
}

/// Authentication configuration for API key authentication
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub api_key: String,
    pub reader_url: Option<String>,
    pub writer_url: Option<String>,
}

/// Authentication configuration for JWT authentication
#[derive(Debug, Clone)]
pub struct JwtAuth {
    pub auth_jwt_url: String,
    pub collection_id: String,
    pub private_api_key: String,
    pub reader_url: Option<String>,
    pub writer_url: Option<String>,
}

/// Authentication configuration enum
#[derive(Debug, Clone)]
pub enum AuthConfig {
    ApiKey(ApiKeyAuth),
    Jwt(JwtAuth),
}

/// Authentication reference containing bearer token and base URL
#[derive(Debug, Clone)]
pub struct AuthRef {
    pub bearer: String,
    pub base_url: String,
}

/// Target for the request (reader or writer)
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Reader,
    Writer,
}

/// Authentication handler
#[derive(Debug, Clone)]
pub struct Auth {
    config: AuthConfig,
    client: Arc<Client>,
}

impl Auth {
    /// Create a new authentication handler
    pub fn new(config: AuthConfig, client: Arc<Client>) -> Self {
        Self { config, client }
    }

    /// Get authentication reference for the specified target
    pub async fn get_ref(&self, target: Target) -> Result<AuthRef> {
        match &self.config {
            AuthConfig::ApiKey(config) => {
                let bearer = config.api_key.clone();
                let base_url = match target {
                    Target::Writer => {
                        config.writer_url.as_ref()
                            .ok_or_else(|| OramaError::config(
                                "Cannot perform a request to a writer without the writerURL. Use cluster.writerURL to configure it"
                            ))?
                            .clone()
                    }
                    Target::Reader => {
                        config.reader_url.as_ref()
                            .ok_or_else(|| OramaError::config(
                                "Cannot perform a request to a reader without the readerURL. Use cluster.readerURL to configure it"
                            ))?
                            .clone()
                    }
                };

                Ok(AuthRef { bearer, base_url })
            }
            AuthConfig::Jwt(config) => {
                let jwt_response = self
                    .get_jwt_token(
                        &config.auth_jwt_url,
                        &config.collection_id,
                        &config.private_api_key,
                        "write",
                    )
                    .await?;

                let (bearer, base_url) = match target {
                    Target::Reader => {
                        let base_url = config
                            .reader_url
                            .as_ref()
                            .unwrap_or(&jwt_response.reader_url)
                            .clone();
                        (jwt_response.reader_api_key, base_url)
                    }
                    Target::Writer => {
                        let base_url = config
                            .writer_url
                            .as_ref()
                            .unwrap_or(&jwt_response.writer_url)
                            .clone();
                        (jwt_response.jwt, base_url)
                    }
                };

                Ok(AuthRef { bearer, base_url })
            }
        }
    }

    /// Get JWT token from authentication endpoint
    async fn get_jwt_token(
        &self,
        auth_jwt_url: &str,
        collection_id: &str,
        private_api_key: &str,
        scope: &str,
    ) -> Result<JwtRequestResponse> {
        let payload = serde_json::json!({
            "collectionId": collection_id,
            "privateApiKey": private_api_key,
            "scope": scope
        });

        let response = self.client.post(auth_jwt_url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(OramaError::api(
                status,
                format!("JWT request to {auth_jwt_url} failed: {text}"),
            ));
        }

        let jwt_response: JwtRequestResponse = response.json().await?;
        Ok(jwt_response)
    }
}

impl ApiKeyAuth {
    /// Create a new API key authentication configuration
    pub fn new<S: Into<String>>(api_key: S) -> Self {
        Self {
            api_key: api_key.into(),
            reader_url: None,
            writer_url: None,
        }
    }

    /// Set the reader URL
    pub fn with_reader_url<S: Into<String>>(mut self, url: S) -> Self {
        self.reader_url = Some(url.into());
        self
    }

    /// Set the writer URL
    pub fn with_writer_url<S: Into<String>>(mut self, url: S) -> Self {
        self.writer_url = Some(url.into());
        self
    }
}

impl JwtAuth {
    /// Create a new JWT authentication configuration
    pub fn new<S: Into<String>>(auth_jwt_url: S, collection_id: S, private_api_key: S) -> Self {
        Self {
            auth_jwt_url: auth_jwt_url.into(),
            collection_id: collection_id.into(),
            private_api_key: private_api_key.into(),
            reader_url: None,
            writer_url: None,
        }
    }

    /// Set the reader URL
    pub fn with_reader_url<S: Into<String>>(mut self, url: S) -> Self {
        self.reader_url = Some(url.into());
        self
    }

    /// Set the writer URL
    pub fn with_writer_url<S: Into<String>>(mut self, url: S) -> Self {
        self.writer_url = Some(url.into());
        self
    }
}
