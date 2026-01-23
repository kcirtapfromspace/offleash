use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize};

use super::error::{SquareError, SquareResult};

const SQUARE_API_BASE: &str = "https://connect.squareup.com/v2";
const SQUARE_SANDBOX_API_BASE: &str = "https://connect.squareupsandbox.com/v2";

/// Square API client
#[derive(Clone)]
pub struct SquareClient {
    client: Client,
    access_token: String,
    pub application_id: Option<String>,
    pub application_secret: Option<String>,
    sandbox: bool,
}

impl SquareClient {
    /// Create a new Square client
    pub fn new(access_token: String, sandbox: bool) -> Self {
        Self {
            client: Client::new(),
            access_token,
            application_id: None,
            application_secret: None,
            sandbox,
        }
    }

    /// Create client with OAuth application credentials
    pub fn with_oauth(
        access_token: String,
        application_id: String,
        application_secret: String,
        sandbox: bool,
    ) -> Self {
        Self {
            client: Client::new(),
            access_token,
            application_id: Some(application_id),
            application_secret: Some(application_secret),
            sandbox,
        }
    }

    fn base_url(&self) -> &str {
        if self.sandbox {
            SQUARE_SANDBOX_API_BASE
        } else {
            SQUARE_API_BASE
        }
    }

    /// Make a GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> SquareResult<T> {
        let url = format!("{}{}", self.base_url(), path);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .header("Square-Version", "2024-01-18")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request with JSON body
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> SquareResult<T> {
        let url = format!("{}{}", self.base_url(), path);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .header("Square-Version", "2024-01-18")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> SquareResult<T> {
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).map_err(|e| SquareError::ParseError(e.to_string()))
        } else {
            // Try to parse Square error response
            if let Ok(error_response) = serde_json::from_str::<SquareErrorResponse>(&body) {
                if let Some(error) = error_response.errors.first() {
                    return Err(SquareError::ApiError {
                        category: error.category.clone(),
                        code: error.code.clone(),
                        message: error.detail.clone().unwrap_or_else(|| error.code.clone()),
                        detail: error.field.clone(),
                    });
                }
            }

            Err(SquareError::ApiError {
                category: "API_ERROR".to_string(),
                code: status.to_string(),
                message: body,
                detail: None,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
struct SquareErrorResponse {
    errors: Vec<SquareErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct SquareErrorDetail {
    category: String,
    code: String,
    detail: Option<String>,
    field: Option<String>,
}
