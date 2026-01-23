//
//  Checkout.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

/// Checkout response from API
struct CheckoutResponse: Codable {
    let transactionId: String
    let status: String
    let subtotalCents: Int
    let customerFeeCents: Int
    let taxCents: Int
    let totalCents: Int
    let providerType: String
    let clientSecret: String?  // For Stripe
    let paymentId: String?     // For Square

    var subtotalDollars: Double {
        Double(subtotalCents) / 100.0
    }

    var totalDollars: Double {
        Double(totalCents) / 100.0
    }

    var customerFeeDollars: Double {
        Double(customerFeeCents) / 100.0
    }

    var taxDollars: Double {
        Double(taxCents) / 100.0
    }

    var formattedTotal: String {
        String(format: "$%.2f", totalDollars)
    }

    var formattedSubtotal: String {
        String(format: "$%.2f", subtotalDollars)
    }

    var formattedFee: String {
        String(format: "$%.2f", customerFeeDollars)
    }

    var formattedTax: String {
        String(format: "$%.2f", taxDollars)
    }
}

/// Request to create a checkout
struct CreateCheckoutRequest: Codable {
    let bookingId: String
    let paymentMethodId: String?
    let subtotalCents: Int
    let tipCents: Int?
    let providerUserId: String
    let customerState: String?
    let customerZip: String?
}

/// Fee preview request
struct FeePreviewRequest: Codable {
    let subtotalCents: Int
    let tipCents: Int?
    let customerState: String?
}

/// Fee preview response
struct FeePreviewResponse: Codable {
    let subtotalCents: Int
    let tipCents: Int
    let customerFeeCents: Int
    let taxCents: Int
    let totalCents: Int
    let customerFeePercent: Double
    let taxRatePercent: Double

    var subtotalDollars: Double {
        Double(subtotalCents) / 100.0
    }

    var totalDollars: Double {
        Double(totalCents) / 100.0
    }

    var tipDollars: Double {
        Double(tipCents) / 100.0
    }

    var customerFeeDollars: Double {
        Double(customerFeeCents) / 100.0
    }

    var taxDollars: Double {
        Double(taxCents) / 100.0
    }

    var formattedTotal: String {
        String(format: "$%.2f", totalDollars)
    }

    var formattedFee: String {
        String(format: "$%.2f", customerFeeDollars)
    }

    var formattedTax: String {
        String(format: "$%.2f", taxDollars)
    }
}

/// Confirm payment request
struct ConfirmPaymentRequest: Codable {
    let paymentMethodId: String?
}

/// Refund request
struct RefundRequest: Codable {
    let amountCents: Int?
    let reason: String?
}

/// Refund response
struct RefundResponse: Codable {
    let success: Bool
    let refundId: String?
    let refundAmountCents: Int
    let status: String

    var refundDollars: Double {
        Double(refundAmountCents) / 100.0
    }
}
