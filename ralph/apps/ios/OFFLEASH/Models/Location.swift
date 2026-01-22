//
//  Location.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Location Model

struct Location: Codable, Identifiable, Equatable {
    let id: String
    let name: String
    let address: String
    let city: String
    let state: String
    let zipCode: String
    let latitude: Double
    let longitude: Double
    let notes: String?
    let isDefault: Bool
    // Note: No CodingKeys needed - APIClient uses convertFromSnakeCase

    var fullAddress: String {
        "\(address), \(city), \(state) \(zipCode)"
    }
}
