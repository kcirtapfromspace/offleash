//
//  PaymentMethodsView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct PaymentMethodsView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var paymentMethods: [PaymentMethod] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showAddPayment = false
    @State private var methodToDelete: PaymentMethod?

    var body: some View {
        Group {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if paymentMethods.isEmpty {
                emptyState
            } else {
                methodsList
            }
        }
        .navigationTitle("Payment Methods")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    showAddPayment = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .onAppear {
            loadPaymentMethods()
            analyticsService.trackScreenView(screenName: "payment_methods")
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
        .alert("Delete Payment Method", isPresented: .init(
            get: { methodToDelete != nil },
            set: { if !$0 { methodToDelete = nil } }
        )) {
            Button("Cancel", role: .cancel) {
                methodToDelete = nil
            }
            Button("Delete", role: .destructive) {
                if let method = methodToDelete {
                    deletePaymentMethod(method)
                }
            }
        } message: {
            Text("Are you sure you want to remove this payment method?")
        }
    }

    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "creditcard")
                .font(.system(size: 60))
                .foregroundColor(.secondary)

            Text("No Payment Methods")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Add a payment method to book walks for your pup.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button {
                showAddPayment = true
            } label: {
                Label("Add Payment Method", systemImage: "plus")
                    .fontWeight(.semibold)
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(themeManager.primaryColor)
                    .foregroundColor(.white)
                    .cornerRadius(12)
            }
            .padding(.horizontal, 40)
            .padding(.top, 8)
        }
        .padding()
    }

    private var methodsList: some View {
        List {
            ForEach(paymentMethods) { method in
                PaymentMethodRow(method: method, onSetDefault: {
                    setDefaultMethod(method)
                })
                .swipeActions(edge: .trailing) {
                    Button(role: .destructive) {
                        methodToDelete = method
                    } label: {
                        Label("Delete", systemImage: "trash")
                    }
                }
            }

            Section {
                Button {
                    showAddPayment = true
                } label: {
                    Label("Add Payment Method", systemImage: "plus.circle")
                        .foregroundColor(themeManager.primaryColor)
                }
            }
        }
    }

    private func loadPaymentMethods() {
        isLoading = true

        Task {
            do {
                let methods: [PaymentMethod] = try await APIClient.shared.get("/payment-methods")
                await MainActor.run {
                    paymentMethods = methods
                    isLoading = false
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to load payment methods"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }

    private func setDefaultMethod(_ method: PaymentMethod) {
        Task {
            do {
                let _: PaymentMethod = try await APIClient.shared.put("/payment-methods/\(method.id)/default", body: EmptyBody())
                await MainActor.run {
                    loadPaymentMethods()
                }
            } catch let error as APIError {
                await MainActor.run {
                    errorMessage = error.errorDescription ?? "Failed to set default"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }

    private func deletePaymentMethod(_ method: PaymentMethod) {
        Task {
            do {
                let _: EmptyResponse = try await APIClient.shared.delete("/payment-methods/\(method.id)")
                await MainActor.run {
                    methodToDelete = nil
                    loadPaymentMethods()
                }
            } catch let error as APIError {
                await MainActor.run {
                    methodToDelete = nil
                    errorMessage = error.errorDescription ?? "Failed to delete payment method"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    methodToDelete = nil
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }
}

struct EmptyBody: Codable {}

struct PaymentMethodRow: View {
    @EnvironmentObject private var themeManager: ThemeManager
    let method: PaymentMethod
    let onSetDefault: () -> Void

    var body: some View {
        HStack(spacing: 12) {
            // Icon
            ZStack {
                Circle()
                    .fill(themeManager.primaryColor.opacity(0.1))
                    .frame(width: 44, height: 44)

                Image(systemName: method.icon)
                    .font(.system(size: 20))
                    .foregroundColor(themeManager.primaryColor)
            }

            // Details
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(method.displayName)
                        .font(.headline)

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

                HStack(spacing: 8) {
                    if let expiration = method.expirationText {
                        Text("Exp: \(expiration)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }

                    if method.isExpired {
                        Text("Expired")
                            .font(.caption)
                            .fontWeight(.medium)
                            .foregroundColor(.red)
                    }
                }
            }

            Spacer()

            // Set as default button
            if !method.isDefault && !method.isExpired {
                Button {
                    onSetDefault()
                } label: {
                    Text("Set Default")
                        .font(.caption)
                        .foregroundColor(themeManager.primaryColor)
                }
                .buttonStyle(.borderless)
            }
        }
        .padding(.vertical, 4)
    }
}

#Preview {
    NavigationStack {
        PaymentMethodsView()
    }
    .withThemeManager()
}
