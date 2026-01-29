//
//  AddPaymentMethodView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import PassKit

struct AddPaymentMethodView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss

    var onSuccess: () -> Void

    @State private var selectedType: PaymentType = .card
    @State private var cardNumber = ""
    @State private var cardExpMonth = ""
    @State private var cardExpYear = ""
    @State private var cardCVV = ""
    @State private var cardholderName = ""
    @State private var nickname = ""
    @State private var setAsDefault = true
    @State private var isSaving = false
    @State private var showError = false
    @State private var errorMessage = ""

    enum PaymentType: String, CaseIterable {
        case card = "Credit/Debit Card"
        case applePay = "Apple Pay"
    }

    var body: some View {
        NavigationStack {
            Form {
                // Payment Type Selection
                Section("Payment Type") {
                    Picker("Type", selection: $selectedType) {
                        ForEach(PaymentType.allCases, id: \.self) { type in
                            Text(type.rawValue).tag(type)
                        }
                    }
                    .pickerStyle(.segmented)
                }

                if selectedType == .card {
                    cardForm
                } else {
                    applePaySection
                }

                // Options
                Section {
                    Toggle("Set as default payment method", isOn: $setAsDefault)
                }

                // Save Button
                Section {
                    Button {
                        savePaymentMethod()
                    } label: {
                        HStack {
                            Spacer()
                            if isSaving {
                                ProgressView()
                                    .tint(.white)
                            } else {
                                Text("Add Payment Method")
                                    .fontWeight(.semibold)
                            }
                            Spacer()
                        }
                    }
                    .listRowBackground(isValid ? themeManager.primaryColor : Color(.systemGray4))
                    .foregroundColor(.white)
                    .disabled(!isValid || isSaving)
                    .accessibilityIdentifier("card-save-button")
                }
            }
            .navigationTitle("Add Payment")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                analyticsService.trackScreenView(screenName: "add_payment_method")
            }
            .alert("Error", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage)
            }
        }
    }

    private var cardForm: some View {
        Group {
            Section("Card Details") {
                TextField("Cardholder Name", text: $cardholderName)
                    .textContentType(.name)
                    .autocapitalization(.words)
                    .accessibilityIdentifier("cardholder-name-field")

                TextField("Card Number", text: $cardNumber)
                    .keyboardType(.numberPad)
                    .onChange(of: cardNumber) { newValue in
                        let formatted = formatCardNumber(newValue)
                        if formatted != newValue {
                            cardNumber = formatted
                        }
                    }
                    .accessibilityIdentifier("card-number-field")

                HStack {
                    TextField("MM", text: $cardExpMonth)
                        .keyboardType(.numberPad)
                        .frame(width: 50)
                        .onChange(of: cardExpMonth) { newValue in
                            let cleaned = String(newValue.prefix(2).filter { $0.isNumber })
                            if cleaned != newValue {
                                cardExpMonth = cleaned
                            }
                        }
                        .accessibilityIdentifier("card-exp-month-field")

                    Text("/")
                        .foregroundColor(.secondary)

                    TextField("YY", text: $cardExpYear)
                        .keyboardType(.numberPad)
                        .frame(width: 50)
                        .onChange(of: cardExpYear) { newValue in
                            let cleaned = String(newValue.prefix(2).filter { $0.isNumber })
                            if cleaned != newValue {
                                cardExpYear = cleaned
                            }
                        }
                        .accessibilityIdentifier("card-exp-year-field")

                    Spacer()

                    SecureField("CVV", text: $cardCVV)
                        .keyboardType(.numberPad)
                        .frame(width: 60)
                        .onChange(of: cardCVV) { newValue in
                            let cleaned = String(newValue.prefix(4).filter { $0.isNumber })
                            if cleaned != newValue {
                                cardCVV = cleaned
                            }
                        }
                        .accessibilityIdentifier("card-cvc-field")
                }
            }

            Section("Optional") {
                TextField("Nickname (e.g., Personal Card)", text: $nickname)
            }
        }
    }

    private var applePaySection: some View {
        Section {
            VStack(spacing: 16) {
                Image(systemName: "apple.logo")
                    .font(.system(size: 48))
                    .foregroundColor(.primary)

                Text("Apple Pay")
                    .font(.title2)
                    .fontWeight(.semibold)

                Text("Use Apple Pay for quick and secure payments. Your card information is never stored on our servers.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)

                if PKPaymentAuthorizationController.canMakePayments() {
                    Text("Apple Pay is available on this device")
                        .font(.caption)
                        .foregroundColor(.green)
                } else {
                    Text("Apple Pay is not set up on this device. Please add a card in Wallet settings.")
                        .font(.caption)
                        .foregroundColor(.orange)
                        .multilineTextAlignment(.center)
                }
            }
            .padding(.vertical, 20)
            .frame(maxWidth: .infinity)
        }
    }

    private var isValid: Bool {
        switch selectedType {
        case .card:
            let cleanNumber = cardNumber.filter { $0.isNumber }
            return cleanNumber.count >= 15 &&
                   cardExpMonth.count == 2 &&
                   cardExpYear.count == 2 &&
                   cardCVV.count >= 3 &&
                   !cardholderName.isEmpty
        case .applePay:
            return PKPaymentAuthorizationController.canMakePayments()
        }
    }

    private func formatCardNumber(_ number: String) -> String {
        let cleaned = number.filter { $0.isNumber }
        let limited = String(cleaned.prefix(16))
        var formatted = ""
        for (index, char) in limited.enumerated() {
            if index > 0 && index % 4 == 0 {
                formatted += " "
            }
            formatted.append(char)
        }
        return formatted
    }

    private func detectCardBrand(_ number: String) -> String {
        let cleaned = number.filter { $0.isNumber }
        if cleaned.hasPrefix("4") {
            return "visa"
        } else if cleaned.hasPrefix("5") || cleaned.hasPrefix("2") {
            return "mastercard"
        } else if cleaned.hasPrefix("3") {
            return "amex"
        } else if cleaned.hasPrefix("6") {
            return "discover"
        }
        return "other"
    }

    private func savePaymentMethod() {
        isSaving = true

        Task {
            do {
                let request: CreatePaymentMethodRequest

                switch selectedType {
                case .card:
                    let cleanNumber = cardNumber.filter { $0.isNumber }
                    let lastFour = String(cleanNumber.suffix(4))
                    let brand = detectCardBrand(cleanNumber)
                    let expMonth = Int(cardExpMonth)
                    let expYear = 2000 + (Int(cardExpYear) ?? 0)

                    request = CreatePaymentMethodRequest(
                        methodType: "card",
                        cardNonce: nil, // In production, this would come from Square SDK
                        cardLastFour: lastFour,
                        cardBrand: brand,
                        cardExpMonth: expMonth,
                        cardExpYear: expYear,
                        nickname: nickname.isEmpty ? nil : nickname,
                        isDefault: setAsDefault
                    )

                case .applePay:
                    request = CreatePaymentMethodRequest(
                        methodType: "apple_pay",
                        cardNonce: nil,
                        cardLastFour: nil,
                        cardBrand: nil,
                        cardExpMonth: nil,
                        cardExpYear: nil,
                        nickname: nil,
                        isDefault: setAsDefault
                    )
                }

                let _: PaymentMethod = try await PaymentService.shared.addPaymentMethod(request)

                await MainActor.run {
                    isSaving = false
                    analyticsService.trackEvent(name: "payment_method_added", params: [
                        "type": selectedType == .card ? "card" : "apple_pay"
                    ])
                    onSuccess()
                    dismiss()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSaving = false
                    errorMessage = error.errorDescription ?? "Failed to add payment method"
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
    AddPaymentMethodView(onSuccess: {})
        .withThemeManager()
}
