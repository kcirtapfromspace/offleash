//
//  TokenValidation.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Token Validation Response

struct TokenValidationResponse: Decodable {
    let valid: Bool
    let expiresAt: Date?
}
