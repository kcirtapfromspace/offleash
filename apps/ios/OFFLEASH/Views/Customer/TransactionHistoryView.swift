//
//  TransactionHistoryView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct TransactionHistoryView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    @State private var transactions: [TransactionListItem] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var selectedTransaction: TransactionListItem?

    var body: some View {
        Group {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if transactions.isEmpty {
                emptyState
            } else {
                transactionList
            }
        }
        .navigationTitle("Transaction History")
        .onAppear {
            loadTransactions()
            analyticsService.trackScreenView(screenName: "transaction_history")
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .sheet(item: $selectedTransaction) { transaction in
            TransactionDetailView(transactionId: transaction.id)
                .environmentObject(themeManager)
        }
    }

    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "clock.arrow.circlepath")
                .font(.system(size: 60))
                .foregroundColor(.secondary)

            Text("No Transactions")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Your transaction history will appear here after you complete a booking.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
        }
        .padding()
    }

    private var transactionList: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Transactions")
                .font(.caption)
                .foregroundColor(.secondary)
                .accessibilityIdentifier("payment-history-list")

            List {
                ForEach(transactions) { transaction in
                    TransactionRow(transaction: transaction, themeManager: themeManager)
                        .onTapGesture {
                            selectedTransaction = transaction
                        }
                }
            }
            .refreshable {
                await refreshTransactions()
            }
        }
    }

    private func loadTransactions() {
        isLoading = true

        Task {
            do {
                let items = try await PaymentService.shared.getTransactions()
                await MainActor.run {
                    transactions = items
                    isLoading = false
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to load transactions"
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

    private func refreshTransactions() async {
        do {
            let items = try await PaymentService.shared.getTransactions()
            await MainActor.run {
                transactions = items
            }
        } catch {
            // Silent refresh failure
        }
    }
}

// MARK: - Transaction Row

struct TransactionRow: View {
    let transaction: TransactionListItem
    let themeManager: ThemeManager

    var body: some View {
        HStack(spacing: 12) {
            // Status Icon
            ZStack {
                Circle()
                    .fill(statusColor.opacity(0.1))
                    .frame(width: 44, height: 44)

                Image(systemName: statusIcon)
                    .foregroundColor(statusColor)
            }

            // Details
            VStack(alignment: .leading, spacing: 4) {
                Text(transaction.isCustomer ? "Payment" : "Received")
                    .font(.subheadline)
                    .fontWeight(.medium)

                Text(formatDate(transaction.createdAt))
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            // Amount
            VStack(alignment: .trailing, spacing: 4) {
                Text(transaction.formattedTotal)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundColor(transaction.isCustomer ? .primary : .green)

                Text(statusText)
                    .font(.caption2)
                    .fontWeight(.medium)
                    .foregroundColor(statusColor)
            }
        }
        .padding(.vertical, 4)
    }

    private var statusText: String {
        switch transaction.status.lowercased() {
        case "pending": return "Pending"
        case "processing": return "Processing"
        case "succeeded": return "Completed"
        case "failed": return "Failed"
        case "refunded": return "Refunded"
        case "partially_refunded": return "Partial Refund"
        case "disputed": return "Disputed"
        case "canceled", "cancelled": return "Canceled"
        default: return transaction.status.capitalized
        }
    }

    private var statusIcon: String {
        switch transaction.status.lowercased() {
        case "pending", "processing": return "clock"
        case "succeeded": return "checkmark"
        case "failed", "disputed": return "exclamationmark.triangle"
        case "refunded", "partially_refunded": return "arrow.uturn.backward"
        case "canceled", "cancelled": return "xmark"
        default: return "dollarsign.circle"
        }
    }

    private var statusColor: Color {
        switch transaction.status.lowercased() {
        case "pending", "processing": return .yellow
        case "succeeded": return .green
        case "failed", "disputed": return .red
        case "refunded", "partially_refunded", "canceled", "cancelled": return .gray
        default: return themeManager.primaryColor
        }
    }

    private func formatDate(_ dateString: String) -> String {
        let isoFormatter = ISO8601DateFormatter()
        isoFormatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]

        if let date = isoFormatter.date(from: dateString) {
            let formatter = DateFormatter()
            formatter.dateStyle = .medium
            formatter.timeStyle = .short
            return formatter.string(from: date)
        }

        // Try without fractional seconds
        isoFormatter.formatOptions = [.withInternetDateTime]
        if let date = isoFormatter.date(from: dateString) {
            let formatter = DateFormatter()
            formatter.dateStyle = .medium
            formatter.timeStyle = .short
            return formatter.string(from: date)
        }

        return dateString
    }
}

// MARK: - Transaction Detail View

struct TransactionDetailView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss

    let transactionId: String

    @State private var transaction: Transaction?
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showRefundConfirm = false
    @State private var isRefunding = false

    var body: some View {
        NavigationStack {
            Group {
                if isLoading {
                    ProgressView()
                } else if let txn = transaction {
                    transactionDetails(txn)
                } else {
                    Text("Transaction not found")
                        .foregroundColor(.secondary)
                }
            }
            .navigationTitle("Transaction Details")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                loadTransaction()
            }
            .alert("Error", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage)
            }
            .alert("Request Refund", isPresented: $showRefundConfirm) {
                Button("Cancel", role: .cancel) {}
                Button("Request Refund", role: .destructive) {
                    requestRefund()
                }
            } message: {
                Text("Are you sure you want to request a refund for this transaction?")
            }
        }
    }

    private func transactionDetails(_ txn: Transaction) -> some View {
        ScrollView {
            VStack(spacing: 24) {
                // Status Header
                VStack(spacing: 8) {
                    Image(systemName: txn.status == .succeeded ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                        .font(.system(size: 48))
                        .foregroundColor(txn.status == .succeeded ? .green : .orange)

                    Text(txn.statusText)
                        .font(.headline)

                    Text(txn.formattedDate)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding()

                // Amount Breakdown
                VStack(spacing: 12) {
                    detailRow(label: "Subtotal", value: formatCents(txn.subtotalCents))

                    if let tip = txn.tipCents, tip > 0 {
                        detailRow(label: "Tip", value: formatCents(tip))
                    }

                    detailRow(label: "Service Fee", value: formatCents(txn.customerFeeCents))

                    if txn.taxCents > 0 {
                        detailRow(label: "Tax", value: formatCents(txn.taxCents))
                    }

                    Divider()

                    HStack {
                        Text("Total")
                            .font(.headline)
                        Spacer()
                        Text(formatCents(txn.totalCents))
                            .font(.headline)
                            .foregroundColor(themeManager.primaryColor)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)

                // Transaction Info
                VStack(spacing: 12) {
                    detailRow(label: "Transaction ID", value: String(txn.id.prefix(12)) + "...")

                    if let bookingId = txn.bookingId {
                        detailRow(label: "Booking", value: String(bookingId.prefix(12)) + "...")
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)

                // Refund Button (only show if eligible)
                if txn.status == .succeeded {
                    Button {
                        showRefundConfirm = true
                    } label: {
                        HStack {
                            if isRefunding {
                                ProgressView()
                                    .tint(Color.red)
                            } else {
                                Text("Request Refund")
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(Color.red.opacity(0.1))
                        .foregroundColor(.red)
                        .cornerRadius(12)
                    }
                    .disabled(isRefunding)
                }
            }
            .padding()
        }
    }

    private func detailRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .foregroundColor(.secondary)
            Spacer()
            Text(value)
        }
    }

    private func formatCents(_ cents: Int) -> String {
        String(format: "$%.2f", Double(cents) / 100.0)
    }

    private func loadTransaction() {
        Task {
            do {
                let txn = try await PaymentService.shared.getTransaction(transactionId)
                await MainActor.run {
                    transaction = txn
                    isLoading = false
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to load transaction"
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

    private func requestRefund() {
        isRefunding = true

        Task {
            do {
                let _ = try await PaymentService.shared.requestRefund(transactionId)
                await MainActor.run {
                    isRefunding = false
                    loadTransaction() // Reload to show updated status
                }
            } catch let error as APIError {
                await MainActor.run {
                    isRefunding = false
                    errorMessage = error.errorDescription ?? "Refund request failed"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isRefunding = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }
}

// Extension for Identifiable conformance
extension TransactionListItem: @retroactive Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    public static func == (lhs: TransactionListItem, rhs: TransactionListItem) -> Bool {
        lhs.id == rhs.id
    }
}

#Preview {
    NavigationStack {
        TransactionHistoryView()
    }
    .withThemeManager()
}
