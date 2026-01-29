//
//  WalkerTabView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct WalkerTabView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @State private var selectedTab = 0

    var body: some View {
        TabView(selection: $selectedTab) {
            WalkerDashboardView()
                .accessibilityIdentifier("walker-dashboard")
                .tabItem {
                    Label("Home", systemImage: "house.fill")
                }
                .tag(0)

            NavigationStack {
                WalkerCalendarView()
            }
            .accessibilityIdentifier("tab-calendar")
            .tabItem {
                Label("Calendar", systemImage: "calendar")
            }
            .tag(1)

            NavigationStack {
                WalkerMapView()
            }
            .accessibilityIdentifier("tab-map")
            .tabItem {
                Label("Map", systemImage: "map.fill")
            }
            .tag(2)

            NavigationStack {
                PendingBookingsView()
            }
            .accessibilityIdentifier("tab-requests")
            .tabItem {
                Label("Requests", systemImage: "tray.full.fill")
            }
            .tag(3)
        }
        .tint(themeManager.primaryColor)
    }
}

#Preview {
    WalkerTabView()
        .withThemeManager()
}
