use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{client::StripeClient, error::StripeResult};

/// Payment Intent status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentIntentStatus {
    RequiresPaymentMethod,
    RequiresConfirmation,
    RequiresAction,
    Processing,
    RequiresCapture,
    Canceled,
    Succeeded,
}

/// Payment Intent object
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub amount_received: Option<i64>,
    pub currency: String,
    pub status: PaymentIntentStatus,
    pub client_secret: Option<String>,
    pub customer: Option<String>,
    pub payment_method: Option<String>,
    pub transfer_data: Option<TransferData>,
    pub application_fee_amount: Option<i64>,
    pub latest_charge: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub created: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransferData {
    pub destination: String,
    pub amount: Option<i64>,
}

/// Charge object
#[derive(Debug, Clone, Deserialize)]
pub struct Charge {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub amount_refunded: i64,
    pub currency: String,
    pub paid: bool,
    pub refunded: bool,
    pub status: String,
    pub payment_intent: Option<String>,
    pub payment_method_details: Option<PaymentMethodDetails>,
    pub receipt_url: Option<String>,
    pub created: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentMethodDetails {
    pub card: Option<CardDetails>,
    #[serde(rename = "type")]
    pub method_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardDetails {
    pub brand: String,
    pub last4: String,
    pub exp_month: i32,
    pub exp_year: i32,
    pub funding: String,
    pub country: Option<String>,
}

/// Refund object
#[derive(Debug, Clone, Deserialize)]
pub struct Refund {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub charge: Option<String>,
    pub currency: String,
    pub payment_intent: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub created: i64,
}

/// Parameters for creating a Payment Intent
#[derive(Debug, Clone, Default)]
pub struct CreatePaymentIntentParams {
    pub amount: i64,
    pub currency: String,
    pub customer: Option<String>,
    pub payment_method: Option<String>,
    pub confirm: bool,
    pub automatic_payment_methods: bool,
    pub return_url: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    // Connect-specific
    pub application_fee_amount: Option<i64>,
    pub transfer_data_destination: Option<String>,
    pub on_behalf_of: Option<String>,
}

impl StripeClient {
    // ============ Payment Intents ============

    /// Create a payment intent
    pub async fn create_payment_intent(
        &self,
        params: CreatePaymentIntentParams,
    ) -> StripeResult<PaymentIntent> {
        let mut form = HashMap::new();
        form.insert("amount".to_string(), params.amount.to_string());
        form.insert("currency".to_string(), params.currency);

        if let Some(customer) = params.customer {
            form.insert("customer".to_string(), customer);
        }

        if let Some(pm) = params.payment_method {
            form.insert("payment_method".to_string(), pm);
        }

        if params.confirm {
            form.insert("confirm".to_string(), "true".to_string());
        }

        if params.automatic_payment_methods {
            form.insert(
                "automatic_payment_methods[enabled]".to_string(),
                "true".to_string(),
            );
        }

        if let Some(return_url) = params.return_url {
            form.insert("return_url".to_string(), return_url);
        }

        // Connect parameters
        if let Some(fee) = params.application_fee_amount {
            form.insert("application_fee_amount".to_string(), fee.to_string());
        }

        if let Some(dest) = params.transfer_data_destination {
            form.insert("transfer_data[destination]".to_string(), dest);
        }

        if let Some(on_behalf_of) = params.on_behalf_of {
            form.insert("on_behalf_of".to_string(), on_behalf_of);
        }

        if let Some(metadata) = params.metadata {
            for (key, value) in metadata {
                form.insert(format!("metadata[{}]", key), value);
            }
        }

        self.post("/payment_intents", &form).await
    }

    /// Retrieve a payment intent
    pub async fn get_payment_intent(&self, id: &str) -> StripeResult<PaymentIntent> {
        self.get(&format!("/payment_intents/{}", id)).await
    }

    /// Confirm a payment intent
    pub async fn confirm_payment_intent(
        &self,
        id: &str,
        payment_method: Option<&str>,
        return_url: Option<&str>,
    ) -> StripeResult<PaymentIntent> {
        let mut params = HashMap::new();

        if let Some(pm) = payment_method {
            params.insert("payment_method".to_string(), pm.to_string());
        }

        if let Some(url) = return_url {
            params.insert("return_url".to_string(), url.to_string());
        }

        self.post(&format!("/payment_intents/{}/confirm", id), &params)
            .await
    }

    /// Cancel a payment intent
    pub async fn cancel_payment_intent(&self, id: &str) -> StripeResult<PaymentIntent> {
        let params = HashMap::new();
        self.post(&format!("/payment_intents/{}/cancel", id), &params)
            .await
    }

    /// Capture a payment intent (for manual capture)
    pub async fn capture_payment_intent(
        &self,
        id: &str,
        amount_to_capture: Option<i64>,
    ) -> StripeResult<PaymentIntent> {
        let mut params = HashMap::new();

        if let Some(amount) = amount_to_capture {
            params.insert("amount_to_capture".to_string(), amount.to_string());
        }

        self.post(&format!("/payment_intents/{}/capture", id), &params)
            .await
    }

    // ============ Charges ============

    /// Retrieve a charge
    pub async fn get_charge(&self, id: &str) -> StripeResult<Charge> {
        self.get(&format!("/charges/{}", id)).await
    }

    // ============ Refunds ============

    /// Create a refund
    pub async fn create_refund(
        &self,
        payment_intent: &str,
        amount: Option<i64>,
        reason: Option<&str>,
    ) -> StripeResult<Refund> {
        let mut params = HashMap::new();
        params.insert("payment_intent".to_string(), payment_intent.to_string());

        if let Some(amt) = amount {
            params.insert("amount".to_string(), amt.to_string());
        }

        if let Some(r) = reason {
            params.insert("reason".to_string(), r.to_string());
        }

        self.post("/refunds", &params).await
    }

    /// Retrieve a refund
    pub async fn get_refund(&self, id: &str) -> StripeResult<Refund> {
        self.get(&format!("/refunds/{}", id)).await
    }

    // ============ Transfers (for Connect) ============

    /// Create a transfer to connected account
    pub async fn create_transfer(
        &self,
        amount: i64,
        currency: &str,
        destination: &str,
        source_transaction: Option<&str>,
        metadata: Option<HashMap<String, String>>,
    ) -> StripeResult<Transfer> {
        let mut params = HashMap::new();
        params.insert("amount".to_string(), amount.to_string());
        params.insert("currency".to_string(), currency.to_string());
        params.insert("destination".to_string(), destination.to_string());

        if let Some(source) = source_transaction {
            params.insert("source_transaction".to_string(), source.to_string());
        }

        if let Some(meta) = metadata {
            for (key, value) in meta {
                params.insert(format!("metadata[{}]", key), value);
            }
        }

        self.post("/transfers", &params).await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transfer {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub currency: String,
    pub destination: String,
    pub source_transaction: Option<String>,
    pub created: i64,
}
