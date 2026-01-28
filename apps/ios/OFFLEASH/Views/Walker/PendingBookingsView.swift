//
//  PendingBookingsView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct PendingBookingsView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @StateObject private var viewModel = PendingBookingsViewModel()

    var body: some View {
        Group {
            if viewModel.isLoading {
                ProgressView()
            } else if viewModel.pendingBookings.isEmpty {
                emptyState
            } else {
                bookingsList
            }
        }
        .navigationTitle("Pending Requests")
        .navigationBarTitleDisplayMode(.inline)
        .refreshable {
            await viewModel.loadBookings()
        }
        .sheet(item: $viewModel.selectedBooking) { booking in
            BookingDetailView(booking: booking) { action in
                Task {
                    await viewModel.handleBookingAction(booking: booking, action: action)
                }
            }
        }
        .task {
            await viewModel.loadBookings()
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "pending_bookings")
        }
    }

    // MARK: - Empty State

    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "tray")
                .font(.system(size: 48))
                .foregroundColor(.secondary)

            Text("No Pending Requests")
                .font(.headline)

            Text("New booking requests will appear here for you to accept or decline")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)
        }
    }

    // MARK: - Bookings List

    private var bookingsList: some View {
        List {
            ForEach(viewModel.pendingBookings) { booking in
                PendingBookingRow(booking: booking, primaryColor: themeManager.primaryColor) {
                    viewModel.selectedBooking = booking
                } onAccept: {
                    Task {
                        await viewModel.confirmBooking(booking)
                    }
                } onDecline: {
                    Task {
                        await viewModel.cancelBooking(booking)
                    }
                }
            }
        }
        .listStyle(.plain)
    }
}

// MARK: - Pending Booking Row

struct PendingBookingRow: View {
    let booking: Booking
    let primaryColor: Color
    let onTap: () -> Void
    let onAccept: () -> Void
    let onDecline: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header with customer and time
            Button(action: onTap) {
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(booking.customerName ?? "Customer")
                            .font(.headline)
                            .foregroundColor(.primary)

                        Text(booking.serviceName ?? "Service")
                            .font(.subheadline)
                            .foregroundColor(.secondary)

                        HStack(spacing: 4) {
                            Image(systemName: "calendar")
                                .font(.caption)
                            Text(booking.dateString)
                            Text("at")
                            Text(booking.timeString)
                        }
                        .font(.caption)
                        .foregroundColor(.secondary)

                        if let address = booking.locationAddress {
                            HStack(spacing: 4) {
                                Image(systemName: "mappin")
                                    .font(.caption)
                                Text(address)
                                    .lineLimit(1)
                            }
                            .font(.caption)
                            .foregroundColor(.secondary)
                        }
                    }

                    Spacer()

                    VStack(alignment: .trailing, spacing: 4) {
                        Text("\(booking.duration) min")
                            .font(.headline)
                            .foregroundColor(primaryColor)
                    }
                }
            }
            .buttonStyle(.plain)

            // Action buttons
            HStack(spacing: 12) {
                Button(action: onDecline) {
                    Text("Decline")
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 10)
                        .background(Color(.systemGray5))
                        .foregroundColor(.red)
                        .cornerRadius(8)
                }

                Button(action: onAccept) {
                    Text("Accept")
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 10)
                        .background(Color.green)
                        .foregroundColor(.white)
                        .cornerRadius(8)
                }
            }
        }
        .padding(.vertical, 8)
    }
}

// MARK: - View Model

@MainActor
class PendingBookingsViewModel: ObservableObject {
    @Published var pendingBookings: [Booking] = []
    @Published var isLoading = false
    @Published var selectedBooking: Booking?

    func loadBookings() async {
        isLoading = true
        defer { isLoading = false }

        do {
            let allBookings: [Booking] = try await APIClient.shared.get("/bookings/walker")
            pendingBookings = allBookings
                .filter { $0.status == .pending }
                .sorted { $0.scheduledStart < $1.scheduledStart }
        } catch {
            print("Error loading bookings: \(error)")
        }
    }

    func confirmBooking(_ booking: Booking) async {
        do {
            let _: Booking = try await APIClient.shared.post("/bookings/\(booking.id)/confirm")
            await loadBookings()
        } catch {
            print("Error confirming booking: \(error)")
        }
    }

    func cancelBooking(_ booking: Booking) async {
        do {
            let _: Booking = try await APIClient.shared.post("/bookings/\(booking.id)/cancel")
            await loadBookings()
        } catch {
            print("Error cancelling booking: \(error)")
        }
    }

    func handleBookingAction(booking: Booking, action: BookingAction) async {
        switch action {
        case .confirm:
            await confirmBooking(booking)
        case .cancel:
            await cancelBooking(booking)
        default:
            break
        }
        selectedBooking = nil
    }
}

#Preview {
    NavigationStack {
        PendingBookingsView()
    }
    .withThemeManager()
}
