//
//  RecurringBooking.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Recurrence Frequency

enum RecurrenceFrequency: String, Codable, CaseIterable {
    case weekly
    case biweekly
    case monthly

    var displayName: String {
        switch self {
        case .weekly: return "Weekly"
        case .biweekly: return "Every 2 Weeks"
        case .monthly: return "Monthly"
        }
    }

    var description: String {
        switch self {
        case .weekly: return "Repeats every week"
        case .biweekly: return "Repeats every 2 weeks"
        case .monthly: return "Repeats every month"
        }
    }

    var icon: String {
        switch self {
        case .weekly: return "repeat"
        case .biweekly: return "repeat.circle"
        case .monthly: return "calendar.badge.clock"
        }
    }
}

// MARK: - Recurrence End Type

enum RecurrenceEndType: String, Codable, CaseIterable {
    case occurrences
    case date

    var displayName: String {
        switch self {
        case .occurrences: return "After occurrences"
        case .date: return "On specific date"
        }
    }
}

// MARK: - Recurring Booking Status

enum RecurringBookingStatus: String, Codable {
    case active
    case paused
    case cancelled
    case completed

    var displayName: String {
        switch self {
        case .active: return "Active"
        case .paused: return "Paused"
        case .cancelled: return "Cancelled"
        case .completed: return "Completed"
        }
    }

    var color: String {
        switch self {
        case .active: return "green"
        case .paused: return "orange"
        case .cancelled: return "red"
        case .completed: return "gray"
        }
    }
}

// MARK: - Recurring Booking Model

struct RecurringBooking: Identifiable, Codable {
    let id: String
    let customerId: String
    let walkerId: String?
    let walkerName: String?
    let serviceId: String
    let serviceName: String?
    let locationId: String
    let locationAddress: String?
    let frequency: RecurrenceFrequency
    let dayOfWeek: Int?
    let timeSlot: String
    let status: RecurringBookingStatus
    let startDate: Date
    let endDate: Date?
    let occurrences: Int?
    let completedOccurrences: Int
    let nextBookingDate: Date?
    let priceCents: Int
    let priceDisplay: String
    let notes: String?
    let createdAt: Date?

    // MARK: - Computed Properties

    var frequencyDescription: String {
        frequency.displayName
    }

    var scheduleDescription: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "EEEE"
        let dayName = dayOfWeek.map { getDayName(from: $0) } ?? "Scheduled day"

        return "\(dayName)s at \(timeSlot)"
    }

    var progressDescription: String {
        if let total = occurrences {
            return "\(completedOccurrences) of \(total) completed"
        } else if let endDate = endDate {
            let formatter = DateFormatter()
            formatter.dateStyle = .medium
            return "Until \(formatter.string(from: endDate))"
        }
        return "Ongoing"
    }

    var nextBookingDescription: String? {
        guard let nextDate = nextBookingDate else { return nil }

        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short

        return formatter.string(from: nextDate)
    }

    var isActive: Bool {
        status == .active
    }

    private func getDayName(from dayOfWeek: Int) -> String {
        let days = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
        guard dayOfWeek >= 0 && dayOfWeek < 7 else { return "Unknown" }
        return days[dayOfWeek]
    }
}

// MARK: - Recurring Booking List Response

struct RecurringBookingListResponse: Codable {
    let recurringBookings: [RecurringBooking]
}

// MARK: - Create Recurring Booking Request

struct CreateRecurringBookingRequest: Codable {
    let serviceId: String
    let locationId: String
    let walkerId: String?
    let frequency: String
    let dayOfWeek: Int
    let startTime: String
    let startDate: String
    let endDate: String?
    let occurrences: Int?
    let notes: String?
}

// MARK: - Cancel Recurring Booking Response

struct CancelRecurringBookingResponse: Codable {
    let success: Bool
    let message: String?
    let cancelledBookingsCount: Int?
}
