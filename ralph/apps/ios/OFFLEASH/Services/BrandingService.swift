//
//  BrandingService.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation
import SwiftUI

// MARK: - Branding Service

@MainActor
final class BrandingService: ObservableObject, Sendable {
    static let shared = BrandingService()

    @Published private(set) var config: BrandingConfig

    private let defaults = UserDefaults.standard
    private let cacheKey = "branding_config"

    // MARK: - Initialization

    private init() {
        self.config = Self.loadFromCache() ?? .default
    }

    // MARK: - Public Methods

    /// Fetch branding configuration from API
    func fetchBranding() async throws {
        let freshConfig: BrandingConfig = try await APIClient.shared.get("/api/branding")
        config = freshConfig
        Self.saveToCache(freshConfig)
    }

    /// Reset branding to default values
    func resetToDefault() {
        config = .default
        clearCache()
    }

    /// Check if branding is cached
    var hasCachedBranding: Bool {
        defaults.data(forKey: cacheKey) != nil
    }

    // MARK: - Cache Management

    private static func saveToCache(_ config: BrandingConfig) {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        if let data = try? encoder.encode(config) {
            UserDefaults.standard.set(data, forKey: "branding_config")
        }
    }

    private static func loadFromCache() -> BrandingConfig? {
        guard let data = UserDefaults.standard.data(forKey: "branding_config") else {
            return nil
        }
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return try? decoder.decode(BrandingConfig.self, from: data)
    }

    private func clearCache() {
        defaults.removeObject(forKey: cacheKey)
    }
}

// MARK: - Environment Key

private struct BrandingServiceKey: @preconcurrency EnvironmentKey {
    @MainActor static let defaultValue = BrandingService.shared
}

extension EnvironmentValues {
    var brandingService: BrandingService {
        get { self[BrandingServiceKey.self] }
        set { self[BrandingServiceKey.self] = newValue }
    }
}

// MARK: - View Extension

extension View {
    /// Apply the branding service as an environment object
    func withBrandingService(_ brandingService: BrandingService = .shared) -> some View {
        self
            .environmentObject(brandingService)
            .environment(\.brandingService, brandingService)
    }
}
