//
//  PersonaSwitcherView.swift
//  OFFLEASH
//
//  A dropdown/sheet component for switching between user personas (memberships).
//  Users can have multiple roles across different organizations.
//

import SwiftUI

// MARK: - Persona Switcher View

struct PersonaSwitcherView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @ObservedObject private var session = UserSession.shared
    @State private var isExpanded = false
    @State private var isLoading = false
    @State private var errorMessage: String?

    var body: some View {
        VStack(spacing: 0) {
            // Current persona button
            Button(action: { withAnimation { isExpanded.toggle() } }) {
                HStack(spacing: 12) {
                    // Role icon
                    Image(systemName: session.currentMembership?.role.iconName ?? "person.fill")
                        .font(.system(size: 18))
                        .foregroundColor(themeManager.primaryColor)
                        .frame(width: 36, height: 36)
                        .background(themeManager.primaryColor.opacity(0.1))
                        .clipShape(Circle())

                    VStack(alignment: .leading, spacing: 2) {
                        Text(session.currentOrganizationName)
                            .font(.system(size: 16, weight: .semibold))
                            .foregroundColor(.primary)

                        Text(session.currentMembership?.role.displayName ?? "Customer")
                            .font(.system(size: 13))
                            .foregroundColor(.secondary)
                    }

                    Spacer()

                    if session.hasMultipleMemberships {
                        Image(systemName: isExpanded ? "chevron.up" : "chevron.down")
                            .font(.system(size: 14, weight: .medium))
                            .foregroundColor(.secondary)
                    }
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 12)
                .background(Color(.systemBackground))
                .cornerRadius(12)
            }
            .buttonStyle(PlainButtonStyle())

            // Expanded membership list
            if isExpanded && session.hasMultipleMemberships {
                VStack(spacing: 0) {
                    ForEach(session.memberships.filter { $0.id != session.currentMembership?.id }) { membership in
                        Button(action: { switchTo(membership) }) {
                            HStack(spacing: 12) {
                                Image(systemName: membership.role.iconName)
                                    .font(.system(size: 16))
                                    .foregroundColor(roleColor(for: membership.role))
                                    .frame(width: 32, height: 32)
                                    .background(roleColor(for: membership.role).opacity(0.1))
                                    .clipShape(Circle())

                                VStack(alignment: .leading, spacing: 2) {
                                    Text(membership.organizationName)
                                        .font(.system(size: 15, weight: .medium))
                                        .foregroundColor(.primary)

                                    HStack(spacing: 4) {
                                        Text(membership.role.displayName)
                                            .font(.system(size: 12))
                                            .foregroundColor(.secondary)

                                        if membership.role.isWalkerOrAdmin {
                                            Text("Dashboard")
                                                .font(.system(size: 11, weight: .medium))
                                                .foregroundColor(themeManager.primaryColor)
                                                .padding(.horizontal, 6)
                                                .padding(.vertical, 2)
                                                .background(themeManager.primaryColor.opacity(0.1))
                                                .cornerRadius(4)
                                        }
                                    }
                                }

                                Spacer()

                                if isLoading {
                                    ProgressView()
                                        .scaleEffect(0.8)
                                }
                            }
                            .padding(.horizontal, 16)
                            .padding(.vertical, 10)
                        }
                        .buttonStyle(PlainButtonStyle())
                        .disabled(isLoading)

                        if membership.id != session.memberships.last?.id {
                            Divider()
                                .padding(.leading, 60)
                        }
                    }
                }
                .background(Color(.systemBackground))
                .cornerRadius(12)
                .padding(.top, 4)
            }

            // Error message
            if let error = errorMessage {
                Text(error)
                    .font(.system(size: 13))
                    .foregroundColor(.red)
                    .padding(.top, 8)
            }
        }
    }

    private func roleColor(for role: MembershipRole) -> Color {
        switch role {
        case .customer:
            return .blue
        case .walker:
            return .green
        case .admin:
            return .orange
        case .owner:
            return .purple
        }
    }

    private func switchTo(_ membership: Membership) {
        isLoading = true
        errorMessage = nil

        Task {
            do {
                _ = try await APIClient.shared.switchContext(membershipId: membership.id)
                await MainActor.run {
                    isLoading = false
                    isExpanded = false
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    errorMessage = "Failed to switch: \(error.localizedDescription)"
                }
            }
        }
    }
}

// MARK: - Compact Persona Indicator

/// A smaller version of the persona switcher for use in navigation bars
struct PersonaIndicator: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @ObservedObject private var session = UserSession.shared
    @Binding var showingSwitcher: Bool

    var body: some View {
        Button(action: { showingSwitcher = true }) {
            HStack(spacing: 6) {
                Image(systemName: session.currentMembership?.role.iconName ?? "person.fill")
                    .font(.system(size: 14))

                if session.hasMultipleMemberships {
                    Image(systemName: "chevron.down")
                        .font(.system(size: 10, weight: .semibold))
                }
            }
            .foregroundColor(themeManager.primaryColor)
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(themeManager.primaryColor.opacity(0.1))
            .cornerRadius(16)
        }
    }
}

// MARK: - Persona Switcher Sheet

/// Full-screen sheet for switching personas with more details
struct PersonaSwitcherSheet: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    @ObservedObject private var session = UserSession.shared
    @State private var isLoading = false
    @State private var loadingMembershipId: String?
    @State private var errorMessage: String?

    var body: some View {
        NavigationView {
            List {
                Section(header: Text("Your Personas")) {
                    ForEach(session.memberships) { membership in
                        Button(action: { switchTo(membership) }) {
                            HStack(spacing: 14) {
                                // Role icon
                                ZStack {
                                    Circle()
                                        .fill(roleColor(for: membership.role).opacity(0.15))
                                        .frame(width: 44, height: 44)

                                    Image(systemName: membership.role.iconName)
                                        .font(.system(size: 20))
                                        .foregroundColor(roleColor(for: membership.role))
                                }

                                VStack(alignment: .leading, spacing: 4) {
                                    Text(membership.organizationName)
                                        .font(.system(size: 16, weight: .semibold))
                                        .foregroundColor(.primary)

                                    HStack(spacing: 8) {
                                        Text(membership.role.displayName)
                                            .font(.system(size: 14))
                                            .foregroundColor(.secondary)

                                        if membership.role.isWalkerOrAdmin {
                                            Label("Dashboard Access", systemImage: "rectangle.grid.2x2")
                                                .font(.system(size: 11))
                                                .foregroundColor(.white)
                                                .padding(.horizontal, 6)
                                                .padding(.vertical, 2)
                                                .background(roleColor(for: membership.role))
                                                .cornerRadius(4)
                                                .accessibilityIdentifier("org-role-badge")
                                        }
                                    }
                                }

                                Spacer()

                                // Current indicator or loading
                                if loadingMembershipId == membership.id {
                                    ProgressView()
                                } else if membership.id == session.currentMembership?.id {
                                    Image(systemName: "checkmark.circle.fill")
                                        .font(.system(size: 22))
                                        .foregroundColor(themeManager.primaryColor)
                                }
                            }
                            .padding(.vertical, 4)
                        }
                        .buttonStyle(PlainButtonStyle())
                        .accessibilityIdentifier("org-item-\(membership.organizationName)")
                        .disabled(isLoading || membership.id == session.currentMembership?.id)
                    }
                }

                if let error = errorMessage {
                    Section {
                        Text(error)
                            .foregroundColor(.red)
                            .font(.system(size: 14))
                    }
                }
            }
            .listStyle(InsetGroupedListStyle())
            .navigationTitle("Switch Persona")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }

    private func roleColor(for role: MembershipRole) -> Color {
        switch role {
        case .customer:
            return .blue
        case .walker:
            return .green
        case .admin:
            return .orange
        case .owner:
            return .purple
        }
    }

    private func switchTo(_ membership: Membership) {
        guard membership.id != session.currentMembership?.id else { return }

        isLoading = true
        loadingMembershipId = membership.id
        errorMessage = nil

        Task {
            do {
                _ = try await APIClient.shared.switchContext(membershipId: membership.id)
                await MainActor.run {
                    isLoading = false
                    loadingMembershipId = nil
                    dismiss()
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    loadingMembershipId = nil
                    errorMessage = "Failed to switch: \(error.localizedDescription)"
                }
            }
        }
    }
}

// MARK: - Preview

#Preview {
    VStack(spacing: 20) {
        PersonaSwitcherView()
            .padding()

        Spacer()
    }
    .background(Color.gray.opacity(0.1))
    .withThemeManager()
}
