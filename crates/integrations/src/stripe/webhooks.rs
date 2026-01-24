use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

use super::error::{StripeError, StripeResult};

type HmacSha256 = Hmac<Sha256>;

/// Stripe webhook event
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub object: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub created: i64,
    pub livemode: bool,
    pub data: WebhookEventData,
    pub api_version: Option<String>,
    pub request: Option<WebhookRequest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEventData {
    pub object: serde_json::Value,
    pub previous_attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookRequest {
    pub id: Option<String>,
    pub idempotency_key: Option<String>,
}

/// Common event types
pub mod event_types {
    pub const PAYMENT_INTENT_SUCCEEDED: &str = "payment_intent.succeeded";
    pub const PAYMENT_INTENT_FAILED: &str = "payment_intent.payment_failed";
    pub const PAYMENT_INTENT_CANCELED: &str = "payment_intent.canceled";
    pub const PAYMENT_INTENT_CREATED: &str = "payment_intent.created";
    pub const PAYMENT_INTENT_PROCESSING: &str = "payment_intent.processing";
    pub const PAYMENT_INTENT_REQUIRES_ACTION: &str = "payment_intent.requires_action";

    pub const CHARGE_SUCCEEDED: &str = "charge.succeeded";
    pub const CHARGE_FAILED: &str = "charge.failed";
    pub const CHARGE_REFUNDED: &str = "charge.refunded";
    pub const CHARGE_DISPUTE_CREATED: &str = "charge.dispute.created";
    pub const CHARGE_DISPUTE_CLOSED: &str = "charge.dispute.closed";
    pub const CHARGE_DISPUTE_UPDATED: &str = "charge.dispute.updated";

    pub const CUSTOMER_SUBSCRIPTION_CREATED: &str = "customer.subscription.created";
    pub const CUSTOMER_SUBSCRIPTION_UPDATED: &str = "customer.subscription.updated";
    pub const CUSTOMER_SUBSCRIPTION_DELETED: &str = "customer.subscription.deleted";
    pub const CUSTOMER_SUBSCRIPTION_TRIAL_WILL_END: &str = "customer.subscription.trial_will_end";

    pub const INVOICE_PAID: &str = "invoice.paid";
    pub const INVOICE_PAYMENT_FAILED: &str = "invoice.payment_failed";
    pub const INVOICE_UPCOMING: &str = "invoice.upcoming";

    pub const ACCOUNT_UPDATED: &str = "account.updated";
    pub const ACCOUNT_APPLICATION_DEAUTHORIZED: &str = "account.application.deauthorized";

    pub const PAYOUT_CREATED: &str = "payout.created";
    pub const PAYOUT_PAID: &str = "payout.paid";
    pub const PAYOUT_FAILED: &str = "payout.failed";

    pub const TRANSFER_CREATED: &str = "transfer.created";
    pub const TRANSFER_REVERSED: &str = "transfer.reversed";
}

/// Verify webhook signature
pub fn verify_signature(payload: &str, signature: &str, secret: &str) -> StripeResult<()> {
    // Parse the signature header
    let parts: std::collections::HashMap<&str, &str> = signature
        .split(',')
        .filter_map(|part| {
            let mut split = part.splitn(2, '=');
            Some((split.next()?, split.next()?))
        })
        .collect();

    let timestamp = parts.get("t").ok_or(StripeError::WebhookSignatureError)?;

    let v1_signature = parts.get("v1").ok_or(StripeError::WebhookSignatureError)?;

    // Prepare signed payload
    let signed_payload = format!("{}.{}", timestamp, payload);

    // Compute expected signature
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| StripeError::WebhookSignatureError)?;
    mac.update(signed_payload.as_bytes());

    let expected_signature = hex::encode(mac.finalize().into_bytes());

    // Compare signatures
    if expected_signature != *v1_signature {
        return Err(StripeError::WebhookSignatureError);
    }

    // Check timestamp tolerance (5 minutes)
    let timestamp_int: i64 = timestamp
        .parse()
        .map_err(|_| StripeError::WebhookSignatureError)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| StripeError::WebhookSignatureError)?
        .as_secs() as i64;

    if (now - timestamp_int).abs() > 300 {
        return Err(StripeError::WebhookSignatureError);
    }

    Ok(())
}

/// Parse webhook event from payload
pub fn parse_event(payload: &str) -> StripeResult<WebhookEvent> {
    serde_json::from_str(payload).map_err(|e| StripeError::ParseError(e.to_string()))
}

/// Verify and parse webhook
pub fn construct_event(payload: &str, signature: &str, secret: &str) -> StripeResult<WebhookEvent> {
    verify_signature(payload, signature, secret)?;
    parse_event(payload)
}

/// Dispute object from webhook
#[derive(Debug, Clone, Deserialize)]
pub struct Dispute {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub charge: String,
    pub currency: String,
    pub reason: String,
    pub status: String,
    pub evidence_details: Option<EvidenceDetails>,
    pub created: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvidenceDetails {
    pub due_by: Option<i64>,
    pub has_evidence: bool,
    pub past_due: bool,
    pub submission_count: i32,
}

/// Payout object from webhook
#[derive(Debug, Clone, Deserialize)]
pub struct Payout {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub arrival_date: i64,
    pub currency: String,
    pub status: String,
    pub destination: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub created: i64,
}
