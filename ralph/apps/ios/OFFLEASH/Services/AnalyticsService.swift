//
//  AnalyticsService.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation
import SwiftUI

// MARK: - Analytics Service Protocol

/// Protocol defining the analytics service interface for tracking user events and errors.
/// Use this abstraction to enable testability and swap implementations.
@MainActor
protocol AnalyticsService: AnyObject {
    /// Track a screen view event
    /// - Parameter screenName: The name of the screen being viewed
    func trackScreenView(screenName: String)

    /// Track a custom event with optional parameters
    /// - Parameters:
    ///   - name: The name of the event
    ///   - params: Optional dictionary of event parameters
    func trackEvent(name: String, params: [String: Any]?)

    /// Track an error with context information
    /// - Parameters:
    ///   - error: The error that occurred
    ///   - context: Additional context about where/why the error occurred
    func trackError(error: Error, context: String?)

    /// Track a booking funnel step
    /// - Parameters:
    ///   - step: The funnel step name (services_viewed, service_selected, location_selected, booking_started, booking_confirmed)
    ///   - serviceId: Optional service ID (included where applicable)
    ///   - locationId: Optional location ID (included where applicable)
    func trackFunnelStep(step: String, serviceId: String?, locationId: String?)
}

// MARK: - Stub Analytics Service

/// A stub implementation of AnalyticsService that logs to console.
/// Use this for development and testing purposes.
@MainActor
final class StubAnalyticsService: AnalyticsService, ObservableObject, Sendable {
    static let shared = StubAnalyticsService()

    private let isLoggingEnabled: Bool

    init(loggingEnabled: Bool = true) {
        self.isLoggingEnabled = loggingEnabled
    }

    func trackScreenView(screenName: String) {
        guard isLoggingEnabled else { return }
        print("[Analytics] Screen View: \(screenName)")
    }

    func trackEvent(name: String, params: [String: Any]?) {
        guard isLoggingEnabled else { return }
        if let params = params {
            print("[Analytics] Event: \(name), params: \(params)")
        } else {
            print("[Analytics] Event: \(name)")
        }
    }

    func trackError(error: Error, context: String?) {
        guard isLoggingEnabled else { return }
        let contextInfo = context.map { ", context: \($0)" } ?? ""
        print("[Analytics] Error: \(error.localizedDescription)\(contextInfo)")
    }

    func trackFunnelStep(step: String, serviceId: String?, locationId: String?) {
        guard isLoggingEnabled else { return }
        var params: [String: Any] = ["step": step]
        if let serviceId = serviceId {
            params["service_id"] = serviceId
        }
        if let locationId = locationId {
            params["location_id"] = locationId
        }
        print("[Analytics] Funnel Step: \(step), params: \(params)")
    }
}

// MARK: - Environment Key

private struct AnalyticsServiceKey: @preconcurrency EnvironmentKey {
    @MainActor static let defaultValue: AnalyticsService = StubAnalyticsService.shared
}

extension EnvironmentValues {
    var analyticsService: AnalyticsService {
        get { self[AnalyticsServiceKey.self] }
        set { self[AnalyticsServiceKey.self] = newValue }
    }
}

// MARK: - View Extension

extension View {
    /// Apply the analytics service to the view environment
    func withAnalyticsService(_ analyticsService: AnalyticsService = StubAnalyticsService.shared) -> some View {
        self.environment(\.analyticsService, analyticsService)
    }
}
