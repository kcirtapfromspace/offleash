//
//  BookingFlowView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Booking Flow Step

enum BookingFlowStep: Int, CaseIterable {
    case location = 0
    case dateTime = 1
    case review = 2
    case confirmation = 3

    var title: String {
        switch self {
        case .location: return "Location"
        case .dateTime: return "Date & Time"
        case .review: return "Review"
        case .confirmation: return "Confirmed"
        }
    }
}

// MARK: - Booking Flow View

struct BookingFlowView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss

    let service: Service

    @State private var currentStep: BookingFlowStep = .location
    @State private var selectedLocation: Location?
    @State private var selectedDate: Date = Date()
    @State private var selectedTimeSlot: TimeSlot?
    @State private var notes: String = ""
    @State private var isSubmitting = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var createdBooking: Booking?

    // Available time slots (in production, these would come from API)
    @State private var availableTimeSlots: [TimeSlot] = []
    @State private var isLoadingSlots = false
    @State private var showAddLocation = false

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Progress Indicator
                if currentStep != .confirmation {
                    progressIndicator
                }

                // Content
                Group {
                    switch currentStep {
                    case .location:
                        locationStepView
                    case .dateTime:
                        dateTimeStepView
                    case .review:
                        reviewStepView
                    case .confirmation:
                        confirmationView
                    }
                }
            }
            .navigationTitle(currentStep.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    if currentStep != .confirmation {
                        Button("Cancel") {
                            dismiss()
                        }
                    }
                }

                ToolbarItem(placement: .navigationBarLeading) {
                    if currentStep.rawValue > 0 && currentStep != .confirmation {
                        Button {
                            withAnimation {
                                currentStep = BookingFlowStep(rawValue: currentStep.rawValue - 1) ?? .location
                            }
                        } label: {
                            Image(systemName: "chevron.left")
                        }
                    }
                }
            }
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "booking_flow")
            analyticsService.trackFunnelStep(step: "booking_started", serviceId: service.id, locationId: nil)
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .sheet(isPresented: $showAddLocation) {
            AddLocationView(onLocationAdded: { location in
                selectedLocation = location
                analyticsService.trackFunnelStep(step: "location_selected", serviceId: service.id, locationId: location.id)
                withAnimation {
                    currentStep = .dateTime
                }
            })
            .withThemeManager(themeManager)
        }
    }

    // MARK: - Progress Indicator

    private var progressIndicator: some View {
        HStack(spacing: 8) {
            ForEach(0..<3) { index in
                VStack(spacing: 4) {
                    Circle()
                        .fill(index <= currentStep.rawValue ? themeManager.primaryColor : Color(.systemGray4))
                        .frame(width: 24, height: 24)
                        .overlay {
                            if index < currentStep.rawValue {
                                Image(systemName: "checkmark")
                                    .font(.caption)
                                    .fontWeight(.bold)
                                    .foregroundColor(.white)
                            } else {
                                Text("\(index + 1)")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                    .foregroundColor(index <= currentStep.rawValue ? .white : .secondary)
                            }
                        }

                    Text(BookingFlowStep(rawValue: index)?.title ?? "")
                        .font(.caption2)
                        .foregroundColor(index <= currentStep.rawValue ? themeManager.primaryColor : .secondary)
                }

                if index < 2 {
                    Rectangle()
                        .fill(index < currentStep.rawValue ? themeManager.primaryColor : Color(.systemGray4))
                        .frame(height: 2)
                        .padding(.bottom, 16)
                }
            }
        }
        .padding()
    }

    // MARK: - Location Step

    private var locationStepView: some View {
        LocationSelectionView(
            serviceId: service.id,
            onLocationSelected: { location in
                selectedLocation = location
                analyticsService.trackFunnelStep(step: "location_selected", serviceId: service.id, locationId: location.id)
                withAnimation {
                    currentStep = .dateTime
                }
            },
            onAddLocationTapped: {
                showAddLocation = true
            }
        )
    }

    // MARK: - Date & Time Step

    private var dateTimeStepView: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                // Date Picker
                VStack(alignment: .leading, spacing: 12) {
                    Text("Select Date")
                        .font(.headline)

                    DatePicker(
                        "Date",
                        selection: $selectedDate,
                        in: Date()...,
                        displayedComponents: .date
                    )
                    .datePickerStyle(.graphical)
                    .tint(themeManager.primaryColor)
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)

                // Time Slots
                VStack(alignment: .leading, spacing: 12) {
                    Text("Select Time")
                        .font(.headline)

                    if isLoadingSlots {
                        HStack {
                            Spacer()
                            ProgressView()
                            Spacer()
                        }
                        .padding()
                    } else if availableTimeSlots.isEmpty {
                        Text("No available time slots for this date")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .padding()
                    } else {
                        LazyVGrid(columns: [
                            GridItem(.flexible()),
                            GridItem(.flexible()),
                            GridItem(.flexible())
                        ], spacing: 12) {
                            ForEach(availableTimeSlots) { slot in
                                TimeSlotButton(
                                    slot: slot,
                                    isSelected: selectedTimeSlot?.id == slot.id,
                                    themeManager: themeManager
                                ) {
                                    selectedTimeSlot = slot
                                }
                            }
                        }
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)

                // Notes
                VStack(alignment: .leading, spacing: 12) {
                    Text("Notes (Optional)")
                        .font(.headline)

                    TextField("Any special instructions...", text: $notes, axis: .vertical)
                        .textFieldStyle(.roundedBorder)
                        .lineLimit(3...6)
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)

                // Continue Button
                Button {
                    withAnimation {
                        currentStep = .review
                    }
                } label: {
                    Text("Continue to Review")
                        .fontWeight(.semibold)
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(selectedTimeSlot != nil ? themeManager.primaryColor : Color(.systemGray4))
                        .foregroundColor(.white)
                        .cornerRadius(12)
                }
                .disabled(selectedTimeSlot == nil)
            }
            .padding()
        }
        .onChange(of: selectedDate) { _ in
            loadTimeSlots()
        }
        .task {
            loadTimeSlots()
        }
    }

    private func loadTimeSlots() {
        isLoadingSlots = true
        selectedTimeSlot = nil

        // Generate time slots (in production, fetch from API based on walker availability)
        Task {
            try? await Task.sleep(nanoseconds: 500_000_000) // Simulate API call

            let calendar = Calendar.current
            var slots: [TimeSlot] = []

            // Generate slots from 8 AM to 6 PM
            for hour in 8..<18 {
                if let slotTime = calendar.date(bySettingHour: hour, minute: 0, second: 0, of: selectedDate) {
                    // Skip past times for today
                    if calendar.isDateInToday(selectedDate) && slotTime <= Date() {
                        continue
                    }

                    let slot = TimeSlot(
                        id: "\(hour):00",
                        startTime: slotTime,
                        endTime: calendar.date(byAdding: .minute, value: service.durationMinutes, to: slotTime) ?? slotTime,
                        isAvailable: Bool.random() // In production, check actual availability
                    )
                    if slot.isAvailable {
                        slots.append(slot)
                    }
                }
            }

            await MainActor.run {
                availableTimeSlots = slots
                isLoadingSlots = false
            }
        }
    }

    // MARK: - Review Step

    private var reviewStepView: some View {
        ScrollView {
            VStack(spacing: 24) {
                // Service Summary
                VStack(alignment: .leading, spacing: 12) {
                    Text("Service")
                        .font(.headline)

                    HStack {
                        Image(systemName: "pawprint.fill")
                            .font(.title2)
                            .foregroundColor(themeManager.primaryColor)

                        VStack(alignment: .leading) {
                            Text(service.name)
                                .font(.subheadline)
                                .fontWeight(.medium)
                            Text("\(formatDuration(service.durationMinutes))")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }

                        Spacer()

                        Text(service.priceDisplay)
                            .font(.headline)
                            .foregroundColor(themeManager.primaryColor)
                    }
                    .padding()
                    .background(Color(.systemGray6))
                    .cornerRadius(8)
                }

                // Location Summary
                if let location = selectedLocation {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Location")
                            .font(.headline)

                        HStack {
                            Image(systemName: "mappin.circle.fill")
                                .font(.title2)
                                .foregroundColor(themeManager.primaryColor)

                            VStack(alignment: .leading) {
                                Text(location.name)
                                    .font(.subheadline)
                                    .fontWeight(.medium)
                                Text(location.fullAddress)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }

                            Spacer()
                        }
                        .padding()
                        .background(Color(.systemGray6))
                        .cornerRadius(8)
                    }
                }

                // Date & Time Summary
                if let timeSlot = selectedTimeSlot {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Date & Time")
                            .font(.headline)

                        HStack {
                            Image(systemName: "calendar")
                                .font(.title2)
                                .foregroundColor(themeManager.primaryColor)

                            VStack(alignment: .leading) {
                                Text(formatDate(selectedDate))
                                    .font(.subheadline)
                                    .fontWeight(.medium)
                                Text(formatTimeRange(timeSlot))
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }

                            Spacer()
                        }
                        .padding()
                        .background(Color(.systemGray6))
                        .cornerRadius(8)
                    }
                }

                // Notes Summary
                if !notes.isEmpty {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Notes")
                            .font(.headline)

                        Text(notes)
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .padding()
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Color(.systemGray6))
                            .cornerRadius(8)
                    }
                }

                Spacer()

                // Confirm Button
                Button {
                    submitBooking()
                } label: {
                    HStack {
                        if isSubmitting {
                            ProgressView()
                                .tint(.white)
                        } else {
                            Text("Confirm Booking")
                                .fontWeight(.semibold)
                        }
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(themeManager.primaryColor)
                    .foregroundColor(.white)
                    .cornerRadius(12)
                }
                .disabled(isSubmitting)
            }
            .padding()
        }
    }

    // MARK: - Confirmation View

    private var confirmationView: some View {
        VStack(spacing: 32) {
            Spacer()

            // Success Icon
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 80))
                .foregroundColor(.green)

            VStack(spacing: 8) {
                Text("Booking Confirmed!")
                    .font(.title2)
                    .fontWeight(.bold)

                Text("Your booking has been submitted successfully.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
            }

            // Booking Summary
            if let booking = createdBooking {
                VStack(spacing: 16) {
                    HStack {
                        Text("Service")
                            .foregroundColor(.secondary)
                        Spacer()
                        Text(booking.serviceName ?? service.name)
                    }

                    HStack {
                        Text("Date")
                            .foregroundColor(.secondary)
                        Spacer()
                        Text(booking.dateString)
                    }

                    HStack {
                        Text("Time")
                            .foregroundColor(.secondary)
                        Spacer()
                        Text(booking.timeRangeString)
                    }

                    Divider()

                    HStack {
                        Text("Total")
                            .fontWeight(.semibold)
                        Spacer()
                        Text(booking.priceDisplay)
                            .fontWeight(.semibold)
                            .foregroundColor(themeManager.primaryColor)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)
            }

            Spacer()

            // Done Button
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
        }
        .padding()
    }

    // MARK: - Submit Booking

    private func submitBooking() {
        guard let location = selectedLocation,
              let timeSlot = selectedTimeSlot else { return }

        isSubmitting = true

        let formatter = ISO8601DateFormatter()
        let request = CreateBookingRequest(
            walkerId: nil, // Will be assigned by backend based on availability
            serviceId: service.id,
            locationId: location.id,
            startTime: formatter.string(from: timeSlot.startTime),
            notes: notes.isEmpty ? nil : notes
        )

        Task {
            do {
                let booking: Booking = try await APIClient.shared.post("/bookings", body: request)
                await MainActor.run {
                    createdBooking = booking
                    isSubmitting = false
                    analyticsService.trackFunnelStep(step: "booking_confirmed", serviceId: service.id, locationId: location.id)
                    withAnimation {
                        currentStep = .confirmation
                    }
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSubmitting = false
                    errorMessage = error.errorDescription ?? "Failed to create booking"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isSubmitting = false
                    errorMessage = "An unexpected error occurred. Please try again."
                    showError = true
                }
            }
        }
    }

    // MARK: - Helpers

    private func formatDuration(_ minutes: Int) -> String {
        if minutes < 60 {
            return "\(minutes) min"
        } else if minutes % 60 == 0 {
            let hours = minutes / 60
            return hours == 1 ? "1 hr" : "\(hours) hrs"
        } else {
            let hours = minutes / 60
            let remainingMinutes = minutes % 60
            return "\(hours) hr \(remainingMinutes) min"
        }
    }

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .full
        return formatter.string(from: date)
    }

    private func formatTimeRange(_ slot: TimeSlot) -> String {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return "\(formatter.string(from: slot.startTime)) - \(formatter.string(from: slot.endTime))"
    }
}

// MARK: - Time Slot Model

struct TimeSlot: Identifiable {
    let id: String
    let startTime: Date
    let endTime: Date
    let isAvailable: Bool

    var displayTime: String {
        let formatter = DateFormatter()
        formatter.timeStyle = .short
        return formatter.string(from: startTime)
    }
}

// MARK: - Time Slot Button

struct TimeSlotButton: View {
    let slot: TimeSlot
    let isSelected: Bool
    let themeManager: ThemeManager
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(slot.displayTime)
                .font(.subheadline)
                .fontWeight(isSelected ? .semibold : .regular)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
                .background(isSelected ? themeManager.primaryColor : Color(.systemGray6))
                .foregroundColor(isSelected ? .white : .primary)
                .cornerRadius(8)
        }
    }
}

#Preview {
    BookingFlowView(service: Service(
        id: "preview",
        name: "30-Minute Walk",
        description: "A quick walk for your pup",
        durationMinutes: 30,
        priceCents: 2500,
        priceDisplay: "$25.00",
        isActive: true
    ))
    .withThemeManager()
}
