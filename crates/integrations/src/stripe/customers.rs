use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{client::{Address, StripeClient, StripeList}, error::StripeResult};

/// Stripe Customer
#[derive(Debug, Clone, Deserialize)]
pub struct Customer {
    pub id: String,
    pub object: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub address: Option<Address>,
    pub default_source: Option<String>,
    pub invoice_settings: Option<InvoiceSettings>,
    pub metadata: Option<HashMap<String, String>>,
    pub created: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceSettings {
    pub default_payment_method: Option<String>,
}

/// Payment Method object
#[derive(Debug, Clone, Deserialize)]
pub struct PaymentMethod {
    pub id: String,
    pub object: String,
    #[serde(rename = "type")]
    pub method_type: String,
    pub card: Option<CardPaymentMethod>,
    pub billing_details: Option<BillingDetails>,
    pub customer: Option<String>,
    pub created: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardPaymentMethod {
    pub brand: String,
    pub last4: String,
    pub exp_month: i32,
    pub exp_year: i32,
    pub funding: String,
    pub country: Option<String>,
    pub fingerprint: Option<String>,
    pub wallet: Option<WalletInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WalletInfo {
    #[serde(rename = "type")]
    pub wallet_type: String,
    pub apple_pay: Option<serde_json::Value>,
    pub google_pay: Option<serde_json::Value>,
    pub link: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingDetails {
    pub address: Option<Address>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
}

/// Setup Intent for saving payment methods
#[derive(Debug, Clone, Deserialize)]
pub struct SetupIntent {
    pub id: String,
    pub object: String,
    pub client_secret: Option<String>,
    pub customer: Option<String>,
    pub payment_method: Option<String>,
    pub status: String,
    pub usage: String,
    pub created: i64,
}

impl StripeClient {
    // ============ Customers ============

    /// Create a customer
    pub async fn create_customer(
        &self,
        email: Option<&str>,
        name: Option<&str>,
        metadata: Option<HashMap<String, String>>,
    ) -> StripeResult<Customer> {
        let mut params = HashMap::new();

        if let Some(e) = email {
            params.insert("email".to_string(), e.to_string());
        }

        if let Some(n) = name {
            params.insert("name".to_string(), n.to_string());
        }

        if let Some(meta) = metadata {
            for (key, value) in meta {
                params.insert(format!("metadata[{}]", key), value);
            }
        }

        self.post("/customers", &params).await
    }

    /// Retrieve a customer
    pub async fn get_customer(&self, id: &str) -> StripeResult<Customer> {
        self.get(&format!("/customers/{}", id)).await
    }

    /// Update customer
    pub async fn update_customer(
        &self,
        id: &str,
        email: Option<&str>,
        name: Option<&str>,
        default_payment_method: Option<&str>,
    ) -> StripeResult<Customer> {
        let mut params = HashMap::new();

        if let Some(e) = email {
            params.insert("email".to_string(), e.to_string());
        }

        if let Some(n) = name {
            params.insert("name".to_string(), n.to_string());
        }

        if let Some(pm) = default_payment_method {
            params.insert("invoice_settings[default_payment_method]".to_string(), pm.to_string());
        }

        self.post(&format!("/customers/{}", id), &params).await
    }

    // ============ Payment Methods ============

    /// Retrieve a payment method
    pub async fn get_payment_method(&self, id: &str) -> StripeResult<PaymentMethod> {
        self.get(&format!("/payment_methods/{}", id)).await
    }

    /// List payment methods for customer
    pub async fn list_payment_methods(
        &self,
        customer_id: &str,
        method_type: Option<&str>,
    ) -> StripeResult<StripeList<PaymentMethod>> {
        let type_param = method_type.unwrap_or("card");
        self.get(&format!(
            "/payment_methods?customer={}&type={}",
            customer_id, type_param
        ))
        .await
    }

    /// Attach payment method to customer
    pub async fn attach_payment_method(
        &self,
        payment_method_id: &str,
        customer_id: &str,
    ) -> StripeResult<PaymentMethod> {
        let mut params = HashMap::new();
        params.insert("customer".to_string(), customer_id.to_string());

        self.post(&format!("/payment_methods/{}/attach", payment_method_id), &params).await
    }

    /// Detach payment method from customer
    pub async fn detach_payment_method(&self, payment_method_id: &str) -> StripeResult<PaymentMethod> {
        let params = HashMap::new();
        self.post(&format!("/payment_methods/{}/detach", payment_method_id), &params).await
    }

    // ============ Setup Intents ============

    /// Create a setup intent for saving payment methods
    pub async fn create_setup_intent(
        &self,
        customer_id: &str,
        payment_method_types: Vec<&str>,
        metadata: Option<HashMap<String, String>>,
    ) -> StripeResult<SetupIntent> {
        let mut params = HashMap::new();
        params.insert("customer".to_string(), customer_id.to_string());

        for (i, pm_type) in payment_method_types.iter().enumerate() {
            params.insert(format!("payment_method_types[{}]", i), pm_type.to_string());
        }

        if let Some(meta) = metadata {
            for (key, value) in meta {
                params.insert(format!("metadata[{}]", key), value);
            }
        }

        self.post("/setup_intents", &params).await
    }

    /// Retrieve a setup intent
    pub async fn get_setup_intent(&self, id: &str) -> StripeResult<SetupIntent> {
        self.get(&format!("/setup_intents/{}", id)).await
    }

    /// Confirm setup intent
    pub async fn confirm_setup_intent(
        &self,
        id: &str,
        payment_method: &str,
        return_url: Option<&str>,
    ) -> StripeResult<SetupIntent> {
        let mut params = HashMap::new();
        params.insert("payment_method".to_string(), payment_method.to_string());

        if let Some(url) = return_url {
            params.insert("return_url".to_string(), url.to_string());
        }

        self.post(&format!("/setup_intents/{}/confirm", id), &params).await
    }
}
