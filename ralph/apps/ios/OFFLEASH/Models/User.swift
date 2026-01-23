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
    let role: UserRole

    var fullName: String {
        [firstName, lastName].compactMap { $0 }.joined(separator: " ")
    }

    var displayName: String {
        fullName.isEmpty ? email : fullName
    }
}

// MARK: - User Session

@MainActor
class UserSession: ObservableObject {
    static let shared = UserSession()

    @Published private(set) var currentUser: User?
    @Published private(set) var isLoggedIn: Bool = false

    private let userDefaultsKey = "currentUser"

    private init() {
        loadStoredUser()
    }

    func setUser(_ user: User) {
        currentUser = user
        isLoggedIn = true
        saveUser(user)
    }

    func clearUser() {
        currentUser = nil
        isLoggedIn = false
        UserDefaults.standard.removeObject(forKey: userDefaultsKey)
    }

    var isWalker: Bool {
        currentUser?.role.isWalker ?? false
    }

    var isCustomer: Bool {
        currentUser?.role.isCustomer ?? false
    }

    var isAdmin: Bool {
        currentUser?.role.isAdmin ?? false
    }

    private func loadStoredUser() {
        guard let data = UserDefaults.standard.data(forKey: userDefaultsKey),
              let user = try? JSONDecoder().decode(User.self, from: data) else {
            return
        }
        currentUser = user
        isLoggedIn = KeychainHelper.shared.hasToken
    }

    private func saveUser(_ user: User) {
        guard let data = try? JSONEncoder().encode(user) else { return }
        UserDefaults.standard.set(data, forKey: userDefaultsKey)
    }
}
