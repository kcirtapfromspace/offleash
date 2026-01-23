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

    /// Optional invite token from deep link for walker onboarding
    var inviteToken: String?

    var body: some View {
        Group {
            // Show different views based on user role and onboarding status
            if userSession.isWalker {
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
        } catch {
            // Handle error silently - user will stay on current view
            print("Failed to refresh user session: \(error)")
        }
    }
}

#Preview {
    ContentView(inviteToken: nil)
        .withThemeManager()
        .environmentObject(SessionStateManager.shared)
}
