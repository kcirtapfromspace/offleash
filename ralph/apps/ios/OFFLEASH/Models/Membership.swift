//
//  Membership.swift
//  OFFLEASH
//
//  Represents a user's membership in an organization with a specific role.
//  Users can have multiple memberships across different organizations.
//

import Foundation

// MARK: - Membership Role

enum MembershipRole: String, Codable, CaseIterable {
    case customer
    case walker
    case admin
    case owner

    var displayName: String {
        switch self {
        case .customer: return "Customer"
        case .walker: return "Walker"
        case .admin: return "Admin"
        case .owner: return "Owner"
        }
    }

    var isWalkerOrAdmin: Bool {
        self == .walker || self == .admin || self == .owner
    }

    var isCustomer: Bool {
        self == .customer
    }

    /// Icon name for SF Symbols
    var iconName: String {
        switch self {
        case .customer: return "person.fill"
        case .walker: return "figure.walk"
        case .admin: return "gearshape.fill"
        case .owner: return "building.2.fill"
        }
    }
}

// MARK: - Membership Model

/// Note: APIClient uses keyDecodingStrategy = .convertFromSnakeCase
/// so we don't need explicit CodingKeys for snake_case conversion
struct Membership: Codable, Identifiable, Equatable {
    let id: String
    let organizationId: String
    let organizationName: String
    let organizationSlug: String
    let role: MembershipRole
    // API also returns these optional fields - we include them to avoid decoding issues
    let title: String?
    let joinedAt: String?

    static func == (lhs: Membership, rhs: Membership) -> Bool {
        lhs.id == rhs.id
    }
}

// MARK: - Context Switch Response

struct SwitchContextResponse: Codable {
    let token: String
    let membership: Membership
}

// MARK: - Contexts Response

/// Note: APIClient uses keyDecodingStrategy = .convertFromSnakeCase
/// so defaultMembershipId automatically maps from default_membership_id
struct ContextsResponse: Decodable {
    let memberships: [Membership]
    let defaultMembershipId: String?
    // API also returns user info - we include to avoid decoding issues
    let userId: String?
    let email: String?
    let firstName: String?
    let lastName: String?

    /// The current/default membership based on defaultMembershipId
    var currentMembership: Membership? {
        guard let defaultId = defaultMembershipId else {
            return memberships.first
        }
        return memberships.first { $0.id == defaultId } ?? memberships.first
    }
}
