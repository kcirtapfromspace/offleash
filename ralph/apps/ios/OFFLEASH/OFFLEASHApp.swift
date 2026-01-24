//
//  OFFLEASHApp.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import FirebaseCore
import FirebaseCrashlytics
import GoogleSignIn

// MARK: - App State

enum AppState {
    case launching
    case validating
    case authenticated
    case unauthenticated
}

// MARK: - Auth Flow State

enum AuthFlowState {
    case roleSelection
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
    @State private var authFlowState: AuthFlowState = .roleSelection
    @State private var selectedRole: SelectedRole = .customer
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
                    ContentView(selectedRole: selectedRole)
                        .withThemeManager(themeManager)
                        .environmentObject(sessionStateManager)

                case .unauthenticated:
                    switch authFlowState {
                    case .roleSelection:
                        RoleSelectionView { role in
                            selectedRole = role
                            authFlowState = .login
                        }
                        .withThemeManager(themeManager)

                    case .login:
                        LoginView(
                            selectedRole: selectedRole,
                            onLoginSuccess: {
                                appState = .authenticated
                            },
                            onNavigateToRegister: {
                                authFlowState = .register
                            },
                            onBack: {
                                authFlowState = .roleSelection
                            }
                        )
                        .withThemeManager(themeManager)

                    case .register:
                        RegisterView(
                            selectedRole: selectedRole,
                            onRegisterSuccess: {
                                appState = .authenticated
                            },
                            onNavigateToLogin: {
                                authFlowState = .login
                            },
                            onBack: {
                                authFlowState = .roleSelection
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
                    // Clear session state and navigate to role selection
                    sessionStateManager.resetSessionState()
                    appState = .unauthenticated
                    authFlowState = .roleSelection
                }
            } message: {
                Text("Your session has expired. Please log in again.")
            }
            .onOpenURL { url in
                // Handle Google Sign-In callback
                GIDSignIn.sharedInstance.handle(url)
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
                // Update user data from token validation response
                if let userData = response.user {
                    let role = UserRole(rawValue: userData.role ?? "customer") ?? .customer
                    let user = User(
                        id: userData.id,
                        email: userData.email,
                        firstName: userData.firstName,
                        lastName: userData.lastName,
                        role: role,
                        organizationId: userData.organizationId
                    )
                    await MainActor.run {
                        UserSession.shared.setUser(user)
                        // Set selectedRole based on user's actual role for proper routing
                        selectedRole = role == .walker ? .walker : .customer
                    }
                }
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
