//
//  Subscription.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

/// Subscription status
enum SubscriptionStatus: String, Codable {
    case active
    case paused
    case canceled
    case pastDue = "past_due"
    case trialing
    case incomplete
}

/// Customer subscription (service package)
struct CustomerSubscription: Codable, Identifiable {
    let id: String
    let serviceId: String?
    let name: String
    let description: String?
    let status: String
    let priceCents: Int
    let interval: String
    let intervalCount: Int
    let currentPeriodStart: String?
    let currentPeriodEnd: String?
    let cancelAtPeriodEnd: Bool
    let autoCreateBookings: Bool
    let createdAt: String

    var priceDollars: Double {
        Double(priceCents) / 100.0
    }

    var formattedPrice: String {
        String(format: "$%.2f", priceDollars)
    }

    var intervalDisplay: String {
        switch (interval, intervalCount) {
        case ("week", 1): return "Weekly"
        case ("week", let n): return "Every \(n) weeks"
        case ("month", 1): return "Monthly"
        case ("month", let n): return "Every \(n) months"
        case ("year", 1): return "Yearly"
        case ("year", let n): return "Every \(n) years"
        default: return "Every \(intervalCount) \(interval)"
        }
    }

    var statusText: String {
        switch status {
        case "active": return "Active"
        case "paused": return "Paused"
        case "canceled": return "Canceled"
        case "past_due": return "Past Due"
        case "trialing": return "Trial"
        case "incomplete": return "Incomplete"
        default: return status.capitalized
        }
    }

    var isActive: Bool {
        status == "active" || status == "trialing"
    }
}

/// Fee tier info (pricing plans)
struct FeeTier: Codable, Identifiable {
    let planTier: String
    let displayName: String
    let customerFeePercent: Double
    let providerFeePercent: Double
    let minCustomerFeeCents: Int
    let minProviderFeeCents: Int
    let monthlyPriceCents: Int
    let annualPriceCents: Int
    let features: [String]

    var id: String { planTier }

    var monthlyPriceDollars: Double {
        Double(monthlyPriceCents) / 100.0
    }

    var annualPriceDollars: Double {
        Double(annualPriceCents) / 100.0
    }

    var formattedMonthlyPrice: String {
        if monthlyPriceCents == 0 {
            return "Free"
        }
        return String(format: "$%.2f/mo", monthlyPriceDollars)
    }

    var formattedAnnualPrice: String {
        if annualPriceCents == 0 {
            return "Free"
        }
        return String(format: "$%.2f/yr", annualPriceDollars)
    }

    var annualMonthlySavings: Double {
        let annualMonthly = annualPriceDollars / 12.0
        return monthlyPriceDollars - annualMonthly
    }
}

/// Tenant/Business subscription
struct TenantSubscription: Codable {
    let id: String
    let planTier: String
    let status: String
    let monthlyPriceCents: Int
    let annualPriceCents: Int
    let customerFeePercent: Double
    let providerFeePercent: Double
    let currentPeriodStart: String?
    let currentPeriodEnd: String?
    let cancelAtPeriodEnd: Bool
    let createdAt: String

    var monthlyPriceDollars: Double {
        Double(monthlyPriceCents) / 100.0
    }

    var isActive: Bool {
        status == "active" || status == "trialing"
    }
}

/// Create subscription request
struct CreateCustomerSubscriptionRequest: Codable {
    let serviceId: String?
    let name: String
    let description: String?
    let priceCents: Int
    let interval: String
    let intervalCount: Int?
    let autoCreateBookings: Bool?
    let preferredDayOfWeek: Int?
    let paymentMethodId: String?
}

/// Create tenant subscription request
struct CreateSubscriptionRequest: Codable {
    let planTier: String
    let billingPeriod: String  // "monthly" or "annual"
    let paymentMethodId: String?
}
