use axum::{extract::State, Json};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    auth::AuthUser,
    error::{ApiError, ApiResult},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    pub feedback_type: FeedbackType,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackType {
    Bug,
    Feature,
}

impl FeedbackType {
    fn label(&self) -> &'static str {
        match self {
            FeedbackType::Bug => "bug",
            FeedbackType::Feature => "enhancement",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            FeedbackType::Bug => "bug",
            FeedbackType::Feature => "sparkles",
        }
    }

    fn title_prefix(&self) -> &'static str {
        match self {
            FeedbackType::Bug => "[Bug]",
            FeedbackType::Feature => "[Feature Request]",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FeedbackResponse {
    pub success: bool,
    pub issue_url: Option<String>,
    pub issue_number: Option<i64>,
    pub message: String,
}

#[derive(Debug, Serialize)]
struct GitHubIssueRequest {
    title: String,
    body: String,
    labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubIssueResponse {
    number: i64,
    html_url: String,
}

/// Submit feedback (bug report or feature request)
pub async fn submit_feedback(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<FeedbackRequest>,
) -> ApiResult<Json<FeedbackResponse>> {
    // Validate input
    if req.title.trim().len() < 5 {
        return Err(ApiError::from(shared::AppError::Validation(
            "Title must be at least 5 characters".to_string(),
        )));
    }

    if req.description.trim().len() < 20 {
        return Err(ApiError::from(shared::AppError::Validation(
            "Description must be at least 20 characters".to_string(),
        )));
    }

    // Get GitHub token from config
    let github_token = match &state.config.github_token {
        Some(token) if !token.is_empty() => token.clone(),
        _ => {
            tracing::warn!("GitHub token not configured, feedback will not create an issue");
            return Ok(Json(FeedbackResponse {
                success: true,
                issue_url: None,
                issue_number: None,
                message: "Feedback received. Thank you!".to_string(),
            }));
        }
    };

    let github_repo = state
        .config
        .github_feedback_repo
        .clone()
        .unwrap_or_else(|| "kcirtapfromspace/offleash".to_string());

    // Format the issue body
    let issue_body = format!(
        r#"## :{}: {} Report

**Submitted by user:** {}

### Description

{}

---

*This issue was automatically created from user feedback in the OFFLEASH app.*
"#,
        req.feedback_type.emoji(),
        match req.feedback_type {
            FeedbackType::Bug => "Bug",
            FeedbackType::Feature => "Feature Request",
        },
        auth_user.user_id,
        req.description.trim()
    );

    // Create the GitHub issue
    let github_issue = GitHubIssueRequest {
        title: format!("{} {}", req.feedback_type.title_prefix(), req.title.trim()),
        body: issue_body,
        labels: vec![
            req.feedback_type.label().to_string(),
            "user-feedback".to_string(),
        ],
    };

    let client = Client::new();
    let response = client
        .post(format!(
            "https://api.github.com/repos/{}/issues",
            github_repo
        ))
        .header("Authorization", format!("Bearer {}", github_token))
        .header("User-Agent", "OFFLEASH-App")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&github_issue)
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => match res.json::<GitHubIssueResponse>().await {
            Ok(issue) => {
                tracing::info!(
                    "Created GitHub issue #{} for user {}",
                    issue.number,
                    auth_user.user_id
                );
                Ok(Json(FeedbackResponse {
                    success: true,
                    issue_url: Some(issue.html_url),
                    issue_number: Some(issue.number),
                    message: "Feedback submitted successfully!".to_string(),
                }))
            }
            Err(e) => {
                tracing::error!("Failed to parse GitHub response: {:?}", e);
                Ok(Json(FeedbackResponse {
                    success: true,
                    issue_url: None,
                    issue_number: None,
                    message: "Feedback received, but could not retrieve issue details.".to_string(),
                }))
            }
        },
        Ok(res) => {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            tracing::error!("GitHub API error: {} - {}", status, body);
            Err(ApiError::from(shared::AppError::Internal(
                "Failed to create GitHub issue".to_string(),
            )))
        }
        Err(e) => {
            tracing::error!("Failed to call GitHub API: {:?}", e);
            Err(ApiError::from(shared::AppError::Internal(
                "Failed to connect to GitHub".to_string(),
            )))
        }
    }
}
