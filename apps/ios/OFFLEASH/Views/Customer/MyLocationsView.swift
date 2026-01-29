//
//  MyLocationsView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct MyLocationsView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var locations: [Location] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showAddLocation = false
    @State private var showToast = false
    @State private var toastMessage = ""

    private var isAuthMockMode: Bool {
        TestAuthMode.isMock
    }

    var body: some View {
        Group {
            if isLoading {
                ProgressView("Loading locations...")
                    .accessibilityIdentifier("loading-indicator")
            } else if locations.isEmpty {
                emptyState
            } else {
                locationsList
            }
        }
        .navigationTitle("My Locations")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    showAddLocation = true
                } label: {
                    Image(systemName: "plus")
                }
                .accessibilityIdentifier("location-add-button")
            }
        }
        .sheet(isPresented: $showAddLocation) {
            AddLocationView { newLocation in
                locations.insert(newLocation, at: 0)
                showAddLocation = false
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
                    showToastMessage("Location saved")
                }
            }
        }
        .task {
            await loadLocations()
        }
        .refreshable {
            await loadLocations()
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "my_locations")
        }
        .overlay(alignment: .top) {
            if showToast {
                ToastBanner(message: toastMessage)
                    .padding(.top, 8)
                    .transition(.move(edge: .top).combined(with: .opacity))
            }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "mappin.slash")
                .font(.system(size: 48))
                .foregroundColor(.secondary)

            Text("No Locations Yet")
                .font(.headline)

            Text("Add your home, work, or other locations for quick booking.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button {
                showAddLocation = true
            } label: {
                Label("Add Location", systemImage: "plus")
                    .fontWeight(.semibold)
            }
            .buttonStyle(.borderedProminent)
            .tint(themeManager.primaryColor)
            .accessibilityIdentifier("location-add-button")
            .padding(.top, 8)
        }
        .accessibilityIdentifier("empty-state")
        .padding()
    }

    private var locationsList: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Locations")
                .font(.caption)
                .foregroundColor(.secondary)
                .accessibilityIdentifier("locations-list")

            List {
                ForEach(locations) { location in
                    LocationRow(location: location, onSetDefault: {
                        await setDefault(location)
                    }, onDelete: {
                        await deleteLocation(location)
                    })
                }
            }
        }
    }

    private func loadLocations() async {
        isLoading = true
        if isAuthMockMode {
            locations = await MockDataStore.shared.getLocations()
            isLoading = false
            return
        }
        do {
            locations = try await APIClient.shared.get("/locations")
            isLoading = false
        } catch {
            isLoading = false
            errorMessage = "Failed to load locations"
            showError = true
        }
    }

    private func setDefault(_ location: Location) async {
        if isAuthMockMode {
            _ = await MockDataStore.shared.setDefaultLocation(id: location.id)
            locations = await MockDataStore.shared.getLocations()
            showToastMessage("Default location updated")
            return
        }
        do {
            let _: Location = try await APIClient.shared.put("/locations/\(location.id)/default")
            await loadLocations()
            showToastMessage("Default location updated")
        } catch {
            errorMessage = "Failed to set default location"
            showError = true
        }
    }

    private func deleteLocation(_ location: Location) async {
        if isAuthMockMode {
            await MockDataStore.shared.deleteLocation(id: location.id)
            locations = await MockDataStore.shared.getLocations()
            showToastMessage("Location deleted")
            return
        }
        do {
            try await APIClient.shared.delete("/locations/\(location.id)")
            locations.removeAll { $0.id == location.id }
            showToastMessage("Location deleted")
        } catch {
            errorMessage = "Failed to delete location"
            showError = true
        }
    }

    private func showToastMessage(_ message: String) {
        toastMessage = message
        showToast = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) {
            showToast = false
        }
    }
}

// MARK: - Location Row

struct LocationRow: View {
    let location: Location
    let onSetDefault: () async -> Void
    let onDelete: () async -> Void

    @State private var isProcessing = false
    private var isAuthMockMode: Bool {
        TestAuthMode.isMock
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(location.name)
                    .font(.headline)

                if location.isDefault {
                    Text("Default")
                        .font(.caption)
                        .fontWeight(.medium)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 2)
                        .background(Color.blue.opacity(0.1))
                        .foregroundColor(.blue)
                        .cornerRadius(4)
                }

                Spacer()

                if isAuthMockMode && !location.isDefault {
                    Button {
                        Task {
                            isProcessing = true
                            await onSetDefault()
                            isProcessing = false
                        }
                    } label: {
                        Text("Set Default")
                            .font(.caption)
                            .fontWeight(.medium)
                    }
                    .buttonStyle(.borderless)
                    .tint(.orange)
                    .accessibilityIdentifier("location-default-button")
                }
            }

            Text(location.fullAddress)
                .font(.subheadline)
                .foregroundColor(.secondary)

            if let notes = location.notes, !notes.isEmpty {
                Text(notes)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
        .swipeActions(edge: .trailing, allowsFullSwipe: false) {
            Button(role: .destructive) {
                Task {
                    isProcessing = true
                    await onDelete()
                    isProcessing = false
                }
            } label: {
                Label("Delete", systemImage: "trash")
            }

            if !location.isDefault && !isAuthMockMode {
                Button {
                    Task {
                        isProcessing = true
                        await onSetDefault()
                        isProcessing = false
                    }
                } label: {
                    Label("Set Default", systemImage: "star")
                }
                .tint(.orange)
                .accessibilityIdentifier("location-default-button")
            }
        }
        .disabled(isProcessing)
    }
}

#Preview {
    NavigationStack {
        MyLocationsView()
    }
    .withThemeManager()
}
