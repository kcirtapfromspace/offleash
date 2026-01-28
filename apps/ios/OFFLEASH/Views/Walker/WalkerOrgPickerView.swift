//
//  WalkerOrgPickerView.swift
//  OFFLEASH
//
//  Shows when a walker user has multiple organizations to choose from.
//  Allows them to pick which org to work with or create/join a new one.
//

import SwiftUI

struct WalkerOrgPickerView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @ObservedObject private var session = UserSession.shared
    @State private var isLoading = false
    @State private var loadingMembershipId: String?
    @State private var errorMessage: String?

    var walkerMemberships: [Membership]
    var onOrgSelected: () -> Void
    var onCreateNew: () -> Void
    var onJoinExisting: () -> Void
    var onBack: (() -> Void)?

    var body: some View {
        GeometryReader { geometry in
            ScrollView {
                VStack(spacing: 24) {
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
                    } else {
                        Spacer(minLength: 40)
                    }

                    // Header
                    VStack(spacing: 16) {
                        Image(systemName: "building.2.fill")
                            .font(.system(size: 64))
                            .foregroundColor(themeManager.primaryColor)

                        Text("Welcome Back!")
                            .font(.largeTitle)
                            .fontWeight(.bold)
                            .foregroundColor(.primary)

                        Text("Which business would you like to manage?")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding(.bottom, 16)

                    // Existing organizations
                    VStack(spacing: 12) {
                        ForEach(walkerMemberships) { membership in
                            OrgSelectionCard(
                                membership: membership,
                                isLoading: loadingMembershipId == membership.id,
                                themeManager: themeManager
                            ) {
                                selectOrg(membership)
                            }
                        }
                    }
                    .padding(.horizontal, 24)

                    // Divider
                    HStack {
                        Rectangle()
                            .fill(Color(.systemGray4))
                            .frame(height: 1)
                        Text("or")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .padding(.horizontal, 16)
                        Rectangle()
                            .fill(Color(.systemGray4))
                            .frame(height: 1)
                    }
                    .padding(.horizontal, 24)
                    .padding(.vertical, 8)

                    // Create/Join options
                    VStack(spacing: 12) {
                        Button(action: onCreateNew) {
                            HStack {
                                Image(systemName: "plus.circle.fill")
                                    .font(.system(size: 20))
                                Text("Start a new business")
                                    .fontWeight(.medium)
                                Spacer()
                                Image(systemName: "chevron.right")
                                    .font(.system(size: 14, weight: .semibold))
                                    .foregroundColor(Color(.systemGray3))
                            }
                            .padding()
                            .background(
                                RoundedRectangle(cornerRadius: 12)
                                    .fill(Color(.systemGray6))
                            )
                        }
                        .buttonStyle(PlainButtonStyle())
                        .foregroundColor(.primary)

                        Button(action: onJoinExisting) {
                            HStack {
                                Image(systemName: "person.badge.plus.fill")
                                    .font(.system(size: 20))
                                Text("Join with invite code")
                                    .fontWeight(.medium)
                                Spacer()
                                Image(systemName: "chevron.right")
                                    .font(.system(size: 14, weight: .semibold))
                                    .foregroundColor(Color(.systemGray3))
                            }
                            .padding()
                            .background(
                                RoundedRectangle(cornerRadius: 12)
                                    .fill(Color(.systemGray6))
                            )
                        }
                        .buttonStyle(PlainButtonStyle())
                        .foregroundColor(.primary)
                    }
                    .padding(.horizontal, 24)

                    // Error message
                    if let error = errorMessage {
                        Text(error)
                            .font(.caption)
                            .foregroundColor(.red)
                            .padding(.horizontal, 24)
                    }

                    Spacer(minLength: 32)
                }
                .frame(minHeight: geometry.size.height)
            }
        }
    }

    private func selectOrg(_ membership: Membership) {
        isLoading = true
        loadingMembershipId = membership.id
        errorMessage = nil

        Task {
            do {
                _ = try await APIClient.shared.switchContext(membershipId: membership.id)
                await MainActor.run {
                    isLoading = false
                    loadingMembershipId = nil
                    onOrgSelected()
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

// MARK: - Org Selection Card

private struct OrgSelectionCard: View {
    let membership: Membership
    let isLoading: Bool
    let themeManager: ThemeManager
    let action: () -> Void

    private var roleColor: Color {
        switch membership.role {
        case .customer: return .blue
        case .walker: return .green
        case .admin: return .orange
        case .owner: return .purple
        }
    }

    var body: some View {
        Button(action: action) {
            HStack(spacing: 14) {
                // Role icon
                ZStack {
                    Circle()
                        .fill(roleColor.opacity(0.15))
                        .frame(width: 50, height: 50)

                    Image(systemName: membership.role.iconName)
                        .font(.system(size: 22))
                        .foregroundColor(roleColor)
                }

                VStack(alignment: .leading, spacing: 4) {
                    Text(membership.organizationName)
                        .font(.system(size: 17, weight: .semibold))
                        .foregroundColor(.primary)

                    Text(membership.role.displayName)
                        .font(.system(size: 14))
                        .foregroundColor(.secondary)
                }

                Spacer()

                if isLoading {
                    ProgressView()
                } else {
                    Image(systemName: "chevron.right")
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundColor(Color(.systemGray3))
                }
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
        .buttonStyle(PlainButtonStyle())
        .disabled(isLoading)
    }
}

#Preview {
    WalkerOrgPickerView(
        walkerMemberships: [],
        onOrgSelected: { print("Selected") },
        onCreateNew: { print("Create") },
        onJoinExisting: { print("Join") },
        onBack: { print("Back") }
    )
    .withThemeManager()
}
