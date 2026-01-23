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

    var body: some View {
        Group {
            if isLoading {
                ProgressView("Loading locations...")
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
            }
        }
        .sheet(isPresented: $showAddLocation) {
            AddLocationView { newLocation in
                locations.insert(newLocation, at: 0)
                showAddLocation = false
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
            .padding(.top, 8)
        }
        .padding()
    }

    private var locationsList: some View {
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

    private func loadLocations() async {
        isLoading = true
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
        do {
            let _: Location = try await APIClient.shared.put("/locations/\(location.id)/default")
            await loadLocations()
        } catch {
            errorMessage = "Failed to set default location"
            showError = true
        }
    }

    private func deleteLocation(_ location: Location) async {
        do {
            try await APIClient.shared.delete("/locations/\(location.id)")
            locations.removeAll { $0.id == location.id }
        } catch {
            errorMessage = "Failed to delete location"
            showError = true
        }
    }
}

// MARK: - Location Row

struct LocationRow: View {
    let location: Location
    let onSetDefault: () async -> Void
    let onDelete: () async -> Void

    @State private var isProcessing = false

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

            if !location.isDefault {
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
