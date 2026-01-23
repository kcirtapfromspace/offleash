//
//  CreateTenantView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Request/Response Models

struct CreateTenantRequest: Encodable {
    let name: String
    let slug: String
}

struct CreateTenantResponse: Decodable {
    let id: String
    let name: String
    let slug: String
}

// MARK: - Create Tenant View

struct CreateTenantView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService

    @State private var businessName = ""
    @State private var isLoading = false
    @State private var showError = false
    @State private var errorMessage = ""

    var onTenantCreated: () -> Void
    var onBack: () -> Void

    private var slug: String {
        businessName
            .lowercased()
            .replacingOccurrences(of: " ", with: "-")
            .replacingOccurrences(of: "[^a-z0-9-]", with: "", options: .regularExpression)
    }

    private var isFormValid: Bool {
        !businessName.trimmingCharacters(in: .whitespaces).isEmpty &&
        businessName.count >= 3
    }

    var body: some View {
        GeometryReader { geometry in
            ScrollView {
                VStack(spacing: 24) {
                    // Back Button
                    HStack {
                        Button(action: onBack) {
                            HStack(spacing: 4) {
                                Image(systemName: "chevron.left")
                                    .font(.system(size: 16, weight: .semibold))
                                Text("Back")
                                    .font(.body)
                            }
                            .foregroundColor(themeManager.primaryColor)
                        }
                        Spacer()
                    }
                    .padding(.top, 16)

                    // Header
                    VStack(spacing: 16) {
                        Image(systemName: "storefront.fill")
                            .font(.system(size: 64))
                            .foregroundColor(themeManager.primaryColor)

                        Text("Create Your Business")
                            .font(.largeTitle)
                            .fontWeight(.bold)
                            .foregroundColor(.primary)

                        Text("Set up your dog walking business in just a few steps")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding(.bottom, 24)

                    // Business Name Field
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Business Name")
                            .font(.subheadline)
                            .fontWeight(.medium)
                            .foregroundColor(.secondary)

                        TextField("e.g., Happy Paws Dog Walking", text: $businessName)
                            .textFieldStyle(.plain)
                            .textContentType(.organizationName)
                            .autocapitalization(.words)
                            .padding()
                            .background(
                                RoundedRectangle(cornerRadius: 12)
                                    .fill(Color(.systemGray6))
                            )
                            .overlay(
                                RoundedRectangle(cornerRadius: 12)
                                    .stroke(Color(.systemGray4), lineWidth: 1)
                            )

                        if !businessName.isEmpty {
                            Text("Your URL: offleash.app/\(slug)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }

                    // Info Box
                    HStack(alignment: .top, spacing: 12) {
                        Image(systemName: "info.circle.fill")
                            .foregroundColor(themeManager.accentColor)

                        VStack(alignment: .leading, spacing: 4) {
                            Text("What you'll get:")
                                .font(.subheadline)
                                .fontWeight(.semibold)

                            VStack(alignment: .leading, spacing: 2) {
                                BulletPoint(text: "Your own booking page")
                                BulletPoint(text: "Client management tools")
                                BulletPoint(text: "Schedule & availability settings")
                                BulletPoint(text: "Payment processing")
                            }
                            .font(.caption)
                            .foregroundColor(.secondary)
                        }
                    }
                    .padding()
                    .background(
                        RoundedRectangle(cornerRadius: 12)
                            .fill(themeManager.accentColor.opacity(0.1))
                    )

                    Spacer(minLength: 24)

                    // Create Button
                    Button(action: createTenant) {
                        HStack {
                            if isLoading {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Text("Create Business")
                                    .fontWeight(.semibold)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(
                            RoundedRectangle(cornerRadius: 12)
                                .fill(isFormValid ? themeManager.primaryColor : Color.gray)
                        )
                        .foregroundColor(.white)
                    }
                    .disabled(!isFormValid || isLoading)

                    Spacer(minLength: 32)
                }
                .padding(.horizontal, 24)
                .frame(minHeight: geometry.size.height)
            }
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "create_tenant")
        }
    }

    private func createTenant() {
        guard isFormValid else { return }

        isLoading = true

        Task {
            do {
                let request = CreateTenantRequest(
                    name: businessName.trimmingCharacters(in: .whitespaces),
                    slug: slug
                )

                let _: CreateTenantResponse = try await APIClient.shared.post("/walker/create-tenant", body: request)

                await MainActor.run {
                    isLoading = false
                    analyticsService.trackEvent(name: "tenant_created", params: ["slug": slug])
                    onTenantCreated()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to create business"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    errorMessage = "An unexpected error occurred. Please try again."
                    showError = true
                }
            }
        }
    }
}

// MARK: - Bullet Point

struct BulletPoint: View {
    let text: String

    var body: some View {
        HStack(alignment: .top, spacing: 6) {
            Text("•")
            Text(text)
        }
    }
}

#Preview {
    CreateTenantView(
        onTenantCreated: { print("Created") },
        onBack: { print("Back") }
    )
    .withThemeManager()
}
