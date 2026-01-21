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

// MARK: - Session State Manager

/// Manages session state across the app, allowing child views to clear their state on session expiry
final class SessionStateManager: ObservableObject {
    static let shared = SessionStateManager()

    @Published var sessionExpired = false

    private init() {}

    func notifySessionExpired() {
        sessionExpired = true
    }

    func resetSessionState() {
        sessionExpired = false
    }
}

@main
struct OFFLEASHApp: App {
    @StateObject private var themeManager = ThemeManager.shared
    @StateObject private var sessionStateManager = SessionStateManager.shared
    @State private var isAuthenticated = KeychainHelper.shared.hasToken
    @State private var currentAuthScreen: AuthScreen = .login
    @State private var showSessionExpiredAlert = false

    var body: some Scene {
        WindowGroup {
            Group {
                if isAuthenticated {
                    ContentView()
                        .withThemeManager(themeManager)
                        .environmentObject(sessionStateManager)
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
            .onReceive(NotificationCenter.default.publisher(for: .authStateChanged)) { notification in
                if let userInfo = notification.userInfo,
                   let authenticated = userInfo["isAuthenticated"] as? Bool {
                    if !authenticated && isAuthenticated {
                        // Session expired - notify child views to clear state
                        sessionStateManager.notifySessionExpired()
                        // Show alert before navigating to login
                        showSessionExpiredAlert = true
                    } else {
                        isAuthenticated = authenticated
                    }
                }
            }
            .alert("Session Expired", isPresented: $showSessionExpiredAlert) {
                Button("OK") {
                    // Clear session state and navigate to login
                    sessionStateManager.resetSessionState()
                    isAuthenticated = false
                    currentAuthScreen = .login
                }
            } message: {
                Text("Your session has expired. Please log in again.")
            }
        }
    }
}
