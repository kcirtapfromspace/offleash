//
//  PaymentService.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

/// Service for handling payment-related API calls
actor PaymentService {
    static let shared = PaymentService()

    private init() {}

    // MARK: - Payment Methods

    /// Get all payment methods for the user
    func getPaymentMethods() async throws -> [PaymentMethod] {
        try await APIClient.shared.get("/payment-methods")
    }

    /// Add a new payment method
    func addPaymentMethod(_ request: CreatePaymentMethodRequest) async throws -> PaymentMethod {
        try await APIClient.shared.post("/payment-methods", body: request)
    }

    /// Set a payment method as default
    func setDefaultPaymentMethod(_ id: String) async throws -> PaymentMethod {
        try await APIClient.shared.put("/payment-methods/\(id)/default", body: EmptyBody())
    }

    /// Delete a payment method
    func deletePaymentMethod(_ id: String) async throws {
        let _: EmptyResponse = try await APIClient.shared.delete("/payment-methods/\(id)")
    }

    // MARK: - Checkout

    /// Create a checkout session
    func createCheckout(_ request: CreateCheckoutRequest) async throws -> CheckoutResponse {
        try await APIClient.shared.post("/checkout", body: request)
    }

    /// Get checkout/transaction details
    func getCheckout(_ transactionId: String) async throws -> CheckoutResponse {
        try await APIClient.shared.get("/checkout/\(transactionId)")
    }

    /// Preview fees before checkout
    func previewFees(_ request: FeePreviewRequest) async throws -> FeePreviewResponse {
        try await APIClient.shared.post("/checkout/preview-fees", body: request)
    }

    /// Confirm a payment
    func confirmPayment(_ transactionId: String, paymentMethodId: String?) async throws -> CheckoutResponse {
        let request = ConfirmPaymentRequest(paymentMethodId: paymentMethodId)
        return try await APIClient.shared.post("/checkout/\(transactionId)/confirm", body: request)
    }

    /// Request a refund
    func requestRefund(_ transactionId: String, amountCents: Int? = nil, reason: String? = nil) async throws -> RefundResponse {
        let request = RefundRequest(amountCents: amountCents, reason: reason)
        return try await APIClient.shared.post("/checkout/\(transactionId)/refund", body: request)
    }

    // MARK: - Transactions

    /// Get user's transactions
    func getTransactions() async throws -> [TransactionListItem] {
        try await APIClient.shared.get("/transactions")
    }

    /// Get a specific transaction
    func getTransaction(_ id: String) async throws -> Transaction {
        try await APIClient.shared.get("/transactions/\(id)")
    }

    // MARK: - Customer Subscriptions

    /// Get user's subscriptions
    func getSubscriptions() async throws -> [CustomerSubscription] {
        try await APIClient.shared.get("/subscriptions/customer")
    }

    /// Get a specific subscription
    func getSubscription(_ id: String) async throws -> CustomerSubscription {
        try await APIClient.shared.get("/subscriptions/customer/\(id)")
    }

    /// Create a customer subscription
    func createSubscription(_ request: CreateCustomerSubscriptionRequest) async throws -> CustomerSubscription {
        try await APIClient.shared.post("/subscriptions/customer", body: request)
    }

    /// Cancel a subscription
    func cancelSubscription(_ id: String) async throws -> CustomerSubscription {
        try await APIClient.shared.delete("/subscriptions/customer/\(id)")
    }

    // MARK: - Fee Tiers (for businesses)

    /// Get available fee tiers/plans
    func getFeeTiers() async throws -> [FeeTier] {
        try await APIClient.shared.get("/subscriptions/tiers")
    }

    /// Get tenant subscription
    func getTenantSubscription() async throws -> TenantSubscription? {
        try await APIClient.shared.get("/subscriptions/tenant")
    }

    /// Create/upgrade tenant subscription
    func createTenantSubscription(_ request: CreateSubscriptionRequest) async throws -> TenantSubscription {
        try await APIClient.shared.post("/subscriptions/tenant", body: request)
    }

    /// Cancel tenant subscription
    func cancelTenantSubscription() async throws -> TenantSubscription {
        try await APIClient.shared.delete("/subscriptions/tenant")
    }
}

// MARK: - Helper Types

// EmptyResponse is defined in APIClient.swift

// Extend PaymentMethod with creation support
extension CreatePaymentMethodRequest {
    /// Create a request for Apple Pay
    static func applePay(token: String) -> CreatePaymentMethodRequest {
        CreatePaymentMethodRequest(
            methodType: "apple_pay",
            cardNonce: token,
            cardLastFour: nil,
            cardBrand: nil,
            cardExpMonth: nil,
            cardExpYear: nil,
            nickname: "Apple Pay",
            isDefault: true
        )
    }

    /// Create a request for a card (from Stripe/Square tokenization)
    static func card(
        nonce: String,
        lastFour: String,
        brand: String,
        expMonth: Int,
        expYear: Int,
        nickname: String? = nil,
        isDefault: Bool = false
    ) -> CreatePaymentMethodRequest {
        CreatePaymentMethodRequest(
            methodType: "card",
            cardNonce: nonce,
            cardLastFour: lastFour,
            cardBrand: brand,
            cardExpMonth: expMonth,
            cardExpYear: expYear,
            nickname: nickname,
            isDefault: isDefault
        )
    }
}
