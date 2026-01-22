use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::types::{OrganizationId, UserId};
use sqlx::FromRow;
use uuid::Uuid;

/// Payment method type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_method_type", rename_all = "snake_case")]
pub enum PaymentMethodType {
    Card,
    ApplePay,
    BankAccount,
}

impl std::fmt::Display for PaymentMethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethodType::Card => write!(f, "card"),
            PaymentMethodType::ApplePay => write!(f, "apple_pay"),
            PaymentMethodType::BankAccount => write!(f, "bank_account"),
        }
    }
}

/// Card brand for display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "card_brand", rename_all = "snake_case")]
pub enum CardBrand {
    Visa,
    Mastercard,
    Amex,
    Discover,
    Other,
}

impl std::fmt::Display for CardBrand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardBrand::Visa => write!(f, "Visa"),
            CardBrand::Mastercard => write!(f, "Mastercard"),
            CardBrand::Amex => write!(f, "American Express"),
            CardBrand::Discover => write!(f, "Discover"),
            CardBrand::Other => write!(f, "Card"),
        }
    }
}

/// Customer payment method database model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CustomerPaymentMethod {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub customer_id: UserId,
    pub method_type: PaymentMethodType,
    pub card_last_four: Option<String>,
    pub card_brand: Option<CardBrand>,
    pub card_exp_month: Option<i32>,
    pub card_exp_year: Option<i32>,
    pub square_card_id: Option<String>,
    pub nickname: Option<String>,
    pub is_default: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CustomerPaymentMethod {
    /// Get a display-friendly name for this payment method
    pub fn display_name(&self) -> String {
        if let Some(nickname) = &self.nickname {
            return nickname.clone();
        }

        match self.method_type {
            PaymentMethodType::ApplePay => "Apple Pay".to_string(),
            PaymentMethodType::BankAccount => "Bank Account".to_string(),
            PaymentMethodType::Card => {
                if let (Some(brand), Some(last_four)) = (&self.card_brand, &self.card_last_four) {
                    format!("{} ****{}", brand, last_four)
                } else {
                    "Card".to_string()
                }
            }
        }
    }

    /// Check if card is expired
    pub fn is_expired(&self) -> bool {
        if self.method_type != PaymentMethodType::Card {
            return false;
        }

        if let (Some(exp_month), Some(exp_year)) = (self.card_exp_month, self.card_exp_year) {
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
}

/// Input for creating a new payment method
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCustomerPaymentMethod {
    pub method_type: PaymentMethodType,
    pub card_last_four: Option<String>,
    pub card_brand: Option<CardBrand>,
    pub card_exp_month: Option<i32>,
    pub card_exp_year: Option<i32>,
    pub square_card_id: Option<String>,
    pub nickname: Option<String>,
    pub is_default: bool,
}
