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
    @ObservedObject private var userSession = UserSession.shared

    var body: some View {
        Group {
            // Show different views based on user role
            if userSession.isWalker {
                WalkerTabView()
            } else {
                // Customer view - tab-based navigation
                CustomerTabView()
            }
        }
    }
}

#Preview {
    ContentView()
        .withThemeManager()
        .environmentObject(SessionStateManager.shared)
}
