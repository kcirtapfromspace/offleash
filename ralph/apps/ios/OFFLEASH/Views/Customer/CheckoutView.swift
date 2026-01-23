//
//  CheckoutView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct CheckoutView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss

    let bookingId: String
    let serviceName: String
    let subtotalCents: Int
    let providerUserId: String

    @State private var selectedPaymentMethod: PaymentMethod?
    @State private var paymentMethods: [PaymentMethod] = []
    @State private var feePreview: FeePreviewResponse?
    @State private var tipCents: Int = 0
    @State private var isLoadingMethods = true
    @State private var isLoadingPreview = false
    @State private var isProcessing = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showAddPayment = false
    @State private var checkoutComplete = false
    @State private var checkoutResponse: CheckoutResponse?

    private let tipOptions = [0, 15, 20, 25] // percentages

    var body: some View {
        NavigationStack {
            if checkoutComplete, let response = checkoutResponse {
                checkoutSuccessView(response)
            } else {
                checkoutFormView
            }
        }
    }

    private var checkoutFormView: some View {
        ScrollView {
            VStack(spacing: 24) {
                // Order Summary
                orderSummarySection

                // Tip Selection
                tipSection

                // Fee Breakdown
                feeBreakdownSection

                // Payment Method
                paymentMethodSection

                // Pay Button
                payButton
            }
            .padding()
        }
        .navigationTitle("Checkout")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") {
                    dismiss()
                }
            }
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "checkout")
            loadPaymentMethods()
            loadFeePreview()
        }
        .onChange(of: tipCents) { _ in
            loadFeePreview()
        }
        .sheet(isPresented: $showAddPayment) {
            AddPaymentMethodView(onSuccess: {
                loadPaymentMethods()
            })
            .environmentObject(themeManager)
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
    }

    // MARK: - Order Summary

    private var orderSummarySection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Order Summary")
                .font(.headline)

            HStack {
                Image(systemName: "pawprint.fill")
                    .foregroundColor(themeManager.primaryColor)

                Text(serviceName)
                    .font(.subheadline)

                Spacer()

                Text(formatCents(subtotalCents))
                    .font(.subheadline)
                    .fontWeight(.medium)
            }
            .padding()
            .background(Color(.systemGray6))
            .cornerRadius(12)
        }
    }

    // MARK: - Tip Section

    private var tipSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Add a Tip")
                .font(.headline)

            Text("Show appreciation for your walker")
                .font(.caption)
                .foregroundColor(.secondary)

            HStack(spacing: 12) {
                ForEach(tipOptions, id: \.self) { percent in
                    Button {
                        let tip = percent == 0 ? 0 : (subtotalCents * percent) / 100
                        tipCents = tip
                    } label: {
                        let isSelected = tipCents == (percent == 0 ? 0 : (subtotalCents * percent) / 100)
                        VStack(spacing: 4) {
                            Text(percent == 0 ? "None" : "\(percent)%")
                                .font(.subheadline)
                                .fontWeight(isSelected ? .semibold : .regular)

                            if percent > 0 {
                                Text(formatCents((subtotalCents * percent) / 100))
                                    .font(.caption2)
                                    .foregroundColor(isSelected ? .white.opacity(0.8) : .secondary)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 12)
                        .background(isSelected ? themeManager.primaryColor : Color(.systemGray6))
                        .foregroundColor(isSelected ? .white : .primary)
                        .cornerRadius(8)
                    }
                }
            }
        }
    }

    // MARK: - Fee Breakdown

    private var feeBreakdownSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Total")
                .font(.headline)

            if isLoadingPreview {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .padding()
            } else if let preview = feePreview {
                VStack(spacing: 8) {
                    feeRow(label: "Subtotal", amount: preview.subtotalCents)

                    if preview.tipCents > 0 {
                        feeRow(label: "Tip", amount: preview.tipCents)
                    }

                    feeRow(label: "Service Fee", amount: preview.customerFeeCents, info: String(format: "%.1f%%", preview.customerFeePercent))

                    if preview.taxCents > 0 {
                        feeRow(label: "Tax", amount: preview.taxCents, info: String(format: "%.1f%%", preview.taxRatePercent))
                    }

                    Divider()

                    HStack {
                        Text("Total")
                            .font(.headline)
                        Spacer()
                        Text(formatCents(preview.totalCents))
                            .font(.headline)
                            .foregroundColor(themeManager.primaryColor)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)
            } else {
                VStack(spacing: 8) {
                    feeRow(label: "Subtotal", amount: subtotalCents)

                    if tipCents > 0 {
                        feeRow(label: "Tip", amount: tipCents)
                    }

                    Divider()

                    HStack {
                        Text("Total")
                            .font(.headline)
                        Spacer()
                        Text(formatCents(subtotalCents + tipCents))
                            .font(.headline)
                            .foregroundColor(themeManager.primaryColor)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)
            }
        }
    }

    private func feeRow(label: String, amount: Int, info: String? = nil) -> some View {
        HStack {
            Text(label)
                .foregroundColor(.secondary)
            if let info = info {
                Text("(\(info))")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
            Text(formatCents(amount))
        }
    }

    // MARK: - Payment Method

    private var paymentMethodSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Payment Method")
                    .font(.headline)

                Spacer()

                Button("Add New") {
                    showAddPayment = true
                }
                .font(.subheadline)
                .foregroundColor(themeManager.primaryColor)
            }

            if isLoadingMethods {
                HStack {
                    Spacer()
                    ProgressView()
                    Spacer()
                }
                .padding()
            } else if paymentMethods.isEmpty {
                Button {
                    showAddPayment = true
                } label: {
                    HStack {
                        Image(systemName: "plus.circle")
                        Text("Add a payment method")
                    }
                    .font(.subheadline)
                    .foregroundColor(themeManager.primaryColor)
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color(.systemGray6))
                    .cornerRadius(12)
                }
            } else {
                ForEach(paymentMethods) { method in
                    PaymentMethodSelectionRow(
                        method: method,
                        isSelected: selectedPaymentMethod?.id == method.id,
                        themeManager: themeManager
                    ) {
                        selectedPaymentMethod = method
                    }
                }
            }
        }
    }

    // MARK: - Pay Button

    private var payButton: some View {
        Button {
            processPayment()
        } label: {
            HStack {
                if isProcessing {
                    ProgressView()
                        .tint(.white)
                } else {
                    Text("Pay \(feePreview?.formattedTotal ?? formatCents(subtotalCents + tipCents))")
                        .fontWeight(.semibold)
                }
            }
            .frame(maxWidth: .infinity)
            .padding()
            .background(canPay ? themeManager.primaryColor : Color(.systemGray4))
            .foregroundColor(.white)
            .cornerRadius(12)
        }
        .disabled(!canPay || isProcessing)
    }

    private var canPay: Bool {
        selectedPaymentMethod != nil && !isLoadingPreview
    }

    // MARK: - Success View

    private func checkoutSuccessView(_ response: CheckoutResponse) -> some View {
        VStack(spacing: 32) {
            Spacer()

            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 80))
                .foregroundColor(.green)

            VStack(spacing: 8) {
                Text("Payment Successful!")
                    .font(.title2)
                    .fontWeight(.bold)

                Text("Your booking has been confirmed.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }

            VStack(spacing: 16) {
                HStack {
                    Text("Amount Paid")
                        .foregroundColor(.secondary)
                    Spacer()
                    Text(response.formattedTotal)
                        .fontWeight(.semibold)
                }

                HStack {
                    Text("Transaction ID")
                        .foregroundColor(.secondary)
                    Spacer()
                    Text(String(response.transactionId.prefix(8)) + "...")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            .padding()
            .background(Color(.systemGray6))
            .cornerRadius(12)
            .padding(.horizontal)

            Spacer()

            Button {
                dismiss()
            } label: {
                Text("Done")
                    .fontWeight(.semibold)
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(themeManager.primaryColor)
                    .foregroundColor(.white)
                    .cornerRadius(12)
            }
            .padding(.horizontal)
        }
        .padding()
        .navigationTitle("Payment Complete")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar(.hidden, for: .navigationBar)
    }

    // MARK: - Actions

    private func loadPaymentMethods() {
        isLoadingMethods = true

        Task {
            do {
                let methods = try await PaymentService.shared.getPaymentMethods()
                await MainActor.run {
                    paymentMethods = methods
                    // Auto-select default method
                    selectedPaymentMethod = methods.first { $0.isDefault } ?? methods.first
                    isLoadingMethods = false
                }
            } catch {
                await MainActor.run {
                    isLoadingMethods = false
                    // Non-critical error, user can add a method
                }
            }
        }
    }

    private func loadFeePreview() {
        isLoadingPreview = true

        Task {
            do {
                let request = FeePreviewRequest(
                    subtotalCents: subtotalCents,
                    tipCents: tipCents > 0 ? tipCents : nil,
                    customerState: nil // Could get from user's location
                )
                let preview = try await PaymentService.shared.previewFees(request)
                await MainActor.run {
                    feePreview = preview
                    isLoadingPreview = false
                }
            } catch {
                await MainActor.run {
                    isLoadingPreview = false
                    // Preview failed, show basic total
                }
            }
        }
    }

    private func processPayment() {
        guard let paymentMethod = selectedPaymentMethod else { return }

        isProcessing = true

        Task {
            do {
                // Create checkout session
                let checkoutRequest = CreateCheckoutRequest(
                    bookingId: bookingId,
                    paymentMethodId: paymentMethod.id,
                    subtotalCents: subtotalCents,
                    tipCents: tipCents > 0 ? tipCents : nil,
                    providerUserId: providerUserId,
                    customerState: nil,
                    customerZip: nil
                )

                let checkout = try await PaymentService.shared.createCheckout(checkoutRequest)

                // Confirm the payment
                let confirmed = try await PaymentService.shared.confirmPayment(
                    checkout.transactionId,
                    paymentMethodId: paymentMethod.id
                )

                await MainActor.run {
                    isProcessing = false
                    checkoutResponse = confirmed
                    checkoutComplete = true
                    analyticsService.trackEvent(name: "checkout_completed", params: [
                        "amount_cents": String(confirmed.totalCents),
                        "provider": confirmed.providerType
                    ])
                }
            } catch let error as APIError {
                await MainActor.run {
                    isProcessing = false
                    errorMessage = error.errorDescription ?? "Payment failed. Please try again."
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isProcessing = false
                    errorMessage = "An unexpected error occurred. Please try again."
                    showError = true
                }
            }
        }
    }

    private func formatCents(_ cents: Int) -> String {
        String(format: "$%.2f", Double(cents) / 100.0)
    }
}

// MARK: - Payment Method Selection Row

struct PaymentMethodSelectionRow: View {
    let method: PaymentMethod
    let isSelected: Bool
    let themeManager: ThemeManager
    let onSelect: () -> Void

    var body: some View {
        Button(action: onSelect) {
            HStack(spacing: 12) {
                // Selection indicator
                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .foregroundColor(isSelected ? themeManager.primaryColor : .secondary)

                // Icon
                Image(systemName: method.icon)
                    .font(.title3)
                    .foregroundColor(themeManager.primaryColor)
                    .frame(width: 32)

                // Details
                VStack(alignment: .leading, spacing: 2) {
                    Text(method.displayName)
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundColor(.primary)

                    if let exp = method.expirationText {
                        Text("Exp: \(exp)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }

                Spacer()

                if method.isDefault {
                    Text("Default")
                        .font(.caption2)
                        .fontWeight(.medium)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(themeManager.primaryColor.opacity(0.1))
                        .foregroundColor(themeManager.primaryColor)
                        .cornerRadius(4)
                }
            }
            .padding()
            .background(isSelected ? themeManager.primaryColor.opacity(0.05) : Color(.systemGray6))
            .cornerRadius(12)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(isSelected ? themeManager.primaryColor : Color.clear, lineWidth: 2)
            )
        }
    }
}

#Preview {
    CheckoutView(
        bookingId: "preview-booking",
        serviceName: "30-Minute Walk",
        subtotalCents: 2500,
        providerUserId: "provider-1"
    )
    .withThemeManager()
}
