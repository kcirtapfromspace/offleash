//
//  JoinTenantView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Request/Response Models

struct JoinTenantRequest: Encodable {
    let inviteToken: String
}

struct JoinTenantResponse: Decodable {
    let success: Bool
    let tenantName: String?
    let message: String?
}

// MARK: - Join Tenant View

struct JoinTenantView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    @State private var isLoading = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showSuccess = false
    @State private var joinedTenantName = ""

    /// Invite token passed from deep link
    var inviteToken: String?
    var onTenantJoined: () -> Void
    var onBack: () -> Void

    var body: some View {
        GeometryReader { geometry in
            ScrollView {
                VStack(spacing: 24) {
                    // Back Button
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
                    .padding(.top, 16)

                    Spacer()

                    // Content depends on whether we have an invite token
                    if let token = inviteToken, !token.isEmpty {
                        // Auto-joining with token from deep link
                        joiningContent
                    } else {
                        // No token - show instructions to get an invite
                        noInviteContent
                    }

                    Spacer()
                }
                .padding(.horizontal, 24)
                .frame(minHeight: geometry.size.height)
            }
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .alert("Welcome!", isPresented: $showSuccess) {
            Button("Get Started") {
                onTenantJoined()
            }
        } message: {
            Text("You've successfully joined \(joinedTenantName). You can now start accepting bookings.")
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "join_tenant")
            // Auto-join if we have a token
            if let token = inviteToken, !token.isEmpty {
                joinTenant(with: token)
            }
        }
    }

    // MARK: - View Components

    private var joiningContent: some View {
        VStack(spacing: 24) {
            ProgressView()
                .scaleEffect(1.5)
                .padding(.bottom, 16)

            Text("Joining Business...")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Please wait while we add you to the team.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
        }
    }

    private var noInviteContent: some View {
        VStack(spacing: 24) {
            Image(systemName: "envelope.badge.fill")
                .font(.system(size: 64))
                .foregroundColor(themeManager.primaryColor)

            Text("Need an Invitation")
                .font(.title)
                .fontWeight(.bold)

            Text("To join a business, you'll need an invitation from the business owner.")
                .font(.body)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            // Instructions
            VStack(alignment: .leading, spacing: 16) {
                InstructionRow(
                    number: "1",
                    text: "Ask the business owner to send you an invite",
                    color: themeManager.primaryColor
                )
                InstructionRow(
                    number: "2",
                    text: "Check your email or text messages for the invitation link",
                    color: themeManager.primaryColor
                )
                InstructionRow(
                    number: "3",
                    text: "Tap the link to automatically join the team",
                    color: themeManager.primaryColor
                )
            }
            .padding()
            .background(
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color(.systemGray6))
            )

            // Info
            HStack(spacing: 8) {
                Image(systemName: "info.circle")
                    .foregroundColor(.secondary)
                Text("Invitation links expire after 7 days")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding(.top, 8)
        }
    }

    // MARK: - Actions

    private func joinTenant(with token: String) {
        isLoading = true

        Task {
            do {
                let request = JoinTenantRequest(inviteToken: token)
                let response: JoinTenantResponse = try await APIClient.shared.post("/walker/join-tenant", body: request)

                await MainActor.run {
                    isLoading = false
                    if response.success {
                        joinedTenantName = response.tenantName ?? "the business"
                        analyticsService.trackEvent(name: "tenant_joined", params: [:])
                        showSuccess = true
                    } else {
                        errorMessage = response.message ?? "This invitation is invalid or has expired."
                        showError = true
                    }
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "This invitation is invalid or has expired."
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    errorMessage = "An unexpected error occurred. Please try again."
                    showError = true
                }
            }
        }
    }
}

// MARK: - Instruction Row

private struct InstructionRow: View {
    let number: String
    let text: String
    let color: Color

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Text(number)
                .font(.subheadline)
                .fontWeight(.bold)
                .foregroundColor(.white)
                .frame(width: 24, height: 24)
                .background(Circle().fill(color))

            Text(text)
                .font(.subheadline)
                .foregroundColor(.primary)
        }
    }
}

#Preview("No Token") {
    JoinTenantView(
        inviteToken: nil,
        onTenantJoined: { print("Joined") },
        onBack: { print("Back") }
    )
    .withThemeManager()
}

#Preview("With Token") {
    JoinTenantView(
        inviteToken: "abc123token",
        onTenantJoined: { print("Joined") },
        onBack: { print("Back") }
    )
    .withThemeManager()
}
