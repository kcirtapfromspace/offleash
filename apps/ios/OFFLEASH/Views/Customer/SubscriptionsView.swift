//
//  SubscriptionsView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct SubscriptionsView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    @State private var subscriptions: [CustomerSubscription] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var subscriptionToCancel: CustomerSubscription?

    var body: some View {
        Group {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if subscriptions.isEmpty {
                emptyState
            } else {
                subscriptionList
            }
        }
        .navigationTitle("Subscriptions")
        .onAppear {
            loadSubscriptions()
            analyticsService.trackScreenView(screenName: "subscriptions")
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .alert("Cancel Subscription", isPresented: .init(
            get: { subscriptionToCancel != nil },
            set: { if !$0 { subscriptionToCancel = nil } }
        )) {
            Button("Keep Subscription", role: .cancel) {
                subscriptionToCancel = nil
            }
            Button("Cancel Subscription", role: .destructive) {
                if let sub = subscriptionToCancel {
                    cancelSubscription(sub)
                }
            }
        } message: {
            if let sub = subscriptionToCancel {
                Text("Are you sure you want to cancel \(sub.name)? You will still have access until the end of the current billing period.")
            }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "repeat.circle")
                .font(.system(size: 60))
                .foregroundColor(.secondary)

            Text("No Active Subscriptions")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Subscribe to a service package to get regular walks for your pup at a discounted rate.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
        }
        .padding()
    }

    private var subscriptionList: some View {
        List {
            ForEach(subscriptions) { subscription in
                SubscriptionCard(subscription: subscription)
                    .swipeActions(edge: .trailing) {
                        if subscription.isActive && !subscription.cancelAtPeriodEnd {
                            Button(role: .destructive) {
                                subscriptionToCancel = subscription
                            } label: {
                                Label("Cancel", systemImage: "xmark.circle")
                            }
                        }
                    }
            }
        }
        .refreshable {
            await refreshSubscriptions()
        }
    }

    private func loadSubscriptions() {
        isLoading = true

        Task {
            do {
                let subs = try await PaymentService.shared.getSubscriptions()
                await MainActor.run {
                    subscriptions = subs
                    isLoading = false
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to load subscriptions"
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

    private func refreshSubscriptions() async {
        do {
            let subs = try await PaymentService.shared.getSubscriptions()
            await MainActor.run {
                subscriptions = subs
            }
        } catch {
            // Silent refresh failure
        }
    }

    private func cancelSubscription(_ subscription: CustomerSubscription) {
        Task {
            do {
                let _ = try await PaymentService.shared.cancelSubscription(subscription.id)
                await MainActor.run {
                    subscriptionToCancel = nil
                    loadSubscriptions()
                    analyticsService.trackEvent(name: "subscription_canceled", params: [
                        "subscription_id": subscription.id
                    ])
                }
            } catch let error as APIError {
                await MainActor.run {
                    subscriptionToCancel = nil
                    errorMessage = error.errorDescription ?? "Failed to cancel subscription"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    subscriptionToCancel = nil
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }
}

// MARK: - Subscription Card

struct SubscriptionCard: View {
    @EnvironmentObject private var themeManager: ThemeManager
    let subscription: CustomerSubscription

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(subscription.name)
                        .font(.headline)

                    if let description = subscription.description {
                        Text(description)
                            .font(.caption)
                            .foregroundColor(.secondary)
                            .lineLimit(2)
                    }
                }

                Spacer()

                SubscriptionStatusBadge(status: subscription.status)
            }

            Divider()

            // Details
            HStack {
                // Price
                VStack(alignment: .leading, spacing: 2) {
                    Text("Price")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("\(subscription.formattedPrice)/\(subscription.interval)")
                        .font(.subheadline)
                        .fontWeight(.medium)
                }

                Spacer()

                // Interval
                VStack(alignment: .trailing, spacing: 2) {
                    Text("Billing")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(subscription.intervalDisplay)
                        .font(.subheadline)
                }
            }

            // Period info
            if let periodEnd = subscription.currentPeriodEnd {
                HStack {
                    Image(systemName: "calendar")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    if subscription.cancelAtPeriodEnd {
                        Text("Cancels on \(formatDate(periodEnd))")
                            .font(.caption)
                            .foregroundColor(.orange)
                    } else {
                        Text("Renews on \(formatDate(periodEnd))")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }

            // Auto-booking indicator
            if subscription.autoCreateBookings {
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.caption)
                        .foregroundColor(.green)
                    Text("Auto-schedules bookings")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }

    private func formatDate(_ dateString: String) -> String {
        let isoFormatter = ISO8601DateFormatter()
        isoFormatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]

        if let date = isoFormatter.date(from: dateString) {
            let formatter = DateFormatter()
            formatter.dateStyle = .medium
            return formatter.string(from: date)
        }

        isoFormatter.formatOptions = [.withInternetDateTime]
        if let date = isoFormatter.date(from: dateString) {
            let formatter = DateFormatter()
            formatter.dateStyle = .medium
            return formatter.string(from: date)
        }

        return dateString
    }
}

// MARK: - Subscription Status Badge

struct SubscriptionStatusBadge: View {
    @EnvironmentObject private var themeManager: ThemeManager
    let status: String

    var body: some View {
        Text(statusText)
            .font(.caption2)
            .fontWeight(.medium)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(statusColor.opacity(0.1))
            .foregroundColor(statusColor)
            .cornerRadius(4)
    }

    private var statusText: String {
        switch status.lowercased() {
        case "active": return "Active"
        case "paused": return "Paused"
        case "canceled": return "Canceled"
        case "past_due": return "Past Due"
        case "trialing": return "Trial"
        case "incomplete": return "Incomplete"
        default: return status.capitalized
        }
    }

    private var statusColor: Color {
        switch status.lowercased() {
        case "active", "trialing": return .green
        case "paused": return .yellow
        case "canceled": return .gray
        case "past_due", "incomplete": return .red
        default: return themeManager.primaryColor
        }
    }
}

#Preview {
    NavigationStack {
        SubscriptionsView()
    }
    .withThemeManager()
}
