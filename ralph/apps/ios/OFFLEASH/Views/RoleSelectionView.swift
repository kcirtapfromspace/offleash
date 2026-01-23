//
//  RoleSelectionView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct RoleSelectionView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    let onRoleSelected: (SelectedRole) -> Void

    var body: some View {
        GeometryReader { geometry in
            VStack(spacing: 0) {
                Spacer()

                // Logo and App Name
                VStack(spacing: 16) {
                    Image(systemName: "pawprint.fill")
                        .font(.system(size: 80))
                        .foregroundColor(themeManager.primaryColor)

                    Text(themeManager.branding.companyName)
                        .font(.largeTitle)
                        .fontWeight(.bold)
                        .foregroundColor(themeManager.primaryColor)

                    Text("Premium pet care, on demand")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                .padding(.bottom, 60)

                // Role Selection Cards
                VStack(spacing: 16) {
                    Text("How would you like to use the app?")
                        .font(.headline)
                        .foregroundColor(.primary)
                        .padding(.bottom, 8)

                    // Customer Card
                    RoleCard(
                        icon: "dog.fill",
                        title: "My dogs need a walk",
                        subtitle: "Book walks, daycare, and more for your pets",
                        color: themeManager.primaryColor
                    ) {
                        analyticsService.trackEvent(name: "role_selected", params: ["role": "customer"])
                        onRoleSelected(.customer)
                    }

                    // Walker Card
                    RoleCard(
                        icon: "figure.walk",
                        title: "I walk dogs",
                        subtitle: "Manage your schedule and bookings",
                        color: themeManager.accentColor
                    ) {
                        analyticsService.trackEvent(name: "role_selected", params: ["role": "walker"])
                        onRoleSelected(.walker)
                    }
                }
                .padding(.horizontal, 24)

                Spacer()

                // Footer
                VStack(spacing: 8) {
                    Text("Need help?")
                        .font(.footnote)
                        .foregroundColor(.secondary)

                    Text(themeManager.branding.supportEmail)
                        .font(.footnote)
                        .foregroundColor(themeManager.accentColor)
                }
                .padding(.bottom, 32)
            }
            .frame(minHeight: geometry.size.height)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "role_selection")
        }
    }
}

// MARK: - Role Card

struct RoleCard: View {
    let icon: String
    let title: String
    let subtitle: String
    let color: Color
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 16) {
                // Icon
                ZStack {
                    Circle()
                        .fill(color.opacity(0.1))
                        .frame(width: 56, height: 56)

                    Image(systemName: icon)
                        .font(.system(size: 24))
                        .foregroundColor(color)
                }

                // Text
                VStack(alignment: .leading, spacing: 4) {
                    Text(title)
                        .font(.headline)
                        .foregroundColor(.primary)

                    Text(subtitle)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                        .multilineTextAlignment(.leading)
                }

                Spacer()

                // Arrow
                Image(systemName: "chevron.right")
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundColor(Color(.systemGray3))
            }
            .padding(16)
            .background(
                RoundedRectangle(cornerRadius: 16)
                    .fill(Color(.systemBackground))
                    .shadow(color: Color.black.opacity(0.05), radius: 8, x: 0, y: 2)
            )
            .overlay(
                RoundedRectangle(cornerRadius: 16)
                    .stroke(Color(.systemGray5), lineWidth: 1)
            )
        }
        .buttonStyle(.plain)
    }
}

#Preview {
    RoleSelectionView { role in
        print("Selected role: \(role)")
    }
    .withThemeManager()
}
