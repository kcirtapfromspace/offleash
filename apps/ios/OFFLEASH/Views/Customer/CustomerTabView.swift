//
//  CustomerTabView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct CustomerTabView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @EnvironmentObject private var sessionStateManager: SessionStateManager
    @State private var selectedTab = 0
    @State private var selectedService: Service?

    var body: some View {
        TabView(selection: $selectedTab) {
            // Services Tab
            ServicesView(onServiceSelected: { service in
                selectedService = service
            })
            .tabItem {
                Label("Services", systemImage: "pawprint.fill")
            }
            .tag(0)

            // Bookings Tab
            NavigationStack {
                CustomerBookingsView()
            }
            .tabItem {
                Label("Bookings", systemImage: "calendar")
            }
            .tag(1)

            // Profile Tab
            NavigationStack {
                CustomerProfileView()
            }
            .tabItem {
                Label("Profile", systemImage: "person.fill")
            }
            .tag(2)
        }
        .tint(themeManager.primaryColor)
        .sheet(item: $selectedService) { service in
            BookingFlowView(service: service)
                .withThemeManager(themeManager)
        }
        .onChange(of: sessionStateManager.sessionExpired) { expired in
            if expired {
                // Clear any in-progress booking state when session expires
                selectedService = nil
            }
        }
    }
}

#Preview {
    CustomerTabView()
        .withThemeManager()
        .environmentObject(SessionStateManager.shared)
}
