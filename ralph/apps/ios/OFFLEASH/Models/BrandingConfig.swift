//
//  BrandingConfig.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Branding Colors

struct BrandingColors: Codable, Equatable {
    let primary: String
    let secondary: String
    let accent: String

    static let `default` = BrandingColors(
        primary: "#3B82F6",
        secondary: "#1E40AF",
        accent: "#10B981"
    )
}

// MARK: - Branding Config

struct BrandingConfig: Codable, Equatable {
    let companyName: String
    let logoUrl: String
    let colors: BrandingColors

    enum CodingKeys: String, CodingKey {
        case companyName = "company_name"
        case logoUrl = "logo_url"
        case colors
    }

    static let `default` = BrandingConfig(
        companyName: "Offleash",
        logoUrl: "",
        colors: .default
    )
}
