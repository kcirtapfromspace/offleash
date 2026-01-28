//
//  AppConfiguration.swift
//  OFFLEASH
//
//  Configuration service for app-level settings including organization.
//  Supports white-labeling via Info.plist configuration.
//

import Foundation

/// App-wide configuration service
/// Reads from Info.plist and environment for flexibility
final class AppConfiguration: @unchecked Sendable {
    static let shared = AppConfiguration()

    // MARK: - Organization Configuration

    /// The default organization slug for new user registration
    /// Priority order:
    /// 1. Info.plist "OFFLEASH_ORG_SLUG" key (for white-label builds)
    /// 2. Environment variable "ORG_SLUG" (for development)
    /// 3. Default "offleash-demo" organization
    var defaultOrganizationSlug: String {
        // Check Info.plist first (for white-label builds)
        if let plistValue = Bundle.main.object(forInfoDictionaryKey: "OFFLEASH_ORG_SLUG") as? String,
           !plistValue.isEmpty,
           !plistValue.hasPrefix("$(") {  // Skip unresolved build variables
            return plistValue
        }

        // Check environment variable (for development)
        if let envValue = ProcessInfo.processInfo.environment["ORG_SLUG"],
           !envValue.isEmpty {
            return envValue
        }

        // Default organization
        return "offleash-demo"
    }

    // MARK: - API Configuration

    /// API base URL
    var apiBaseURL: String {
        if let plistValue = Bundle.main.object(forInfoDictionaryKey: "OFFLEASH_API_URL") as? String,
           !plistValue.isEmpty,
           !plistValue.hasPrefix("$(") {
            return plistValue
        }

        if let envValue = ProcessInfo.processInfo.environment["API_URL"],
           !envValue.isEmpty {
            return envValue
        }

        #if DEBUG
        return "http://localhost:8080"
        #else
        return "https://api.offleash.pro"
        #endif
    }

    // MARK: - Feature Flags

    /// Whether certificate pinning is enabled
    var certificatePinningEnabled: Bool {
        #if DEBUG
        return false
        #else
        return true
        #endif
    }

    private init() {}
}
