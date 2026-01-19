//
//  ServicesView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Services View

struct ServicesView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @State private var services: [Service] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var selectedService: Service?

    var onServiceSelected: ((Service) -> Void)?

    var body: some View {
        NavigationStack {
            Group {
                if isLoading && services.isEmpty {
                    loadingView
                } else if showError && services.isEmpty {
                    errorView
                } else {
                    servicesList
                }
            }
            .navigationTitle("Services")
            .refreshable {
                await fetchServices()
            }
        }
        .task {
            await fetchServices()
        }
        .alert("Error", isPresented: $showError) {
            Button("Retry") {
                Task {
                    await fetchServices()
                }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
    }

    // MARK: - Loading View

    private var loadingView: some View {
        VStack(spacing: 16) {
            ProgressView()
                .scaleEffect(1.5)
            Text("Loading services...")
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Error View

    private var errorView: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 48))
                .foregroundColor(.orange)

            Text("Unable to load services")
                .font(.headline)

            Text(errorMessage)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button(action: {
                Task {
                    await fetchServices()
                }
            }) {
                Text("Try Again")
                    .fontWeight(.semibold)
                    .padding(.horizontal, 24)
                    .padding(.vertical, 12)
                    .background(themeManager.primaryColor)
                    .foregroundColor(.white)
                    .cornerRadius(8)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Services List

    private var servicesList: some View {
        List(services) { service in
            ServiceRowView(service: service, themeManager: themeManager)
                .contentShape(Rectangle())
                .onTapGesture {
                    onServiceSelected?(service)
                }
        }
        .listStyle(.plain)
    }

    // MARK: - Data Fetching

    private func fetchServices() async {
        if services.isEmpty {
            isLoading = true
        }

        do {
            let fetchedServices: [Service] = try await APIClient.shared.get("/services")
            await MainActor.run {
                services = fetchedServices.filter { $0.isActive }
                isLoading = false
                showError = false
            }
        } catch let error as APIError {
            await MainActor.run {
                isLoading = false
                errorMessage = error.errorDescription ?? "An unexpected error occurred"
                if services.isEmpty {
                    showError = true
                }
            }
        } catch {
            await MainActor.run {
                isLoading = false
                errorMessage = "An unexpected error occurred. Please try again."
                if services.isEmpty {
                    showError = true
                }
            }
        }
    }
}

// MARK: - Service Row View

struct ServiceRowView: View {
    let service: Service
    let themeManager: ThemeManager

    var body: some View {
        HStack(spacing: 12) {
            // Service icon
            Image(systemName: "pawprint.fill")
                .font(.title2)
                .foregroundColor(themeManager.primaryColor)
                .frame(width: 44, height: 44)
                .background(themeManager.primaryColor.opacity(0.1))
                .cornerRadius(8)

            // Service details
            VStack(alignment: .leading, spacing: 4) {
                Text(service.name)
                    .font(.headline)
                    .foregroundColor(.primary)

                HStack(spacing: 8) {
                    // Duration
                    HStack(spacing: 4) {
                        Image(systemName: "clock")
                            .font(.caption)
                        Text(formatDuration(service.durationMinutes))
                            .font(.subheadline)
                    }
                    .foregroundColor(.secondary)

                    Text("•")
                        .foregroundColor(.secondary)

                    // Price
                    Text(service.priceDisplay)
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundColor(themeManager.primaryColor)
                }
            }

            Spacer()

            // Chevron indicator
            Image(systemName: "chevron.right")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 8)
    }

    private func formatDuration(_ minutes: Int) -> String {
        if minutes < 60 {
            return "\(minutes) min"
        } else if minutes % 60 == 0 {
            let hours = minutes / 60
            return hours == 1 ? "1 hr" : "\(hours) hrs"
        } else {
            let hours = minutes / 60
            let remainingMinutes = minutes % 60
            return "\(hours) hr \(remainingMinutes) min"
        }
    }
}

// MARK: - Preview

#Preview {
    ServicesView(onServiceSelected: { service in
        print("Selected service: \(service.name)")
    })
    .withThemeManager()
}
