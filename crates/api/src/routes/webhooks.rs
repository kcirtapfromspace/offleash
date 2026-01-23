use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use base64::Engine;
use db::{
    models::{CreateDispute, CreateWebhookEvent, DisputeStatus, TransactionStatus, UpdateDispute},
    DisputeRepository, PaymentProviderRepository, TransactionRepository, WebhookEventRepository,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use shared::types::OrganizationId;
use uuid::Uuid;

use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

/// Stripe webhook handler
/// POST /webhooks/stripe/:org_id
pub async fn stripe_webhook(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let org_id: Uuid = org_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid org ID".to_string()))?;

    // Get tenant pool
    let pool = state
        .tenant_pool_manager
        .get_pool(OrganizationId::from_uuid(org_id))
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid organization".to_string()))?;

    // Get the provider to get the webhook secret
    let provider = PaymentProviderRepository::get_by_type(
        &pool,
        OrganizationId::from_uuid(org_id),
        db::models::PaymentProviderType::Stripe,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::BAD_REQUEST, "Stripe not configured".to_string()))?;

    // Get signature header
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing signature".to_string()))?;

    // Verify webhook signature
    let webhook_secret = provider
        .webhook_secret
        .as_ref()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Webhook secret not configured".to_string()))?;

    verify_stripe_signature(&body, signature, webhook_secret)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid signature: {}", e)))?;

    // Parse the event
    let payload_str = std::str::from_utf8(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload".to_string()))?;

    let event: StripeWebhookEvent = serde_json::from_str(payload_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

    // Check for idempotency
    let already_processed = WebhookEventRepository::is_processed(&pool, "stripe", &event.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if already_processed {
        return Ok(StatusCode::OK);
    }

    // Record the event
    let webhook_input = CreateWebhookEvent {
        provider: "stripe".to_string(),
        event_id: event.id.clone(),
        event_type: event.event_type.clone(),
        payload: serde_json::from_str(payload_str).unwrap_or_default(),
    };

    WebhookEventRepository::create(&pool, webhook_input)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Process the event
    match event.event_type.as_str() {
        "payment_intent.succeeded" => {
            if let Some(payment_intent_id) = event.data.object.get("id").and_then(|v| v.as_str()) {
                // Find and update the transaction
                if let Some(transaction) = TransactionRepository::get_by_external_id(
                    &pool,
                    OrganizationId::from_uuid(org_id),
                    payment_intent_id,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                {
                    TransactionRepository::update_status(
                        &pool,
                        transaction.id,
                        TransactionStatus::Succeeded,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        "payment_intent.payment_failed" => {
            if let Some(payment_intent_id) = event.data.object.get("id").and_then(|v| v.as_str()) {
                if let Some(transaction) = TransactionRepository::get_by_external_id(
                    &pool,
                    OrganizationId::from_uuid(org_id),
                    payment_intent_id,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                {
                    TransactionRepository::update_status(
                        &pool,
                        transaction.id,
                        TransactionStatus::Failed,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        "charge.refunded" => {
            if let Some(payment_intent_id) = event
                .data
                .object
                .get("payment_intent")
                .and_then(|v| v.as_str())
            {
                if let Some(transaction) = TransactionRepository::get_by_external_id(
                    &pool,
                    OrganizationId::from_uuid(org_id),
                    payment_intent_id,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                {
                    // Check if full or partial refund
                    let refunded = event
                        .data
                        .object
                        .get("amount_refunded")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;

                    let total = event
                        .data
                        .object
                        .get("amount")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;

                    let status = if refunded >= total {
                        TransactionStatus::Refunded
                    } else {
                        TransactionStatus::PartiallyRefunded
                    };

                    TransactionRepository::update_status(&pool, transaction.id, status)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        "charge.dispute.created" => {
            if let Some(payment_intent_id) = event
                .data
                .object
                .get("payment_intent")
                .and_then(|v| v.as_str())
            {
                if let Some(transaction) = TransactionRepository::get_by_external_id(
                    &pool,
                    OrganizationId::from_uuid(org_id),
                    payment_intent_id,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                {
                    let dispute_id = event
                        .data
                        .object
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let amount = event
                        .data
                        .object
                        .get("amount")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32;

                    let reason = event
                        .data
                        .object
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("general")
                        .to_string();

                    // Create dispute record
                    let dispute_input = CreateDispute {
                        transaction_id: transaction.id,
                        amount_cents: amount,
                        currency: "USD".to_string(),
                        stripe_dispute_id: Some(dispute_id.to_string()),
                        square_dispute_id: None,
                        reason,
                        evidence_due_by: None, // TODO: Parse from Stripe response
                    };

                    DisputeRepository::create(&pool, OrganizationId::from_uuid(org_id), dispute_input)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                    // Update transaction status
                    TransactionRepository::update_status(
                        &pool,
                        transaction.id,
                        TransactionStatus::Disputed,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        "charge.dispute.closed" => {
            let dispute_id = event
                .data
                .object
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let status = event
                .data
                .object
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let dispute_status = match status {
                "won" => DisputeStatus::Won,
                _ => DisputeStatus::Lost,
            };

            if let Some(dispute) =
                DisputeRepository::get_by_stripe_dispute(&pool, dispute_id)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            {
                let update_input = UpdateDispute {
                    status: Some(dispute_status),
                    resolved_at: Some(chrono::Utc::now()),
                    outcome: Some(status.to_string()),
                    ..Default::default()
                };

                DisputeRepository::update(&pool, dispute.id, update_input)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                // If dispute was won, revert transaction to succeeded
                if dispute_status == DisputeStatus::Won {
                    TransactionRepository::update_status(
                        &pool,
                        dispute.transaction_id,
                        TransactionStatus::Succeeded,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        _ => {
            // Unhandled event type - that's OK
            tracing::debug!("Unhandled Stripe webhook event: {}", event.event_type);
        }
    }

    // Mark event as processed
    WebhookEventRepository::mark_processed(&pool, "stripe", &event.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

/// Square webhook handler
/// POST /webhooks/square/:org_id
pub async fn square_webhook(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    let org_id: Uuid = org_id
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid org ID".to_string()))?;

    // Get tenant pool
    let pool = state
        .tenant_pool_manager
        .get_pool(OrganizationId::from_uuid(org_id))
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid organization".to_string()))?;

    // Get the provider to get the webhook secret
    let provider = PaymentProviderRepository::get_by_type(
        &pool,
        OrganizationId::from_uuid(org_id),
        db::models::PaymentProviderType::Square,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::BAD_REQUEST, "Square not configured".to_string()))?;

    // Get signature header
    let signature = headers
        .get("x-square-hmacsha256-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing signature".to_string()))?;

    // Verify webhook signature
    let webhook_secret = provider
        .webhook_secret
        .as_ref()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Webhook secret not configured".to_string()))?;

    // Get the webhook URL for signature verification
    let webhook_url = std::env::var("SQUARE_WEBHOOK_URL").unwrap_or_default();

    verify_square_signature(&body, signature, webhook_secret, &webhook_url)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid signature: {}", e)))?;

    // Parse the event
    let payload_str = std::str::from_utf8(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid payload".to_string()))?;

    let event: SquareWebhookEvent = serde_json::from_str(payload_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

    // Check for idempotency
    let already_processed = WebhookEventRepository::is_processed(&pool, "square", &event.event_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if already_processed {
        return Ok(StatusCode::OK);
    }

    // Record the event
    let webhook_input = CreateWebhookEvent {
        provider: "square".to_string(),
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        payload: serde_json::from_str(payload_str).unwrap_or_default(),
    };

    WebhookEventRepository::create(&pool, webhook_input)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Process the event
    match event.event_type.as_str() {
        "payment.completed" => {
            if let Some(payment_id) = event.data.object.get("payment").and_then(|p| p.get("id")).and_then(|v| v.as_str()) {
                if let Some(transaction) = TransactionRepository::get_by_external_id(
                    &pool,
                    OrganizationId::from_uuid(org_id),
                    payment_id,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                {
                    TransactionRepository::update_status(
                        &pool,
                        transaction.id,
                        TransactionStatus::Succeeded,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        "payment.failed" => {
            if let Some(payment_id) = event.data.object.get("payment").and_then(|p| p.get("id")).and_then(|v| v.as_str()) {
                if let Some(transaction) = TransactionRepository::get_by_external_id(
                    &pool,
                    OrganizationId::from_uuid(org_id),
                    payment_id,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                {
                    TransactionRepository::update_status(
                        &pool,
                        transaction.id,
                        TransactionStatus::Failed,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        "refund.created" | "refund.updated" => {
            // Handle refunds
            if let Some(refund) = event.data.object.get("refund") {
                let payment_id = refund.get("payment_id").and_then(|v| v.as_str()).unwrap_or("");
                let status = refund.get("status").and_then(|v| v.as_str()).unwrap_or("");

                if status == "COMPLETED" {
                    if let Some(transaction) = TransactionRepository::get_by_external_id(
                        &pool,
                        OrganizationId::from_uuid(org_id),
                        payment_id,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                    {
                        TransactionRepository::update_status(
                            &pool,
                            transaction.id,
                            TransactionStatus::Refunded,
                        )
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    }
                }
            }
        }
        "dispute.created" => {
            if let Some(dispute) = event.data.object.get("dispute") {
                let dispute_id = dispute.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let payment_id = dispute.get("disputed_payment").and_then(|dp| dp.get("payment_id")).and_then(|v| v.as_str()).unwrap_or("");

                let amount = dispute
                    .get("amount_money")
                    .and_then(|m| m.get("amount"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                let reason = dispute.get("reason").and_then(|v| v.as_str()).unwrap_or("general").to_string();

                if let Some(transaction) = TransactionRepository::get_by_external_id(
                    &pool,
                    OrganizationId::from_uuid(org_id),
                    payment_id,
                )
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                {
                    let dispute_input = CreateDispute {
                        transaction_id: transaction.id,
                        amount_cents: amount,
                        currency: "USD".to_string(),
                        stripe_dispute_id: None,
                        square_dispute_id: Some(dispute_id.to_string()),
                        reason,
                        evidence_due_by: None,
                    };

                    DisputeRepository::create(&pool, OrganizationId::from_uuid(org_id), dispute_input)
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

                    TransactionRepository::update_status(
                        &pool,
                        transaction.id,
                        TransactionStatus::Disputed,
                    )
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }
            }
        }
        _ => {
            tracing::debug!("Unhandled Square webhook event: {}", event.event_type);
        }
    }

    // Mark event as processed
    WebhookEventRepository::mark_processed(&pool, "square", &event.event_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

// Webhook event structures

#[derive(Debug, Deserialize)]
struct StripeWebhookEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    data: StripeEventData,
}

#[derive(Debug, Deserialize)]
struct StripeEventData {
    object: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct SquareWebhookEvent {
    event_id: String,
    #[serde(rename = "type")]
    event_type: String,
    data: SquareEventData,
}

#[derive(Debug, Deserialize)]
struct SquareEventData {
    object: serde_json::Value,
}

// Signature verification

fn verify_stripe_signature(payload: &[u8], signature: &str, secret: &str) -> Result<(), String> {
    // Parse the signature header
    let mut timestamp: Option<&str> = None;
    let mut signatures: Vec<&str> = Vec::new();

    for part in signature.split(',') {
        let kv: Vec<&str> = part.split('=').collect();
        if kv.len() == 2 {
            match kv[0] {
                "t" => timestamp = Some(kv[1]),
                "v1" => signatures.push(kv[1]),
                _ => {}
            }
        }
    }

    let timestamp = timestamp.ok_or("Missing timestamp")?;

    if signatures.is_empty() {
        return Err("Missing signature".to_string());
    }

    // Create signed payload
    let signed_payload = format!(
        "{}.{}",
        timestamp,
        std::str::from_utf8(payload).map_err(|_| "Invalid payload encoding")?
    );

    // Compute expected signature
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| "Invalid secret")?;
    mac.update(signed_payload.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    // Check if any signature matches
    for sig in signatures {
        if constant_time_eq(sig.as_bytes(), expected.as_bytes()) {
            return Ok(());
        }
    }

    Err("Signature mismatch".to_string())
}

fn verify_square_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
    webhook_url: &str,
) -> Result<(), String> {
    // Square signature is HMAC-SHA256 of URL + body
    let signed_payload = format!(
        "{}{}",
        webhook_url,
        std::str::from_utf8(payload).map_err(|_| "Invalid payload encoding")?
    );

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| "Invalid secret")?;
    mac.update(signed_payload.as_bytes());
    let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    if constant_time_eq(signature.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err("Signature mismatch".to_string())
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}
