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

    /// Optional invite token from deep link for walker onboarding
    var inviteToken: String?

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
            } else if userSession.isWalkerOrAdmin {
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
            showWalkerOnboarding = userSession.needsOnboarding
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
    ContentView(inviteToken: nil)
        .withThemeManager()
        .environmentObject(SessionStateManager.shared)
}
