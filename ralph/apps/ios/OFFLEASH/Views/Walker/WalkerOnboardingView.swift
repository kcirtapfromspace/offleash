//
//  WalkerOnboardingView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

enum WalkerOnboardingStep {
    case orgPicker      // Show existing orgs to pick from
    case tenantChoice   // Create or join (no existing orgs)
    case createTenant
    case joinTenant
}

struct WalkerOnboardingView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @ObservedObject private var session = UserSession.shared

    @State private var currentStep: WalkerOnboardingStep = .tenantChoice

    /// Optional invite token from deep link - if present, goes directly to join flow
    var inviteToken: String?
    var onOnboardingComplete: () -> Void
    /// Called when user wants to go back to role selection
    var onBack: (() -> Void)?

    /// Walker/admin memberships the user already has
    private var existingWalkerMemberships: [Membership] {
        session.memberships.filter { $0.role.isWalkerOrAdmin }
    }

    var body: some View {
        Group {
            switch currentStep {
            case .orgPicker:
                WalkerOrgPickerView(
                    walkerMemberships: existingWalkerMemberships,
                    onOrgSelected: {
                        onOnboardingComplete()
                    },
                    onCreateNew: {
                        currentStep = .createTenant
                    },
                    onJoinExisting: {
                        currentStep = .joinTenant
                    },
                    onBack: onBack
                )
                .withThemeManager(themeManager)

            case .tenantChoice:
                TenantChoiceView(
                    onCreateTenant: {
                        currentStep = .createTenant
                    },
                    onJoinTenant: {
                        currentStep = .joinTenant
                    },
                    onBack: onBack
                )
                .withThemeManager(themeManager)

            case .createTenant:
                CreateTenantView(
                    onTenantCreated: {
                        onOnboardingComplete()
                    },
                    onBack: {
                        // Go back to appropriate view
                        currentStep = existingWalkerMemberships.isEmpty ? .tenantChoice : .orgPicker
                    }
                )
                .withThemeManager(themeManager)

            case .joinTenant:
                JoinTenantView(
                    inviteToken: inviteToken,
                    onTenantJoined: {
                        onOnboardingComplete()
                    },
                    onBack: {
                        // Go back to appropriate view
                        currentStep = existingWalkerMemberships.isEmpty ? .tenantChoice : .orgPicker
                    }
                )
                .withThemeManager(themeManager)
            }
        }
        .onAppear {
            updateStepBasedOnMemberships()
        }
        .onChange(of: session.memberships) { _ in
            // Re-evaluate step when memberships load/change
            updateStepBasedOnMemberships()
        }
    }

    private func updateStepBasedOnMemberships() {
        // Don't change step if user has navigated to create/join
        guard currentStep == .orgPicker || currentStep == .tenantChoice else { return }

        if inviteToken != nil {
            // If we have an invite token, go directly to join flow
            currentStep = .joinTenant
        } else if !existingWalkerMemberships.isEmpty {
            // User has existing walker/owner orgs - let them pick
            currentStep = .orgPicker
        } else {
            // No existing orgs - show create/join choice
            currentStep = .tenantChoice
        }
    }
}

// MARK: - Tenant Choice View

struct TenantChoiceView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    var onCreateTenant: () -> Void
    var onJoinTenant: () -> Void
    var onBack: (() -> Void)?

    var body: some View {
        GeometryReader { geometry in
            VStack(spacing: 0) {
                // Back button
                if let onBack = onBack {
                    HStack {
                        Button(action: onBack) {
                            HStack(spacing: 4) {
                                Image(systemName: "chevron.left")
                                    .font(.system(size: 16, weight: .semibold))
                                Text("Back")
                                    .font(.body)
                            }
                            .foregroundColor(themeManager.primaryColor)
                        }
                        Spacer()
                    }
                    .padding(.horizontal, 24)
                    .padding(.top, 16)
                }

                Spacer()

                // Header
                VStack(spacing: 16) {
                    Image(systemName: "building.2.fill")
                        .font(.system(size: 64))
                        .foregroundColor(themeManager.primaryColor)

                    Text("Welcome, Walker!")
                        .font(.largeTitle)
                        .fontWeight(.bold)
                        .foregroundColor(.primary)

                    Text("How would you like to get started?")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                        .multilineTextAlignment(.center)
                }
                .padding(.bottom, 48)

                // Options
                VStack(spacing: 16) {
                    // Create Business Card
                    TenantOptionCard(
                        icon: "plus.circle.fill",
                        title: "Start my own business",
                        subtitle: "Create your dog walking business and manage your own clients",
                        color: themeManager.primaryColor
                    ) {
                        analyticsService.trackEvent(name: "walker_onboarding_choice", params: ["choice": "create_tenant"])
                        onCreateTenant()
                    }

                    // Join Business Card
                    TenantOptionCard(
                        icon: "person.badge.plus.fill",
                        title: "Join an existing business",
                        subtitle: "Work with an established dog walking company",
                        color: themeManager.accentColor
                    ) {
                        analyticsService.trackEvent(name: "walker_onboarding_choice", params: ["choice": "join_tenant"])
                        onJoinTenant()
                    }
                }
                .padding(.horizontal, 24)

                Spacer()

                // Footer
                Text("You can change this later in settings")
                    .font(.footnote)
                    .foregroundColor(.secondary)
                    .padding(.bottom, 32)
            }
            .frame(minHeight: geometry.size.height)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "walker_tenant_choice")
        }
    }
}

// MARK: - Tenant Option Card

struct TenantOptionCard: View {
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
    WalkerOnboardingView(
        inviteToken: nil,
        onOnboardingComplete: {
            print("Onboarding complete")
        }
    )
    .withThemeManager()
}
