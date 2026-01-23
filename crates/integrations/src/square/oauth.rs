use serde::{Deserialize, Serialize};

use super::{client::SquareClient, error::{SquareError, SquareResult}};

const SQUARE_OAUTH_BASE: &str = "https://connect.squareup.com/oauth2";
const SQUARE_SANDBOX_OAUTH_BASE: &str = "https://connect.squareupsandbox.com/oauth2";

/// OAuth token response
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: String,
    pub merchant_id: String,
    pub refresh_token: Option<String>,
}

/// Revoke token response
#[derive(Debug, Clone, Deserialize)]
pub struct RevokeTokenResponse {
    pub success: bool,
}

#[derive(Debug, Serialize)]
struct ObtainTokenRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uri: Option<String>,
}

#[derive(Debug, Serialize)]
struct RevokeTokenRequest {
    client_id: String,
    access_token: Option<String>,
    merchant_id: Option<String>,
    revoke_only_access_token: Option<bool>,
}

impl SquareClient {
    /// Generate OAuth authorization URL
    pub fn get_oauth_url(
        application_id: &str,
        scopes: &[&str],
        state: &str,
        redirect_uri: &str,
        sandbox: bool,
    ) -> String {
        let base = if sandbox {
            SQUARE_SANDBOX_OAUTH_BASE
        } else {
            SQUARE_OAUTH_BASE
        };

        let scope = scopes.join("+");

        format!(
            "{}/authorize?client_id={}&scope={}&session=false&state={}&redirect_uri={}",
            base,
            application_id,
            scope,
            urlencoding::encode(state),
            urlencoding::encode(redirect_uri)
        )
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_oauth_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> SquareResult<OAuthTokenResponse> {
        let app_id = self.application_id.as_ref()
            .ok_or_else(|| SquareError::MissingField("application_id".to_string()))?;
        let app_secret = self.application_secret.as_ref()
            .ok_or_else(|| SquareError::MissingField("application_secret".to_string()))?;

        let request = ObtainTokenRequest {
            client_id: app_id.clone(),
            client_secret: app_secret.clone(),
            grant_type: "authorization_code".to_string(),
            code: Some(code.to_string()),
            refresh_token: None,
            redirect_uri: Some(redirect_uri.to_string()),
        };

        self.post("/oauth2/token", &request).await
    }

    /// Refresh OAuth token
    pub async fn refresh_oauth_token(
        &self,
        refresh_token: &str,
    ) -> SquareResult<OAuthTokenResponse> {
        let app_id = self.application_id.as_ref()
            .ok_or_else(|| SquareError::MissingField("application_id".to_string()))?;
        let app_secret = self.application_secret.as_ref()
            .ok_or_else(|| SquareError::MissingField("application_secret".to_string()))?;

        let request = ObtainTokenRequest {
            client_id: app_id.clone(),
            client_secret: app_secret.clone(),
            grant_type: "refresh_token".to_string(),
            code: None,
            refresh_token: Some(refresh_token.to_string()),
            redirect_uri: None,
        };

        self.post("/oauth2/token", &request).await
    }

    /// Revoke OAuth token
    pub async fn revoke_oauth_token(
        &self,
        access_token: &str,
        merchant_id: &str,
    ) -> SquareResult<RevokeTokenResponse> {
        let app_id = self.application_id.as_ref()
            .ok_or_else(|| SquareError::MissingField("application_id".to_string()))?;

        let request = RevokeTokenRequest {
            client_id: app_id.clone(),
            access_token: Some(access_token.to_string()),
            merchant_id: Some(merchant_id.to_string()),
            revoke_only_access_token: Some(false),
        };

        self.post("/oauth2/revoke", &request).await
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                _ => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        result
    }
}
