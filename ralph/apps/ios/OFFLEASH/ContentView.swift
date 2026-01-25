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

    /// Optional invite token from deep link for walker onboarding
    var inviteToken: String?

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
                    Text("Loading...")
                        .foregroundColor(.secondary)
                }
            } else if shouldShowWalkerExperience {
                // Walker/Admin view - show onboarding if needed, else dashboard
                if showWalkerOnboarding || userSession.needsOnboarding {
                    WalkerOnboardingView(
                        inviteToken: inviteToken,
                        onOnboardingComplete: {
                            showWalkerOnboarding = false
                            // Refresh user session to get updated organization
                            Task {
                                await refreshUserSession()
                            }
                        }
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
            // Show onboarding flow (which includes org picker) when:
            // - User selected walker role during login, OR
            // - User's base role is walker
            // The WalkerOnboardingView will then decide whether to show:
            // - Org picker (if they have existing walker/owner memberships)
            // - Create/join choice (if they have no walker memberships)
            let isWalkerFlow = selectedRole == .walker || userSession.currentUser?.role == .walker
            let hasWalkerMembership = userSession.memberships.contains { $0.role.isWalkerOrAdmin }

            // Show onboarding if walker flow AND either no membership OR no current walker membership selected
            showWalkerOnboarding = isWalkerFlow && (
                !hasWalkerMembership ||
                userSession.currentMembership?.role.isWalkerOrAdmin != true
            )

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
            let contextsResponse = try await APIClient.shared.fetchContexts()
            await MainActor.run {
                UserSession.shared.setMemberships(
                    contextsResponse.memberships,
                    current: contextsResponse.currentMembership
                )
            }
        } catch {
            print("Failed to load contexts: \(error)")
            // Continue without contexts - user can still use the app
        }
    }
}

#Preview {
    ContentView(selectedRole: .customer, inviteToken: nil)
        .withThemeManager()
        .environmentObject(SessionStateManager.shared)
}
