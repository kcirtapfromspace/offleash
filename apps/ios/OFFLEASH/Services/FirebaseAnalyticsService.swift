//
//  FirebaseAnalyticsService.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation
import FirebaseAnalytics

// MARK: - Firebase Analytics Service

/// Firebase implementation of AnalyticsService for production analytics tracking.
/// Uses Firebase Analytics SDK to track screen views, events, and errors.
@MainActor
final class FirebaseAnalyticsService: AnalyticsService, ObservableObject {
    static let shared = FirebaseAnalyticsService()

    private init() {}

    func trackScreenView(screenName: String) {
        Analytics.logEvent(AnalyticsEventScreenView, parameters: [
            AnalyticsParameterScreenName: screenName
        ])
    }

    func trackEvent(name: String, params: [String: Any]?) {
        Analytics.logEvent(name, parameters: params)
    }

    func trackError(error: Error, context: String?) {
        var parameters: [String: Any] = [
            "error_description": error.localizedDescription
        ]
        if let context = context {
            parameters["error_context"] = context
        }
        Analytics.logEvent("app_error", parameters: parameters)
    }

    func trackFunnelStep(step: String, serviceId: String?, locationId: String?) {
        var parameters: [String: Any] = ["step": step]
        if let serviceId = serviceId {
            parameters["service_id"] = serviceId
        }
        if let locationId = locationId {
            parameters["location_id"] = locationId
        }
        Analytics.logEvent("funnel_step", parameters: parameters)
    }
}
