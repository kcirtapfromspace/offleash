//
//  ContentView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @EnvironmentObject private var sessionStateManager: SessionStateManager
    @ObservedObject private var userSession = UserSession.shared
    @State private var showWalkerOnboarding: Bool = false
    @State private var isLoadingContexts: Bool = false

    /// The role selected during login/register flow - used for routing new users
    var selectedRole: SelectedRole

    /// Whether this is a fresh login (user just authenticated) vs returning user
    var isFreshLogin: Bool = true

    /// Optional invite token from deep link for walker onboarding
    var inviteToken: String?

    /// Called when user wants to go back to role selection (logout and restart)
    var onBackToRoleSelection: (() -> Void)?

    /// Determines if the current user should see the walker/admin experience
    private var shouldShowWalkerExperience: Bool {
        // If user has walker/admin memberships, they're definitely a walker
        if userSession.isWalkerOrAdmin {
            return true
        }
        // If user selected walker role during login but has no memberships yet,
        // they need to go through walker onboarding
        if selectedRole == .walker && userSession.memberships.isEmpty {
            return true
        }
        // Also check if user's base role is walker (set by backend)
        if userSession.currentUser?.role == .walker {
            return true
        }
        return false
    }

    var body: some View {
        Group {
            if isLoadingContexts {
                // Show loading while fetching contexts
                VStack(spacing: 16) {
                    ProgressView()
                        .scaleEffect(1.5)
                        .accessibilityIdentifier("loading-indicator")
                    Text("Loading...")
                        .foregroundColor(.secondary)
                }
            } else if shouldShowWalkerExperience {
                // Walker/Admin view - show onboarding/picker if needed, else dashboard
                if showWalkerOnboarding || userSession.needsOnboarding {
                    WalkerOnboardingView(
                        inviteToken: inviteToken,
                        onOnboardingComplete: {
                            showWalkerOnboarding = false
                            // Refresh user session to get updated organization
                            Task {
                                await refreshUserSession()
                            }
                        },
                        onBack: onBackToRoleSelection
                    )
                } else {
                    WalkerTabView()
                }
            } else {
                // Customer view - tab-based navigation
                CustomerTabView()
            }
        }
        .onAppear {
            // Determine if walker onboarding/org picker is needed
            let isWalkerFlow = selectedRole == .walker || userSession.currentUser?.role == .walker
            let hasWalkerMembership = userSession.memberships.contains { $0.role.isWalkerOrAdmin }

            // Show onboarding/org picker when:
            // 1. Fresh login as walker (isFreshLogin && selectedRole == .walker) - always show picker
            // 2. Walker with no walker memberships yet (needs to create/join)
            // 3. Returning walker with no current walker membership selected
            if isFreshLogin && selectedRole == .walker {
                // Fresh login as walker - always show org picker (or create/join if no orgs)
                showWalkerOnboarding = true
            } else if isWalkerFlow && !hasWalkerMembership {
                // Walker who needs to join/create an org
                showWalkerOnboarding = true
            } else if isWalkerFlow && userSession.currentMembership?.role.isWalkerOrAdmin != true {
                // Walker but current membership isn't a walker role - show picker
                showWalkerOnboarding = true
            } else {
                showWalkerOnboarding = false
            }

            // Load contexts on appear if not already loaded
            if userSession.memberships.isEmpty {
                Task {
                    await loadContexts()
                }
            }
        }
    }

    private func refreshUserSession() async {
        // Re-validate token to get updated user info with organization
        do {
            let response = try await APIClient.shared.validateToken()
            if response.valid, let userData = response.user {
                let user = User(
                    id: userData.id,
                    email: userData.email,
                    firstName: userData.firstName,
                    lastName: userData.lastName,
                    phone: userData.phone,
                    role: UserRole(rawValue: userData.role ?? "customer") ?? .customer,
                    organizationId: userData.organizationId
                )
                await MainActor.run {
                    UserSession.shared.setUser(user)
                }
            }
            // Also reload contexts after refreshing user
            await loadContexts()
        } catch {
            // Handle error silently - user will stay on current view
            print("Failed to refresh user session: \(error)")
        }
    }

    private func loadContexts() async {
        isLoadingContexts = true
        defer { isLoadingContexts = false }

        do {
            print("[ContentView] Fetching contexts...")
            let contextsResponse = try await APIClient.shared.fetchContexts()
            print("[ContentView] Got \(contextsResponse.memberships.count) memberships")
            for m in contextsResponse.memberships {
                print("[ContentView]   - \(m.organizationName): \(m.role.rawValue)")
            }
            await MainActor.run {
                UserSession.shared.setMemberships(
                    contextsResponse.memberships,
                    current: contextsResponse.currentMembership
                )
                print("[ContentView] Set memberships in UserSession")
            }
        } catch {
            print("[ContentView] Failed to load contexts: \(error)")
            // Continue without contexts - user can still use the app
        }
    }
}

#Preview {
    ContentView(
        selectedRole: .customer,
        isFreshLogin: true,
        inviteToken: nil,
        onBackToRoleSelection: { print("Back to role selection") }
    )
    .withThemeManager()
    .environmentObject(SessionStateManager.shared)
}
