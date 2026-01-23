use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::error::{TaxError, TaxResult};

const TAXJAR_API_BASE: &str = "https://api.taxjar.com/v2";
const TAXJAR_SANDBOX_API_BASE: &str = "https://api.sandbox.taxjar.com/v2";

/// TaxJar API client
#[derive(Clone)]
pub struct TaxJarClient {
    client: Client,
    api_key: String,
    sandbox: bool,
}

/// Address for tax calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: String,
    pub zip: String,
    pub country: String,
}

impl Address {
    pub fn us(state: &str, zip: &str) -> Self {
        Self {
            street: None,
            city: None,
            state: state.to_string(),
            zip: zip.to_string(),
            country: "US".to_string(),
        }
    }

    pub fn full(street: &str, city: &str, state: &str, zip: &str, country: &str) -> Self {
        Self {
            street: Some(street.to_string()),
            city: Some(city.to_string()),
            state: state.to_string(),
            zip: zip.to_string(),
            country: country.to_string(),
        }
    }
}

/// Tax category for services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxCategory {
    /// General services (default)
    Services,
    /// Pet services - may have specific rules
    PetServices,
    /// Digital services
    Digital,
    /// Physical goods
    Goods,
}

impl Default for TaxCategory {
    fn default() -> Self {
        Self::Services
    }
}

impl TaxCategory {
    /// Get TaxJar product tax code
    pub fn tax_code(&self) -> &str {
        match self {
            TaxCategory::Services => "19000",      // General services
            TaxCategory::PetServices => "19000",   // Pet services (general services)
            TaxCategory::Digital => "31000",       // Digital goods
            TaxCategory::Goods => "00000",         // General goods
        }
    }
}

/// Line item for tax calculation
#[derive(Debug, Clone, Serialize)]
pub struct TaxLineItem {
    pub id: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub product_tax_code: Option<String>,
    pub description: Option<String>,
}

impl TaxLineItem {
    pub fn new(id: &str, quantity: i32, unit_price_cents: i64) -> Self {
        Self {
            id: id.to_string(),
            quantity,
            unit_price: unit_price_cents as f64 / 100.0,
            product_tax_code: None,
            description: None,
        }
    }

    pub fn with_tax_code(mut self, category: TaxCategory) -> Self {
        self.product_tax_code = Some(category.tax_code().to_string());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
}

/// Tax rate for a location
#[derive(Debug, Clone, Deserialize)]
pub struct TaxRate {
    pub zip: String,
    pub state: String,
    pub state_rate: f64,
    pub county: Option<String>,
    pub county_rate: f64,
    pub city: Option<String>,
    pub city_rate: f64,
    pub combined_district_rate: f64,
    pub combined_rate: f64,
    pub freight_taxable: bool,
}

impl TaxRate {
    /// Get the total tax rate as a percentage (e.g., 8.25 for 8.25%)
    pub fn total_rate_percent(&self) -> f64 {
        self.combined_rate * 100.0
    }

    /// Calculate tax amount for a given subtotal in cents
    pub fn calculate_tax_cents(&self, subtotal_cents: i64) -> i64 {
        let tax = (subtotal_cents as f64 * self.combined_rate).round() as i64;
        tax
    }
}

/// Tax calculation result
#[derive(Debug, Clone, Deserialize)]
pub struct TaxCalculation {
    pub order_total_amount: f64,
    pub shipping: f64,
    pub taxable_amount: f64,
    pub amount_to_collect: f64,
    pub rate: f64,
    pub has_nexus: bool,
    pub freight_taxable: bool,
    pub tax_source: Option<String>,
    pub jurisdictions: Option<TaxJurisdictions>,
    pub breakdown: Option<TaxBreakdown>,
}

impl TaxCalculation {
    /// Get tax amount in cents
    pub fn tax_cents(&self) -> i64 {
        (self.amount_to_collect * 100.0).round() as i64
    }

    /// Get the effective tax rate as a percentage
    pub fn effective_rate_percent(&self) -> f64 {
        self.rate * 100.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaxJurisdictions {
    pub country: String,
    pub state: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaxBreakdown {
    pub taxable_amount: f64,
    pub tax_collectable: f64,
    pub combined_tax_rate: f64,
    pub state_taxable_amount: Option<f64>,
    pub state_tax_rate: Option<f64>,
    pub state_tax_collectable: Option<f64>,
    pub county_taxable_amount: Option<f64>,
    pub county_tax_rate: Option<f64>,
    pub county_tax_collectable: Option<f64>,
    pub city_taxable_amount: Option<f64>,
    pub city_tax_rate: Option<f64>,
    pub city_tax_collectable: Option<f64>,
    pub special_district_taxable_amount: Option<f64>,
    pub special_tax_rate: Option<f64>,
    pub special_district_tax_collectable: Option<f64>,
}

#[derive(Debug, Serialize)]
struct TaxCalculationRequest {
    from_country: String,
    from_zip: String,
    from_state: String,
    to_country: String,
    to_zip: String,
    to_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to_street: Option<String>,
    amount: f64,
    shipping: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_items: Option<Vec<TaxLineItem>>,
}

#[derive(Debug, Deserialize)]
struct TaxRateResponse {
    rate: TaxRate,
}

#[derive(Debug, Deserialize)]
struct TaxCalculationResponse {
    tax: TaxCalculation,
}

#[derive(Debug, Deserialize)]
struct TaxJarErrorResponse {
    error: String,
    detail: String,
    status: u16,
}

impl TaxJarClient {
    /// Create a new TaxJar client
    pub fn new(api_key: String, sandbox: bool) -> Self {
        Self {
            client: Client::new(),
            api_key,
            sandbox,
        }
    }

    fn base_url(&self) -> &str {
        if self.sandbox {
            TAXJAR_SANDBOX_API_BASE
        } else {
            TAXJAR_API_BASE
        }
    }

    /// Get tax rate for a location
    pub async fn get_rate(&self, zip: &str, params: Option<&Address>) -> TaxResult<TaxRate> {
        let mut url = format!("{}/rates/{}", self.base_url(), zip);

        // Add query parameters if address provided
        if let Some(addr) = params {
            let mut query_parts = vec![];
            if !addr.country.is_empty() {
                query_parts.push(format!("country={}", addr.country));
            }
            if !addr.state.is_empty() {
                query_parts.push(format!("state={}", addr.state));
            }
            if let Some(city) = &addr.city {
                query_parts.push(format!("city={}", city));
            }
            if let Some(street) = &addr.street {
                query_parts.push(format!("street={}", street));
            }
            if !query_parts.is_empty() {
                url = format!("{}?{}", url, query_parts.join("&"));
            }
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response::<TaxRateResponse>(response)
            .await
            .map(|r| r.rate)
    }

    /// Calculate tax for an order
    pub async fn calculate_tax(
        &self,
        from_address: &Address,
        to_address: &Address,
        amount_cents: i64,
        shipping_cents: i64,
        line_items: Option<Vec<TaxLineItem>>,
    ) -> TaxResult<TaxCalculation> {
        let request = TaxCalculationRequest {
            from_country: from_address.country.clone(),
            from_zip: from_address.zip.clone(),
            from_state: from_address.state.clone(),
            to_country: to_address.country.clone(),
            to_zip: to_address.zip.clone(),
            to_state: to_address.state.clone(),
            to_city: to_address.city.clone(),
            to_street: to_address.street.clone(),
            amount: amount_cents as f64 / 100.0,
            shipping: shipping_cents as f64 / 100.0,
            line_items,
        };

        let url = format!("{}/taxes", self.base_url());

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        self.handle_response::<TaxCalculationResponse>(response)
            .await
            .map(|r| r.tax)
    }

    /// Simple tax calculation for a single service at a location
    pub async fn calculate_service_tax(
        &self,
        provider_state: &str,
        provider_zip: &str,
        customer_state: &str,
        customer_zip: &str,
        amount_cents: i64,
    ) -> TaxResult<TaxCalculation> {
        let from = Address::us(provider_state, provider_zip);
        let to = Address::us(customer_state, customer_zip);

        let line_item = TaxLineItem::new("service", 1, amount_cents)
            .with_tax_code(TaxCategory::PetServices);

        self.calculate_tax(&from, &to, amount_cents, 0, Some(vec![line_item])).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> TaxResult<T> {
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).map_err(|e| TaxError::ParseError(e.to_string()))
        } else {
            // Try to parse TaxJar error response
            if let Ok(error_response) = serde_json::from_str::<TaxJarErrorResponse>(&body) {
                return Err(TaxError::ApiError {
                    status: error_response.status,
                    message: format!("{}: {}", error_response.error, error_response.detail),
                });
            }

            Err(TaxError::ApiError {
                status: status.as_u16(),
                message: body,
            })
        }
    }
}

/// Simplified tax calculation without external API (for fallback/testing)
pub struct SimpleTaxCalculator;

impl SimpleTaxCalculator {
    /// Calculate tax using a simple flat rate (fallback when API unavailable)
    /// This should only be used as a fallback
    pub fn calculate_with_rate(subtotal_cents: i64, rate_percent: f64) -> i64 {
        ((subtotal_cents as f64) * (rate_percent / 100.0)).round() as i64
    }

    /// Get default tax rate for a state (simplified - for fallback only)
    pub fn default_state_rate(state: &str) -> f64 {
        match state.to_uppercase().as_str() {
            "CA" => 7.25,
            "TX" => 6.25,
            "FL" => 6.00,
            "NY" => 4.00,  // State only, cities add more
            "WA" => 6.50,
            "OR" | "MT" | "NH" | "DE" => 0.0, // No sales tax
            _ => 6.0, // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tax_line_item() {
        let item = TaxLineItem::new("walk-001", 1, 2500)
            .with_tax_code(TaxCategory::PetServices)
            .with_description("Dog walking service");

        assert_eq!(item.id, "walk-001");
        assert_eq!(item.quantity, 1);
        assert_eq!(item.unit_price, 25.0);
        assert_eq!(item.product_tax_code, Some("19000".to_string()));
    }

    #[test]
    fn test_simple_tax_calculator() {
        // $25.00 at 8.25% = $2.06
        let tax = SimpleTaxCalculator::calculate_with_rate(2500, 8.25);
        assert_eq!(tax, 206);

        // $100.00 at 7.25% = $7.25
        let tax = SimpleTaxCalculator::calculate_with_rate(10000, 7.25);
        assert_eq!(tax, 725);
    }

    #[test]
    fn test_address() {
        let addr = Address::us("CA", "90210");
        assert_eq!(addr.state, "CA");
        assert_eq!(addr.zip, "90210");
        assert_eq!(addr.country, "US");
    }
}
