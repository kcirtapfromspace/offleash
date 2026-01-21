//
//  ContentView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @EnvironmentObject private var sessionStateManager: SessionStateManager
    @State private var selectedService: Service?

    var body: some View {
        ServicesView(onServiceSelected: { service in
            selectedService = service
        })
        .sheet(item: $selectedService) { service in
            BookingStartView(service: service)
                .withThemeManager(themeManager)
        }
        .onChange(of: sessionStateManager.sessionExpired) { _, expired in
            if expired {
                // Clear any in-progress booking state when session expires
                selectedService = nil
            }
        }
    }
}

// MARK: - Booking Start View (Placeholder)
// TODO: When booking confirmation is implemented, call:
// analyticsService.trackFunnelStep(step: "booking_confirmed", serviceId: service.id, locationId: selectedLocationId)

struct BookingStartView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss
    let service: Service

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Image(systemName: "calendar.badge.plus")
                    .font(.system(size: 64))
                    .foregroundColor(themeManager.primaryColor)

                Text("Book \(service.name)")
                    .font(.title2)
                    .fontWeight(.bold)

                VStack(spacing: 8) {
                    HStack {
                        Text("Duration:")
                            .foregroundColor(.secondary)
                        Spacer()
                        Text(formatDuration(service.durationMinutes))
                    }
                    HStack {
                        Text("Price:")
                            .foregroundColor(.secondary)
                        Spacer()
                        Text(service.priceDisplay)
                            .fontWeight(.semibold)
                            .foregroundColor(themeManager.primaryColor)
                    }
                }
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)

                Text("Booking flow coming soon...")
                    .foregroundColor(.secondary)
                    .font(.subheadline)

                Spacer()
            }
            .padding()
            .navigationTitle("Start Booking")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "booking_start")
            analyticsService.trackFunnelStep(step: "booking_started", serviceId: service.id, locationId: nil)
        }
    }

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
}

#Preview {
    ContentView()
        .withThemeManager()
        .environmentObject(SessionStateManager.shared)
}
