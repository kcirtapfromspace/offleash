//
//  SessionExpiredAlert.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Session Expired Alert View Modifier

struct SessionExpiredAlertModifier: ViewModifier {
    @Binding var isPresented: Bool
    var onDismiss: () -> Void

    func body(content: Content) -> some View {
        content
            .alert("Session Expired", isPresented: $isPresented) {
                Button("OK", role: .cancel) {
                    onDismiss()
                }
            } message: {
                Text("Please log in again")
            }
    }
}

// MARK: - View Extension

extension View {
    /// Shows an alert when the user's session has expired
    /// - Parameters:
    ///   - isPresented: Binding to control alert visibility
    ///   - onDismiss: Callback triggered when user taps OK, typically used to navigate to login
    func sessionExpiredAlert(isPresented: Binding<Bool>, onDismiss: @escaping () -> Void) -> some View {
        modifier(SessionExpiredAlertModifier(isPresented: isPresented, onDismiss: onDismiss))
    }
}

// MARK: - Preview

#if DEBUG
struct SessionExpiredAlertPreview: View {
    @State private var showAlert = true

    var body: some View {
        VStack {
            Text("Content View")
            Button("Show Session Expired Alert") {
                showAlert = true
            }
        }
        .sessionExpiredAlert(isPresented: $showAlert) {
            print("Navigating to login...")
        }
    }
}

#Preview {
    SessionExpiredAlertPreview()
}
#endif
