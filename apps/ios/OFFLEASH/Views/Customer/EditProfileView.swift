//
//  EditProfileView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct UpdateProfileRequest: Codable {
    let firstName: String?
    let lastName: String?
    let phone: String?
}

struct EditProfileView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss
    @ObservedObject private var userSession = UserSession.shared

    @State private var firstName: String = ""
    @State private var lastName: String = ""
    @State private var phone: String = ""

    @State private var isLoading = false
    @State private var isSaving = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showSuccess = false
    @State private var showToast = false
    @State private var toastMessage = ""

    private var isAuthMockMode: Bool {
        TestAuthMode.isMock
    }

    var body: some View {
        Form {
            Section("Personal Information") {
                TextField("First Name", text: $firstName)
                    .textContentType(.givenName)
                    .autocapitalization(.words)
                    .accessibilityIdentifier("profile-name-field")

                TextField("Last Name", text: $lastName)
                    .textContentType(.familyName)
                    .autocapitalization(.words)
                    .accessibilityIdentifier("profile-last-name-field")
            }

            Section("Contact") {
                TextField("Phone Number", text: $phone)
                    .textContentType(.telephoneNumber)
                    .keyboardType(.phonePad)
                    .accessibilityIdentifier("profile-phone-field")
            }

            Section {
                HStack {
                    Text("Email")
                        .foregroundColor(.secondary)
                    Spacer()
                    Text(userSession.currentUser?.email ?? "")
                        .foregroundColor(.secondary)
                }
            } footer: {
                Text("Email cannot be changed. Contact support if you need to update your email.")
                    .font(.caption)
            }

            Section {
                Button {
                    saveProfile()
                } label: {
                    HStack {
                        Spacer()
                        if isSaving {
                            ProgressView()
                                .tint(.white)
                        } else {
                            Text("Save Changes")
                                .fontWeight(.semibold)
                        }
                        Spacer()
                    }
                }
                .listRowBackground(hasChanges ? themeManager.primaryColor : Color(.systemGray4))
                .foregroundColor(.white)
                .disabled(!hasChanges || isSaving)
                .accessibilityIdentifier("profile-save-button")
            }
        }
        .navigationTitle("Edit Profile")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            loadCurrentValues()
            analyticsService.trackScreenView(screenName: "edit_profile")
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .alert("Success", isPresented: $showSuccess) {
            Button("OK") {
                dismiss()
            }
        } message: {
            Text("Your profile has been updated.")
        }
        .overlay(alignment: .top) {
            if showToast {
                ToastBanner(message: toastMessage)
                    .padding(.top, 8)
                    .transition(.move(edge: .top).combined(with: .opacity))
            }
        }
    }

    private var hasChanges: Bool {
        let user = userSession.currentUser
        return firstName != (user?.firstName ?? "") ||
               lastName != (user?.lastName ?? "") ||
               phone != (user?.phone ?? "")
    }

    private func loadCurrentValues() {
        if let user = userSession.currentUser {
            firstName = user.firstName ?? ""
            lastName = user.lastName ?? ""
            phone = user.phone ?? ""
        }
    }

    private func saveProfile() {
        isSaving = true

        let request = UpdateProfileRequest(
            firstName: firstName.isEmpty ? nil : firstName,
            lastName: lastName.isEmpty ? nil : lastName,
            phone: phone.isEmpty ? nil : phone
        )

        Task {
            do {
                if isAuthMockMode {
                    let existing = userSession.currentUser
                    let updated = User(
                        id: existing?.id ?? "test-user",
                        email: existing?.email ?? "test-customer@offleash.test",
                        firstName: request.firstName ?? existing?.firstName,
                        lastName: request.lastName ?? existing?.lastName,
                        phone: request.phone ?? existing?.phone,
                        role: existing?.role ?? .customer,
                        organizationId: existing?.organizationId
                    )
                    await MainActor.run {
                        userSession.setUser(updated)
                        isSaving = false
                        toastMessage = "Profile updated"
                        showToast = true
                        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
                            showToast = false
                            dismiss()
                        }
                    }
                    return
                }

                let updatedUser: User = try await APIClient.shared.put("/users/me", body: request)

                await MainActor.run {
                    userSession.setUser(updatedUser)
                    isSaving = false
                    showSuccess = true
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSaving = false
                    errorMessage = error.errorDescription ?? "Failed to update profile"
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
        EditProfileView()
    }
    .withThemeManager()
}

// MARK: - Toast Banner

struct ToastBanner: View {
    let message: String

    var body: some View {
        Text(message)
            .font(.footnote)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(Color.black.opacity(0.85))
            .foregroundColor(.white)
            .cornerRadius(8)
            .accessibilityIdentifier("toast-message")
    }
}
