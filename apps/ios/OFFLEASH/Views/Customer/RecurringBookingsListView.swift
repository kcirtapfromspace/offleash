//
//  RecurringBookingsListView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct RecurringBookingsListView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    @State private var recurringBookings: [RecurringBooking] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var selectedBooking: RecurringBooking?

    var body: some View {
        Group {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if recurringBookings.isEmpty {
                emptyState
            } else {
                bookingsList
            }
        }
        .navigationTitle("Recurring Walks")
        .onAppear {
            loadRecurringBookings()
            analyticsService.trackScreenView(screenName: "recurring_bookings")
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .sheet(item: $selectedBooking) { booking in
            NavigationStack {
                RecurringBookingDetailView(
                    booking: booking,
                    onCancel: {
                        selectedBooking = nil
                        loadRecurringBookings()
                    }
                )
                .environmentObject(themeManager)
            }
        }
    }

    // MARK: - Empty State

    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "repeat.circle")
                .font(.system(size: 60))
                .foregroundColor(.secondary)

            Text("No Recurring Walks")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Schedule regular walks for your pup. Create a recurring booking when booking a new walk.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
        }
        .padding()
    }

    // MARK: - Bookings List

    private var bookingsList: some View {
        List {
            // Active bookings
            let activeBookings = recurringBookings.filter { $0.isActive }
            if !activeBookings.isEmpty {
                Section("Active") {
                    ForEach(activeBookings) { booking in
                        RecurringBookingRow(booking: booking, themeManager: themeManager)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                selectedBooking = booking
                            }
                    }
                }
            }

            // Paused bookings
            let pausedBookings = recurringBookings.filter { $0.status == .paused }
            if !pausedBookings.isEmpty {
                Section("Paused") {
                    ForEach(pausedBookings) { booking in
                        RecurringBookingRow(booking: booking, themeManager: themeManager)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                selectedBooking = booking
                            }
                    }
                }
            }

            // Completed/Cancelled bookings
            let pastBookings = recurringBookings.filter { $0.status == .completed || $0.status == .cancelled }
            if !pastBookings.isEmpty {
                Section("Past") {
                    ForEach(pastBookings) { booking in
                        RecurringBookingRow(booking: booking, themeManager: themeManager)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                selectedBooking = booking
                            }
                    }
                }
            }
        }
    }

    // MARK: - Load Data

    private func loadRecurringBookings() {
        isLoading = true

        Task {
            do {
                let response: RecurringBookingListResponse = try await APIClient.shared.get("/bookings/recurring")
                await MainActor.run {
                    recurringBookings = response.recurringBookings
                    isLoading = false
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to load recurring bookings"
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
}

// MARK: - Recurring Booking Row

struct RecurringBookingRow: View {
    let booking: RecurringBooking
    let themeManager: ThemeManager

    var body: some View {
        HStack(spacing: 12) {
            // Status Icon
            ZStack {
                Circle()
                    .fill(statusColor.opacity(0.1))
                    .frame(width: 44, height: 44)

                Image(systemName: booking.frequency.icon)
                    .font(.system(size: 20))
                    .foregroundColor(statusColor)
            }

            // Details
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(booking.serviceName ?? "Walk")
                        .font(.headline)

                    Spacer()

                    RecurringStatusBadge(status: booking.status, themeManager: themeManager)
                }

                Text(booking.scheduleDescription)
                    .font(.subheadline)
                    .foregroundColor(.secondary)

                HStack {
                    Text(booking.frequencyDescription)
                        .font(.caption)
                        .foregroundColor(.secondary)

                    Text("•")
                        .foregroundColor(.secondary)

                    Text(booking.progressDescription)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(.vertical, 4)
    }

    private var statusColor: Color {
        switch booking.status {
        case .active: return .green
        case .paused: return .orange
        case .cancelled: return .red
        case .completed: return .gray
        }
    }
}

// MARK: - Recurring Status Badge

struct RecurringStatusBadge: View {
    let status: RecurringBookingStatus
    let themeManager: ThemeManager

    var body: some View {
        Text(status.displayName)
            .font(.caption2)
            .fontWeight(.medium)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(backgroundColor.opacity(0.1))
            .foregroundColor(backgroundColor)
            .cornerRadius(4)
    }

    private var backgroundColor: Color {
        switch status {
        case .active: return .green
        case .paused: return .orange
        case .cancelled: return .red
        case .completed: return .gray
        }
    }
}

#Preview {
    NavigationStack {
        RecurringBookingsListView()
    }
    .withThemeManager()
}
