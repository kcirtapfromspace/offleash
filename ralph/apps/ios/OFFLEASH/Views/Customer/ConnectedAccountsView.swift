//
//  ConnectedAccountsView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import UIKit
import AuthenticationServices
import GoogleSignIn

struct ConnectedAccountsView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    @State private var identities: [UserIdentity] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var identityToUnlink: UserIdentity?
    @State private var showLinkOptions = false
    @State private var isLinking = false
    @State private var showChangePassword = false

    // Available providers that can be linked
    private var linkableProviders: [IdentityProvider] {
        let linkedProviders = Set(identities.map { $0.provider })
        return IdentityProvider.allCases.filter { !linkedProviders.contains($0) }
    }

    // Check if user has email identity (for password management)
    private var hasEmailIdentity: Bool {
        identities.contains { $0.provider == .email }
    }

    var body: some View {
        Group {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                List {
                    // Connected Accounts Section
                    Section {
                        ForEach(identities) { identity in
                            IdentityRow(identity: identity, themeManager: themeManager)
                                .swipeActions(edge: .trailing) {
                                    if !identity.isPrimary && identities.count > 1 {
                                        Button(role: .destructive) {
                                            identityToUnlink = identity
                                        } label: {
                                            Label("Unlink", systemImage: "link.badge.minus")
                                        }
                                    }
                                }
                        }
                    } header: {
                        Text("Connected Accounts")
                    } footer: {
                        Text("Your primary account cannot be unlinked. You must have at least one connected account.")
                    }

                    // Link New Account Section
                    if !linkableProviders.isEmpty {
                        Section("Link New Account") {
                            ForEach(linkableProviders, id: \.self) { provider in
                                Button {
                                    linkAccount(provider: provider)
                                } label: {
                                    HStack(spacing: 12) {
                                        Image(systemName: provider.icon)
                                            .font(.title3)
                                            .foregroundColor(providerColor(provider))
                                            .frame(width: 30)

                                        Text("Link \(provider.displayName)")
                                            .foregroundColor(.primary)

                                        Spacer()

                                        if isLinking {
                                            ProgressView()
                                        } else {
                                            Image(systemName: "plus.circle")
                                                .foregroundColor(themeManager.primaryColor)
                                        }
                                    }
                                }
                                .disabled(isLinking)
                            }
                        }
                    }

                    // Password Section (only if email identity exists)
                    if hasEmailIdentity {
                        Section("Security") {
                            Button {
                                showChangePassword = true
                            } label: {
                                HStack {
                                    Label("Change Password", systemImage: "key.fill")
                                        .foregroundColor(.primary)
                                    Spacer()
                                    Image(systemName: "chevron.right")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                            }
                        }
                    }
                }
            }
        }
        .navigationTitle("Connected Accounts")
        .onAppear {
            loadIdentities()
            analyticsService.trackScreenView(screenName: "connected_accounts")
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .alert("Unlink Account", isPresented: .init(
            get: { identityToUnlink != nil },
            set: { if !$0 { identityToUnlink = nil } }
        )) {
            Button("Cancel", role: .cancel) {
                identityToUnlink = nil
            }
            Button("Unlink", role: .destructive) {
                if let identity = identityToUnlink {
                    unlinkIdentity(identity)
                }
            }
        } message: {
            if let identity = identityToUnlink {
                Text("Are you sure you want to unlink \(identity.provider.displayName) (\(identity.displayIdentifier))? You won't be able to sign in with this account anymore.")
            }
        }
        .sheet(isPresented: $showChangePassword) {
            NavigationStack {
                ChangePasswordView(onSuccess: {
                    showChangePassword = false
                })
                .environmentObject(themeManager)
            }
        }
    }

    // MARK: - Provider Color

    private func providerColor(_ provider: IdentityProvider) -> Color {
        switch provider {
        case .email: return .blue
        case .google: return .red
        case .apple: return .primary
        case .phone: return .green
        }
    }

    // MARK: - Load Identities

    private func loadIdentities() {
        isLoading = true

        Task {
            do {
                let response: IdentityListResponse = try await APIClient.shared.get("/users/me/identities")
                await MainActor.run {
                    identities = response.identities
                    isLoading = false
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to load accounts"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }

    // MARK: - Unlink Identity

    private func unlinkIdentity(_ identity: UserIdentity) {
        Task {
            do {
                try await APIClient.shared.delete("/users/me/identities/\(identity.id)")
                await MainActor.run {
                    identityToUnlink = nil
                    loadIdentities()
                    analyticsService.trackEvent(name: "identity_unlinked", params: ["provider": identity.provider.rawValue])
                }
            } catch let error as APIError {
                await MainActor.run {
                    identityToUnlink = nil
                    errorMessage = error.errorDescription ?? "Failed to unlink account"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    identityToUnlink = nil
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }

    // MARK: - Link Account

    private func linkAccount(provider: IdentityProvider) {
        switch provider {
        case .google:
            linkGoogleAccount()
        case .apple:
            linkAppleAccount()
        case .email:
            // Would need a separate flow to add email/password
            break
        case .phone:
            // Would need a separate flow to verify phone
            break
        }
    }

    // MARK: - Link Google

    private func linkGoogleAccount() {
        guard let clientID = Bundle.main.object(forInfoDictionaryKey: "GIDClientID") as? String,
              !clientID.isEmpty,
              !clientID.hasPrefix("$(") else {
            errorMessage = "Google Sign-In is not configured."
            showError = true
            return
        }

        guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
              let rootViewController = windowScene.windows.first?.rootViewController else {
            errorMessage = "Unable to present Google Sign-In"
            showError = true
            return
        }

        isLinking = true

        GIDSignIn.sharedInstance.signIn(withPresenting: rootViewController) { result, error in
            if let error = error {
                DispatchQueue.main.async {
                    isLinking = false
                    if (error as NSError).code != GIDSignInError.canceled.rawValue {
                        errorMessage = error.localizedDescription
                        showError = true
                    }
                }
                return
            }

            guard let user = result?.user,
                  let idToken = user.idToken?.tokenString else {
                DispatchQueue.main.async {
                    isLinking = false
                    errorMessage = "Failed to get Google credentials"
                    showError = true
                }
                return
            }

            Task {
                do {
                    let request = LinkGoogleRequest(idToken: idToken)
                    let _: IdentityResponse = try await APIClient.shared.post("/users/me/identities/google", body: request)
                    await MainActor.run {
                        isLinking = false
                        loadIdentities()
                        analyticsService.trackEvent(name: "identity_linked", params: ["provider": "google"])
                    }
                } catch let error as APIError {
                    await MainActor.run {
                        isLinking = false
                        errorMessage = error.errorDescription ?? "Failed to link Google account"
                        showError = true
                    }
                } catch {
                    await MainActor.run {
                        isLinking = false
                        errorMessage = "An unexpected error occurred"
                        showError = true
                    }
                }
            }
        }
    }

    // MARK: - Link Apple (placeholder - requires full Sign in with Apple flow)

    private func linkAppleAccount() {
        // Apple Sign In requires the full ASAuthorizationController flow
        // For simplicity, show a message that it needs to be done from settings
        errorMessage = "To link Apple ID, please use Sign in with Apple from the login screen."
        showError = true
    }
}

// MARK: - Identity Row

struct IdentityRow: View {
    let identity: UserIdentity
    let themeManager: ThemeManager

    var body: some View {
        HStack(spacing: 12) {
            // Provider Icon
            ZStack {
                Circle()
                    .fill(providerColor.opacity(0.1))
                    .frame(width: 44, height: 44)

                Image(systemName: identity.provider.icon)
                    .font(.system(size: 20))
                    .foregroundColor(providerColor)
            }

            // Details
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(identity.provider.displayName)
                        .font(.headline)

                    if identity.isPrimary {
                        Text("Primary")
                            .font(.caption2)
                            .fontWeight(.medium)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(themeManager.primaryColor.opacity(0.1))
                            .foregroundColor(themeManager.primaryColor)
                            .cornerRadius(4)
                    }
                }

                Text(identity.displayIdentifier)
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }

            Spacer()
        }
        .padding(.vertical, 4)
    }

    private var providerColor: Color {
        switch identity.provider {
        case .email: return .blue
        case .google: return .red
        case .apple: return .primary
        case .phone: return .green
        }
    }
}

// MARK: - Change Password View

struct ChangePasswordView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss

    var onSuccess: () -> Void

    @State private var currentPassword = ""
    @State private var newPassword = ""
    @State private var confirmPassword = ""
    @State private var isSaving = false
    @State private var showError = false
    @State private var errorMessage = ""

    private var isValid: Bool {
        !newPassword.isEmpty &&
        newPassword.count >= 8 &&
        newPassword == confirmPassword
    }

    private var passwordError: String? {
        if newPassword.isEmpty { return nil }
        if newPassword.count < 8 { return "Password must be at least 8 characters" }
        if !confirmPassword.isEmpty && newPassword != confirmPassword { return "Passwords don't match" }
        return nil
    }

    var body: some View {
        Form {
            Section {
                SecureField("Current Password (if set)", text: $currentPassword)
                    .textContentType(.password)
            } footer: {
                Text("Leave blank if you don't have a password set yet.")
            }

            Section {
                SecureField("New Password", text: $newPassword)
                    .textContentType(.newPassword)

                SecureField("Confirm Password", text: $confirmPassword)
                    .textContentType(.newPassword)
            } footer: {
                if let error = passwordError {
                    Text(error)
                        .foregroundColor(.red)
                } else {
                    Text("Password must be at least 8 characters.")
                }
            }
        }
        .navigationTitle("Change Password")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") {
                    dismiss()
                }
            }

            ToolbarItem(placement: .confirmationAction) {
                Button {
                    changePassword()
                } label: {
                    if isSaving {
                        ProgressView()
                    } else {
                        Text("Save")
                    }
                }
                .disabled(!isValid || isSaving)
            }
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
    }

    private func changePassword() {
        isSaving = true

        Task {
            do {
                let request = ChangePasswordRequest(
                    currentPassword: currentPassword.isEmpty ? nil : currentPassword,
                    newPassword: newPassword
                )
                let _: ChangePasswordResponse = try await APIClient.shared.put("/users/me/password", body: request)
                await MainActor.run {
                    isSaving = false
                    onSuccess()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSaving = false
                    errorMessage = error.errorDescription ?? "Failed to change password"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isSaving = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }
}

#Preview {
    NavigationStack {
        ConnectedAccountsView()
    }
    .withThemeManager()
}
