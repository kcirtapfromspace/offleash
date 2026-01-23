use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{client::StripeClient, error::StripeResult};

/// Stripe Connect account types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Standard,
    Express,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Standard => write!(f, "standard"),
            AccountType::Express => write!(f, "express"),
        }
    }
}

/// Stripe Connect account
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub id: String,
    pub object: String,
    #[serde(rename = "type")]
    pub account_type: Option<String>,
    pub business_profile: Option<BusinessProfile>,
    pub charges_enabled: bool,
    pub payouts_enabled: bool,
    pub details_submitted: bool,
    pub email: Option<String>,
    pub requirements: Option<AccountRequirements>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BusinessProfile {
    pub name: Option<String>,
    pub url: Option<String>,
    pub support_email: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountRequirements {
    pub currently_due: Vec<String>,
    pub eventually_due: Vec<String>,
    pub past_due: Vec<String>,
    pub disabled_reason: Option<String>,
}

/// OAuth token response
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub livemode: bool,
    pub stripe_user_id: String,
    pub scope: Option<String>,
}

/// Account link for onboarding
#[derive(Debug, Clone, Deserialize)]
pub struct AccountLink {
    pub object: String,
    pub url: String,
    pub expires_at: i64,
    pub created: i64,
}

impl StripeClient {
    // ============ OAuth Flow ============

    /// Generate OAuth authorization URL
    pub fn get_oauth_url(&self, state: &str, redirect_uri: &str) -> Option<String> {
        let client_id = self.connect_client_id.as_ref()?;

        Some(format!(
            "https://connect.stripe.com/oauth/authorize?response_type=code&client_id={}&scope=read_write&state={}&redirect_uri={}",
            client_id,
            urlencoding::encode(state),
            urlencoding::encode(redirect_uri)
        ))
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_oauth_code(&self, code: &str) -> StripeResult<OAuthTokenResponse> {
        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), "authorization_code".to_string());
        params.insert("code".to_string(), code.to_string());

        self.post("/oauth/token", &params).await
    }

    /// Refresh OAuth token
    pub async fn refresh_oauth_token(&self, refresh_token: &str) -> StripeResult<OAuthTokenResponse> {
        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), "refresh_token".to_string());
        params.insert("refresh_token".to_string(), refresh_token.to_string());

        self.post("/oauth/token", &params).await
    }

    /// Deauthorize connected account
    pub async fn deauthorize_account(&self, stripe_user_id: &str) -> StripeResult<DeauthorizeResponse> {
        let client_id = self.connect_client_id.as_ref()
            .ok_or_else(|| super::error::StripeError::MissingField("connect_client_id".to_string()))?;

        let mut params = HashMap::new();
        params.insert("client_id".to_string(), client_id.clone());
        params.insert("stripe_user_id".to_string(), stripe_user_id.to_string());

        self.post("/oauth/deauthorize", &params).await
    }

    // ============ Account Management ============

    /// Create Express account
    pub async fn create_express_account(
        &self,
        email: &str,
        country: &str,
        business_type: &str,
    ) -> StripeResult<Account> {
        let mut params = HashMap::new();
        params.insert("type".to_string(), "express".to_string());
        params.insert("email".to_string(), email.to_string());
        params.insert("country".to_string(), country.to_string());
        params.insert("business_type".to_string(), business_type.to_string());
        params.insert("capabilities[card_payments][requested]".to_string(), "true".to_string());
        params.insert("capabilities[transfers][requested]".to_string(), "true".to_string());

        self.post("/accounts", &params).await
    }

    /// Retrieve a connected account
    pub async fn get_account(&self, account_id: &str) -> StripeResult<Account> {
        self.get(&format!("/accounts/{}", account_id)).await
    }

    /// Create account link for onboarding
    pub async fn create_account_link(
        &self,
        account_id: &str,
        refresh_url: &str,
        return_url: &str,
        link_type: &str, // "account_onboarding" or "account_update"
    ) -> StripeResult<AccountLink> {
        let mut params = HashMap::new();
        params.insert("account".to_string(), account_id.to_string());
        params.insert("refresh_url".to_string(), refresh_url.to_string());
        params.insert("return_url".to_string(), return_url.to_string());
        params.insert("type".to_string(), link_type.to_string());

        self.post("/account_links", &params).await
    }

    /// Create login link for Express dashboard
    pub async fn create_login_link(&self, account_id: &str) -> StripeResult<LoginLink> {
        let params = HashMap::new();
        self.post(&format!("/accounts/{}/login_links", account_id), &params).await
    }

    /// Delete (disconnect) a connected account
    pub async fn delete_account(&self, account_id: &str) -> StripeResult<DeletedAccount> {
        self.delete(&format!("/accounts/{}", account_id)).await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeauthorizeResponse {
    pub stripe_user_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginLink {
    pub object: String,
    pub url: String,
    pub created: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeletedAccount {
    pub id: String,
    pub object: String,
    pub deleted: bool,
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
