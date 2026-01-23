//
//  ServiceAreasView.swift
//  OFFLEASH
//
//  Service area configuration with map polygon drawing (Zillow-style)
//

import SwiftUI
import MapKit
import UIKit

struct ServiceAreasView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    @StateObject private var viewModel = ServiceAreasViewModel()

    var body: some View {
        VStack(spacing: 0) {
            // Mode selector
            Picker("Mode", selection: $viewModel.mode) {
                Text("View").tag(ServiceAreaMode.view)
                Text("Draw").tag(ServiceAreaMode.draw)
            }
            .pickerStyle(.segmented)
            .padding()

            // Map
            ZStack {
                MapReader { proxy in
                    Map(position: $viewModel.cameraPosition) {
                        // Existing service areas
                        ForEach(viewModel.serviceAreas) { area in
                            MapPolygon(coordinates: area.coordinates)
                                .foregroundStyle(area.color.opacity(0.3))
                                .stroke(area.color, lineWidth: 2)
                        }

                        // Currently drawing polygon
                        if !viewModel.currentPolygonPoints.isEmpty {
                            MapPolygon(coordinates: viewModel.currentPolygonPoints)
                                .foregroundStyle(themeManager.primaryColor.opacity(0.2))
                                .stroke(themeManager.primaryColor, lineWidth: 3)

                            // Points
                            ForEach(Array(viewModel.currentPolygonPoints.enumerated()), id: \.offset) { index, point in
                                Annotation("", coordinate: point, anchor: .center) {
                                    Circle()
                                        .fill(themeManager.primaryColor)
                                        .frame(width: 12, height: 12)
                                        .overlay(
                                            Circle()
                                                .stroke(Color.white, lineWidth: 2)
                                        )
                                }
                            }
                        }
                    }
                    .mapStyle(.standard)
                    .onTapGesture { screenCoord in
                        guard viewModel.mode == .draw else { return }
                        if let coordinate = proxy.convert(screenCoord, from: .local) {
                            viewModel.addCoordinate(coordinate)
                        }
                    }
                }

                // Drawing instructions overlay
                if viewModel.mode == .draw {
                    VStack {
                        HStack {
                            Image(systemName: "hand.tap.fill")
                            Text("Tap to add points. Connect to first point to close.")
                        }
                        .font(.caption)
                        .padding(.horizontal, 12)
                        .padding(.vertical, 8)
                        .background(.ultraThinMaterial)
                        .cornerRadius(8)
                        .padding(.top, 8)

                        Spacer()
                    }
                }

                // Drawing controls
                if viewModel.mode == .draw && !viewModel.currentPolygonPoints.isEmpty {
                    VStack {
                        Spacer()

                        HStack(spacing: 16) {
                            Button {
                                viewModel.undoLastPoint()
                            } label: {
                                Label("Undo", systemImage: "arrow.uturn.backward")
                                    .font(.subheadline)
                                    .padding(.horizontal, 16)
                                    .padding(.vertical, 10)
                                    .background(.ultraThinMaterial)
                                    .cornerRadius(8)
                            }

                            Button {
                                viewModel.clearCurrentPolygon()
                            } label: {
                                Label("Clear", systemImage: "trash")
                                    .font(.subheadline)
                                    .foregroundColor(.red)
                                    .padding(.horizontal, 16)
                                    .padding(.vertical, 10)
                                    .background(.ultraThinMaterial)
                                    .cornerRadius(8)
                            }

                            if viewModel.currentPolygonPoints.count >= 3 {
                                Button {
                                    viewModel.showNameDialog = true
                                } label: {
                                    Label("Save Area", systemImage: "checkmark.circle.fill")
                                        .font(.subheadline)
                                        .foregroundColor(.white)
                                        .padding(.horizontal, 16)
                                        .padding(.vertical, 10)
                                        .background(themeManager.primaryColor)
                                        .cornerRadius(8)
                                }
                            }
                        }
                        .padding(.bottom, 16)
                    }
                }
            }

            // Service Areas List
            if !viewModel.serviceAreas.isEmpty && viewModel.mode == .view {
                List {
                    Section("Your Service Areas") {
                        ForEach(viewModel.serviceAreas) { area in
                            ServiceAreaRow(area: area) {
                                viewModel.focusOnArea(area)
                            }
                        }
                        .onDelete { indexSet in
                            viewModel.deleteAreas(at: indexSet)
                        }
                    }
                }
                .frame(height: 200)
            }
        }
        .navigationTitle("Service Areas")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Done") {
                    Task {
                        await viewModel.saveAreas()
                        dismiss()
                    }
                }
            }
        }
        .alert("Name This Area", isPresented: $viewModel.showNameDialog) {
            TextField("Area name", text: $viewModel.newAreaName)
            Button("Cancel", role: .cancel) {
                viewModel.newAreaName = ""
            }
            Button("Save") {
                viewModel.saveCurrentPolygon()
            }
        } message: {
            Text("Give this service area a name (e.g., \"Downtown Denver\", \"Capitol Hill\")")
        }
        .task {
            await viewModel.loadAreas()
        }
    }
}

// MARK: - Service Area Row

struct ServiceAreaRow: View {
    let area: ServiceArea
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack {
                Circle()
                    .fill(area.color)
                    .frame(width: 12, height: 12)

                VStack(alignment: .leading, spacing: 2) {
                    Text(area.name)
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundColor(.primary)

                    Text("\(area.coordinates.count) points")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                Image(systemName: "location.circle")
                    .foregroundColor(.secondary)
            }
        }
    }
}

// MARK: - Service Area Model

struct ServiceArea: Identifiable {
    let id: String
    var name: String
    var coordinates: [CLLocationCoordinate2D]
    var color: Color
    var isActive: Bool
    var priority: Int
    var priceAdjustmentPercent: Int
    var notes: String?

    static let colors: [Color] = [.blue, .green, .orange, .purple, .pink, .cyan]

    init(id: String, name: String, coordinates: [CLLocationCoordinate2D], color: Color, isActive: Bool = true, priority: Int = 0, priceAdjustmentPercent: Int = 0, notes: String? = nil) {
        self.id = id
        self.name = name
        self.coordinates = coordinates
        self.color = color
        self.isActive = isActive
        self.priority = priority
        self.priceAdjustmentPercent = priceAdjustmentPercent
        self.notes = notes
    }

    init(from response: ServiceAreaResponse) {
        self.id = response.id
        self.name = response.name
        self.coordinates = response.polygon.map { CLLocationCoordinate2D(latitude: $0.lat, longitude: $0.lng) }
        self.color = Color(hex: response.color)
        self.isActive = response.isActive
        self.priority = response.priority
        self.priceAdjustmentPercent = response.priceAdjustmentPercent
        self.notes = response.notes
    }
}

// MARK: - API Models

struct ServiceAreaResponse: Codable {
    let id: String
    let walkerId: String
    let name: String
    let color: String
    let polygon: [PolygonPointResponse]
    let isActive: Bool
    let priority: Int
    let priceAdjustmentPercent: Int
    let notes: String?
    let createdAt: String
    let updatedAt: String

    enum CodingKeys: String, CodingKey {
        case id
        case walkerId = "walker_id"
        case name
        case color
        case polygon
        case isActive = "is_active"
        case priority
        case priceAdjustmentPercent = "price_adjustment_percent"
        case notes
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct PolygonPointResponse: Codable {
    let lat: Double
    let lng: Double
}

struct CreateServiceAreaRequest: Codable {
    let name: String
    let color: String?
    let polygon: [PolygonPointResponse]
    let isActive: Bool?
    let priority: Int?
    let priceAdjustmentPercent: Int?
    let notes: String?

    enum CodingKeys: String, CodingKey {
        case name
        case color
        case polygon
        case isActive = "is_active"
        case priority
        case priceAdjustmentPercent = "price_adjustment_percent"
        case notes
    }
}

struct DeleteResponse: Codable {
    let success: Bool
}

// MARK: - Color Extension

extension Color {
    func toHex() -> String {
        guard let components = UIColor(self).cgColor.components, components.count >= 3 else {
            return "#3B82F6"
        }
        let r = Int(components[0] * 255)
        let g = Int(components[1] * 255)
        let b = Int(components[2] * 255)
        return String(format: "#%02X%02X%02X", r, g, b)
    }
}

// MARK: - Mode

enum ServiceAreaMode {
    case view
    case draw
}

// MARK: - View Model

@MainActor
class ServiceAreasViewModel: ObservableObject {
    @Published var serviceAreas: [ServiceArea] = []
    @Published var currentPolygonPoints: [CLLocationCoordinate2D] = []
    @Published var mode: ServiceAreaMode = .view
    @Published var cameraPosition: MapCameraPosition = .automatic
    @Published var showNameDialog = false
    @Published var newAreaName = ""

    private var mapProxy: MapProxy?

    init() {
        // Default to Denver area
        cameraPosition = .region(MKCoordinateRegion(
            center: CLLocationCoordinate2D(latitude: 39.7392, longitude: -104.9903),
            span: MKCoordinateSpan(latitudeDelta: 0.15, longitudeDelta: 0.15)
        ))
    }

    func addCoordinate(_ coordinate: CLLocationCoordinate2D) {
        currentPolygonPoints.append(coordinate)
    }

    func undoLastPoint() {
        guard !currentPolygonPoints.isEmpty else { return }
        currentPolygonPoints.removeLast()
    }

    func clearCurrentPolygon() {
        currentPolygonPoints.removeAll()
    }

    func saveCurrentPolygon() {
        Task {
            await saveCurrentPolygonToBackend()
        }
    }

    func deleteAreas(at indexSet: IndexSet) {
        for index in indexSet {
            let area = serviceAreas[index]
            Task {
                await deleteArea(area)
            }
        }
    }

    func focusOnArea(_ area: ServiceArea) {
        guard !area.coordinates.isEmpty else { return }

        // Calculate region that encompasses all points
        let lats = area.coordinates.map { $0.latitude }
        let lons = area.coordinates.map { $0.longitude }

        let center = CLLocationCoordinate2D(
            latitude: (lats.min()! + lats.max()!) / 2,
            longitude: (lons.min()! + lons.max()!) / 2
        )

        let span = MKCoordinateSpan(
            latitudeDelta: (lats.max()! - lats.min()!) * 1.5,
            longitudeDelta: (lons.max()! - lons.min()!) * 1.5
        )

        withAnimation {
            cameraPosition = .region(MKCoordinateRegion(center: center, span: span))
        }
    }

    func loadAreas() async {
        do {
            let response: [ServiceAreaResponse] = try await APIClient.shared.get("/walker/service-areas")
            serviceAreas = response.map { ServiceArea(from: $0) }

            // Focus on areas if we have any
            if let firstArea = serviceAreas.first {
                focusOnArea(firstArea)
            }
        } catch {
            print("Error loading service areas: \(error)")
            // Initialize with empty array on error
            serviceAreas = []
        }
    }

    func saveAreas() async {
        // Individual areas are saved as they are created
        // This method is called on dismiss - nothing to do
    }

    func saveCurrentPolygonToBackend() async {
        guard currentPolygonPoints.count >= 3 else { return }

        let polygon = currentPolygonPoints.map { PolygonPointResponse(lat: $0.latitude, lng: $0.longitude) }
        let colorIndex = serviceAreas.count % ServiceArea.colors.count
        let color = ServiceArea.colors[colorIndex].toHex()

        let request = CreateServiceAreaRequest(
            name: newAreaName.isEmpty ? "Area \(serviceAreas.count + 1)" : newAreaName,
            color: color,
            polygon: polygon,
            isActive: true,
            priority: serviceAreas.count,
            priceAdjustmentPercent: 0,
            notes: nil
        )

        do {
            let response: ServiceAreaResponse = try await APIClient.shared.post("/walker/service-areas", body: request)
            let newArea = ServiceArea(from: response)
            serviceAreas.append(newArea)
            currentPolygonPoints.removeAll()
            newAreaName = ""
            mode = .view
        } catch {
            print("Error saving service area: \(error)")
        }
    }

    func deleteArea(_ area: ServiceArea) async {
        do {
            let _: DeleteResponse = try await APIClient.shared.delete("/walker/service-areas/\(area.id)")
            serviceAreas.removeAll { $0.id == area.id }
        } catch {
            print("Error deleting service area: \(error)")
        }
    }
}

#Preview {
    NavigationStack {
        ServiceAreasView()
    }
    .withThemeManager()
}
