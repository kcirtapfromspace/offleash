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
    let zip: String
    let notes: String?
    let isDefault: Bool

    var fullAddress: String {
        "\(address), \(city), \(state) \(zip)"
    }
}
