//
//  AvailabilityService.swift
//  OFFLEASH
//
//  Service for fetching available time slots from the API
//

import Foundation

// MARK: - API Response Models

struct AvailabilitySlotsResponse: Codable {
    let walkerId: String
    let date: String
    let serviceId: String
    let slots: [AvailableSlotResponse]

    enum CodingKeys: String, CodingKey {
        case walkerId = "walker_id"
        case date
        case serviceId = "service_id"
        case slots
    }
}

struct AvailableSlotResponse: Codable {
    let start: String
    let end: String
    let confidence: String
}

struct OptimizedRouteResponse: Codable {
    let date: String
    let isOptimized: Bool
    let stops: [RouteStopResponse]
    let totalTravelMinutes: Int
    let totalDistanceMeters: Int
    let savingsMinutes: Int

    enum CodingKeys: String, CodingKey {
        case date
        case isOptimized = "is_optimized"
        case stops
        case totalTravelMinutes = "total_travel_minutes"
        case totalDistanceMeters = "total_distance_meters"
        case savingsMinutes = "savings_minutes"
    }
}

struct RouteStopResponse: Codable {
    let sequence: Int
    let bookingId: String
    let customerName: String
    let address: String
    let arrivalTime: String
    let departureTime: String
    let travelFromPreviousMinutes: Int
    let serviceDurationMinutes: Int

    enum CodingKeys: String, CodingKey {
        case sequence
        case bookingId = "booking_id"
        case customerName = "customer_name"
        case address
        case arrivalTime = "arrival_time"
        case departureTime = "departure_time"
        case travelFromPreviousMinutes = "travel_from_previous_minutes"
        case serviceDurationMinutes = "service_duration_minutes"
    }
}

// MARK: - AvailabilityService

actor AvailabilityService {
    static let shared = AvailabilityService()

    private let dateFormatter: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()

    private init() {}

    /// Fetch available time slots for a given date, service, and location
    /// - Parameters:
    ///   - date: The date to check availability for
    ///   - serviceId: The service ID
    ///   - locationId: The customer's location ID
    ///   - walkerId: Optional specific walker ID (if not provided, returns slots from any available walker)
    /// - Returns: Array of available time slots
    func getAvailableSlots(
        date: Date,
        serviceId: String,
        locationId: String,
        walkerId: String? = nil
    ) async throws -> [TimeSlot] {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd"
        let dateString = dateFormatter.string(from: date)

        // If walkerId is provided, use the specific walker availability endpoint
        // Otherwise, use the general slots endpoint
        let endpoint: String
        var queryItems: [URLQueryItem] = [
            URLQueryItem(name: "date", value: dateString),
            URLQueryItem(name: "service_id", value: serviceId),
            URLQueryItem(name: "location_id", value: locationId)
        ]

        if let walkerId = walkerId {
            endpoint = "/availability/\(walkerId)"
        } else {
            endpoint = "/availability/slots"
        }

        let response: AvailabilitySlotsResponse = try await APIClient.shared.get(
            endpoint,
            queryItems: queryItems
        )

        return response.slots.compactMap { slot in
            guard let startDate = self.dateFormatter.date(from: slot.start),
                  let endDate = self.dateFormatter.date(from: slot.end) else {
                return nil
            }

            return TimeSlot(
                id: slot.start,
                startTime: startDate,
                endTime: endDate,
                isAvailable: true,
                confidence: SlotConfidence(rawValue: slot.confidence) ?? .high
            )
        }
    }

    /// Fetch the optimized route for a walker on a given date
    /// - Parameters:
    ///   - walkerId: The walker's user ID
    ///   - date: The date to get the route for
    /// - Returns: Optimized route response
    func getOptimizedRoute(
        walkerId: String,
        date: Date
    ) async throws -> OptimizedRouteResponse {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd"
        let dateString = dateFormatter.string(from: date)

        return try await APIClient.shared.get(
            "/walkers/\(walkerId)/route",
            queryItems: [URLQueryItem(name: "date", value: dateString)]
        )
    }

    /// Trigger route optimization for a walker on a given date
    /// - Parameters:
    ///   - walkerId: The walker's user ID
    ///   - date: The date to optimize the route for
    /// - Returns: Optimized route response
    func optimizeRoute(
        walkerId: String,
        date: Date
    ) async throws -> OptimizedRouteResponse {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd"
        let dateString = dateFormatter.string(from: date)

        return try await APIClient.shared.get(
            "/walkers/\(walkerId)/route/optimize",
            queryItems: [URLQueryItem(name: "date", value: dateString)]
        )
    }
}

// MARK: - Slot Confidence

enum SlotConfidence: String, Codable {
    case high = "High"
    case medium = "Medium"
    case low = "Low"

    var displayText: String {
        switch self {
        case .high:
            return "Confirmed"
        case .medium:
            return "Likely"
        case .low:
            return "Estimated"
        }
    }

    var iconName: String {
        switch self {
        case .high:
            return "checkmark.circle.fill"
        case .medium:
            return "clock.fill"
        case .low:
            return "questionmark.circle.fill"
        }
    }
}

// MARK: - Extended TimeSlot

extension TimeSlot {
    init(id: String, startTime: Date, endTime: Date, isAvailable: Bool, confidence: SlotConfidence) {
        self.id = id
        self.startTime = startTime
        self.endTime = endTime
        self.isAvailable = isAvailable
    }
}
