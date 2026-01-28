//
//  WalkerMapView.swift
//  OFFLEASH
//
//  Map view showing all walker appointments with route optimization
//

import SwiftUI
import MapKit

struct WalkerMapView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @StateObject private var viewModel = WalkerMapViewModel()
    @State private var selectedBookingId: String?
    @State private var showOptimizeSheet = false
    @State private var cameraPosition: MapCameraPosition = .automatic
    @State private var selectedDate: Date = Date()

    var body: some View {
        VStack(spacing: 0) {
            // Date picker header
            datePickerHeader

            ZStack {
            // Map with booking annotations
            Map(position: $cameraPosition, selection: $selectedBookingId) {
                // Current location marker
                if let currentLocation = viewModel.currentLocation {
                    Annotation("You", coordinate: currentLocation) {
                        ZStack {
                            Circle()
                                .fill(Color.blue)
                                .frame(width: 24, height: 24)
                            Circle()
                                .fill(Color.white)
                                .frame(width: 10, height: 10)
                        }
                        .shadow(radius: 3)
                    }
                }

                // Booking markers
                ForEach(viewModel.bookings) { booking in
                    if let coord = booking.coordinate {
                        Annotation(booking.customerName ?? "Customer", coordinate: coord, anchor: .bottom) {
                            BookingMapPin(
                                booking: booking,
                                sequence: viewModel.getSequenceNumber(for: booking),
                                isOptimized: viewModel.isOptimized,
                                primaryColor: themeManager.primaryColor
                            )
                        }
                        .tag(booking.id)
                    }
                }

                // Route polyline if optimized
                if viewModel.isOptimized, let route = viewModel.optimizedRoute {
                    MapPolyline(coordinates: route)
                        .stroke(themeManager.primaryColor, lineWidth: 3)
                }
            }
            .mapStyle(.standard(pointsOfInterest: .excludingAll))
            .mapControls {
                MapUserLocationButton()
                MapCompass()
                MapScaleView()
            }

            // Bottom card with summary
            VStack {
                Spacer()

                // Summary card
                summaryCard
            }
        }
        .navigationTitle("Route Map")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: { showOptimizeSheet = true }) {
                    HStack(spacing: 4) {
                        Image(systemName: "wand.and.stars")
                        Text("Optimize")
                    }
                    .font(.subheadline)
                }
            }
        }
        .sheet(isPresented: $showOptimizeSheet) {
            OptimizeRouteSheet(viewModel: viewModel)
                .environmentObject(themeManager)
        }
        .sheet(isPresented: Binding(
            get: { selectedBookingId != nil },
            set: { if !$0 { selectedBookingId = nil } }
        )) {
            if let bookingId = selectedBookingId,
               let booking = viewModel.bookings.first(where: { $0.id == bookingId }) {
                BookingMapDetailSheet(booking: booking)
                    .environmentObject(themeManager)
            }
        }
        .task {
            await viewModel.loadBookings(for: selectedDate)
        }
        .onChange(of: selectedDate) { newDate in
            Task {
                await viewModel.loadBookings(for: newDate)
            }
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "walker_map")
        }
        }
    }

    // MARK: - Date Picker Header

    private var datePickerHeader: some View {
        HStack(spacing: 16) {
            // Previous day button
            Button(action: { moveDate(by: -1) }) {
                Image(systemName: "chevron.left")
                    .font(.title3)
                    .foregroundColor(themeManager.primaryColor)
            }

            Spacer()

            // Today button
            Button(action: { selectedDate = Date() }) {
                Text("Today")
                    .font(.subheadline)
                    .fontWeight(.medium)
                    .foregroundColor(Calendar.current.isDateInToday(selectedDate) ? .secondary : themeManager.primaryColor)
            }
            .disabled(Calendar.current.isDateInToday(selectedDate))

            // Date display with picker
            DatePicker(
                "",
                selection: $selectedDate,
                displayedComponents: .date
            )
            .datePickerStyle(.compact)
            .labelsHidden()
            .accentColor(themeManager.primaryColor)

            Spacer()

            // Next day button
            Button(action: { moveDate(by: 1) }) {
                Image(systemName: "chevron.right")
                    .font(.title3)
                    .foregroundColor(themeManager.primaryColor)
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 12)
        .background(Color(.systemBackground))
    }

    private func moveDate(by days: Int) {
        if let newDate = Calendar.current.date(byAdding: .day, value: days, to: selectedDate) {
            selectedDate = newDate
        }
    }

    // MARK: - Summary Card

    private var summaryCard: some View {
        VStack(spacing: 12) {
            // Stats row
            HStack(spacing: 20) {
                StatItem(
                    icon: "mappin.circle.fill",
                    value: "\(viewModel.bookings.count)",
                    label: "Stops",
                    color: themeManager.primaryColor
                )

                StatItem(
                    icon: "clock.fill",
                    value: viewModel.totalDurationString,
                    label: "Walking",
                    color: .green
                )

                StatItem(
                    icon: "car.fill",
                    value: viewModel.totalDriveTimeString,
                    label: "Driving",
                    color: .orange
                )
            }

            // Optimization status
            if viewModel.isOptimized {
                HStack {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.green)
                    Text("Route optimized - saves \(viewModel.timeSavedString)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                }
            }

            // Next appointment
            if let next = viewModel.nextBooking {
                Divider()
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("NEXT UP")
                            .font(.caption2)
                            .fontWeight(.bold)
                            .foregroundColor(.secondary)
                        Text(next.customerName ?? "Customer")
                            .font(.subheadline)
                            .fontWeight(.semibold)
                        Text(next.locationAddress ?? "")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    VStack(alignment: .trailing, spacing: 2) {
                        Text(next.timeString)
                            .font(.headline)
                            .foregroundColor(themeManager.primaryColor)
                        if let driveTime = viewModel.getDriveTimeTo(next) {
                            Text("\(driveTime) min drive")
                                .font(.caption)
                                .foregroundColor(.orange)
                        }
                    }
                    Button(action: { openNavigation(to: next) }) {
                        Image(systemName: "arrow.triangle.turn.up.right.circle.fill")
                            .font(.title)
                            .foregroundColor(themeManager.primaryColor)
                    }
                }
            }
        }
        .padding()
        .background(.ultraThinMaterial)
        .cornerRadius(16)
        .shadow(color: .black.opacity(0.1), radius: 10, x: 0, y: -5)
        .padding()
    }

    private func openNavigation(to booking: Booking) {
        guard let address = booking.locationAddress,
              let encoded = address.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed),
              let url = URL(string: "maps://?daddr=\(encoded)") else { return }
        UIApplication.shared.open(url)
    }
}

// MARK: - Stat Item

struct StatItem: View {
    let icon: String
    let value: String
    let label: String
    let color: Color

    var body: some View {
        VStack(spacing: 4) {
            Image(systemName: icon)
                .font(.title3)
                .foregroundColor(color)
            Text(value)
                .font(.headline)
                .fontWeight(.bold)
            Text(label)
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity)
    }
}

// MARK: - Booking Map Pin

struct BookingMapPin: View {
    let booking: Booking
    let sequence: Int?
    let isOptimized: Bool
    let primaryColor: Color

    var body: some View {
        VStack(spacing: 0) {
            ZStack {
                // Pin background
                Circle()
                    .fill(statusColor)
                    .frame(width: 36, height: 36)
                    .shadow(color: statusColor.opacity(0.5), radius: 4)

                // Sequence number or icon
                if let seq = sequence, isOptimized {
                    Text("\(seq)")
                        .font(.system(size: 14, weight: .bold))
                        .foregroundColor(.white)
                } else {
                    Image(systemName: "pawprint.fill")
                        .font(.system(size: 16))
                        .foregroundColor(.white)
                }
            }

            // Pin tail
            Triangle()
                .fill(statusColor)
                .frame(width: 12, height: 8)
                .offset(y: -2)
        }
    }

    private var statusColor: Color {
        switch booking.status {
        case .pending: return .orange
        case .confirmed: return primaryColor
        case .inProgress: return .green
        case .completed: return .gray
        case .cancelled, .noShow: return .red
        }
    }
}

// MARK: - Triangle Shape

struct Triangle: Shape {
    func path(in rect: CGRect) -> Path {
        var path = Path()
        path.move(to: CGPoint(x: rect.midX, y: rect.maxY))
        path.addLine(to: CGPoint(x: rect.minX, y: rect.minY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY))
        path.closeSubpath()
        return path
    }
}

// MARK: - Optimize Route Sheet

struct OptimizeRouteSheet: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var viewModel: WalkerMapViewModel
    @State private var isOptimizing = false
    @State private var optimizationMode: OptimizationMode = .minimizeTravel

    enum OptimizationMode: String, CaseIterable {
        case minimizeTravel = "Minimize Travel"
        case earliestFirst = "Earliest First"
        case pendingFirst = "Pending First"

        var description: String {
            switch self {
            case .minimizeTravel:
                return "Reorder stops to minimize total driving time"
            case .earliestFirst:
                return "Keep original time-based order"
            case .pendingFirst:
                return "Prioritize pending bookings that need confirmation"
            }
        }

        var icon: String {
            switch self {
            case .minimizeTravel: return "car.fill"
            case .earliestFirst: return "clock.fill"
            case .pendingFirst: return "exclamationmark.circle.fill"
            }
        }
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                // Header illustration
                ZStack {
                    Circle()
                        .fill(themeManager.primaryColor.opacity(0.1))
                        .frame(width: 100, height: 100)
                    Image(systemName: "wand.and.stars")
                        .font(.system(size: 40))
                        .foregroundColor(themeManager.primaryColor)
                }
                .padding(.top)

                Text("Optimize Your Route")
                    .font(.title2)
                    .fontWeight(.bold)

                Text("Let us find the most efficient order for your appointments today.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal)

                // Optimization modes
                VStack(spacing: 12) {
                    ForEach(OptimizationMode.allCases, id: \.self) { mode in
                        OptimizationModeRow(
                            mode: mode,
                            isSelected: optimizationMode == mode,
                            primaryColor: themeManager.primaryColor
                        ) {
                            optimizationMode = mode
                        }
                    }
                }
                .padding(.horizontal)

                Spacer()

                // Current stats
                if viewModel.bookings.count > 1 {
                    VStack(spacing: 8) {
                        Text("Current Route")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Text("~\(viewModel.estimatedTotalDriveMinutes) min total driving")
                            .font(.headline)
                    }
                    .padding()
                    .frame(maxWidth: .infinity)
                    .background(Color(.systemGray6))
                    .cornerRadius(12)
                    .padding(.horizontal)
                }

                // Optimize button
                Button {
                    Task {
                        isOptimizing = true
                        await viewModel.optimizeRoute(mode: optimizationMode)
                        isOptimizing = false
                        dismiss()
                    }
                } label: {
                    HStack {
                        if isOptimizing {
                            ProgressView()
                                .tint(.white)
                        } else {
                            Image(systemName: "sparkles")
                            Text("Optimize Route")
                        }
                    }
                    .font(.headline)
                    .foregroundColor(.white)
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(themeManager.primaryColor)
                    .cornerRadius(12)
                }
                .disabled(isOptimizing || viewModel.bookings.count < 2)
                .padding(.horizontal)
                .padding(.bottom)
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }
}

// MARK: - Optimization Mode Row

struct OptimizationModeRow: View {
    let mode: OptimizeRouteSheet.OptimizationMode
    let isSelected: Bool
    let primaryColor: Color
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 16) {
                Image(systemName: mode.icon)
                    .font(.title2)
                    .foregroundColor(isSelected ? primaryColor : .secondary)
                    .frame(width: 32)

                VStack(alignment: .leading, spacing: 2) {
                    Text(mode.rawValue)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                        .foregroundColor(.primary)
                    Text(mode.description)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                Image(systemName: isSelected ? "checkmark.circle.fill" : "circle")
                    .foregroundColor(isSelected ? primaryColor : .secondary)
            }
            .padding()
            .background(isSelected ? primaryColor.opacity(0.1) : Color(.systemGray6))
            .cornerRadius(12)
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(isSelected ? primaryColor : Color.clear, lineWidth: 2)
            )
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Booking Map Detail Sheet

struct BookingMapDetailSheet: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    let booking: Booking

    var body: some View {
        NavigationStack {
            VStack(spacing: 20) {
                // Status badge
                HStack {
                    StatusBadge(status: booking.status)
                    Spacer()
                    Text(booking.timeString)
                        .font(.title2)
                        .fontWeight(.bold)
                        .foregroundColor(themeManager.primaryColor)
                }

                Divider()

                // Customer info
                VStack(alignment: .leading, spacing: 12) {
                    DetailRow(icon: "person.fill", title: "Customer", value: booking.customerName ?? "Unknown")
                    DetailRow(icon: "mappin.circle.fill", title: "Location", value: booking.locationAddress ?? "Unknown")
                    DetailRow(icon: "pawprint.fill", title: "Service", value: booking.serviceName ?? "Walk")
                    if let notes = booking.notes, !notes.isEmpty {
                        DetailRow(icon: "note.text", title: "Notes", value: notes)
                    }
                }

                Spacer()

                // Action buttons
                HStack(spacing: 16) {
                    Button {
                        if let address = booking.locationAddress,
                           let encoded = address.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed),
                           let url = URL(string: "maps://?daddr=\(encoded)") {
                            UIApplication.shared.open(url)
                        }
                    } label: {
                        Label("Navigate", systemImage: "arrow.triangle.turn.up.right.circle.fill")
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(themeManager.primaryColor)

                    if let phone = booking.customerPhone {
                        Button {
                            if let url = URL(string: "tel://\(phone)") {
                                UIApplication.shared.open(url)
                            }
                        } label: {
                            Label("Call", systemImage: "phone.fill")
                        }
                        .buttonStyle(.bordered)
                    }
                }
            }
            .padding()
            .navigationTitle("Appointment Details")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium])
    }
}

// MARK: - Detail Row

struct DetailRow: View {
    let icon: String
    let title: String
    let value: String

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: icon)
                .foregroundColor(.secondary)
                .frame(width: 24)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.caption)
                    .foregroundColor(.secondary)
                Text(value)
                    .font(.subheadline)
            }
            Spacer()
        }
    }
}

// MARK: - View Model

@MainActor
class WalkerMapViewModel: ObservableObject {
    @Published var bookings: [Booking] = []
    @Published var currentLocation: CLLocationCoordinate2D?
    @Published var isOptimized = false
    @Published var optimizedRoute: [CLLocationCoordinate2D]?
    @Published var isLoading = false
    @Published var selectedDate: Date = Date()

    private var originalOrder: [Booking] = []
    private var driveTimeCache: [String: Int] = [:]

    var nextBooking: Booking? {
        bookings.first { $0.scheduledStart > Date() && $0.status != .cancelled }
    }

    var totalDurationString: String {
        let totalMinutes = bookings.reduce(0) { $0 + ($1.durationMinutes ?? 30) }
        let hours = totalMinutes / 60
        let minutes = totalMinutes % 60
        if hours > 0 {
            return "\(hours)h \(minutes)m"
        }
        return "\(minutes)m"
    }

    var totalDriveTimeString: String {
        let minutes = estimatedTotalDriveMinutes
        if minutes >= 60 {
            return "\(minutes / 60)h \(minutes % 60)m"
        }
        return "\(minutes)m"
    }

    var estimatedTotalDriveMinutes: Int {
        guard bookings.count > 1 else { return 0 }
        var total = 0
        for i in 1..<bookings.count {
            total += estimateDriveTime(from: bookings[i-1], to: bookings[i])
        }
        return total
    }

    var totalEarningsString: String {
        let totalCents = bookings.reduce(0) { $0 + $1.priceCents }
        return String(format: "$%.0f", Double(totalCents) / 100.0)
    }

    var timeSavedString: String {
        guard isOptimized else { return "0m" }
        let originalTime = calculateTotalDriveTime(for: originalOrder)
        let optimizedTime = estimatedTotalDriveMinutes
        let saved = originalTime - optimizedTime
        return "\(max(0, saved))m"
    }

    func loadBookings(for date: Date) async {
        isLoading = true
        selectedDate = date
        defer { isLoading = false }

        do {
            let fetchedBookings: [Booking] = try await APIClient.shared.get("/bookings/walker")

            // Filter to selected date's bookings only
            let calendar = Calendar.current
            let startOfDay = calendar.startOfDay(for: date)
            let endOfDay = calendar.date(byAdding: .day, value: 1, to: startOfDay)!

            bookings = fetchedBookings.filter { booking in
                booking.scheduledStart >= startOfDay &&
                booking.scheduledStart < endOfDay &&
                booking.status != .cancelled
            }.sorted { $0.scheduledStart < $1.scheduledStart }

            originalOrder = bookings
            isOptimized = false

            // Build route coordinates
            updateRouteCoordinates()

            // Get current location (mock for now)
            currentLocation = CLLocationCoordinate2D(latitude: 39.7392, longitude: -104.9903)

        } catch {
            print("Error loading bookings: \(error)")
        }
    }

    func getSequenceNumber(for booking: Booking) -> Int? {
        guard isOptimized else { return nil }
        return bookings.firstIndex(where: { $0.id == booking.id }).map { $0 + 1 }
    }

    func getDriveTimeTo(_ booking: Booking) -> Int? {
        guard let index = bookings.firstIndex(where: { $0.id == booking.id }) else { return nil }

        if index == 0 {
            // From current location
            return 10 // Mock: 10 min from current location
        } else {
            return estimateDriveTime(from: bookings[index - 1], to: booking)
        }
    }

    func optimizeRoute(mode: OptimizeRouteSheet.OptimizationMode) async {
        guard bookings.count > 1 else { return }

        switch mode {
        case .minimizeTravel:
            await optimizeForMinimalTravel()
        case .earliestFirst:
            bookings = originalOrder
            isOptimized = false
        case .pendingFirst:
            optimizeForPendingFirst()
        }

        updateRouteCoordinates()
    }

    private func optimizeForMinimalTravel() async {
        // Use backend route optimization API
        guard !bookings.isEmpty else { return }

        do {
            // Get the walker's user ID from stored credentials
            guard let userId = UserDefaults.standard.string(forKey: "userId") else {
                // Fallback to local optimization if no user ID
                await localOptimizeForMinimalTravel()
                return
            }

            let response = try await AvailabilityService.shared.optimizeRoute(
                walkerId: userId,
                date: selectedDate
            )

            // Reorder bookings based on API response
            var optimizedBookings: [Booking] = []
            for stop in response.stops {
                if let booking = bookings.first(where: { $0.id == stop.bookingId }) {
                    optimizedBookings.append(booking)
                }
            }

            // Update with optimized order
            if !optimizedBookings.isEmpty {
                bookings = optimizedBookings
                isOptimized = true
                timeSaved = response.savingsMinutes
            } else {
                // Fallback if response doesn't match
                await localOptimizeForMinimalTravel()
            }
        } catch {
            print("Failed to optimize route via API: \(error)")
            // Fallback to local optimization
            await localOptimizeForMinimalTravel()
        }
    }

    private var timeSaved: Int = 0

    private func localOptimizeForMinimalTravel() async {
        // Nearest neighbor algorithm for TSP approximation (local fallback)
        var remaining = bookings
        var optimized: [Booking] = []
        var currentCoord = currentLocation ?? CLLocationCoordinate2D(latitude: 39.7392, longitude: -104.9903)

        while !remaining.isEmpty {
            // Find nearest booking
            var nearestIndex = 0
            var nearestDistance = Double.infinity

            for (index, booking) in remaining.enumerated() {
                if let coord = booking.coordinate {
                    let distance = calculateDistance(from: currentCoord, to: coord)
                    if distance < nearestDistance {
                        nearestDistance = distance
                        nearestIndex = index
                    }
                }
            }

            let nearest = remaining.remove(at: nearestIndex)
            optimized.append(nearest)

            if let coord = nearest.coordinate {
                currentCoord = coord
            }
        }

        bookings = optimized
        isOptimized = true
    }

    private func optimizeForPendingFirst() {
        let pending = bookings.filter { $0.status == .pending }
        let confirmed = bookings.filter { $0.status != .pending }
        bookings = pending + confirmed
        isOptimized = true
    }

    private func updateRouteCoordinates() {
        optimizedRoute = bookings.compactMap { $0.coordinate }
        if let current = currentLocation {
            optimizedRoute?.insert(current, at: 0)
        }
    }

    private func estimateDriveTime(from: Booking, to: Booking) -> Int {
        guard let fromCoord = from.coordinate, let toCoord = to.coordinate else { return 15 }

        // Rough estimate: 2 minutes per km in city traffic
        let distance = calculateDistance(from: fromCoord, to: toCoord)
        return max(5, Int(distance * 2))
    }

    private func calculateDistance(from: CLLocationCoordinate2D, to: CLLocationCoordinate2D) -> Double {
        let fromLocation = CLLocation(latitude: from.latitude, longitude: from.longitude)
        let toLocation = CLLocation(latitude: to.latitude, longitude: to.longitude)
        return fromLocation.distance(from: toLocation) / 1000.0 // km
    }

    private func calculateTotalDriveTime(for bookings: [Booking]) -> Int {
        guard bookings.count > 1 else { return 0 }
        var total = 0
        for i in 1..<bookings.count {
            total += estimateDriveTime(from: bookings[i-1], to: bookings[i])
        }
        return total
    }
}

// MARK: - Booking Extension for Map

extension Booking {
    var coordinate: CLLocationCoordinate2D? {
        guard let lat = latitude, let lon = longitude else { return nil }
        return CLLocationCoordinate2D(latitude: lat, longitude: lon)
    }

    var durationMinutes: Int? {
        Int(scheduledEnd.timeIntervalSince(scheduledStart) / 60)
    }
}

#Preview {
    NavigationStack {
        WalkerMapView()
    }
    .withThemeManager()
}
