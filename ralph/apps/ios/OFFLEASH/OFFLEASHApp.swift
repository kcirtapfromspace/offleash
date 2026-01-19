//
//  OFFLEASHApp.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Auth Screen State

enum AuthScreen {
    case login
    case register
}

@main
struct OFFLEASHApp: App {
    @StateObject private var themeManager = ThemeManager.shared
    @State private var isAuthenticated = KeychainHelper.shared.hasToken
    @State private var currentAuthScreen: AuthScreen = .login

    var body: some Scene {
        WindowGroup {
            if isAuthenticated {
                ContentView()
                    .withThemeManager(themeManager)
            } else {
                switch currentAuthScreen {
                case .login:
                    LoginView(
                        onLoginSuccess: {
                            isAuthenticated = true
                        },
                        onNavigateToRegister: {
                            currentAuthScreen = .register
                        }
                    )
                    .withThemeManager(themeManager)
                case .register:
                    RegisterView(
                        onRegisterSuccess: {
                            isAuthenticated = true
                        },
                        onNavigateToLogin: {
                            currentAuthScreen = .login
                        }
                    )
                    .withThemeManager(themeManager)
                }
            }
        }
    }
}
