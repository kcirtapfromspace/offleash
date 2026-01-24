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

struct Membership: Codable, Identifiable, Equatable {
    let id: String
    let organizationId: String
    let organizationName: String
    let organizationSlug: String
    let role: MembershipRole
    let isDefault: Bool

    enum CodingKeys: String, CodingKey {
        case id
        case organizationId = "organization_id"
        case organizationName = "organization_name"
        case organizationSlug = "organization_slug"
        case role
        case isDefault = "is_default"
    }

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

struct ContextsResponse: Decodable {
    let currentMembership: Membership?
    let memberships: [Membership]

    enum CodingKeys: String, CodingKey {
        case currentMembership = "current_membership"
        case memberships
    }
}
