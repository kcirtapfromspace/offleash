//
//  Theme.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Color Extension for Hex Support

extension Color {
    /// Initialize a Color from a hex string
    /// Supports formats: "#RGB", "#RRGGBB", "RGB", "RRGGBB"
    init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)

        let r, g, b: UInt64
        switch hex.count {
        case 3: // RGB (12-bit)
            (r, g, b) = ((int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6: // RRGGBB (24-bit)
            (r, g, b) = (int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8: // AARRGGBB (32-bit with alpha, ignore alpha)
            (r, g, b) = (int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (r, g, b) = (0, 0, 0)
        }

        self.init(
            .sRGB,
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            opacity: 1
        )
    }

    /// Initialize a Color from a hex string with optional alpha
    init(hex: String, opacity: Double) {
        let color = Color(hex: hex)
        self = color.opacity(opacity)
    }
}

// MARK: - Branding Model

struct Branding: Codable, Equatable {
    let companyName: String
    let logoUrl: String
    let primaryColor: String
    let secondaryColor: String
    let accentColor: String
    let supportEmail: String

    enum CodingKeys: String, CodingKey {
        case companyName = "company_name"
        case logoUrl = "logo_url"
        case primaryColor = "primary_color"
        case secondaryColor = "secondary_color"
        case accentColor = "accent_color"
        case supportEmail = "support_email"
    }

    static let `default` = Branding(
        companyName: "Offleash",
        logoUrl: "",
        primaryColor: "#3B82F6",
        secondaryColor: "#1E40AF",
        accentColor: "#10B981",
        supportEmail: "support@offleash.com"
    )
}

// MARK: - Theme Manager

@MainActor
final class ThemeManager: ObservableObject {
    static let shared = ThemeManager()

    @Published private(set) var branding: Branding

    // MARK: - Computed Color Properties

    var primaryColor: Color {
        Color(hex: branding.primaryColor)
    }

    var secondaryColor: Color {
        Color(hex: branding.secondaryColor)
    }

    var accentColor: Color {
        Color(hex: branding.accentColor)
    }

    // MARK: - Initialization

    private init() {
        self.branding = .default
    }

    // MARK: - Public Methods

    /// Update branding with new values
    func updateBranding(_ newBranding: Branding) {
        branding = newBranding
    }

    /// Update branding from API response data
    func updateBranding(from data: Data) throws {
        let decoder = JSONDecoder()
        let newBranding = try decoder.decode(Branding.self, from: data)
        branding = newBranding
    }

    /// Reset to default branding
    func resetToDefault() {
        branding = .default
    }
}

// MARK: - Environment Key

private struct ThemeManagerKey: EnvironmentKey {
    static let defaultValue = ThemeManager.shared
}

extension EnvironmentValues {
    var themeManager: ThemeManager {
        get { self[ThemeManagerKey.self] }
        set { self[ThemeManagerKey.self] = newValue }
    }
}

// MARK: - View Extension for Easy Access

extension View {
    /// Apply the theme manager as an environment object
    func withThemeManager(_ themeManager: ThemeManager = .shared) -> some View {
        self
            .environmentObject(themeManager)
            .environment(\.themeManager, themeManager)
    }
}

// MARK: - Preview Helpers

#if DEBUG
extension ThemeManager {
    /// Create a preview instance with custom branding
    static func preview(
        primaryColor: String = "#3B82F6",
        secondaryColor: String = "#1E40AF",
        accentColor: String = "#10B981"
    ) -> ThemeManager {
        let manager = ThemeManager.shared
        manager.updateBranding(Branding(
            companyName: "Preview",
            logoUrl: "",
            primaryColor: primaryColor,
            secondaryColor: secondaryColor,
            accentColor: accentColor,
            supportEmail: "preview@example.com"
        ))
        return manager
    }
}
#endif
