use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{StripeError, StripeResult};

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

/// Stripe API client
#[derive(Clone)]
pub struct StripeClient {
    client: Client,
    secret_key: String,
    pub connect_client_id: Option<String>,
}

impl StripeClient {
    /// Create a new Stripe client
    pub fn new(secret_key: String, connect_client_id: Option<String>) -> Self {
        Self {
            client: Client::new(),
            secret_key,
            connect_client_id,
        }
    }

    /// Make a GET request to Stripe API
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> StripeResult<T> {
        let url = format!("{}{}", STRIPE_API_BASE, path);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.secret_key, None::<&str>)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request to Stripe API with form data
    pub async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &HashMap<String, String>,
    ) -> StripeResult<T> {
        let url = format!("{}{}", STRIPE_API_BASE, path);

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.secret_key, None::<&str>)
            .form(params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request on behalf of a connected account
    pub async fn post_on_behalf<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &HashMap<String, String>,
        stripe_account: &str,
    ) -> StripeResult<T> {
        let url = format!("{}{}", STRIPE_API_BASE, path);

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.secret_key, None::<&str>)
            .header("Stripe-Account", stripe_account)
            .form(params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> StripeResult<T> {
        let url = format!("{}{}", STRIPE_API_BASE, path);

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.secret_key, None::<&str>)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> StripeResult<T> {
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).map_err(|e| StripeError::ParseError(e.to_string()))
        } else {
            // Try to parse Stripe error response
            if let Ok(error_response) = serde_json::from_str::<StripeErrorResponse>(&body) {
                Err(StripeError::ApiError {
                    code: error_response.error.code,
                    message: error_response.error.message,
                    param: error_response.error.param,
                    decline_code: error_response.error.decline_code,
                })
            } else {
                Err(StripeError::ApiError {
                    code: Some(status.to_string()),
                    message: body,
                    param: None,
                    decline_code: None,
                })
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct StripeErrorResponse {
    error: StripeErrorDetail,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StripeErrorDetail {
    #[serde(rename = "type")]
    error_type: Option<String>,
    code: Option<String>,
    message: String,
    param: Option<String>,
    decline_code: Option<String>,
}

// ============ Common Stripe Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeList<T> {
    pub object: String,
    pub data: Vec<T>,
    pub has_more: bool,
    pub url: String,
}
