//
//  RescheduleBookingSheet.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct RescheduleBookingSheet: View {
    @EnvironmentObject private var themeManager: ThemeManager
    let booking: Booking
    let onReschedule: (Date) -> Void
    let onDismiss: () -> Void

    @State private var selectedDate: Date
    @State private var isSubmitting = false

    init(booking: Booking, onReschedule: @escaping (Date) -> Void, onDismiss: @escaping () -> Void) {
        self.booking = booking
        self.onReschedule = onReschedule
        self.onDismiss = onDismiss
        // Initialize with the current scheduled time, but at minimum tomorrow
        let minimumDate = Calendar.current.date(byAdding: .day, value: 1, to: Date()) ?? Date()
        self._selectedDate = State(initialValue: max(booking.scheduledStart, minimumDate))
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                // Current booking info
                VStack(alignment: .leading, spacing: 8) {
                    Text("Current Appointment")
                        .font(.subheadline)
                        .foregroundColor(.secondary)

                    HStack {
                        VStack(alignment: .leading, spacing: 4) {
                            Text(booking.serviceName ?? "Service")
                                .font(.headline)
                            Text("\(booking.dateString) at \(booking.timeString)")
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                        }
                        Spacer()
                    }
                    .padding()
                    .background(Color(.systemGray6))
                    .cornerRadius(12)
                }

                Divider()

                // New date/time picker
                VStack(alignment: .leading, spacing: 8) {
                    Text("Select New Date & Time")
                        .font(.subheadline)
                        .foregroundColor(.secondary)

                    DatePicker(
                        "New Date & Time",
                        selection: $selectedDate,
                        in: minimumAllowedDate...,
                        displayedComponents: [.date, .hourAndMinute]
                    )
                    .datePickerStyle(.graphical)
                    .labelsHidden()
                }

                Spacer()

                // Action buttons
                VStack(spacing: 12) {
                    Button(action: {
                        isSubmitting = true
                        onReschedule(selectedDate)
                    }) {
                        HStack {
                            if isSubmitting {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Text("Confirm Reschedule")
                                    .fontWeight(.semibold)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(themeManager.primaryColor)
                        .foregroundColor(.white)
                        .cornerRadius(12)
                    }
                    .disabled(isSubmitting || !isValidNewTime)

                    Button("Cancel") {
                        onDismiss()
                    }
                    .foregroundColor(.secondary)
                }
            }
            .padding()
            .navigationTitle("Reschedule")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        onDismiss()
                    }
                }
            }
        }
    }

    private var minimumAllowedDate: Date {
        // Minimum is tomorrow at the start of the day
        let calendar = Calendar.current
        let tomorrow = calendar.date(byAdding: .day, value: 1, to: Date()) ?? Date()
        return calendar.startOfDay(for: tomorrow)
    }

    private var isValidNewTime: Bool {
        // Must be a different time than current booking
        let calendar = Calendar.current
        return !calendar.isDate(selectedDate, equalTo: booking.scheduledStart, toGranularity: .minute)
    }
}

#Preview {
    RescheduleBookingSheet(
        booking: Booking(
            id: "1",
            customerId: "c1",
            customerName: "John Doe",
            walkerId: "w1",
            walkerName: "Jane Walker",
            serviceId: "s1",
            serviceName: "30-min Walk",
            locationId: "l1",
            locationAddress: "123 Main St",
            latitude: nil,
            longitude: nil,
            status: .confirmed,
            scheduledStart: Date(),
            scheduledEnd: Date().addingTimeInterval(1800),
            priceCents: 2500,
            priceDisplay: "$25.00",
            notes: nil,
            customerPhone: nil,
            petName: "Buddy",
            petBreed: "Golden Retriever"
        ),
        onReschedule: { _ in },
        onDismiss: {}
    )
    .withThemeManager()
}
