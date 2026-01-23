//
//  LoginView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import FirebaseCrashlytics

// MARK: - Login Request/Response Models

struct LoginRequest: Encodable {
    let orgSlug: String
    let email: String
    let password: String
}

struct LoginResponse: Decodable {
    let token: String
    let user: LoginUser?
}

struct LoginUser: Decodable {
    let id: String
    let email: String
    let firstName: String?
    let lastName: String?
    let role: String?
}

// MARK: - Login View

struct LoginView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var email = ""
    @State private var password = ""
    @State private var isLoading = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var emailError: String?

    var onLoginSuccess: () -> Void
    var onNavigateToRegister: (() -> Void)?

    var body: some View {
        GeometryReader { geometry in
            ScrollView {
                VStack(spacing: 24) {
                    Spacer()
                        .frame(height: geometry.size.height * 0.1)

                    // Logo and Title
                    VStack(spacing: 16) {
                        Image(systemName: "pawprint.fill")
                            .font(.system(size: 64))
                            .foregroundColor(themeManager.primaryColor)

                        Text(themeManager.branding.companyName)
                            .font(.largeTitle)
                            .fontWeight(.bold)
                            .foregroundColor(themeManager.primaryColor)

                        Text("Sign in to continue")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.bottom, 32)

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
                            .onChange(of: email) { _ in
                                validateEmailWithDebounce()
                            }

                        if let error = emailError {
                            Text(error)
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                    }

                    // Password Field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Password")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        SecureField("Enter your password", text: $password)
                            .textFieldStyle(.plain)
                            .textContentType(.password)
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

                    // Login Button
                    Button(action: login) {
                        HStack {
                            if isLoading {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Text("Sign In")
                                    .fontWeight(.semibold)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(
                            RoundedRectangle(cornerRadius: 12)
                                .fill(isLoginEnabled ? themeManager.primaryColor : Color.gray)
                        )
                        .foregroundColor(.white)
                    }
                    .disabled(!isLoginEnabled || isLoading)
                    .padding(.top, 16)

                    // Register Link
                    if let onNavigateToRegister = onNavigateToRegister {
                        HStack {
                            Text("Don't have an account?")
                                .font(.subheadline)
                                .foregroundColor(.secondary)

                            Button(action: onNavigateToRegister) {
                                Text("Create Account")
                                    .font(.subheadline)
                                    .fontWeight(.semibold)
                                    .foregroundColor(themeManager.primaryColor)
                            }
                        }
                        .padding(.top, 8)
                    }

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
        .alert("Login Failed", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "login")
        }
    }

    // MARK: - Computed Properties

    private var isLoginEnabled: Bool {
        !email.trimmingCharacters(in: .whitespaces).isEmpty &&
        !password.isEmpty &&
        Validators.isValidEmail(email)
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

    private func login() {
        validateEmail()
        guard isLoginEnabled, emailError == nil else { return }

        isLoading = true

        Task {
            do {
                // TODO: Make org_slug configurable or derive from app configuration
                let orgSlug = ProcessInfo.processInfo.environment["ORG_SLUG"] ?? "demo"
                let request = LoginRequest(orgSlug: orgSlug, email: email.trimmingCharacters(in: .whitespaces), password: password)
                let response: LoginResponse = try await APIClient.shared.post("/auth/login", body: request)

                await APIClient.shared.setAuthToken(response.token)

                // Save user to session
                if let loginUser = response.user {
                    let role = UserRole(rawValue: loginUser.role ?? "customer") ?? .customer
                    let user = User(
                        id: loginUser.id,
                        email: loginUser.email,
                        firstName: loginUser.firstName,
                        lastName: loginUser.lastName,
                        role: role
                    )
                    await MainActor.run {
                        UserSession.shared.setUser(user)
                    }

                    if FirebaseState.isConfigured {
                        Crashlytics.crashlytics().setUserID(loginUser.id)
                    }
                }

                await MainActor.run {
                    isLoading = false
                    onLoginSuccess()
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
    LoginView(onLoginSuccess: {
        print("Login successful!")
    })
    .withThemeManager()
}
