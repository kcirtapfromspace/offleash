//
//  RegisterView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import FirebaseCrashlytics

// MARK: - Registration Request/Response Models

struct RegisterRequest: Encodable {
    let firstName: String
    let lastName: String
    let email: String
    let phone: String
    let password: String
}

struct RegisterResponse: Decodable {
    let token: String
    let user: RegisterUser?
}

struct RegisterUser: Decodable {
    let id: String
    let email: String
    let firstName: String?
    let lastName: String?
}

// MARK: - Register View

struct RegisterView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var firstName = ""
    @State private var lastName = ""
    @State private var email = ""
    @State private var phone = ""
    @State private var password = ""
    @State private var isLoading = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var emailError: String?

    var onRegisterSuccess: () -> Void
    var onNavigateToLogin: () -> Void

    var body: some View {
        GeometryReader { geometry in
            ScrollView {
                VStack(spacing: 24) {
                    Spacer()
                        .frame(height: geometry.size.height * 0.05)

                    // Logo and Title
                    VStack(spacing: 16) {
                        Image(systemName: "pawprint.fill")
                            .font(.system(size: 64))
                            .foregroundColor(themeManager.primaryColor)

                        Text(themeManager.branding.companyName)
                            .font(.largeTitle)
                            .fontWeight(.bold)
                            .foregroundColor(themeManager.primaryColor)

                        Text("Create your account")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.bottom, 24)

                    // First Name Field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("First Name")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        TextField("Enter your first name", text: $firstName)
                            .textFieldStyle(.plain)
                            .textContentType(.givenName)
                            .autocapitalization(.words)
                            .autocorrectionDisabled()
                            .padding()
                            .background(
                                RoundedRectangle(cornerRadius: 12)
                                    .fill(Color(.systemGray6))
                            )
                            .overlay(
                                RoundedRectangle(cornerRadius: 12)
                                    .stroke(Color(.systemGray4), lineWidth: 1)
                            )
                    }

                    // Last Name Field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Last Name")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        TextField("Enter your last name", text: $lastName)
                            .textFieldStyle(.plain)
                            .textContentType(.familyName)
                            .autocapitalization(.words)
                            .autocorrectionDisabled()
                            .padding()
                            .background(
                                RoundedRectangle(cornerRadius: 12)
                                    .fill(Color(.systemGray6))
                            )
                            .overlay(
                                RoundedRectangle(cornerRadius: 12)
                                    .stroke(Color(.systemGray4), lineWidth: 1)
                            )
                    }

                    // Email Field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Email")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        TextField("Enter your email", text: $email)
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
                            .onChange(of: email) { _, _ in
                                validateEmailWithDebounce()
                            }

                        if let error = emailError {
                            Text(error)
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                    }

                    // Phone Field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Phone")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        TextField("Enter your phone number", text: $phone)
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
                                    .stroke(Color(.systemGray4), lineWidth: 1)
                            )
                    }

                    // Password Field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Password")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        SecureField("Create a password", text: $password)
                            .textFieldStyle(.plain)
                            .textContentType(.newPassword)
                            .padding()
                            .background(
                                RoundedRectangle(cornerRadius: 12)
                                    .fill(Color(.systemGray6))
                            )
                            .overlay(
                                RoundedRectangle(cornerRadius: 12)
                                    .stroke(Color(.systemGray4), lineWidth: 1)
                            )

                        PasswordRequirements(password: password)
                            .padding(.top, 4)
                    }

                    // Register Button
                    Button(action: register) {
                        HStack {
                            if isLoading {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Text("Create Account")
                                    .fontWeight(.semibold)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(
                            RoundedRectangle(cornerRadius: 12)
                                .fill(isRegisterEnabled ? themeManager.primaryColor : Color.gray)
                        )
                        .foregroundColor(.white)
                    }
                    .disabled(!isRegisterEnabled || isLoading)
                    .padding(.top, 16)

                    // Login Link
                    HStack {
                        Text("Already have an account?")
                            .font(.subheadline)
                            .foregroundColor(.secondary)

                        Button(action: onNavigateToLogin) {
                            Text("Sign In")
                                .font(.subheadline)
                                .fontWeight(.semibold)
                                .foregroundColor(themeManager.primaryColor)
                        }
                    }
                    .padding(.top, 8)

                    Spacer()

                    // Support Email
                    VStack(spacing: 4) {
                        Text("Need help?")
                            .font(.footnote)
                            .foregroundColor(.secondary)

                        Text(themeManager.branding.supportEmail)
                            .font(.footnote)
                            .foregroundColor(themeManager.accentColor)
                    }
                    .padding(.bottom, 32)
                }
                .padding(.horizontal, 24)
                .frame(minHeight: geometry.size.height)
            }
        }
        .alert("Registration Failed", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "register")
        }
    }

    // MARK: - Computed Properties

    private var isRegisterEnabled: Bool {
        !firstName.trimmingCharacters(in: .whitespaces).isEmpty &&
        !lastName.trimmingCharacters(in: .whitespaces).isEmpty &&
        !email.trimmingCharacters(in: .whitespaces).isEmpty &&
        Validators.isValidEmail(email) &&
        !phone.trimmingCharacters(in: .whitespaces).isEmpty &&
        passwordRequirementsMet
    }

    private var passwordRequirementsMet: Bool {
        password.count >= 8 &&
        password.contains(where: { $0.isUppercase }) &&
        password.contains(where: { $0.isNumber })
    }

    // MARK: - Validation

    @State private var emailValidationTask: Task<Void, Never>?

    private func validateEmailWithDebounce() {
        emailValidationTask?.cancel()
        emailValidationTask = Task {
            try? await Task.sleep(nanoseconds: 300_000_000) // 300ms debounce
            guard !Task.isCancelled else { return }
            await MainActor.run {
                validateEmail()
            }
        }
    }

    private func validateEmail() {
        let trimmedEmail = email.trimmingCharacters(in: .whitespaces)
        if trimmedEmail.isEmpty {
            emailError = nil
        } else if !Validators.isValidEmail(trimmedEmail) {
            emailError = "Please enter a valid email address"
        } else {
            emailError = nil
        }
    }

    // MARK: - Actions

    private func register() {
        validateEmail()
        guard isRegisterEnabled, emailError == nil else { return }

        isLoading = true

        Task {
            do {
                let request = RegisterRequest(
                    firstName: firstName.trimmingCharacters(in: .whitespaces),
                    lastName: lastName.trimmingCharacters(in: .whitespaces),
                    email: email.trimmingCharacters(in: .whitespaces),
                    phone: phone.trimmingCharacters(in: .whitespaces),
                    password: password
                )
                let response: RegisterResponse = try await APIClient.shared.post("/auth/register", body: request)

                await APIClient.shared.setAuthToken(response.token)

                if let userId = response.user?.id {
                    Crashlytics.crashlytics().setUserID(userId)
                }

                await MainActor.run {
                    isLoading = false
                    onRegisterSuccess()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "An unexpected error occurred"
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

// MARK: - Preview

#Preview {
    RegisterView(
        onRegisterSuccess: {
            print("Registration successful!")
        },
        onNavigateToLogin: {
            print("Navigate to login")
        }
    )
    .withThemeManager()
}
