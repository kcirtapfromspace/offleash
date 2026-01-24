use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

use super::PaymentProviderType;

/// Payment method type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_method_type", rename_all = "snake_case")]
pub enum PaymentMethodType {
    Card,
    ApplePay,
    GooglePay,
    ShopPay,
    Link,
    BankAccount,
}

impl std::fmt::Display for PaymentMethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethodType::Card => write!(f, "card"),
            PaymentMethodType::ApplePay => write!(f, "apple_pay"),
            PaymentMethodType::GooglePay => write!(f, "google_pay"),
            PaymentMethodType::ShopPay => write!(f, "shop_pay"),
            PaymentMethodType::Link => write!(f, "link"),
            PaymentMethodType::BankAccount => write!(f, "bank_account"),
        }
    }
}

/// Customer payment method database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CustomerPaymentMethod {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub provider_type: PaymentProviderType,
    pub method_type: PaymentMethodType,
    // Provider references
    pub stripe_payment_method_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub square_card_id: Option<String>,
    pub square_customer_id: Option<String>,
    // Card details (non-sensitive)
    pub last_four: Option<String>,
    pub brand: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub cardholder_name: Option<String>,
    // Bank account
    pub bank_name: Option<String>,
    pub account_last_four: Option<String>,
    // Wallet info
    pub wallet_type: Option<String>,
    // Status
    pub is_default: bool,
    pub is_active: bool,
    // Metadata
    pub billing_address: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CustomerPaymentMethod {
    /// Get a display-friendly name for this payment method
    pub fn display_name(&self) -> String {
        match self.method_type {
            PaymentMethodType::ApplePay => "Apple Pay".to_string(),
            PaymentMethodType::GooglePay => "Google Pay".to_string(),
            PaymentMethodType::ShopPay => "Shop Pay".to_string(),
            PaymentMethodType::Link => "Link".to_string(),
            PaymentMethodType::BankAccount => {
                if let (Some(bank), Some(last_four)) = (&self.bank_name, &self.account_last_four) {
                    format!("{} ****{}", bank, last_four)
                } else {
                    "Bank Account".to_string()
                }
            }
            PaymentMethodType::Card => {
                if let (Some(brand), Some(last_four)) = (&self.brand, &self.last_four) {
                    format!("{} ****{}", Self::format_brand(brand), last_four)
                } else {
                    "Card".to_string()
                }
            }
        }
    }

    /// Format card brand for display
    fn format_brand(brand: &str) -> &str {
        match brand.to_lowercase().as_str() {
            "visa" => "Visa",
            "mastercard" => "Mastercard",
            "amex" | "american_express" => "Amex",
            "discover" => "Discover",
            "diners" | "diners_club" => "Diners",
            "jcb" => "JCB",
            "unionpay" => "UnionPay",
            _ => brand,
        }
    }

    /// Check if card is expired
    pub fn is_expired(&self) -> bool {
        if self.method_type != PaymentMethodType::Card {
            return false;
        }

        if let (Some(exp_month), Some(exp_year)) = (self.exp_month, self.exp_year) {
            let now = Utc::now();
            let current_year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2024);
            let current_month = now.format("%m").to_string().parse::<i32>().unwrap_or(1);

            if exp_year < current_year {
                return true;
            }
            if exp_year == current_year && exp_month < current_month {
                return true;
            }
        }
        false
    }

    /// Get expiry date display
    pub fn expiry_display(&self) -> Option<String> {
        if let (Some(month), Some(year)) = (self.exp_month, self.exp_year) {
            Some(format!("{:02}/{}", month, year % 100))
        } else {
            None
        }
    }

    /// Get icon name for the payment method
    pub fn icon(&self) -> &'static str {
        match self.method_type {
            PaymentMethodType::ApplePay => "apple-pay",
            PaymentMethodType::GooglePay => "google-pay",
            PaymentMethodType::ShopPay => "shop-pay",
            PaymentMethodType::Link => "link",
            PaymentMethodType::BankAccount => "bank",
            PaymentMethodType::Card => {
                match self.brand.as_deref().unwrap_or("").to_lowercase().as_str() {
                    "visa" => "visa",
                    "mastercard" => "mastercard",
                    "amex" | "american_express" => "amex",
                    "discover" => "discover",
                    _ => "card",
                }
            }
        }
    }
}

/// Input for creating a new payment method
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCustomerPaymentMethod {
    pub provider_type: PaymentProviderType,
    pub method_type: PaymentMethodType,
    pub stripe_payment_method_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub square_card_id: Option<String>,
    pub square_customer_id: Option<String>,
    pub last_four: Option<String>,
    pub brand: Option<String>,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
    pub cardholder_name: Option<String>,
    pub bank_name: Option<String>,
    pub account_last_four: Option<String>,
    pub wallet_type: Option<String>,
    pub is_default: bool,
    pub billing_address: Option<serde_json::Value>,
}

/// Input for updating a payment method
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateCustomerPaymentMethod {
    pub is_default: Option<bool>,
    pub is_active: Option<bool>,
    pub billing_address: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Response for client-side payment method display
#[derive(Debug, Clone, Serialize)]
pub struct PaymentMethodResponse {
    pub id: String,
    pub method_type: String,
    pub display_name: String,
    pub icon: String,
    pub last_four: Option<String>,
    pub brand: Option<String>,
    pub expiry: Option<String>,
    pub is_default: bool,
    pub is_expired: bool,
}

impl From<CustomerPaymentMethod> for PaymentMethodResponse {
    fn from(pm: CustomerPaymentMethod) -> Self {
        Self {
            id: pm.id.to_string(),
            method_type: pm.method_type.to_string(),
            display_name: pm.display_name(),
            icon: pm.icon().to_string(),
            last_four: pm.last_four.clone(),
            brand: pm.brand.clone(),
            expiry: pm.expiry_display(),
            is_default: pm.is_default,
            is_expired: pm.is_expired(),
        }
    }
}
