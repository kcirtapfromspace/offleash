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
    @State private var locations: [Location] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var selectedLocation: Location?
    @State private var showAddLocation = false

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
                await fetchLocations()
            }
        }
        .task {
            await fetchLocations()
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

    private func fetchLocations() async {
        if locations.isEmpty {
            isLoading = true
        }

        do {
            let fetchedLocations: [Location] = try await APIClient.shared.get("/locations")
            await MainActor.run {
                locations = fetchedLocations
                isLoading = false
                showError = false
            }
        } catch let error as APIError {
            await MainActor.run {
                isLoading = false
                errorMessage = error.errorDescription ?? "An unexpected error occurred"
                if locations.isEmpty {
                    showError = true
                }
            }
        } catch {
            await MainActor.run {
                isLoading = false
                errorMessage = "An unexpected error occurred. Please try again."
                if locations.isEmpty {
                    showError = true
                }
            }
        }
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
        onLocationSelected: { location in
            print("Selected location: \(location.name)")
        },
        onAddLocationTapped: {
            print("Add location tapped")
        }
    )
    .withThemeManager()
}
