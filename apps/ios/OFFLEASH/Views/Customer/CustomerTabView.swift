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

    private var isUITestMode: Bool {
        TestAuthMode.isMock
    }

    var body: some View {
        ZStack(alignment: .topLeading) {
            TabView(selection: $selectedTab) {
                // Services Tab
                ServicesView(onServiceSelected: { service in
                    selectedService = service
                })
                .accessibilityIdentifier("tab-services")
                .tabItem {
                    Label("Services", systemImage: "pawprint.fill")
                }
                .tag(0)

                // Bookings Tab
                NavigationStack {
                    CustomerBookingsView()
                }
                .accessibilityIdentifier("tab-bookings")
                .tabItem {
                    Label("Bookings", systemImage: "calendar")
                }
                .tag(1)

                // Profile Tab
                NavigationStack {
                    CustomerProfileView()
                }
                .accessibilityIdentifier("tab-profile")
                .tabItem {
                    Label("Profile", systemImage: "person.fill")
                }
                .tag(2)
            }
            .tint(themeManager.primaryColor)

            if isUITestMode {
                HStack(spacing: 8) {
                    Button("S") { selectedTab = 0 }
                        .accessibilityIdentifier("tab-bar-services")
                        .frame(width: 24, height: 24)
                        .background(Color.black.opacity(0.12))
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                    Button("B") { selectedTab = 1 }
                        .accessibilityIdentifier("tab-bar-bookings")
                        .frame(width: 24, height: 24)
                        .background(Color.black.opacity(0.12))
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                    Button("P") { selectedTab = 2 }
                        .accessibilityIdentifier("tab-bar-profile")
                        .frame(width: 24, height: 24)
                        .background(Color.black.opacity(0.12))
                        .clipShape(RoundedRectangle(cornerRadius: 4))
                }
                .padding(6)
                .background(Color.black.opacity(0.2))
                .clipShape(RoundedRectangle(cornerRadius: 6))
                .padding([.top, .leading], 8)
                .zIndex(999)
                .accessibilityElement(children: .contain)
            }
        }
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
