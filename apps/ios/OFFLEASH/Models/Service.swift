//
//  Service.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Service Model

struct Service: Codable, Identifiable, Equatable {
    let id: String
    let name: String
    let description: String?
    let durationMinutes: Int
    let priceCents: Int
    let priceDisplay: String
    let isActive: Bool
}
