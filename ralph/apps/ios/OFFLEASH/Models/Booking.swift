//
//  Booking.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Booking Status

enum BookingStatus: String, Codable, CaseIterable {
    case pending
    case confirmed
    case inProgress = "in_progress"
    case completed
    case cancelled
    case noShow = "no_show"

    var displayName: String {
        switch self {
        case .pending: return "Pending"
        case .confirmed: return "Confirmed"
        case .inProgress: return "In Progress"
        case .completed: return "Completed"
        case .cancelled: return "Cancelled"
        case .noShow: return "No Show"
        }
    }

    var color: String {
        switch self {
        case .pending: return "orange"
        case .confirmed: return "blue"
        case .inProgress: return "green"
        case .completed: return "gray"
        case .cancelled: return "red"
        case .noShow: return "red"
        }
    }
}

// MARK: - Booking Model

struct Booking: Identifiable, Codable {
    let id: String
    let customerId: String
    let customerName: String?
    let walkerId: String
    let walkerName: String?
    let serviceId: String
    let serviceName: String?
    let locationId: String
    let locationAddress: String?
    let latitude: Double?
    let longitude: Double?
    let status: BookingStatus
    let scheduledStart: Date
    let scheduledEnd: Date
    let priceCents: Int
    let priceDisplay: String
    let notes: String?
    let customerPhone: String?
    let petName: String?
    let petBreed: String?

    var duration: Int {
        Int(scheduledEnd.timeIntervalSince(scheduledStart) / 60)
    }

    var isToday: Bool {
        Calendar.current.isDateInToday(scheduledStart)
    }

    var isTomorrow: Bool {
        Calendar.current.isDateInTomorrow(scheduledStart)
    }

    var isPast: Bool {
        scheduledEnd < Date()
    }

    /// Whether the booking can be cancelled (not already cancelled/completed and not in progress)
    var canCancel: Bool {
        switch status {
        case .pending, .confirmed:
            return !isPast
        case .inProgress, .completed, .cancelled, .noShow:
            return false
        }
    }

    /// Whether the booking can be rescheduled (pending or confirmed, not past)
    var canReschedule: Bool {
        switch status {
        case .pending, .confirmed:
            return !isPast
        case .inProgress, .completed, .cancelled, .noShow:
            return false
        }
    }

    var timeString: String {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return formatter.string(from: scheduledStart)
    }

    var dateString: String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        return formatter.string(from: scheduledStart)
    }

    var timeRangeString: String {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return "\(formatter.string(from: scheduledStart)) - \(formatter.string(from: scheduledEnd))"
    }
}

// MARK: - Booking List Response

struct BookingListResponse: Codable {
    let bookings: [Booking]
}

// MARK: - Create Booking Request

struct CreateBookingRequest: Codable {
    let walkerId: String?
    let serviceId: String
    let locationId: String
    let startTime: String
    let notes: String?
    // Note: No CodingKeys needed - APIClient uses convertToSnakeCase
}

// MARK: - Dashboard Metrics

struct WalkerDashboardMetrics: Codable {
    let todayBookingCount: Int
    let pendingBookingCount: Int
    let weekEarningsCents: Int
    let completedThisWeek: Int

    var weekEarningsDisplay: String {
        let dollars = Double(weekEarningsCents) / 100.0
        return String(format: "$%.2f", dollars)
    }
}
