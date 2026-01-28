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
    let user: TokenValidationUser?
}

struct TokenValidationUser: Decodable {
    let id: String
    let email: String
    let firstName: String?
    let lastName: String?
    let role: String?
    let organizationId: String?
}
