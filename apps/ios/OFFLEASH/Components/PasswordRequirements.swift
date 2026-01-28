//
//  PasswordRequirements.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct PasswordRequirements: View {
    let password: String

    private var hasMinLength: Bool {
        password.count >= 8
    }

    private var hasUppercase: Bool {
        password.contains(where: { $0.isUppercase })
    }

    private var hasNumber: Bool {
        password.contains(where: { $0.isNumber })
    }

    var allRequirementsMet: Bool {
        hasMinLength && hasUppercase && hasNumber
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            RequirementRow(
                text: "At least 8 characters",
                isMet: hasMinLength
            )
            RequirementRow(
                text: "At least 1 uppercase letter",
                isMet: hasUppercase
            )
            RequirementRow(
                text: "At least 1 number",
                isMet: hasNumber
            )
        }
    }
}

private struct RequirementRow: View {
    let text: String
    let isMet: Bool

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: isMet ? "checkmark.circle.fill" : "circle")
                .foregroundColor(isMet ? .green : .gray)
                .font(.system(size: 14))

            Text(text)
                .font(.caption)
                .foregroundColor(isMet ? .primary : .secondary)
        }
    }
}

#Preview {
    VStack(spacing: 20) {
        PasswordRequirements(password: "")
        PasswordRequirements(password: "test")
        PasswordRequirements(password: "Test1234")
    }
    .padding()
}
