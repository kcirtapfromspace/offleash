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

    var body: some View {
        Form {
            Section("Personal Information") {
                TextField("First Name", text: $firstName)
                    .textContentType(.givenName)
                    .autocapitalization(.words)

                TextField("Last Name", text: $lastName)
                    .textContentType(.familyName)
                    .autocapitalization(.words)
            }

            Section("Contact") {
                TextField("Phone Number", text: $phone)
                    .textContentType(.telephoneNumber)
                    .keyboardType(.phonePad)
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
