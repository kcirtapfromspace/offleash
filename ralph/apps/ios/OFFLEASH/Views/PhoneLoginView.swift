//
//  PhoneLoginView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Phone Auth Request/Response Models

struct PhoneSendCodeRequest: Encodable {
    let phone: String
    let orgSlug: String
}

struct PhoneSendCodeResponse: Decodable {
    let success: Bool
    let message: String?
    let expiresAt: Date?
}

struct PhoneVerifyRequest: Encodable {
    let phone: String
    let code: String
    let orgSlug: String
    let role: String?
}

struct PhoneVerifyResponse: Decodable {
    let token: String
    let user: PhoneAuthUser
}

struct PhoneAuthUser: Decodable {
    let id: String
    let email: String?
    let phone: String
    let firstName: String?
    let lastName: String?
    let role: String?
    let organizationId: String?
}

// MARK: - Phone Login View

struct PhoneLoginView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    @State private var phoneNumber = ""
    @State private var isLoading = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var phoneError: String?
    @State private var navigateToOTP = false
    @State private var countryCode = "+1"

    let selectedRole: SelectedRole
    var onLoginSuccess: () -> Void
    var onBack: (() -> Void)?

    // Country codes for picker
    private let countryCodes = [
        ("+1", "US/CA"),
        ("+44", "UK"),
        ("+61", "AU"),
        ("+49", "DE"),
        ("+33", "FR"),
        ("+81", "JP"),
        ("+91", "IN"),
        ("+86", "CN")
    ]

    private var formattedPhone: String {
        // Format to E.164
        let digits = phoneNumber.filter { $0.isNumber }
        return countryCode + digits
    }

    private var isPhoneValid: Bool {
        let digits = phoneNumber.filter { $0.isNumber }
        return digits.count >= 10
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
                        Image(systemName: "phone.fill")
                            .font(.system(size: 64))
                            .foregroundColor(themeManager.primaryColor)

                        Text("Sign in with Phone")
                            .font(.largeTitle)
                            .fontWeight(.bold)
                            .foregroundColor(themeManager.primaryColor)

                        Text("We'll send you a verification code")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                    .padding(.bottom, 24)

                    // Phone Number Input
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Phone Number")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        HStack(spacing: 8) {
                            // Country Code Picker
                            Menu {
                                ForEach(countryCodes, id: \.0) { code, label in
                                    Button {
                                        countryCode = code
                                    } label: {
                                        Text("\(code) (\(label))")
                                    }
                                }
                            } label: {
                                HStack(spacing: 4) {
                                    Text(countryCode)
                                        .foregroundColor(.primary)
                                    Image(systemName: "chevron.down")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
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

                            // Phone Number Field
                            TextField("(555) 555-5555", text: $phoneNumber)
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
                                .onChange(of: phoneNumber) { _ in
                                    validatePhone()
                                }
                        }

                        if let error = phoneError {
                            Text(error)
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                    }

                    // Info Text
                    HStack {
                        Image(systemName: "info.circle")
                            .foregroundColor(.secondary)
                        Text("Standard SMS rates may apply")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .padding(.top, 4)

                    // Continue Button
                    Button {
                        sendCode()
                    } label: {
                        HStack {
                            if isLoading {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Text("Send Code")
                                    .fontWeight(.semibold)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(
                            RoundedRectangle(cornerRadius: 12)
                                .fill(isPhoneValid ? themeManager.primaryColor : Color.gray)
                        )
                        .foregroundColor(.white)
                    }
                    .disabled(!isPhoneValid || isLoading)
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
        .navigationDestination(isPresented: $navigateToOTP) {
            OTPVerificationView(
                phoneNumber: formattedPhone,
                selectedRole: selectedRole,
                onVerificationSuccess: onLoginSuccess,
                onBack: { navigateToOTP = false }
            )
            .environmentObject(themeManager)
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "phone_login")
        }
    }

    // MARK: - Validation

    private func validatePhone() {
        let digits = phoneNumber.filter { $0.isNumber }
        if digits.isEmpty {
            phoneError = nil
        } else if digits.count < 10 {
            phoneError = "Please enter a valid phone number"
        } else {
            phoneError = nil
        }
    }

    // MARK: - Send Code

    private func sendCode() {
        guard isPhoneValid, phoneError == nil else { return }

        isLoading = true

        Task {
            do {
                let orgSlug = ProcessInfo.processInfo.environment["ORG_SLUG"] ?? "demo"
                let request = PhoneSendCodeRequest(phone: formattedPhone, orgSlug: orgSlug)
                let _: PhoneSendCodeResponse = try await APIClient.shared.post("/auth/phone/send-code", body: request)

                await MainActor.run {
                    isLoading = false
                    analyticsService.trackEvent(name: "phone_code_sent", params: nil)
                    navigateToOTP = true
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    if case .httpError(let statusCode, _) = error, statusCode == 429 {
                        errorMessage = "Too many requests. Please wait before trying again."
                    } else {
                        errorMessage = error.errorDescription ?? "Failed to send code"
                    }
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

#Preview {
    NavigationStack {
        PhoneLoginView(
            selectedRole: .customer,
            onLoginSuccess: { print("Success") },
            onBack: { print("Back") }
        )
    }
    .withThemeManager()
}
