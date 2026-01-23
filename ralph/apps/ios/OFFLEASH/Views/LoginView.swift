//
//  LoginView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import AuthenticationServices
import FirebaseCrashlytics
import GoogleSignIn

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
    let organizationId: String?
}

// MARK: - Login View

struct LoginView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var email = ""
    @State private var password = ""
    @State private var isLoading = false
    @State private var isOAuthLoading = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var emailError: String?

    let selectedRole: SelectedRole
    var onLoginSuccess: () -> Void
    var onNavigateToRegister: (() -> Void)?
    var onBack: (() -> Void)?

    private var roleDisplayText: String {
        switch selectedRole {
        case .customer:
            return "Sign in to book walks"
        case .walker:
            return "Sign in to walk dogs"
        }
    }

    var body: some View {
        GeometryReader { geometry in
            ScrollView {
                VStack(spacing: 24) {
                    // Back Button
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
                        .padding(.top, 16)
                    } else {
                        Spacer()
                            .frame(height: geometry.size.height * 0.08)
                    }

                    // Logo and Title
                    VStack(spacing: 16) {
                        Image(systemName: "pawprint.fill")
                            .font(.system(size: 64))
                            .foregroundColor(themeManager.primaryColor)

                        Text(themeManager.branding.companyName)
                            .font(.largeTitle)
                            .fontWeight(.bold)
                            .foregroundColor(themeManager.primaryColor)

                        Text(roleDisplayText)
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.bottom, 24)

                    // OAuth Buttons
                    VStack(spacing: 12) {
                        // Sign in with Apple
                        SignInWithAppleButton(
                            .signIn,
                            onRequest: { request in
                                request.requestedScopes = [.email, .fullName]
                            },
                            onCompletion: handleAppleSignIn
                        )
                        .signInWithAppleButtonStyle(.black)
                        .frame(height: 50)
                        .cornerRadius(12)
                        .disabled(isLoading || isOAuthLoading)

                        // Google Sign In Button (styled)
                        Button(action: signInWithGoogle) {
                            HStack(spacing: 12) {
                                Image(systemName: "g.circle.fill")
                                    .font(.title2)
                                Text("Sign in with Google")
                                    .fontWeight(.medium)
                            }
                            .frame(maxWidth: .infinity)
                            .frame(height: 50)
                            .background(Color.white)
                            .foregroundColor(.black)
                            .cornerRadius(12)
                            .overlay(
                                RoundedRectangle(cornerRadius: 12)
                                    .stroke(Color(.systemGray3), lineWidth: 1)
                            )
                        }
                        .disabled(isLoading || isOAuthLoading)
                    }

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
                    .padding(.vertical, 8)

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
                    .disabled(!isLoginEnabled || isLoading || isOAuthLoading)
                    .padding(.top, 8)

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
        .overlay {
            if isOAuthLoading {
                Color.black.opacity(0.3)
                    .ignoresSafeArea()
                ProgressView()
                    .scaleEffect(1.5)
                    .tint(.white)
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
                        role: role,
                        organizationId: loginUser.organizationId
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

    // MARK: - OAuth Actions

    private func handleAppleSignIn(_ result: Result<ASAuthorization, Error>) {
        switch result {
        case .success(let authorization):
            guard let appleIDCredential = authorization.credential as? ASAuthorizationAppleIDCredential,
                  let identityTokenData = appleIDCredential.identityToken,
                  let identityToken = String(data: identityTokenData, encoding: .utf8) else {
                errorMessage = "Invalid credentials received from Apple"
                showError = true
                return
            }

            // Get name (only available on first sign-in)
            let firstName = appleIDCredential.fullName?.givenName
            let lastName = appleIDCredential.fullName?.familyName

            isOAuthLoading = true

            Task {
                do {
                    let orgSlug = ProcessInfo.processInfo.environment["ORG_SLUG"] ?? "demo"
                    let request = OAuthAppleRequest(
                        orgSlug: orgSlug,
                        idToken: identityToken,
                        firstName: firstName,
                        lastName: lastName
                    )

                    let response: OAuthResponse = try await APIClient.shared.post("/auth/apple", body: request)
                    await APIClient.shared.setAuthToken(response.token)

                    // Save user to session
                    let role = UserRole(rawValue: response.user.role ?? "customer") ?? .customer
                    let user = User(
                        id: response.user.id,
                        email: response.user.email,
                        firstName: response.user.firstName,
                        lastName: response.user.lastName,
                        role: role,
                        organizationId: response.user.organizationId
                    )
                    await MainActor.run {
                        UserSession.shared.setUser(user)
                        isOAuthLoading = false
                        analyticsService.trackEvent(name: "login_success", params: ["method": "apple"])
                        onLoginSuccess()
                    }
                } catch let error as APIError {
                    await MainActor.run {
                        isOAuthLoading = false
                        errorMessage = error.errorDescription ?? "Apple Sign-In failed"
                        showError = true
                    }
                } catch {
                    await MainActor.run {
                        isOAuthLoading = false
                        errorMessage = "Apple Sign-In failed. Please try again."
                        showError = true
                    }
                }
            }

        case .failure(let error):
            // User cancelled - don't show error
            if (error as NSError).code == ASAuthorizationError.canceled.rawValue {
                return
            }
            errorMessage = error.localizedDescription
            showError = true
        }
    }

    private func signInWithGoogle() {
        // Check if Google Sign-In is configured
        guard let clientID = Bundle.main.object(forInfoDictionaryKey: "GIDClientID") as? String,
              !clientID.isEmpty,
              !clientID.hasPrefix("$(") else {
            errorMessage = "Google Sign-In is not configured. Please use Apple Sign-In or email/password."
            showError = true
            return
        }

        // Get the root view controller for presenting the sign-in flow
        guard let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
              let rootViewController = windowScene.windows.first?.rootViewController else {
            errorMessage = "Unable to present Google Sign-In"
            showError = true
            return
        }

        isOAuthLoading = true

        GIDSignIn.sharedInstance.signIn(withPresenting: rootViewController) { result, error in
            if let error = error {
                DispatchQueue.main.async {
                    isOAuthLoading = false
                    // Check if user cancelled
                    if (error as NSError).code == GIDSignInError.canceled.rawValue {
                        return
                    }
                    errorMessage = error.localizedDescription
                    showError = true
                }
                return
            }

            guard let user = result?.user,
                  let idToken = user.idToken?.tokenString else {
                DispatchQueue.main.async {
                    isOAuthLoading = false
                    errorMessage = "Failed to get Google credentials"
                    showError = true
                }
                return
            }

            // Send the ID token to the backend
            Task {
                do {
                    let orgSlug = ProcessInfo.processInfo.environment["ORG_SLUG"] ?? "demo"
                    let request = OAuthGoogleRequest(
                        orgSlug: orgSlug,
                        idToken: idToken
                    )

                    let response: OAuthResponse = try await APIClient.shared.post("/auth/google", body: request)
                    await APIClient.shared.setAuthToken(response.token)

                    // Save user to session
                    let role = UserRole(rawValue: response.user.role ?? "customer") ?? .customer
                    let sessionUser = User(
                        id: response.user.id,
                        email: response.user.email,
                        firstName: response.user.firstName,
                        lastName: response.user.lastName,
                        role: role,
                        organizationId: response.user.organizationId
                    )

                    await MainActor.run {
                        UserSession.shared.setUser(sessionUser)
                        isOAuthLoading = false
                        analyticsService.trackEvent(name: "login_success", params: ["method": "google"])
                        onLoginSuccess()
                    }
                } catch let error as APIError {
                    await MainActor.run {
                        isOAuthLoading = false
                        errorMessage = error.errorDescription ?? "Google Sign-In failed"
                        showError = true
                    }
                } catch {
                    await MainActor.run {
                        isOAuthLoading = false
                        errorMessage = "Google Sign-In failed. Please try again."
                        showError = true
                    }
                }
            }
        }
    }
}

// MARK: - Preview

#Preview {
    LoginView(
        selectedRole: .customer,
        onLoginSuccess: {
            print("Login successful!")
        },
        onBack: {
            print("Back to role selection")
        }
    )
    .withThemeManager()
}
