//
//  OAuthService.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - OAuth Request/Response Models

/// Google OAuth request
/// org_slug is optional - if not provided, uses universal auth
struct OAuthGoogleRequest: Encodable {
    let orgSlug: String?
    let idToken: String
    let role: String?
}

/// Apple OAuth request
/// org_slug is optional - if not provided, uses universal auth
struct OAuthAppleRequest: Encodable {
    let orgSlug: String?
    let idToken: String
    let firstName: String?
    let lastName: String?
    let role: String?
}

struct OAuthResponse: Decodable {
    let token: String
    let user: OAuthUser
}

struct OAuthUser: Decodable {
    let id: String
    let email: String
    let firstName: String?
    let lastName: String?
    let phone: String?
    let role: String?
    let organizationId: String?
}

// MARK: - OAuth Errors

enum OAuthError: LocalizedError {
    case invalidCredentials
    case cancelled
    case notConfigured
    case unknown(String)

    var errorDescription: String? {
        switch self {
        case .invalidCredentials:
            return "Invalid credentials received from sign-in provider"
        case .cancelled:
            return "Sign-in was cancelled"
        case .notConfigured:
            return "OAuth is not configured"
        case .unknown(let message):
            return message
        }
    }
}
