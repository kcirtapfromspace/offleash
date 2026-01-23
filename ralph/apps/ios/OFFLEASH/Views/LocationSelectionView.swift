//
//  LocationSelectionView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Location Selection View

struct LocationSelectionView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var locations: [Location] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var selectedLocation: Location?
    @State private var showAddLocation = false
    @State private var startTime: Date?
    @State private var wasCacheHit = false
    @State private var wasLoading = true  // Track previous loading state for iOS 16 onChange

    var serviceId: String?
    var onLocationSelected: ((Location) -> Void)?
    var onAddLocationTapped: (() -> Void)?

    var body: some View {
        NavigationStack {
            Group {
                if isLoading && locations.isEmpty {
                    loadingView
                } else if showError && locations.isEmpty {
                    errorView
                } else if locations.isEmpty {
                    emptyView
                } else {
                    locationsList
                }
            }
            .navigationTitle("Select Location")
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: {
                        onAddLocationTapped?()
                    }) {
                        Image(systemName: "plus")
                            .foregroundColor(themeManager.primaryColor)
                    }
                }
            }
            .refreshable {
                await refreshLocations()
            }
        }
        .task {
            await fetchLocations()
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "location_selection")
            startTime = Date()
        }
        .onChange(of: isLoading) { newValue in
            // Track TTI when loading completes (transitions from true to false)
            if wasLoading == true && newValue == false, let start = startTime {
                let durationMs = Int(Date().timeIntervalSince(start) * 1000)
                analyticsService.trackEvent(name: "tti", params: [
                    "screen": "LocationSelectionView",
                    "duration_ms": durationMs,
                    "cache_hit": wasCacheHit
                ])
                startTime = nil // Reset to avoid duplicate tracking
            }
            wasLoading = newValue
        }
        .alert("Error", isPresented: $showError) {
            Button("Retry") {
                Task {
                    await fetchLocations()
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
            Text("Loading locations...")
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

            Text("Unable to load locations")
                .font(.headline)

            Text(errorMessage)
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button(action: {
                Task {
                    await fetchLocations()
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

    // MARK: - Empty View

    private var emptyView: some View {
        VStack(spacing: 16) {
            Image(systemName: "mappin.slash")
                .font(.system(size: 48))
                .foregroundColor(.secondary)

            Text("No locations yet")
                .font(.headline)

            Text("Add a location to continue with your booking")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button(action: {
                onAddLocationTapped?()
            }) {
                HStack {
                    Image(systemName: "plus")
                    Text("Add Location")
                }
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

    // MARK: - Locations List

    private var locationsList: some View {
        VStack(spacing: 0) {
            List {
                ForEach(locations) { location in
                    LocationRowView(
                        location: location,
                        isSelected: selectedLocation?.id == location.id,
                        themeManager: themeManager
                    )
                    .contentShape(Rectangle())
                    .onTapGesture {
                        selectedLocation = location
                        analyticsService.trackFunnelStep(step: "location_selected", serviceId: serviceId, locationId: location.id)
                        onLocationSelected?(location)
                    }
                }

                // Add new location row
                Button(action: {
                    onAddLocationTapped?()
                }) {
                    HStack(spacing: 12) {
                        Image(systemName: "plus.circle.fill")
                            .font(.title2)
                            .foregroundColor(themeManager.primaryColor)
                            .frame(width: 44, height: 44)

                        Text("Add New Location")
                            .font(.headline)
                            .foregroundColor(themeManager.primaryColor)

                        Spacer()
                    }
                    .padding(.vertical, 8)
                }
            }
            .listStyle(.plain)
        }
    }

    // MARK: - Data Fetching

    static let cacheKey = "locations"
    private static let cacheTTL: TimeInterval = 600 // 10 minutes

    private func loadLocations() async {
        // Check cache first
        if let cachedLocations: [Location] = await CacheManager.shared.get(key: Self.cacheKey) {
            // Cache hit: show cached data immediately
            wasCacheHit = true
            locations = cachedLocations
            isLoading = false

            // Fetch fresh data in background
            await fetchLocationsFromNetwork(updateUIOnSuccess: true)
        } else {
            // Cache miss: show loading state and fetch from network
            wasCacheHit = false
            isLoading = true
            await fetchLocationsFromNetwork(updateUIOnSuccess: true)
        }
    }

    private func fetchLocations() async {
        await loadLocations()
    }

    private func refreshLocations() async {
        // Pull-to-refresh bypasses cache and forces network fetch
        await fetchLocationsFromNetwork(updateUIOnSuccess: true)
    }

    private func fetchLocationsFromNetwork(updateUIOnSuccess: Bool) async {
        do {
            let fetchedLocations: [Location] = try await APIClient.shared.get("/locations")

            // Cache the response with 10-minute TTL
            await CacheManager.shared.set(key: Self.cacheKey, value: fetchedLocations, ttl: Self.cacheTTL)

            if updateUIOnSuccess {
                // Only update UI if data changed
                if fetchedLocations != locations {
                    locations = fetchedLocations
                }
                isLoading = false
                showError = false
            }
        } catch let error as APIError {
            // Only show error if we have no cached data to display
            if locations.isEmpty {
                isLoading = false
                errorMessage = error.errorDescription ?? "An unexpected error occurred"
                showError = true
            } else {
                // Silently fail background refresh if we have cached data
                isLoading = false
            }
        } catch {
            if locations.isEmpty {
                isLoading = false
                errorMessage = "An unexpected error occurred. Please try again."
                showError = true
            } else {
                isLoading = false
            }
        }
    }

    /// Invalidate the locations cache
    /// Call this when a location is added, edited, or deleted
    static func invalidateCache() async {
        await CacheManager.shared.invalidate(key: cacheKey)
    }
}

// MARK: - Location Row View

struct LocationRowView: View {
    let location: Location
    let isSelected: Bool
    let themeManager: ThemeManager

    var body: some View {
        HStack(spacing: 12) {
            // Location icon
            Image(systemName: "mappin.circle.fill")
                .font(.title2)
                .foregroundColor(themeManager.primaryColor)
                .frame(width: 44, height: 44)
                .background(themeManager.primaryColor.opacity(0.1))
                .cornerRadius(8)

            // Location details
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(location.name)
                        .font(.headline)
                        .foregroundColor(.primary)

                    if location.isDefault {
                        Text("Default")
                            .font(.caption)
                            .fontWeight(.medium)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(themeManager.primaryColor.opacity(0.1))
                            .foregroundColor(themeManager.primaryColor)
                            .cornerRadius(4)
                    }
                }

                Text(location.fullAddress)
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            // Selection indicator
            if isSelected {
                Image(systemName: "checkmark.circle.fill")
                    .font(.title2)
                    .foregroundColor(themeManager.primaryColor)
            } else {
                Image(systemName: "circle")
                    .font(.title2)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 8)
    }
}

// MARK: - Preview

#Preview {
    LocationSelectionView(
        serviceId: "preview-service-id",
        onLocationSelected: { location in
            print("Selected location: \(location.name)")
        },
        onAddLocationTapped: {
            print("Add location tapped")
        }
    )
    .withThemeManager()
}
