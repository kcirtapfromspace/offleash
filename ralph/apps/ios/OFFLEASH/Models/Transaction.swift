//
//  Transaction.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

/// Transaction status enum matching API
enum TransactionStatus: String, Codable {
    case pending
    case processing
    case succeeded
    case failed
    case refunded
    case partiallyRefunded = "partially_refunded"
    case disputed
    case canceled
}

/// Transaction model for payment history
struct Transaction: Codable, Identifiable {
    let id: String
    let bookingId: String?
    let status: TransactionStatus
    let subtotalCents: Int
    let customerFeeCents: Int
    let taxCents: Int
    let totalCents: Int
    let providerPayoutCents: Int?
    let tipCents: Int?
    let createdAt: String
    let completedAt: String?
    let isCustomer: Bool?

    // Computed properties
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

    var tipDollars: Double? {
        guard let tip = tipCents else { return nil }
        return Double(tip) / 100.0
    }

    var statusText: String {
        switch status {
        case .pending: return "Pending"
        case .processing: return "Processing"
        case .succeeded: return "Completed"
        case .failed: return "Failed"
        case .refunded: return "Refunded"
        case .partiallyRefunded: return "Partially Refunded"
        case .disputed: return "Disputed"
        case .canceled: return "Canceled"
        }
    }

    var statusColor: String {
        switch status {
        case .pending, .processing: return "yellow"
        case .succeeded: return "green"
        case .failed, .disputed: return "red"
        case .refunded, .partiallyRefunded, .canceled: return "gray"
        }
    }

    var formattedDate: String {
        let isoFormatter = ISO8601DateFormatter()
        isoFormatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]

        if let date = isoFormatter.date(from: createdAt) {
            let displayFormatter = DateFormatter()
            displayFormatter.dateStyle = .medium
            displayFormatter.timeStyle = .short
            return displayFormatter.string(from: date)
        }
        return createdAt
    }
}

/// Transaction list item (lighter weight for lists)
struct TransactionListItem: Codable, Identifiable {
    let id: String
    let bookingId: String?
    let status: String
    let subtotalCents: Int
    let totalCents: Int
    let createdAt: String
    let isCustomer: Bool

    var totalDollars: Double {
        Double(totalCents) / 100.0
    }

    var formattedTotal: String {
        String(format: "$%.2f", totalDollars)
    }
}
