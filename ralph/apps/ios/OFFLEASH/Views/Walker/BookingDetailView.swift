//
//  BookingDetailView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import MapKit

struct BookingDetailView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    let booking: Booking
    let onAction: (BookingAction) -> Void

    @State private var showCancelConfirmation = false
    @State private var showCompleteConfirmation = false

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Status Header
                    statusHeader

                    // Time & Date Section
                    timeSection

                    // Customer Section
                    customerSection

                    // Service Section
                    serviceSection

                    // Location Section
                    locationSection

                    // Notes Section
                    if let notes = booking.notes, !notes.isEmpty {
                        notesSection(notes)
                    }

                    // Action Buttons
                    actionButtons
                }
                .padding()
            }
            .navigationTitle("Booking Details")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") { dismiss() }
                }
            }
            .confirmationDialog("Cancel Booking", isPresented: $showCancelConfirmation) {
                Button("Cancel Booking", role: .destructive) {
                    onAction(.cancel)
                    dismiss()
                }
                Button("Keep Booking", role: .cancel) {}
            } message: {
                Text("Are you sure you want to cancel this booking? The customer will be notified.")
            }
            .confirmationDialog("Complete Booking", isPresented: $showCompleteConfirmation) {
                Button("Mark as Complete") {
                    onAction(.complete)
                    dismiss()
                }
                Button("Not Yet", role: .cancel) {}
            } message: {
                Text("Mark this booking as completed?")
            }
        }
    }

    // MARK: - Status Header

    private var statusHeader: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(booking.status.displayName.uppercased())
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(statusColor)
                Text(booking.serviceName ?? "Service")
                    .font(.title2)
                    .fontWeight(.bold)
            }
            Spacer()
            Text(booking.priceDisplay)
                .font(.title)
                .fontWeight(.bold)
                .foregroundColor(themeManager.primaryColor)
        }
        .padding()
        .background(statusColor.opacity(0.1))
        .cornerRadius(12)
    }

    private var statusColor: Color {
        switch booking.status {
        case .pending: return .orange
        case .confirmed: return .blue
        case .inProgress: return .green
        case .completed: return .gray
        case .cancelled: return .red
        }
    }

    // MARK: - Time Section

    private var timeSection: some View {
        DetailSection(title: "Date & Time", icon: "calendar") {
            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Image(systemName: "calendar")
                        .foregroundColor(.secondary)
                        .frame(width: 24)
                    Text(booking.dateString)
                        .font(.body)
                }
                HStack {
                    Image(systemName: "clock")
                        .foregroundColor(.secondary)
                        .frame(width: 24)
                    Text(booking.timeRangeString)
                        .font(.body)
                }
                HStack {
                    Image(systemName: "hourglass")
                        .foregroundColor(.secondary)
                        .frame(width: 24)
                    Text("\(booking.duration) minutes")
                        .font(.body)
                }
            }
        }
    }

    // MARK: - Customer Section

    private var customerSection: some View {
        DetailSection(title: "Customer", icon: "person") {
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Image(systemName: "person.circle.fill")
                        .font(.title)
                        .foregroundColor(themeManager.primaryColor)
                    VStack(alignment: .leading, spacing: 2) {
                        Text(booking.customerName ?? "Customer")
                            .font(.headline)
                        if let phone = booking.customerPhone {
                            Text(phone)
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                        }
                    }
                    Spacer()
                    if booking.customerPhone != nil {
                        Button(action: { onAction(.call) }) {
                            Image(systemName: "phone.fill")
                                .font(.title2)
                                .foregroundColor(themeManager.primaryColor)
                                .padding(12)
                                .background(themeManager.primaryColor.opacity(0.1))
                                .clipShape(Circle())
                        }
                    }
                }

                if let petName = booking.petName {
                    Divider()
                    HStack {
                        Image(systemName: "pawprint.fill")
                            .foregroundColor(.secondary)
                            .frame(width: 24)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(petName)
                                .font(.body)
                            if let breed = booking.petBreed {
                                Text(breed)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                }
            }
        }
    }

    // MARK: - Service Section

    private var serviceSection: some View {
        DetailSection(title: "Service", icon: "pawprint") {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(booking.serviceName ?? "Service")
                        .font(.body)
                    Text("\(booking.duration) min")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Text(booking.priceDisplay)
                    .font(.headline)
                    .foregroundColor(themeManager.primaryColor)
            }
        }
    }

    // MARK: - Location Section

    private var locationSection: some View {
        DetailSection(title: "Location", icon: "mappin") {
            VStack(alignment: .leading, spacing: 12) {
                if let address = booking.locationAddress {
                    Text(address)
                        .font(.body)
                }

                Button(action: { onAction(.startNavigation) }) {
                    HStack {
                        Image(systemName: "arrow.triangle.turn.up.right.diamond.fill")
                        Text("Get Directions")
                            .fontWeight(.medium)
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(themeManager.primaryColor)
                    .foregroundColor(.white)
                    .cornerRadius(10)
                }
            }
        }
    }

    // MARK: - Notes Section

    private func notesSection(_ notes: String) -> some View {
        DetailSection(title: "Notes", icon: "note.text") {
            Text(notes)
                .font(.body)
                .foregroundColor(.secondary)
        }
    }

    // MARK: - Action Buttons

    private var actionButtons: some View {
        VStack(spacing: 12) {
            switch booking.status {
            case .pending:
                Button(action: {
                    onAction(.confirm)
                    dismiss()
                }) {
                    HStack {
                        Image(systemName: "checkmark.circle.fill")
                        Text("Accept Booking")
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color.green)
                    .foregroundColor(.white)
                    .cornerRadius(12)
                }

                Button(action: { showCancelConfirmation = true }) {
                    HStack {
                        Image(systemName: "xmark.circle")
                        Text("Decline")
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color(.systemGray5))
                    .foregroundColor(.red)
                    .cornerRadius(12)
                }

            case .confirmed:
                if booking.isToday || booking.scheduledStart <= Date() {
                    Button(action: { showCompleteConfirmation = true }) {
                        HStack {
                            Image(systemName: "checkmark.circle.fill")
                            Text("Mark as Complete")
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(Color.green)
                        .foregroundColor(.white)
                        .cornerRadius(12)
                    }
                }

                Button(action: { showCancelConfirmation = true }) {
                    HStack {
                        Image(systemName: "xmark.circle")
                        Text("Cancel Booking")
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color(.systemGray5))
                    .foregroundColor(.red)
                    .cornerRadius(12)
                }

            case .inProgress:
                Button(action: { showCompleteConfirmation = true }) {
                    HStack {
                        Image(systemName: "checkmark.circle.fill")
                        Text("Mark as Complete")
                    }
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(Color.green)
                    .foregroundColor(.white)
                    .cornerRadius(12)
                }

            case .completed, .cancelled:
                EmptyView()
            }
        }
        .padding(.top, 8)
    }
}

// MARK: - Detail Section

struct DetailSection<Content: View>: View {
    let title: String
    let icon: String
    @ViewBuilder let content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 8) {
                Image(systemName: icon)
                    .foregroundColor(.secondary)
                Text(title)
                    .font(.headline)
            }

            content
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}

#Preview {
    BookingDetailView(
        booking: Booking(
            id: "1",
            customerId: "c1",
            customerName: "John Smith",
            walkerId: "w1",
            walkerName: "Alex",
            serviceId: "s1",
            serviceName: "30 Min Walk",
            locationId: "l1",
            locationAddress: "123 Main St, San Francisco, CA",
            latitude: 37.7749,
            longitude: -122.4194,
            status: .confirmed,
            scheduledStart: Date(),
            scheduledEnd: Date().addingTimeInterval(1800),
            priceCents: 3500,
            priceDisplay: "$35.00",
            notes: "Please use the side gate",
            customerPhone: "555-123-4567",
            petName: "Max",
            petBreed: "Golden Retriever"
        ),
        onAction: { _ in }
    )
    .withThemeManager()
}
