//
//  LocationService.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Location Service

/// Service for managing customer locations with automatic cache invalidation
actor LocationService {
    static let shared = LocationService()

    private init() {}

    // MARK: - Create Location

    /// Create a new location
    /// Automatically invalidates the locations cache on success
    func createLocation(_ location: LocationInput) async throws -> Location {
        let result: Location = try await APIClient.shared.post("/locations", body: location)
        await invalidateCache()
        return result
    }

    // MARK: - Update Location

    /// Update an existing location
    /// Automatically invalidates the locations cache on success
    func updateLocation(id: String, _ location: LocationInput) async throws -> Location {
        let result: Location = try await APIClient.shared.put("/locations/\(id)", body: location)
        await invalidateCache()
        return result
    }

    // MARK: - Delete Location

    /// Delete a location
    /// Automatically invalidates the locations cache on success
    func deleteLocation(id: String) async throws {
        try await APIClient.shared.delete("/locations/\(id)")
        await invalidateCache()
    }

    // MARK: - Cache Management

    /// Invalidate the locations cache
    /// Call this when locations are modified from outside this service
    func invalidateCache() async {
        await CacheManager.shared.invalidate(key: LocationSelectionView.cacheKey)
    }
}

// MARK: - Location Input

/// Input model for creating or updating a location
struct LocationInput: Encodable {
    let name: String
    let address: String
    let city: String
    let state: String
    let zip: String
    let notes: String?
    let isDefault: Bool
}
