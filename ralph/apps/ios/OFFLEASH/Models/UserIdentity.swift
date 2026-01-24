//
//  UserIdentity.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Identity Provider

enum IdentityProvider: String, Codable, CaseIterable {
    case email
    case google
    case apple
    case phone

    var displayName: String {
        switch self {
        case .email: return "Email & Password"
        case .google: return "Google"
        case .apple: return "Apple"
        case .phone: return "Phone"
        }
    }

    var icon: String {
        switch self {
        case .email: return "envelope.fill"
        case .google: return "g.circle.fill"
        case .apple: return "apple.logo"
        case .phone: return "phone.fill"
        }
    }

    var color: String {
        switch self {
        case .email: return "blue"
        case .google: return "red"
        case .apple: return "black"
        case .phone: return "green"
        }
    }
}

// MARK: - User Identity Model

struct UserIdentity: Identifiable, Codable {
    let id: String
    let userId: String
    let provider: IdentityProvider
    let providerUserId: String
    let email: String?
    let phone: String?
    let isPrimary: Bool
    let createdAt: Date?

    var displayIdentifier: String {
        if let email = email, !email.isEmpty {
            return email
        }
        if let phone = phone, !phone.isEmpty {
            return formatPhone(phone)
        }
        return provider.displayName
    }

    var subtitle: String {
        if isPrimary {
            return "Primary Account"
        }
        if let createdAt = createdAt {
            return "Linked \(formatDate(createdAt))"
        }
        return "Linked Account"
    }

    private func formatPhone(_ phone: String) -> String {
        let cleaned = phone.filter { $0.isNumber || $0 == "+" }
        if cleaned.hasPrefix("+1") && cleaned.count == 12 {
            let start = cleaned.index(cleaned.startIndex, offsetBy: 2)
            let areaEnd = cleaned.index(start, offsetBy: 3)
            let middleEnd = cleaned.index(areaEnd, offsetBy: 3)
            return "+1 (\(cleaned[start..<areaEnd])) \(cleaned[areaEnd..<middleEnd])-\(cleaned[middleEnd...])"
        }
        return cleaned
    }

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        return formatter.string(from: date)
    }
}

// MARK: - Identity List Response

struct IdentityListResponse: Codable {
    let identities: [UserIdentity]
}

// MARK: - Link Identity Requests

struct LinkGoogleRequest: Codable {
    let idToken: String
}

struct LinkAppleRequest: Codable {
    let idToken: String
    let firstName: String?
    let lastName: String?
}

struct LinkEmailRequest: Codable {
    let email: String
    let password: String
}

struct LinkPhoneRequest: Codable {
    let phone: String
    let code: String
}

// MARK: - Change Password Request

struct ChangePasswordRequest: Codable {
    let currentPassword: String?
    let newPassword: String
}

struct ChangePasswordResponse: Codable {
    let success: Bool
    let message: String?
}

// MARK: - Identity Response

struct IdentityResponse: Codable {
    let identity: UserIdentity
}
