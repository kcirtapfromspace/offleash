//
//  CustomerBookingsView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Customer Bookings View

struct CustomerBookingsView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var bookings: [Booking] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var selectedFilter: BookingFilter = .upcoming

    // Swipe action states
    @State private var bookingToCancel: Booking?
    @State private var showCancelConfirmation = false
    @State private var bookingToReschedule: Booking?
    @State private var showRescheduleSheet = false
    @State private var isCancelling = false
    @State private var cancelError: String?
    @State private var showCancelError = false

    enum BookingFilter: String, CaseIterable {
        case upcoming = "Upcoming"
        case past = "Past"
        case all = "All"
    }

    var body: some View {
        VStack(spacing: 0) {
            // Filter Picker
            Picker("Filter", selection: $selectedFilter) {
                ForEach(BookingFilter.allCases, id: \.self) { filter in
                    Text(filter.rawValue).tag(filter)
                }
            }
            .pickerStyle(.segmented)
            .padding()

            // Content
            Group {
                if isLoading && bookings.isEmpty {
                    loadingView
                } else if showError && bookings.isEmpty {
                    errorView
                } else if filteredBookings.isEmpty {
                    emptyView
                } else {
                    bookingsList
                }
            }
        }
        .navigationTitle("My Bookings")
        .refreshable {
            await refreshBookings()
        }
        .task {
            await fetchBookings()
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "customer_bookings")
        }
        .alert("Error", isPresented: $showError) {
            Button("Retry") {
                Task {
                    await fetchBookings()
                }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .alert("Cancel Booking", isPresented: $showCancelConfirmation) {
            Button("Keep Booking", role: .cancel) {
                bookingToCancel = nil
            }
            Button("Cancel Booking", role: .destructive) {
                if let booking = bookingToCancel {
                    Task {
                        await cancelBooking(booking)
                    }
                }
            }
        } message: {
            if let booking = bookingToCancel {
                Text("Are you sure you want to cancel your \(booking.serviceName ?? "booking") on \(booking.dateString)?")
            } else {
                Text("Are you sure you want to cancel this booking?")
            }
        }
        .alert("Unable to Cancel", isPresented: $showCancelError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(cancelError ?? "An error occurred while cancelling the booking.")
        }
        .sheet(isPresented: $showRescheduleSheet) {
            if let booking = bookingToReschedule {
                RescheduleBookingSheet(
                    booking: booking,
                    onReschedule: { newDate in
                        Task {
                            await rescheduleBooking(booking, to: newDate)
                        }
                    },
                    onDismiss: {
                        showRescheduleSheet = false
                        bookingToReschedule = nil
                    }
                )
                .environmentObject(themeManager)
            }
        }
    }

    private var filteredBookings: [Booking] {
        switch selectedFilter {
        case .upcoming:
            return bookings.filter { !$0.isPast && $0.status != .cancelled }
                .sorted { $0.scheduledStart < $1.scheduledStart }
        case .past:
            return bookings.filter { $0.isPast || $0.status == .completed || $0.status == .cancelled }
                .sorted { $0.scheduledStart > $1.scheduledStart }
        case .all:
            return bookings.sorted { $0.scheduledStart > $1.scheduledStart }
        }
    }

    // MARK: - Loading View

    private var loadingView: some View {
        VStack(spacing: 16) {
            ProgressView()
                .scaleEffect(1.5)
            Text("Loading bookings...")
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Error View

    private var errorView: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 48))
                .foregroundColor(.orange)

            Text("Unable to load bookings")
                .font(.headline)

            Text(errorMessage)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button(action: {
                Task {
                    await fetchBookings()
                }
            }) {
                Text("Try Again")
                    .fontWeight(.semibold)
                    .padding(.horizontal, 24)
                    .padding(.vertical, 12)
                    .background(themeManager.primaryColor)
                    .foregroundColor(.white)
                    .cornerRadius(8)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Empty View

    private var emptyView: some View {
        VStack(spacing: 16) {
            Image(systemName: "calendar.badge.exclamationmark")
                .font(.system(size: 48))
                .foregroundColor(.secondary)

            Text(emptyMessage)
                .font(.headline)

            Text(emptySubtitle)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var emptyMessage: String {
        switch selectedFilter {
        case .upcoming:
            return "No upcoming bookings"
        case .past:
            return "No past bookings"
        case .all:
            return "No bookings yet"
        }
    }

    private var emptySubtitle: String {
        switch selectedFilter {
        case .upcoming:
            return "Book a service to get started"
        case .past:
            return "Your completed bookings will appear here"
        case .all:
            return "Book a service to get started"
        }
    }

    // MARK: - Bookings List

    private var bookingsList: some View {
        List(filteredBookings) { booking in
            CustomerBookingRowView(booking: booking, themeManager: themeManager)
                .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                    if booking.canCancel {
                        Button(role: .destructive) {
                            bookingToCancel = booking
                            showCancelConfirmation = true
                        } label: {
                            Label("Cancel", systemImage: "xmark.circle")
                        }
                    }
                }
                .swipeActions(edge: .leading, allowsFullSwipe: false) {
                    if booking.canReschedule {
                        Button {
                            bookingToReschedule = booking
                            showRescheduleSheet = true
                        } label: {
                            Label("Reschedule", systemImage: "calendar.badge.clock")
                        }
                        .tint(.blue)
                    }
                }
        }
        .listStyle(.plain)
    }

    // MARK: - Data Fetching

    private func fetchBookings() async {
        isLoading = true

        do {
            let fetchedBookings: [Booking] = try await APIClient.shared.get("/bookings/customer")
            bookings = fetchedBookings
            isLoading = false
            showError = false
        } catch let error as APIError {
            isLoading = false
            errorMessage = error.errorDescription ?? "An unexpected error occurred"
            if bookings.isEmpty {
                showError = true
            }
        } catch {
            isLoading = false
            errorMessage = "An unexpected error occurred. Please try again."
            if bookings.isEmpty {
                showError = true
            }
        }
    }

    private func refreshBookings() async {
        await fetchBookings()
    }

    // MARK: - Cancel Booking

    private func cancelBooking(_ booking: Booking) async {
        isCancelling = true

        do {
            let _: EmptyResponse = try await APIClient.shared.post("/bookings/\(booking.id)/cancel", body: EmptyRequest())

            // Remove from local list or refresh
            await MainActor.run {
                bookings.removeAll { $0.id == booking.id }
                bookingToCancel = nil
                isCancelling = false
            }

            analyticsService.trackEvent(name: "booking_cancelled", params: ["booking_id": booking.id])

            // Refresh to get updated list
            await fetchBookings()
        } catch let error as APIError {
            await MainActor.run {
                isCancelling = false
                cancelError = error.errorDescription ?? "Failed to cancel booking"
                showCancelError = true
                bookingToCancel = nil
            }
        } catch {
            await MainActor.run {
                isCancelling = false
                cancelError = "An unexpected error occurred"
                showCancelError = true
                bookingToCancel = nil
            }
        }
    }

    // MARK: - Reschedule Booking

    private func rescheduleBooking(_ booking: Booking, to newDate: Date) async {
        do {
            let request = RescheduleBookingRequest(scheduledStart: ISO8601DateFormatter().string(from: newDate))
            let _: Booking = try await APIClient.shared.post("/bookings/\(booking.id)/reschedule", body: request)

            await MainActor.run {
                showRescheduleSheet = false
                bookingToReschedule = nil
            }

            analyticsService.trackEvent(name: "booking_rescheduled", params: ["booking_id": booking.id])

            // Refresh to get updated list
            await fetchBookings()
        } catch let error as APIError {
            await MainActor.run {
                showRescheduleSheet = false
                cancelError = error.errorDescription ?? "Failed to reschedule booking"
                showCancelError = true
                bookingToReschedule = nil
            }
        } catch {
            await MainActor.run {
                showRescheduleSheet = false
                cancelError = "An unexpected error occurred"
                showCancelError = true
                bookingToReschedule = nil
            }
        }
    }
}

// MARK: - Empty Request for Cancel

struct EmptyRequest: Encodable {}

// MARK: - Reschedule Request

struct RescheduleBookingRequest: Encodable {
    let scheduledStart: String
}

// MARK: - Customer Booking Row View

struct CustomerBookingRowView: View {
    let booking: Booking
    let themeManager: ThemeManager

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Service Name & Status
            HStack {
                Text(booking.serviceName ?? "Service")
                    .font(.headline)

                Spacer()

                CustomerStatusBadge(status: booking.status)
            }

            // Date & Time
            HStack(spacing: 16) {
                HStack(spacing: 4) {
                    Image(systemName: "calendar")
                        .font(.caption)
                    Text(booking.dateString)
                        .font(.subheadline)
                }
                .foregroundColor(.secondary)

                HStack(spacing: 4) {
                    Image(systemName: "clock")
                        .font(.caption)
                    Text(booking.timeRangeString)
                        .font(.subheadline)
                }
                .foregroundColor(.secondary)
            }

            // Walker & Location
            HStack(spacing: 16) {
                if let walkerName = booking.walkerName {
                    HStack(spacing: 4) {
                        Image(systemName: "person")
                            .font(.caption)
                        Text(walkerName)
                            .font(.subheadline)
                    }
                    .foregroundColor(.secondary)
                }

                if let address = booking.locationAddress {
                    HStack(spacing: 4) {
                        Image(systemName: "mappin")
                            .font(.caption)
                        Text(address)
                            .font(.subheadline)
                            .lineLimit(1)
                    }
                    .foregroundColor(.secondary)
                }
            }

            // Price
            HStack {
                Spacer()
                Text(booking.priceDisplay)
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundColor(themeManager.primaryColor)
            }
        }
        .padding(.vertical, 8)
    }
}

// MARK: - Customer Status Badge

struct CustomerStatusBadge: View {
    let status: BookingStatus

    var body: some View {
        Text(status.displayName)
            .font(.caption)
            .fontWeight(.medium)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(statusColor.opacity(0.1))
            .foregroundColor(statusColor)
            .cornerRadius(4)
    }

    private var statusColor: Color {
        switch status {
        case .pending:
            return .orange
        case .confirmed:
            return .blue
        case .inProgress:
            return .green
        case .completed:
            return .gray
        case .cancelled:
            return .red
        }
    }
}

#Preview {
    NavigationStack {
        CustomerBookingsView()
    }
    .withThemeManager()
}
