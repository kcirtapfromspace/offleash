//
//  OFFLEASHApp.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import FirebaseCore
import FirebaseCrashlytics

// MARK: - App State

enum AppState {
    case launching
    case validating
    case authenticated
    case unauthenticated
}

// MARK: - Auth Screen State

enum AuthScreen {
    case login
    case register
}

// MARK: - Session State Manager

/// Manages session state across the app, allowing child views to clear their state on session expiry
@MainActor
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

/// Tracks whether Firebase was successfully configured
@MainActor
enum FirebaseState {
    static var isConfigured = false
}

@main
struct OFFLEASHApp: App {
    @StateObject private var themeManager = ThemeManager.shared
    @StateObject private var sessionStateManager = SessionStateManager.shared
    @State private var appState: AppState = .launching
    @State private var currentAuthScreen: AuthScreen = .login
    @State private var showSessionExpiredAlert = false

    init() {
        // Skip Firebase in UI testing mode or if configuration is invalid
        let isUITesting = ProcessInfo.processInfo.arguments.contains("--uitesting")
        if !isUITesting {
            configureFirebaseSafely()
        }
    }

    private func configureFirebaseSafely() {
        // Check if GoogleService-Info.plist has valid configuration
        guard let path = Bundle.main.path(forResource: "GoogleService-Info", ofType: "plist"),
              let plist = NSDictionary(contentsOfFile: path),
              let appId = plist["GOOGLE_APP_ID"] as? String,
              !appId.isEmpty,
              !appId.hasPrefix("YOUR_"),
              !appId.contains("placeholder"),
              !appId.contains("HERE") else {
            print("⚠️ Firebase: Skipping configuration - invalid or placeholder GoogleService-Info.plist")
            return
        }

        FirebaseApp.configure()
        FirebaseState.isConfigured = true
    }

    var body: some Scene {
        WindowGroup {
            Group {
                switch appState {
                case .launching, .validating:
                    VStack {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle())
                        if appState == .validating {
                            Text("Verifying session...")
                                .font(.subheadline)
                                .foregroundColor(.secondary)
                                .padding(.top, 8)
                        }
                    }
                    .withThemeManager(themeManager)

                case .authenticated:
                    ContentView()
                        .withThemeManager(themeManager)
                        .environmentObject(sessionStateManager)

                case .unauthenticated:
                    switch currentAuthScreen {
                    case .login:
                        LoginView(
                            onLoginSuccess: {
                                appState = .authenticated
                            },
                            onNavigateToRegister: {
                                currentAuthScreen = .register
                            }
                        )
                        .withThemeManager(themeManager)
                    case .register:
                        RegisterView(
                            onRegisterSuccess: {
                                appState = .authenticated
                            },
                            onNavigateToLogin: {
                                currentAuthScreen = .login
                            }
                        )
                        .withThemeManager(themeManager)
                    }
                }
            }
            .task {
                await validateTokenOnLaunch()
            }
            .onReceive(NotificationCenter.default.publisher(for: .authStateChanged)) { notification in
                if let userInfo = notification.userInfo,
                   let authenticated = userInfo["isAuthenticated"] as? Bool {
                    if !authenticated && appState == .authenticated {
                        // Session expired - notify child views to clear state
                        sessionStateManager.notifySessionExpired()
                        // Show alert before navigating to login
                        showSessionExpiredAlert = true
                    } else if authenticated {
                        appState = .authenticated
                    }
                }
            }
            .alert("Session Expired", isPresented: $showSessionExpiredAlert) {
                Button("OK") {
                    // Clear session state and navigate to login
                    sessionStateManager.resetSessionState()
                    appState = .unauthenticated
                    currentAuthScreen = .login
                }
            } message: {
                Text("Your session has expired. Please log in again.")
            }
        }
    }

    private func validateTokenOnLaunch() async {
        // Check if we have a stored token
        guard KeychainHelper.shared.hasToken else {
            appState = .unauthenticated
            return
        }

        // Token exists, validate it with the server
        appState = .validating

        do {
            let response = try await APIClient.shared.validateToken()
            if response.valid {
                appState = .authenticated
            } else {
                // Token is invalid, clear it and go to login
                await APIClient.shared.clearAuthToken()
                appState = .unauthenticated
            }
        } catch {
            // Check if it's a network error for graceful degradation
            if case APIError.networkError = error {
                // Network error - allow user to proceed (graceful degradation)
                appState = .authenticated
            } else if case APIError.unauthorized = error {
                // Token was rejected - go to login
                appState = .unauthenticated
            } else {
                // Other errors - allow user to proceed (graceful degradation)
                appState = .authenticated
            }
        }
    }
}
