//
//  Validators.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

/// Utility functions for validating user input
enum Validators {
    /// Validates if a string is a properly formatted email address
    /// - Parameter email: The email string to validate
    /// - Returns: true if the email is valid, false otherwise
    static func isValidEmail(_ email: String) -> Bool {
        let trimmedEmail = email.trimmingCharacters(in: .whitespaces)

        guard !trimmedEmail.isEmpty else { return false }

        // Email regex pattern: requires @ symbol, characters before and after @,
        // and a dot in the domain with at least 2 characters after the dot
        let emailPattern = #"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$"#

        guard let regex = try? NSRegularExpression(pattern: emailPattern, options: .caseInsensitive) else {
            return false
        }

        let range = NSRange(location: 0, length: trimmedEmail.utf16.count)
        return regex.firstMatch(in: trimmedEmail, options: [], range: range) != nil
    }
}

enum TestAuthMode {
    static var isMock: Bool {
        if ProcessInfo.processInfo.environment["OFFLEASH_TEST_AUTH"] == "mock" {
            return true
        }
        if UserDefaults.standard.string(forKey: "OFFLEASH_TEST_AUTH") == "mock" {
            return true
        }
        let args = ProcessInfo.processInfo.arguments
        if let index = args.firstIndex(of: "-OFFLEASH_TEST_AUTH"), index + 1 < args.count {
            return args[index + 1].lowercased() == "mock"
        }
        return false
    }
}
