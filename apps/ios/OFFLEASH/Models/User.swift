//
//  User.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - User Role

enum UserRole: String, Codable {
    case customer
    case walker
    case admin

    var isWalker: Bool {
        self == .walker
    }

    var isCustomer: Bool {
        self == .customer
    }

    var isAdmin: Bool {
        self == .admin
    }
}

// MARK: - User Model

struct User: Codable, Identifiable {
    let id: String
    let email: String
    let firstName: String?
    let lastName: String?
    let phone: String?
    let role: UserRole
    let organizationId: String?

    var fullName: String {
        [firstName, lastName].compactMap { $0 }.joined(separator: " ")
    }

    var displayName: String {
        fullName.isEmpty ? email : fullName
    }

    var needsOnboarding: Bool {
        role == .walker && organizationId == nil
    }
}

// MARK: - User Session

@MainActor
class UserSession: ObservableObject {
    static let shared = UserSession()

    @Published private(set) var currentUser: User?
    @Published private(set) var isLoggedIn: Bool = false

    // Membership management
    @Published private(set) var currentMembership: Membership?
    @Published private(set) var memberships: [Membership] = []

    private let userDefaultsKey = "currentUser"
    private let membershipDefaultsKey = "currentMembership"
    private let membershipsDefaultsKey = "memberships"

    private init() {
        loadStoredData()
    }

    func setUser(_ user: User) {
        currentUser = user
        isLoggedIn = true
        saveUser(user)
    }

    func setMemberships(_ memberships: [Membership], current: Membership?) {
        self.memberships = memberships
        self.currentMembership = current ?? memberships.first
        saveMemberships()
    }

    func setCurrentMembership(_ membership: Membership) {
        self.currentMembership = membership
        saveMemberships()
    }

    func clearUser() {
        currentUser = nil
        currentMembership = nil
        memberships = []
        isLoggedIn = false
        UserDefaults.standard.removeObject(forKey: userDefaultsKey)
        UserDefaults.standard.removeObject(forKey: membershipDefaultsKey)
        UserDefaults.standard.removeObject(forKey: membershipsDefaultsKey)
    }

    // MARK: - Computed Properties

    /// Current role based on active membership
    var currentRole: MembershipRole {
        currentMembership?.role ?? (currentUser?.role == .walker ? .walker : .customer)
    }

    var isWalker: Bool {
        currentRole == .walker
    }

    var isCustomer: Bool {
        currentRole == .customer
    }

    var isAdmin: Bool {
        currentRole == .admin || currentRole == .owner
    }

    var isWalkerOrAdmin: Bool {
        currentRole.isWalkerOrAdmin
    }

    var needsOnboarding: Bool {
        // Walker needs onboarding if they have no walker/admin memberships
        guard currentUser?.role == .walker else { return false }
        return !memberships.contains { $0.role.isWalkerOrAdmin }
    }

    var hasMultipleMemberships: Bool {
        memberships.count > 1
    }

    var currentOrganizationName: String {
        currentMembership?.organizationName ?? "OFFLEASH"
    }

    // MARK: - Private Methods

    private func loadStoredData() {
        // Load user
        if let data = UserDefaults.standard.data(forKey: userDefaultsKey),
           let user = try? JSONDecoder().decode(User.self, from: data) {
            currentUser = user
            isLoggedIn = KeychainHelper.shared.hasToken
        }

        // Load memberships
        if let data = UserDefaults.standard.data(forKey: membershipsDefaultsKey),
           let memberships = try? JSONDecoder().decode([Membership].self, from: data) {
            self.memberships = memberships
        }

        // Load current membership
        if let data = UserDefaults.standard.data(forKey: membershipDefaultsKey),
           let membership = try? JSONDecoder().decode(Membership.self, from: data) {
            self.currentMembership = membership
        }
    }

    private func saveUser(_ user: User) {
        guard let data = try? JSONEncoder().encode(user) else { return }
        UserDefaults.standard.set(data, forKey: userDefaultsKey)
    }

    private func saveMemberships() {
        if let data = try? JSONEncoder().encode(memberships) {
            UserDefaults.standard.set(data, forKey: membershipsDefaultsKey)
        }
        if let membership = currentMembership,
           let data = try? JSONEncoder().encode(membership) {
            UserDefaults.standard.set(data, forKey: membershipDefaultsKey)
        }
    }
}
