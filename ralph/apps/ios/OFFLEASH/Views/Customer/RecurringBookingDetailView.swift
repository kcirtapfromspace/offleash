//
//  RecurringBookingDetailView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct RecurringBookingDetailView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss

    let booking: RecurringBooking
    var onCancel: () -> Void

    @State private var showCancelConfirmation = false
    @State private var isCancelling = false
    @State private var showError = false
    @State private var errorMessage = ""

    var body: some View {
        List {
            // Status Section
            Section {
                HStack {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(booking.serviceName ?? "Walk")
                            .font(.title2)
                            .fontWeight(.bold)

                        RecurringStatusBadge(status: booking.status, themeManager: themeManager)
                    }

                    Spacer()

                    Text(booking.priceDisplay)
                        .font(.title2)
                        .fontWeight(.semibold)
                        .foregroundColor(themeManager.primaryColor)
                }
                .listRowBackground(Color.clear)
            }

            // Schedule Section
            Section("Schedule") {
                RecurringDetailRow(
                    icon: "repeat",
                    title: "Frequency",
                    value: booking.frequencyDescription
                )

                RecurringDetailRow(
                    icon: "calendar",
                    title: "Schedule",
                    value: booking.scheduleDescription
                )

                if let nextDate = booking.nextBookingDescription {
                    RecurringDetailRow(
                        icon: "clock",
                        title: "Next Booking",
                        value: nextDate
                    )
                }
            }

            // Progress Section
            Section("Progress") {
                if let occurrences = booking.occurrences {
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text("Completed")
                            Spacer()
                            Text("\(booking.completedOccurrences) of \(occurrences)")
                                .foregroundColor(.secondary)
                        }

                        ProgressView(value: Double(booking.completedOccurrences), total: Double(occurrences))
                            .tint(themeManager.primaryColor)
                    }
                } else if let endDate = booking.endDate {
                    RecurringDetailRow(
                        icon: "calendar.badge.clock",
                        title: "End Date",
                        value: formatDate(endDate)
                    )

                    RecurringDetailRow(
                        icon: "checkmark.circle",
                        title: "Completed",
                        value: "\(booking.completedOccurrences) walks"
                    )
                } else {
                    RecurringDetailRow(
                        icon: "infinity",
                        title: "Duration",
                        value: "Ongoing"
                    )

                    RecurringDetailRow(
                        icon: "checkmark.circle",
                        title: "Completed",
                        value: "\(booking.completedOccurrences) walks"
                    )
                }
            }

            // Location Section
            if let address = booking.locationAddress {
                Section("Location") {
                    HStack(spacing: 12) {
                        Image(systemName: "mappin.circle.fill")
                            .font(.title2)
                            .foregroundColor(themeManager.primaryColor)

                        Text(address)
                            .font(.subheadline)
                    }
                }
            }

            // Walker Section
            if let walkerName = booking.walkerName {
                Section("Walker") {
                    HStack(spacing: 12) {
                        ZStack {
                            Circle()
                                .fill(themeManager.primaryColor.opacity(0.1))
                                .frame(width: 40, height: 40)

                            Text(walkerName.prefix(1).uppercased())
                                .font(.headline)
                                .foregroundColor(themeManager.primaryColor)
                        }

                        Text(walkerName)
                            .font(.subheadline)
                    }
                }
            }

            // Notes Section
            if let notes = booking.notes, !notes.isEmpty {
                Section("Notes") {
                    Text(notes)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
            }

            // Cancel Section (only for active bookings)
            if booking.status == .active {
                Section {
                    Button(role: .destructive) {
                        showCancelConfirmation = true
                    } label: {
                        HStack {
                            Spacer()
                            if isCancelling {
                                ProgressView()
                                    .tint(.red)
                            } else {
                                Text("Cancel Recurring Series")
                                    .fontWeight(.medium)
                            }
                            Spacer()
                        }
                    }
                    .disabled(isCancelling)
                } footer: {
                    Text("This will cancel all future bookings in this series. Past bookings will not be affected.")
                }
            }
        }
        .navigationTitle("Recurring Walk")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Done") {
                    dismiss()
                }
            }
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "recurring_booking_detail")
        }
        .alert("Cancel Series", isPresented: $showCancelConfirmation) {
            Button("Keep", role: .cancel) {}
            Button("Cancel Series", role: .destructive) {
                cancelRecurringSeries()
            }
        } message: {
            Text("Are you sure you want to cancel this recurring walk series? All future bookings will be cancelled.")
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
    }

    // MARK: - Helpers

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        return formatter.string(from: date)
    }

    // MARK: - Cancel Series

    private func cancelRecurringSeries() {
        isCancelling = true

        Task {
            do {
                let _: CancelRecurringBookingResponse = try await APIClient.shared.delete("/bookings/recurring/\(booking.id)")
                await MainActor.run {
                    isCancelling = false
                    analyticsService.trackEvent(name: "recurring_booking_cancelled", params: ["booking_id": booking.id])
                    onCancel()
                    dismiss()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isCancelling = false
                    errorMessage = error.errorDescription ?? "Failed to cancel series"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isCancelling = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }
}

// MARK: - Recurring Detail Row

struct RecurringDetailRow: View {
    let icon: String
    let title: String
    let value: String

    var body: some View {
        HStack {
            Image(systemName: icon)
                .foregroundColor(.secondary)
                .frame(width: 24)

            Text(title)
                .foregroundColor(.primary)

            Spacer()

            Text(value)
                .foregroundColor(.secondary)
        }
    }
}

#Preview {
    NavigationStack {
        RecurringBookingDetailView(
            booking: RecurringBooking(
                id: "1",
                customerId: "cust1",
                walkerId: "walker1",
                walkerName: "Mike Thompson",
                serviceId: "svc1",
                serviceName: "30-Minute Walk",
                locationId: "loc1",
                locationAddress: "1301 Pearl St, Boulder, CO 80302",
                frequency: .weekly,
                dayOfWeek: 2,
                timeSlot: "10:00 AM",
                status: .active,
                startDate: Date(),
                endDate: nil,
                occurrences: 12,
                completedOccurrences: 4,
                nextBookingDate: Date().addingTimeInterval(86400 * 7),
                priceCents: 2500,
                priceDisplay: "$25.00",
                notes: "Luna loves exploring the creek trails!",
                createdAt: Date()
            ),
            onCancel: {}
        )
    }
    .withThemeManager()
}
