//
//  InviteWalkerView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Invite Request/Response Models

struct InviteWalkerRequest: Encodable {
    let email: String?
    let phone: String?
}

struct InviteWalkerResponse: Decodable {
    let success: Bool
    let message: String?
}

// MARK: - Invite Walker View

struct InviteWalkerView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss

    @State private var contactMethod: ContactMethod = .email
    @State private var email = ""
    @State private var phone = ""
    @State private var isLoading = false
    @State private var showSuccess = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var emailError: String?
    @State private var phoneError: String?

    enum ContactMethod: String, CaseIterable {
        case email = "Email"
        case phone = "Text Message"
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Header
                    VStack(spacing: 12) {
                        Image(systemName: "person.badge.plus")
                            .font(.system(size: 48))
                            .foregroundColor(themeManager.primaryColor)

                        Text("Invite a Walker")
                            .font(.title2)
                            .fontWeight(.bold)

                        Text("Send an invitation to join your team. They'll receive a link to sign up and will automatically be added to your organization.")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding(.top, 24)

                    // Contact Method Picker
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Send invitation via")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        Picker("Contact Method", selection: $contactMethod) {
                            ForEach(ContactMethod.allCases, id: \.self) { method in
                                Text(method.rawValue).tag(method)
                            }
                        }
                        .pickerStyle(.segmented)
                    }

                    // Input Field
                    if contactMethod == .email {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Email Address")
                                .font(.subheadline)
                                .fontWeight(.medium)
                                .foregroundColor(.secondary)

                            TextField("Enter their email", text: $email)
                                .textFieldStyle(.plain)
                                .keyboardType(.emailAddress)
                                .textContentType(.emailAddress)
                                .autocapitalization(.none)
                                .autocorrectionDisabled()
                                .padding()
                                .background(
                                    RoundedRectangle(cornerRadius: 12)
                                        .fill(Color(.systemGray6))
                                )
                                .overlay(
                                    RoundedRectangle(cornerRadius: 12)
                                        .stroke(emailError != nil ? Color.red : Color(.systemGray4), lineWidth: 1)
                                )
                                .onChange(of: email) { _ in
                                    validateEmail()
                                }

                            if let error = emailError {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                        }
                    } else {
                        VStack(alignment: .leading, spacing: 8) {
                            Text("Phone Number")
                                .font(.subheadline)
                                .fontWeight(.medium)
                                .foregroundColor(.secondary)

                            TextField("Enter their phone number", text: $phone)
                                .textFieldStyle(.plain)
                                .keyboardType(.phonePad)
                                .textContentType(.telephoneNumber)
                                .padding()
                                .background(
                                    RoundedRectangle(cornerRadius: 12)
                                        .fill(Color(.systemGray6))
                                )
                                .overlay(
                                    RoundedRectangle(cornerRadius: 12)
                                        .stroke(phoneError != nil ? Color.red : Color(.systemGray4), lineWidth: 1)
                                )
                                .onChange(of: phone) { _ in
                                    validatePhone()
                                }

                            if let error = phoneError {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                        }
                    }

                    // Send Button
                    Button(action: sendInvite) {
                        HStack {
                            if isLoading {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Image(systemName: "paperplane.fill")
                                Text("Send Invitation")
                                    .fontWeight(.semibold)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(
                            RoundedRectangle(cornerRadius: 12)
                                .fill(isSendEnabled ? themeManager.primaryColor : Color.gray)
                        )
                        .foregroundColor(.white)
                    }
                    .disabled(!isSendEnabled || isLoading)
                    .padding(.top, 8)

                    // Info text
                    VStack(spacing: 8) {
                        HStack(spacing: 8) {
                            Image(systemName: "clock")
                                .foregroundColor(.secondary)
                            Text("Invitation expires in 7 days")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        HStack(spacing: 8) {
                            Image(systemName: "checkmark.shield")
                                .foregroundColor(.secondary)
                            Text("They'll need to complete sign up to join")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(.top, 16)

                    Spacer()
                }
                .padding(.horizontal, 24)
            }
            .navigationTitle("Invite Walker")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .alert("Invitation Sent!", isPresented: $showSuccess) {
                Button("OK") {
                    dismiss()
                }
            } message: {
                if contactMethod == .email {
                    Text("We've sent an invitation to \(email). They'll receive a link to sign up and join your team.")
                } else {
                    Text("We've sent an invitation to \(phone). They'll receive a link to sign up and join your team.")
                }
            }
            .alert("Error", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage)
            }
        }
    }

    // MARK: - Computed Properties

    private var isSendEnabled: Bool {
        if contactMethod == .email {
            return !email.trimmingCharacters(in: .whitespaces).isEmpty &&
                   Validators.isValidEmail(email) &&
                   emailError == nil
        } else {
            return !phone.trimmingCharacters(in: .whitespaces).isEmpty &&
                   phone.count >= 10 &&
                   phoneError == nil
        }
    }

    // MARK: - Validation

    private func validateEmail() {
        let trimmed = email.trimmingCharacters(in: .whitespaces)
        if trimmed.isEmpty {
            emailError = nil
        } else if !Validators.isValidEmail(trimmed) {
            emailError = "Please enter a valid email address"
        } else {
            emailError = nil
        }
    }

    private func validatePhone() {
        let digits = phone.filter { $0.isNumber }
        if phone.isEmpty {
            phoneError = nil
        } else if digits.count < 10 {
            phoneError = "Please enter a valid phone number"
        } else {
            phoneError = nil
        }
    }

    // MARK: - Actions

    private func sendInvite() {
        guard isSendEnabled else { return }

        isLoading = true

        Task {
            do {
                let request = InviteWalkerRequest(
                    email: contactMethod == .email ? email.trimmingCharacters(in: .whitespaces) : nil,
                    phone: contactMethod == .phone ? phone.trimmingCharacters(in: .whitespaces) : nil
                )

                let _: InviteWalkerResponse = try await APIClient.shared.post("/walker/invite", body: request)

                await MainActor.run {
                    isLoading = false
                    showSuccess = true
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to send invitation"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    errorMessage = "Failed to send invitation. Please try again."
                    showError = true
                }
            }
        }
    }
}

// MARK: - Preview

#Preview {
    InviteWalkerView()
        .withThemeManager()
}
