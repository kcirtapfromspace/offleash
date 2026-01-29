//
//  OTPVerificationView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import FirebaseCrashlytics

struct OTPVerificationView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    let phoneNumber: String
    let selectedRole: SelectedRole
    var onVerificationSuccess: () -> Void
    var onBack: (() -> Void)?

    @State private var code = ""
    @State private var isLoading = false
    @State private var isResending = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var resendCooldown = 60
    @State private var canResend = false

    @FocusState private var isCodeFieldFocused: Bool

    private let codeLength = 6
    private let timer = Timer.publish(every: 1, on: .main, in: .common).autoconnect()

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

                    // Icon and Title
                    VStack(spacing: 16) {
                        Image(systemName: "lock.shield.fill")
                            .font(.system(size: 64))
                            .foregroundColor(themeManager.primaryColor)

                        Text("Verify Your Phone")
                            .font(.largeTitle)
                            .fontWeight(.bold)
                            .foregroundColor(themeManager.primaryColor)

                        Text("Enter the 6-digit code sent to")
                            .font(.subheadline)
                            .foregroundColor(.secondary)

                        Text(formatPhoneNumber(phoneNumber))
                            .font(.headline)
                            .foregroundColor(.primary)
                    }
                    .padding(.bottom, 24)

                    // OTP Input
                    VStack(spacing: 16) {
                        // Code Input Boxes
                        HStack(spacing: 12) {
                            ForEach(0..<codeLength, id: \.self) { index in
                                OTPDigitBox(
                                    digit: getDigit(at: index),
                                    isFocused: code.count == index && isCodeFieldFocused,
                                    themeManager: themeManager
                                )
                            }
                        }
                        .onTapGesture {
                            isCodeFieldFocused = true
                        }

                        // Hidden TextField for keyboard input
                        TextField("", text: $code)
                            .keyboardType(.numberPad)
                            .textContentType(.oneTimeCode)
                            .focused($isCodeFieldFocused)
                            .opacity(0)
                            .frame(height: 0)
                            .onChange(of: code) { newValue in
                                // Limit to 6 digits
                                let digits = newValue.filter { $0.isNumber }
                                if digits.count > codeLength {
                                    code = String(digits.prefix(codeLength))
                                } else {
                                    code = digits
                                }

                                // Auto-submit when complete
                                if code.count == codeLength {
                                    verifyCode()
                                }
                            }
                    }

                    // Verify Button
                    Button {
                        verifyCode()
                    } label: {
                        HStack {
                            if isLoading {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Text("Verify")
                                    .fontWeight(.semibold)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(
                            RoundedRectangle(cornerRadius: 12)
                                .fill(code.count == codeLength ? themeManager.primaryColor : Color.gray)
                        )
                        .foregroundColor(.white)
                    }
                    .disabled(code.count != codeLength || isLoading)
                    .padding(.top, 8)

                    // Resend Code
                    VStack(spacing: 8) {
                        Text("Didn't receive the code?")
                            .font(.subheadline)
                            .foregroundColor(.secondary)

                        if canResend {
                            Button {
                                resendCode()
                            } label: {
                                if isResending {
                                    ProgressView()
                                        .tint(themeManager.primaryColor)
                                } else {
                                    Text("Resend Code")
                                        .font(.subheadline)
                                        .fontWeight(.semibold)
                                        .foregroundColor(themeManager.primaryColor)
                                }
                            }
                            .disabled(isResending)
                        } else {
                            Text("Resend in \(resendCooldown)s")
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(.top, 16)

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
        .onReceive(timer) { _ in
            if resendCooldown > 0 {
                resendCooldown -= 1
            } else {
                canResend = true
            }
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .onAppear {
            isCodeFieldFocused = true
            analyticsService.trackScreenView(screenName: "otp_verification")
        }
    }

    // MARK: - Helpers

    private func getDigit(at index: Int) -> String {
        guard index < code.count else { return "" }
        let stringIndex = code.index(code.startIndex, offsetBy: index)
        return String(code[stringIndex])
    }

    private func formatPhoneNumber(_ phone: String) -> String {
        // Simple formatting for display
        guard phone.count >= 10 else { return phone }

        let cleaned = phone.filter { $0.isNumber || $0 == "+" }
        if cleaned.hasPrefix("+1") && cleaned.count == 12 {
            let start = cleaned.index(cleaned.startIndex, offsetBy: 2)
            let areaEnd = cleaned.index(start, offsetBy: 3)
            let middleEnd = cleaned.index(areaEnd, offsetBy: 3)

            return "+1 (\(cleaned[start..<areaEnd])) \(cleaned[areaEnd..<middleEnd])-\(cleaned[middleEnd...])"
        }

        return cleaned
    }

    // MARK: - Verify Code

    private func verifyCode() {
        guard code.count == codeLength else { return }

        isLoading = true
        isCodeFieldFocused = false

        Task {
            do {
                let orgSlug = ProcessInfo.processInfo.environment["ORG_SLUG"] ?? "demo"
                let roleString = selectedRole == .walker ? "walker" : "customer"
                let request = PhoneVerifyRequest(
                    phone: phoneNumber,
                    code: code,
                    orgSlug: orgSlug,
                    role: roleString
                )

                let response: PhoneVerifyResponse = try await APIClient.shared.post("/auth/phone/verify", body: request)

                await APIClient.shared.setAuthToken(response.token)

                // Save user to session
                let role = UserRole(rawValue: response.user.role ?? "customer") ?? .customer
                let user = User(
                    id: response.user.id,
                    email: response.user.email ?? "",
                    firstName: response.user.firstName,
                    lastName: response.user.lastName,
                    phone: response.user.phone,
                    role: role,
                    organizationId: response.user.organizationId
                )

                await MainActor.run {
                    UserSession.shared.setUser(user)
                }

                if FirebaseState.isConfigured {
                    Crashlytics.crashlytics().setUserID(response.user.id)
                }

                // Load contexts
                await loadContexts()

                await MainActor.run {
                    isLoading = false
                    analyticsService.trackEvent(name: "login_success", params: ["method": "phone"])
                    onVerificationSuccess()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    code = ""
                    isCodeFieldFocused = true

                    if case .httpError(let statusCode, _) = error {
                        if statusCode == 400 {
                            errorMessage = "Invalid code. Please try again."
                        } else if statusCode == 410 {
                            errorMessage = "Code expired. Please request a new one."
                        } else {
                            errorMessage = error.errorDescription ?? "Verification failed"
                        }
                    } else {
                        errorMessage = error.errorDescription ?? "Verification failed"
                    }
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    code = ""
                    isCodeFieldFocused = true
                    errorMessage = "An unexpected error occurred. Please try again."
                    showError = true
                }
            }
        }
    }

    // MARK: - Resend Code

    private func resendCode() {
        isResending = true

        Task {
            do {
                let orgSlug = ProcessInfo.processInfo.environment["ORG_SLUG"] ?? "demo"
                let request = PhoneSendCodeRequest(phone: phoneNumber, orgSlug: orgSlug)
                let _: PhoneSendCodeResponse = try await APIClient.shared.post("/auth/phone/send-code", body: request)

                await MainActor.run {
                    isResending = false
                    canResend = false
                    resendCooldown = 60
                    analyticsService.trackEvent(name: "phone_code_resent", params: nil)
                }
            } catch let error as APIError {
                await MainActor.run {
                    isResending = false
                    if case .httpError(let statusCode, _) = error, statusCode == 429 {
                        errorMessage = "Too many requests. Please wait before trying again."
                    } else {
                        errorMessage = error.errorDescription ?? "Failed to resend code"
                    }
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isResending = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }

    // MARK: - Load Contexts

    private func loadContexts() async {
        do {
            let contextsResponse = try await APIClient.shared.fetchContexts()
            await MainActor.run {
                UserSession.shared.setMemberships(
                    contextsResponse.memberships,
                    current: contextsResponse.currentMembership
                )
            }
        } catch {
            print("Failed to load contexts: \(error)")
        }
    }
}

// MARK: - OTP Digit Box

struct OTPDigitBox: View {
    let digit: String
    let isFocused: Bool
    let themeManager: ThemeManager

    var body: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 12)
                .fill(Color(.systemGray6))
                .frame(width: 48, height: 56)
                .overlay(
                    RoundedRectangle(cornerRadius: 12)
                        .stroke(isFocused ? themeManager.primaryColor : Color(.systemGray4), lineWidth: isFocused ? 2 : 1)
                )

            if digit.isEmpty && isFocused {
                RoundedRectangle(cornerRadius: 2)
                    .fill(themeManager.primaryColor)
                    .frame(width: 2, height: 24)
                    .opacity(0.8)
            } else {
                Text(digit)
                    .font(.title)
                    .fontWeight(.semibold)
                    .foregroundColor(.primary)
            }
        }
    }
}

#Preview {
    NavigationStack {
        OTPVerificationView(
            phoneNumber: "+15555555555",
            selectedRole: .customer,
            onVerificationSuccess: { print("Success") },
            onBack: { print("Back") }
        )
    }
    .withThemeManager()
}
