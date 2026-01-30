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

    private var mockPaymentMethods: [PaymentMethod] = [
        PaymentMethod(
            id: "pm_mock_1",
            methodType: "card",
            displayName: "Visa •••• 4242",
            cardLastFour: "4242",
            cardBrand: "visa",
            cardExpMonth: 12,
            cardExpYear: 2029,
            nickname: "Test Card",
            isDefault: true,
            isExpired: false,
            createdAt: "2025-01-01T00:00:00Z"
        )
    ]
    private var mockTransactions: [TransactionListItem] = [
        TransactionListItem(
            id: "txn_mock_1",
            bookingId: "booking_mock_1",
            status: "succeeded",
            subtotalCents: 2500,
            totalCents: 2900,
            createdAt: "2025-01-02T12:00:00Z",
            isCustomer: true
        )
    ]
    private var mockSubscriptions: [CustomerSubscription] = [
        CustomerSubscription(
            id: "sub_mock_1",
            serviceId: "mock-service",
            name: "Weekly Walks",
            description: "Weekly 30-minute walk package",
            status: "active",
            priceCents: 9900,
            interval: "week",
            intervalCount: 1,
            currentPeriodStart: "2025-01-01T00:00:00Z",
            currentPeriodEnd: "2025-02-01T00:00:00Z",
            cancelAtPeriodEnd: false,
            autoCreateBookings: true,
            createdAt: "2025-01-01T00:00:00Z"
        )
    ]

    private init() {}

    // MARK: - Payment Methods

    /// Get all payment methods for the user
    func getPaymentMethods() async throws -> [PaymentMethod] {
        if TestAuthMode.isMock {
            return mockPaymentMethods
        }
        return try await APIClient.shared.get("/payment-methods")
    }

    /// Add a new payment method
    func addPaymentMethod(_ request: CreatePaymentMethodRequest) async throws -> PaymentMethod {
        if TestAuthMode.isMock {
            let methodId = "pm_mock_\(UUID().uuidString.prefix(8))"
            let brand = request.cardBrand ?? (request.methodType == "apple_pay" ? "apple" : "card")
            let lastFour = request.cardLastFour ?? "0000"
            let displayName = request.methodType == "apple_pay"
                ? "Apple Pay"
                : "\(brand.capitalized) •••• \(lastFour)"
            let newMethod = PaymentMethod(
                id: methodId,
                methodType: request.methodType,
                displayName: displayName,
                cardLastFour: request.cardLastFour,
                cardBrand: request.cardBrand,
                cardExpMonth: request.cardExpMonth,
                cardExpYear: request.cardExpYear,
                nickname: request.nickname,
                isDefault: request.isDefault ?? false,
                isExpired: false,
                createdAt: ISO8601DateFormatter().string(from: Date())
            )
            if newMethod.isDefault {
                mockPaymentMethods = mockPaymentMethods.map { method in
                    PaymentMethod(
                        id: method.id,
                        methodType: method.methodType,
                        displayName: method.displayName,
                        cardLastFour: method.cardLastFour,
                        cardBrand: method.cardBrand,
                        cardExpMonth: method.cardExpMonth,
                        cardExpYear: method.cardExpYear,
                        nickname: method.nickname,
                        isDefault: false,
                        isExpired: method.isExpired,
                        createdAt: method.createdAt
                    )
                }
            }
            mockPaymentMethods.append(newMethod)
            return newMethod
        }
        return try await APIClient.shared.post("/payment-methods", body: request)
    }

    /// Set a payment method as default
    func setDefaultPaymentMethod(_ id: String) async throws -> PaymentMethod {
        if TestAuthMode.isMock {
            guard let method = mockPaymentMethods.first(where: { $0.id == id }) else {
                throw APIError.noData
            }
            mockPaymentMethods = mockPaymentMethods.map { entry in
                PaymentMethod(
                    id: entry.id,
                    methodType: entry.methodType,
                    displayName: entry.displayName,
                    cardLastFour: entry.cardLastFour,
                    cardBrand: entry.cardBrand,
                    cardExpMonth: entry.cardExpMonth,
                    cardExpYear: entry.cardExpYear,
                    nickname: entry.nickname,
                    isDefault: entry.id == id,
                    isExpired: entry.isExpired,
                    createdAt: entry.createdAt
                )
            }
            return PaymentMethod(
                id: method.id,
                methodType: method.methodType,
                displayName: method.displayName,
                cardLastFour: method.cardLastFour,
                cardBrand: method.cardBrand,
                cardExpMonth: method.cardExpMonth,
                cardExpYear: method.cardExpYear,
                nickname: method.nickname,
                isDefault: true,
                isExpired: method.isExpired,
                createdAt: method.createdAt
            )
        }
        return try await APIClient.shared.put("/payment-methods/\(id)/default", body: EmptyBody())
    }

    /// Delete a payment method
    func deletePaymentMethod(_ id: String) async throws {
        if TestAuthMode.isMock {
            mockPaymentMethods.removeAll { $0.id == id }
            if mockPaymentMethods.allSatisfy({ !$0.isDefault }), let first = mockPaymentMethods.first {
                mockPaymentMethods = mockPaymentMethods.map { entry in
                    PaymentMethod(
                        id: entry.id,
                        methodType: entry.methodType,
                        displayName: entry.displayName,
                        cardLastFour: entry.cardLastFour,
                        cardBrand: entry.cardBrand,
                        cardExpMonth: entry.cardExpMonth,
                        cardExpYear: entry.cardExpYear,
                        nickname: entry.nickname,
                        isDefault: entry.id == first.id,
                        isExpired: entry.isExpired,
                        createdAt: entry.createdAt
                    )
                }
            }
            return
        }
        let _: EmptyResponse = try await APIClient.shared.delete("/payment-methods/\(id)")
    }

    // MARK: - Checkout

    /// Create a checkout session
    func createCheckout(_ request: CreateCheckoutRequest) async throws -> CheckoutResponse {
        if TestAuthMode.isMock {
            return CheckoutResponse(
                transactionId: "txn_mock_checkout",
                status: "succeeded",
                subtotalCents: request.subtotalCents,
                customerFeeCents: 400,
                taxCents: 0,
                totalCents: request.subtotalCents + 400,
                providerType: "mock",
                clientSecret: nil,
                paymentId: "pay_mock"
            )
        }
        return try await APIClient.shared.post("/checkout", body: request)
    }

    /// Get checkout/transaction details
    func getCheckout(_ transactionId: String) async throws -> CheckoutResponse {
        if TestAuthMode.isMock {
            return CheckoutResponse(
                transactionId: transactionId,
                status: "succeeded",
                subtotalCents: 2500,
                customerFeeCents: 400,
                taxCents: 0,
                totalCents: 2900,
                providerType: "mock",
                clientSecret: nil,
                paymentId: "pay_mock"
            )
        }
        return try await APIClient.shared.get("/checkout/\(transactionId)")
    }

    /// Preview fees before checkout
    func previewFees(_ request: FeePreviewRequest) async throws -> FeePreviewResponse {
        if TestAuthMode.isMock {
            return FeePreviewResponse(
                subtotalCents: request.subtotalCents,
                tipCents: request.tipCents ?? 0,
                customerFeeCents: 400,
                taxCents: 0,
                totalCents: request.subtotalCents + 400 + (request.tipCents ?? 0),
                customerFeePercent: 0.1,
                taxRatePercent: 0
            )
        }
        return try await APIClient.shared.post("/checkout/preview-fees", body: request)
    }

    /// Confirm a payment
    func confirmPayment(_ transactionId: String, paymentMethodId: String?) async throws -> CheckoutResponse {
        if TestAuthMode.isMock {
            return CheckoutResponse(
                transactionId: transactionId,
                status: "succeeded",
                subtotalCents: 2500,
                customerFeeCents: 400,
                taxCents: 0,
                totalCents: 2900,
                providerType: "mock",
                clientSecret: nil,
                paymentId: "pay_mock"
            )
        }
        let request = ConfirmPaymentRequest(paymentMethodId: paymentMethodId)
        return try await APIClient.shared.post("/checkout/\(transactionId)/confirm", body: request)
    }

    /// Request a refund
    func requestRefund(_ transactionId: String, amountCents: Int? = nil, reason: String? = nil) async throws -> RefundResponse {
        if TestAuthMode.isMock {
            return RefundResponse(
                success: true,
                refundId: "refund_mock",
                refundAmountCents: amountCents ?? 0,
                status: "succeeded"
            )
        }
        let request = RefundRequest(amountCents: amountCents, reason: reason)
        return try await APIClient.shared.post("/checkout/\(transactionId)/refund", body: request)
    }

    // MARK: - Transactions

    /// Get user's transactions
    func getTransactions() async throws -> [TransactionListItem] {
        if TestAuthMode.isMock {
            return mockTransactions
        }
        return try await APIClient.shared.get("/transactions")
    }

    /// Get a specific transaction
    func getTransaction(_ id: String) async throws -> Transaction {
        if TestAuthMode.isMock {
            return Transaction(
                id: id,
                bookingId: "booking_mock_1",
                status: .succeeded,
                subtotalCents: 2500,
                customerFeeCents: 400,
                taxCents: 0,
                totalCents: 2900,
                providerPayoutCents: 2000,
                tipCents: 0,
                createdAt: "2025-01-02T12:00:00Z",
                completedAt: "2025-01-02T12:05:00Z",
                isCustomer: true
            )
        }
        return try await APIClient.shared.get("/transactions/\(id)")
    }

    // MARK: - Customer Subscriptions

    /// Get user's subscriptions
    func getSubscriptions() async throws -> [CustomerSubscription] {
        if TestAuthMode.isMock {
            return mockSubscriptions
        }
        return try await APIClient.shared.get("/subscriptions/customer")
    }

    /// Get a specific subscription
    func getSubscription(_ id: String) async throws -> CustomerSubscription {
        if TestAuthMode.isMock {
            guard let sub = mockSubscriptions.first(where: { $0.id == id }) else {
                throw APIError.noData
            }
            return sub
        }
        return try await APIClient.shared.get("/subscriptions/customer/\(id)")
    }

    /// Create a customer subscription
    func createSubscription(_ request: CreateCustomerSubscriptionRequest) async throws -> CustomerSubscription {
        if TestAuthMode.isMock {
            let newSub = CustomerSubscription(
                id: "sub_mock_\(UUID().uuidString.prefix(8))",
                serviceId: request.serviceId,
                name: "Mock Subscription",
                description: "Mocked subscription for tests",
                status: "active",
                priceCents: 9900,
                interval: "month",
                intervalCount: 1,
                currentPeriodStart: "2025-01-01T00:00:00Z",
                currentPeriodEnd: "2025-02-01T00:00:00Z",
                cancelAtPeriodEnd: false,
                autoCreateBookings: true,
                createdAt: ISO8601DateFormatter().string(from: Date())
            )
            mockSubscriptions.append(newSub)
            return newSub
        }
        return try await APIClient.shared.post("/subscriptions/customer", body: request)
    }

    /// Cancel a subscription
    func cancelSubscription(_ id: String) async throws -> CustomerSubscription {
        if TestAuthMode.isMock {
            guard let sub = mockSubscriptions.first(where: { $0.id == id }) else {
                throw APIError.noData
            }
            let canceled = CustomerSubscription(
                id: sub.id,
                serviceId: sub.serviceId,
                name: sub.name,
                description: sub.description,
                status: "canceled",
                priceCents: sub.priceCents,
                interval: sub.interval,
                intervalCount: sub.intervalCount,
                currentPeriodStart: sub.currentPeriodStart,
                currentPeriodEnd: sub.currentPeriodEnd,
                cancelAtPeriodEnd: true,
                autoCreateBookings: sub.autoCreateBookings,
                createdAt: sub.createdAt
            )
            mockSubscriptions = mockSubscriptions.map { $0.id == id ? canceled : $0 }
            return canceled
        }
        return try await APIClient.shared.delete("/subscriptions/customer/\(id)")
    }

    // MARK: - Fee Tiers (for businesses)

    /// Get available fee tiers/plans
    func getFeeTiers() async throws -> [FeeTier] {
        return try await APIClient.shared.get("/subscriptions/tiers")
    }

    /// Get tenant subscription
    func getTenantSubscription() async throws -> TenantSubscription? {
        return try await APIClient.shared.get("/subscriptions/tenant")
    }

    /// Create/upgrade tenant subscription
    func createTenantSubscription(_ request: CreateSubscriptionRequest) async throws -> TenantSubscription {
        return try await APIClient.shared.post("/subscriptions/tenant", body: request)
    }

    /// Cancel tenant subscription
    func cancelTenantSubscription() async throws -> TenantSubscription {
        return try await APIClient.shared.delete("/subscriptions/tenant")
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
