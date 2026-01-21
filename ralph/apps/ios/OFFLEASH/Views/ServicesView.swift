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
    @Environment(\.analyticsService) private var analyticsService
    @State private var services: [Service] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var selectedService: Service?
    @State private var startTime: Date?
    @State private var wasCacheHit = false

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
                await refreshServices()
            }
        }
        .task {
            await fetchServices()
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "services")
            startTime = Date()
        }
        .onChange(of: isLoading) { oldValue, newValue in
            // Track TTI when loading completes (transitions from true to false)
            if oldValue == true && newValue == false, let start = startTime {
                let durationMs = Int(Date().timeIntervalSince(start) * 1000)
                analyticsService.trackEvent(name: "tti", params: [
                    "screen": "ServicesView",
                    "duration_ms": durationMs,
                    "cache_hit": wasCacheHit
                ])
                startTime = nil // Reset to avoid duplicate tracking
            }
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

    private static let cacheKey = "services"
    private static let cacheTTL: TimeInterval = 300 // 5 minutes

    private func loadServices() async {
        // Check cache first
        if let cachedServices: [Service] = await CacheManager.shared.get(key: Self.cacheKey) {
            // Cache hit: show cached data immediately (already filtered by server)
            wasCacheHit = true
            services = cachedServices
            isLoading = false

            // Fetch fresh data in background
            await fetchServicesFromNetwork(updateUIOnSuccess: true)
        } else {
            // Cache miss: show loading state and fetch from network
            wasCacheHit = false
            isLoading = true
            await fetchServicesFromNetwork(updateUIOnSuccess: true)
        }
    }

    private func fetchServices() async {
        await loadServices()
    }

    private func refreshServices() async {
        // Pull-to-refresh bypasses cache and forces network fetch
        await fetchServicesFromNetwork(updateUIOnSuccess: true)
    }

    private func fetchServicesFromNetwork(updateUIOnSuccess: Bool) async {
        do {
            let queryItems = [URLQueryItem(name: "active", value: "true")]
            let fetchedServices: [Service] = try await APIClient.shared.get("/services", queryItems: queryItems)

            // Cache the response with 5-minute TTL (already filtered by server)
            await CacheManager.shared.set(key: Self.cacheKey, value: fetchedServices, ttl: Self.cacheTTL)

            if updateUIOnSuccess {
                // Only update UI if data changed
                if fetchedServices != services {
                    services = fetchedServices
                }
                isLoading = false
                showError = false
            }
        } catch let error as APIError {
            // Only show error if we have no cached data to display
            if services.isEmpty {
                isLoading = false
                errorMessage = error.errorDescription ?? "An unexpected error occurred"
                showError = true
            } else {
                // Silently fail background refresh if we have cached data
                isLoading = false
            }
        } catch {
            if services.isEmpty {
                isLoading = false
                errorMessage = "An unexpected error occurred. Please try again."
                showError = true
            } else {
                isLoading = false
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
