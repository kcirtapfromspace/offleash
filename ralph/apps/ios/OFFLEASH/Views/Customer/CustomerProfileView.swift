//
//  CustomerProfileView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct CustomerProfileView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @ObservedObject private var userSession = UserSession.shared
    @State private var showLogoutConfirmation = false
    @State private var isLoggingOut = false

    var body: some View {
        List {
            // User Info Section
            Section {
                HStack(spacing: 16) {
                    // Avatar
                    ZStack {
                        Circle()
                            .fill(themeManager.primaryColor.opacity(0.1))
                            .frame(width: 64, height: 64)

                        Text(userInitials)
                            .font(.title2)
                            .fontWeight(.semibold)
                            .foregroundColor(themeManager.primaryColor)
                    }

                    VStack(alignment: .leading, spacing: 4) {
                        Text(userSession.currentUser?.displayName ?? "Guest")
                            .font(.headline)

                        Text(userSession.currentUser?.email ?? "")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }

                    Spacer()
                }
                .padding(.vertical, 8)
            }

            // Account Section
            Section("Account") {
                NavigationLink {
                    Text("Edit Profile")
                        .navigationTitle("Edit Profile")
                } label: {
                    Label("Edit Profile", systemImage: "person.circle")
                }

                NavigationLink {
                    Text("My Locations")
                        .navigationTitle("My Locations")
                } label: {
                    Label("My Locations", systemImage: "mappin.circle")
                }

                NavigationLink {
                    Text("Payment Methods")
                        .navigationTitle("Payment Methods")
                } label: {
                    Label("Payment Methods", systemImage: "creditcard")
                }
            }

            // Preferences Section
            Section("Preferences") {
                NavigationLink {
                    Text("Notifications")
                        .navigationTitle("Notifications")
                } label: {
                    Label("Notifications", systemImage: "bell")
                }
            }

            // Support Section
            Section("Support") {
                NavigationLink {
                    Text("Help Center")
                        .navigationTitle("Help Center")
                } label: {
                    Label("Help Center", systemImage: "questionmark.circle")
                }

                NavigationLink {
                    Text("Contact Us")
                        .navigationTitle("Contact Us")
                } label: {
                    Label("Contact Us", systemImage: "envelope")
                }
            }

            // Logout Section
            Section {
                Button(role: .destructive) {
                    showLogoutConfirmation = true
                } label: {
                    HStack {
                        Spacer()
                        if isLoggingOut {
                            ProgressView()
                                .tint(.red)
                        } else {
                            Text("Log Out")
                                .fontWeight(.medium)
                        }
                        Spacer()
                    }
                }
                .disabled(isLoggingOut)
            }

            // App Version
            Section {
                HStack {
                    Text("Version")
                        .foregroundColor(.secondary)
                    Spacer()
                    Text(appVersion)
                        .foregroundColor(.secondary)
                }
            }
        }
        .navigationTitle("Profile")
        .onAppear {
            analyticsService.trackScreenView(screenName: "customer_profile")
        }
        .alert("Log Out", isPresented: $showLogoutConfirmation) {
            Button("Cancel", role: .cancel) {}
            Button("Log Out", role: .destructive) {
                logout()
            }
        } message: {
            Text("Are you sure you want to log out?")
        }
    }

    private var userInitials: String {
        guard let user = userSession.currentUser else { return "?" }
        let first = user.firstName?.prefix(1) ?? ""
        let last = user.lastName?.prefix(1) ?? ""
        let initials = "\(first)\(last)"
        return initials.isEmpty ? String(user.email.prefix(1)).uppercased() : initials.uppercased()
    }

    private var appVersion: String {
        let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
        let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1"
        return "\(version) (\(build))"
    }

    private func logout() {
        isLoggingOut = true
        analyticsService.trackEvent(name: "logout", params: nil)

        Task {
            await APIClient.shared.clearAuthToken()
            // Post notification to trigger app state change
            NotificationCenter.default.post(
                name: .authStateChanged,
                object: nil,
                userInfo: ["isAuthenticated": false]
            )
            isLoggingOut = false
        }
    }
}

#Preview {
    NavigationStack {
        CustomerProfileView()
    }
    .withThemeManager()
}
