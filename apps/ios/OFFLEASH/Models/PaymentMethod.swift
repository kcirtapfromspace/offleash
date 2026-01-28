//
//  PaymentMethod.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

struct PaymentMethod: Codable, Identifiable {
    let id: String
    let methodType: String
    let displayName: String
    let cardLastFour: String?
    let cardBrand: String?
    let cardExpMonth: Int?
    let cardExpYear: Int?
    let nickname: String?
    let isDefault: Bool
    let isExpired: Bool
    let createdAt: String
}

struct CreatePaymentMethodRequest: Codable {
    let methodType: String
    let cardNonce: String?
    let cardLastFour: String?
    let cardBrand: String?
    let cardExpMonth: Int?
    let cardExpYear: Int?
    let nickname: String?
    let isDefault: Bool?
}

extension PaymentMethod {
    var icon: String {
        switch methodType {
        case "apple_pay":
            return "apple.logo"
        case "bank_account":
            return "building.columns"
        default:
            return "creditcard"
        }
    }

    var brandIcon: String? {
        guard let brand = cardBrand?.lowercased() else { return nil }
        switch brand {
        case "visa":
            return "v.circle.fill"
        case "mastercard":
            return "m.circle.fill"
        case "amex", "american express":
            return "a.circle.fill"
        case "discover":
            return "d.circle.fill"
        default:
            return nil
        }
    }

    var expirationText: String? {
        guard let month = cardExpMonth, let year = cardExpYear else { return nil }
        return String(format: "%02d/%02d", month, year % 100)
    }
}
