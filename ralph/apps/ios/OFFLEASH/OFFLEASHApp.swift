//
//  OFFLEASHApp.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

@main
struct OFFLEASHApp: App {
    @StateObject private var themeManager = ThemeManager.shared
    @State private var isAuthenticated = KeychainHelper.shared.hasToken

    var body: some Scene {
        WindowGroup {
            if isAuthenticated {
                ContentView()
                    .withThemeManager(themeManager)
            } else {
                LoginView(onLoginSuccess: {
                    isAuthenticated = true
                })
                .withThemeManager(themeManager)
            }
        }
    }
}
