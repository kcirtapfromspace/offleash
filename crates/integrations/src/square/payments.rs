use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{client::SquareClient, error::SquareResult};

/// Money amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Money {
    pub amount: i64,
    pub currency: String,
}

impl Money {
    pub fn usd(cents: i64) -> Self {
        Self {
            amount: cents,
            currency: "USD".to_string(),
        }
    }
}

/// Payment object
#[derive(Debug, Clone, Deserialize)]
pub struct Payment {
    pub id: String,
    pub status: String,
    pub amount_money: Money,
    pub total_money: Option<Money>,
    pub tip_money: Option<Money>,
    pub app_fee_money: Option<Money>,
    pub source_type: Option<String>,
    pub card_details: Option<CardDetails>,
    pub receipt_number: Option<String>,
    pub receipt_url: Option<String>,
    pub order_id: Option<String>,
    pub customer_id: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardDetails {
    pub card: Option<Card>,
    pub entry_method: Option<String>,
    pub status: String,
}

/// Card information
#[derive(Debug, Clone, Deserialize)]
pub struct Card {
    pub id: Option<String>,
    pub card_brand: Option<String>,
    pub last_4: Option<String>,
    pub exp_month: Option<i64>,
    pub exp_year: Option<i64>,
    pub cardholder_name: Option<String>,
    pub fingerprint: Option<String>,
}

/// Refund object
#[derive(Debug, Clone, Deserialize)]
pub struct Refund {
    pub id: String,
    pub status: String,
    pub amount_money: Money,
    pub payment_id: String,
    pub reason: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}

/// Create payment request
#[derive(Debug, Serialize)]
pub struct CreatePaymentRequest {
    pub source_id: String,
    pub idempotency_key: String,
    pub amount_money: Money,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tip_money: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_fee_money: Option<Money>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub autocomplete: bool,
}

impl CreatePaymentRequest {
    pub fn new(source_id: String, amount_cents: i64) -> Self {
        Self {
            source_id,
            idempotency_key: Uuid::new_v4().to_string(),
            amount_money: Money::usd(amount_cents),
            tip_money: None,
            app_fee_money: None,
            customer_id: None,
            location_id: None,
            reference_id: None,
            note: None,
            autocomplete: true,
        }
    }

    pub fn with_app_fee(mut self, fee_cents: i64) -> Self {
        self.app_fee_money = Some(Money::usd(fee_cents));
        self
    }

    pub fn with_customer(mut self, customer_id: String) -> Self {
        self.customer_id = Some(customer_id);
        self
    }

    pub fn with_location(mut self, location_id: String) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn with_reference(mut self, reference_id: String) -> Self {
        self.reference_id = Some(reference_id);
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }
}

#[derive(Debug, Deserialize)]
struct PaymentResponse {
    payment: Payment,
}

#[derive(Debug, Deserialize)]
struct RefundResponse {
    refund: Refund,
}

#[derive(Debug, Serialize)]
struct RefundPaymentRequest {
    idempotency_key: String,
    payment_id: String,
    amount_money: Money,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl SquareClient {
    /// Create a payment
    pub async fn create_payment(&self, request: CreatePaymentRequest) -> SquareResult<Payment> {
        let response: PaymentResponse = self.post("/payments", &request).await?;
        Ok(response.payment)
    }

    /// Get a payment
    pub async fn get_payment(&self, payment_id: &str) -> SquareResult<Payment> {
        let response: PaymentResponse = self.get(&format!("/payments/{}", payment_id)).await?;
        Ok(response.payment)
    }

    /// Complete a payment (if autocomplete was false)
    pub async fn complete_payment(&self, payment_id: &str) -> SquareResult<Payment> {
        #[derive(Serialize)]
        struct EmptyBody {}
        let response: PaymentResponse = self
            .post(&format!("/payments/{}/complete", payment_id), &EmptyBody {})
            .await?;
        Ok(response.payment)
    }

    /// Cancel a payment
    pub async fn cancel_payment(&self, payment_id: &str) -> SquareResult<Payment> {
        #[derive(Serialize)]
        struct EmptyBody {}
        let response: PaymentResponse = self
            .post(&format!("/payments/{}/cancel", payment_id), &EmptyBody {})
            .await?;
        Ok(response.payment)
    }

    /// Refund a payment
    pub async fn refund_payment(
        &self,
        payment_id: &str,
        amount_cents: i64,
        reason: Option<&str>,
    ) -> SquareResult<Refund> {
        let request = RefundPaymentRequest {
            idempotency_key: Uuid::new_v4().to_string(),
            payment_id: payment_id.to_string(),
            amount_money: Money::usd(amount_cents),
            reason: reason.map(|s| s.to_string()),
        };

        let response: RefundResponse = self.post("/refunds", &request).await?;
        Ok(response.refund)
    }

    /// Get a refund
    pub async fn get_refund(&self, refund_id: &str) -> SquareResult<Refund> {
        let response: RefundResponse = self.get(&format!("/refunds/{}", refund_id)).await?;
        Ok(response.refund)
    }
}
