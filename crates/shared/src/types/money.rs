use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// Currency enum (USD only for MVP)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    #[default]
    USD,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::USD => write!(f, "USD"),
        }
    }
}

/// Money amount stored in cents to avoid floating point issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    cents: i64,
    currency: Currency,
}

impl Money {
    /// Create money from cents
    pub fn from_cents(cents: i64) -> Self {
        Self {
            cents,
            currency: Currency::USD,
        }
    }

    /// Create money from dollars (converts to cents)
    pub fn from_dollars(dollars: f64) -> Self {
        Self {
            cents: (dollars * 100.0).round() as i64,
            currency: Currency::USD,
        }
    }

    /// Get the amount in cents
    pub fn cents(&self) -> i64 {
        self.cents
    }

    /// Get the amount in dollars
    pub fn dollars(&self) -> f64 {
        self.cents as f64 / 100.0
    }

    /// Get the currency
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Check if the amount is zero
    pub fn is_zero(&self) -> bool {
        self.cents == 0
    }

    /// Check if the amount is positive
    pub fn is_positive(&self) -> bool {
        self.cents > 0
    }

    /// Zero amount
    pub fn zero() -> Self {
        Self::from_cents(0)
    }
}

impl Default for Money {
    fn default() -> Self {
        Self::zero()
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(
            self.currency, other.currency,
            "Cannot add different currencies"
        );
        Self {
            cents: self.cents + other.cents,
            currency: self.currency,
        }
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert_eq!(
            self.currency, other.currency,
            "Cannot subtract different currencies"
        );
        Self {
            cents: self.cents - other.cents,
            currency: self.currency,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.2}", self.dollars())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_from_cents() {
        let money = Money::from_cents(5000);
        assert_eq!(money.cents(), 5000);
        assert_eq!(money.dollars(), 50.0);
    }

    #[test]
    fn test_money_from_dollars() {
        let money = Money::from_dollars(50.0);
        assert_eq!(money.cents(), 5000);
        assert_eq!(money.dollars(), 50.0);
    }

    #[test]
    fn test_money_add() {
        let a = Money::from_cents(1000);
        let b = Money::from_cents(500);
        let sum = a + b;
        assert_eq!(sum.cents(), 1500);
    }

    #[test]
    fn test_money_sub() {
        let a = Money::from_cents(1000);
        let b = Money::from_cents(400);
        let diff = a - b;
        assert_eq!(diff.cents(), 600);
    }

    #[test]
    fn test_money_display() {
        let money = Money::from_cents(5099);
        assert_eq!(money.to_string(), "$50.99");
    }

    #[test]
    fn test_money_rounding() {
        let money = Money::from_dollars(10.999);
        assert_eq!(money.cents(), 1100); // Rounds to nearest cent
    }
}
