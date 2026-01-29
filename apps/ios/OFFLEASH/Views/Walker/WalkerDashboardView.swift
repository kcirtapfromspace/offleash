//
//  WalkerDashboardView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct WalkerDashboardView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @StateObject private var viewModel = WalkerDashboardViewModel()

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 20) {
                    // Quick Stats
                    statsSection

                    // Next Appointment
                    if let nextBooking = viewModel.nextBooking {
                        nextAppointmentSection(nextBooking)
                    }

                    // Pending Requests
                    if !viewModel.pendingBookings.isEmpty {
                        pendingRequestsSection
                    }

                    // Today's Schedule
                    todayScheduleSection
                }
                .padding()
            }
            .navigationTitle("Dashboard")
            .refreshable {
                await viewModel.refresh()
            }
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { viewModel.showSettings = true }) {
                        Image(systemName: "gearshape")
                    }
                }
            }
            .sheet(isPresented: $viewModel.showSettings) {
                WalkerSettingsView()
            }
            .sheet(item: $viewModel.selectedBooking) { booking in
                BookingDetailView(booking: booking, onAction: { action in
                    Task {
                        await viewModel.handleBookingAction(booking: booking, action: action)
                    }
                })
            }
        }
        .task {
            await viewModel.loadData()
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "walker_dashboard")
        }
    }

    // MARK: - Stats Section

    private var statsSection: some View {
        LazyVGrid(columns: [
            GridItem(.flexible()),
            GridItem(.flexible())
        ], spacing: 16) {
            StatCard(
                title: "Today",
                value: "\(viewModel.todayCount)",
                subtitle: "appointments",
                icon: "calendar.day.timeline.left",
                color: themeManager.primaryColor
            )
            .accessibilityIdentifier("today-bookings-count")

            StatCard(
                title: "Pending",
                value: "\(viewModel.pendingCount)",
                subtitle: "requests",
                icon: "clock.badge.questionmark",
                color: .orange
            )

            StatCard(
                title: "This Week",
                value: "\(viewModel.weekBookingsCount)",
                subtitle: "scheduled",
                icon: "calendar.badge.clock",
                color: .blue
            )
            .accessibilityIdentifier("weekly-earnings")

            StatCard(
                title: "Completed",
                value: "\(viewModel.completedThisWeek)",
                subtitle: "this week",
                icon: "checkmark.circle",
                color: .green
            )
        }
        .accessibilityIdentifier("performance-metrics")
    }

    // MARK: - Next Appointment Section

    private func nextAppointmentSection(_ booking: Booking) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Next Appointment")
                    .font(.headline)
                Spacer()
                if booking.isToday {
                    Text("TODAY")
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(.white)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(themeManager.primaryColor)
                        .cornerRadius(4)
                }
            }

            Button(action: { viewModel.selectedBooking = booking }) {
                HStack(spacing: 16) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(booking.timeString)
                            .font(.title2)
                            .fontWeight(.bold)
                            .foregroundColor(themeManager.primaryColor)
                        Text(booking.dateString)
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }

                    Divider()

                    VStack(alignment: .leading, spacing: 4) {
                        Text(booking.customerName ?? "Customer")
                            .font(.headline)
                            .foregroundColor(.primary)
                        Text(booking.serviceName ?? "Service")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                        if let address = booking.locationAddress {
                            HStack(spacing: 4) {
                                Image(systemName: "mappin")
                                    .font(.caption)
                                Text(address)
                                    .font(.caption)
                                    .lineLimit(1)
                            }
                            .foregroundColor(.secondary)
                        }
                    }

                    Spacer()

                    Image(systemName: "chevron.right")
                        .foregroundColor(.secondary)
                }
                .padding()
                .background(Color(.systemBackground))
                .cornerRadius(12)
                .shadow(color: .black.opacity(0.1), radius: 4, x: 0, y: 2)
            }
            .buttonStyle(.plain)
        }
    }

    // MARK: - Pending Requests Section

    private var pendingRequestsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Pending Requests")
                    .font(.headline)
                Spacer()
                NavigationLink(destination: PendingBookingsView()) {
                    Text("See All")
                        .font(.subheadline)
                        .foregroundColor(themeManager.primaryColor)
                }
            }

            ForEach(viewModel.pendingBookings.prefix(3)) { booking in
                PendingBookingCard(booking: booking) {
                    viewModel.selectedBooking = booking
                }
            }
        }
    }

    // MARK: - Today's Schedule Section

    private var todayScheduleSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Today's Schedule")
                    .font(.headline)
                Spacer()
                NavigationLink(destination: WalkerCalendarView()) {
                    Text("Calendar")
                        .font(.subheadline)
                        .foregroundColor(themeManager.primaryColor)
                }
            }

            if viewModel.todayBookings.isEmpty {
                HStack {
                    Spacer()
                    VStack(spacing: 8) {
                        Image(systemName: "calendar.badge.checkmark")
                            .font(.largeTitle)
                            .foregroundColor(.secondary)
                        Text("No appointments today")
                            .foregroundColor(.secondary)
                    }
                    .padding(.vertical, 32)
                    Spacer()
                }
                .background(Color(.systemGray6))
                .cornerRadius(12)
            } else {
                ForEach(viewModel.todayBookings) { booking in
                    ScheduleBookingRow(booking: booking) {
                        viewModel.selectedBooking = booking
                    }
                }
            }
        }
    }
}

// MARK: - Stat Card

struct StatCard: View {
    let title: String
    let value: String
    let subtitle: String
    let icon: String
    let color: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .foregroundColor(color)
                Spacer()
            }
            Text(value)
                .font(.title)
                .fontWeight(.bold)
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
                Text(subtitle)
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .cornerRadius(12)
    }
}

// MARK: - Pending Booking Card

struct PendingBookingCard: View {
    @EnvironmentObject private var themeManager: ThemeManager
    let booking: Booking
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(booking.customerName ?? "Customer")
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundColor(.primary)
                    Text("\(booking.dateString) at \(booking.timeString)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(booking.serviceName ?? "Service")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                Spacer()
                Text("Review")
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundColor(themeManager.primaryColor)
            }
            .padding()
            .background(Color.orange.opacity(0.1))
            .cornerRadius(8)
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Schedule Booking Row

struct ScheduleBookingRow: View {
    @EnvironmentObject private var themeManager: ThemeManager
    let booking: Booking
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 12) {
                VStack {
                    Text(booking.timeString)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                        .foregroundColor(themeManager.primaryColor)
                }
                .frame(width: 60)

                Rectangle()
                    .fill(statusColor)
                    .frame(width: 3)
                    .cornerRadius(1.5)

                VStack(alignment: .leading, spacing: 2) {
                    Text(booking.customerName ?? "Customer")
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundColor(.primary)
                    Text(booking.serviceName ?? "Service")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                StatusBadge(status: booking.status)
            }
            .padding(.vertical, 8)
        }
        .buttonStyle(.plain)
    }

    private var statusColor: Color {
        switch booking.status {
        case .confirmed: return .blue
        case .inProgress: return .green
        case .pending: return .orange
        case .completed: return .gray
        case .cancelled, .noShow: return .red
        }
    }
}

// MARK: - Status Badge

struct StatusBadge: View {
    let status: BookingStatus

    var body: some View {
        Text(status.displayName)
            .font(.caption2)
            .fontWeight(.medium)
            .foregroundColor(color)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(color.opacity(0.15))
            .cornerRadius(4)
    }

    private var color: Color {
        switch status {
        case .pending: return .orange
        case .confirmed: return .blue
        case .inProgress: return .green
        case .completed: return .gray
        case .cancelled, .noShow: return .red
        }
    }
}

// MARK: - View Model

@MainActor
class WalkerDashboardViewModel: ObservableObject {
    @Published var todayBookings: [Booking] = []
    @Published var pendingBookings: [Booking] = []
    @Published var nextBooking: Booking?
    @Published var isLoading = false
    @Published var showSettings = false
    @Published var selectedBooking: Booking?

    @Published var todayCount = 0
    @Published var pendingCount = 0
    @Published var completedThisWeek = 0
    @Published var weekBookingsCount = 0

    func loadData() async {
        isLoading = true
        defer { isLoading = false }

        do {
            // Fetch all bookings for the walker
            let allBookings: [Booking] = try await APIClient.shared.get("/bookings/walker")

            let today = Calendar.current.startOfDay(for: Date())

            // Filter today's bookings
            todayBookings = allBookings.filter { booking in
                let bookingDate = Calendar.current.startOfDay(for: booking.scheduledStart)
                return bookingDate == today && booking.status != .cancelled
            }.sorted { $0.scheduledStart < $1.scheduledStart }

            // Filter pending bookings
            pendingBookings = allBookings.filter { $0.status == .pending }
                .sorted { $0.scheduledStart < $1.scheduledStart }

            // Find next upcoming booking
            nextBooking = allBookings
                .filter { $0.scheduledStart > Date() && $0.status == .confirmed }
                .sorted { $0.scheduledStart < $1.scheduledStart }
                .first

            // Calculate stats
            todayCount = todayBookings.count
            pendingCount = pendingBookings.count

            // Calculate stats for this week
            let weekStart = Calendar.current.date(from: Calendar.current.dateComponents([.yearForWeekOfYear, .weekOfYear], from: Date()))!
            let weekEnd = Calendar.current.date(byAdding: .weekOfYear, value: 1, to: weekStart)!

            completedThisWeek = allBookings.filter { booking in
                booking.status == .completed && booking.scheduledEnd >= weekStart
            }.count

            // Count all scheduled bookings this week (confirmed + pending)
            weekBookingsCount = allBookings.filter { booking in
                booking.scheduledStart >= weekStart &&
                booking.scheduledStart < weekEnd &&
                (booking.status == .confirmed || booking.status == .pending)
            }.count

        } catch {
            print("Error loading dashboard: \(error)")
        }
    }

    func refresh() async {
        await loadData()
    }

    func handleBookingAction(booking: Booking, action: BookingAction) async {
        do {
            switch action {
            case .confirm:
                let _: Booking = try await APIClient.shared.post("/bookings/\(booking.id)/confirm")
            case .cancel:
                let _: Booking = try await APIClient.shared.post("/bookings/\(booking.id)/cancel")
            case .complete:
                let _: Booking = try await APIClient.shared.post("/bookings/\(booking.id)/complete")
            case .startNavigation:
                // Open Maps with the location
                if let address = booking.locationAddress,
                   let encoded = address.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed),
                   let url = URL(string: "maps://?address=\(encoded)") {
                    await MainActor.run {
                        UIApplication.shared.open(url)
                    }
                }
                return
            case .call:
                if let phone = booking.customerPhone,
                   let url = URL(string: "tel://\(phone)") {
                    await MainActor.run {
                        UIApplication.shared.open(url)
                    }
                }
                return
            }
            selectedBooking = nil
            await loadData()
        } catch {
            print("Error performing action: \(error)")
        }
    }
}

// MARK: - Booking Action

enum BookingAction {
    case confirm
    case cancel
    case complete
    case startNavigation
    case call
}

// MARK: - Placeholder Views

struct WalkerSettingsView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    @ObservedObject private var userSession = UserSession.shared
    @State private var pushNotificationsEnabled = true
    @State private var emailNotificationsEnabled = true
    @State private var showPersonaSwitcher = false

    var body: some View {
        NavigationStack {
            List {
                // Persona Switcher (only if multiple memberships)
                if userSession.hasMultipleMemberships {
                    Section("Current Role") {
                        Button {
                            showPersonaSwitcher = true
                        } label: {
                            HStack(spacing: 12) {
                                Image(systemName: userSession.currentMembership?.role.iconName ?? "person.fill")
                                    .font(.system(size: 18))
                                    .foregroundColor(themeManager.primaryColor)
                                    .frame(width: 32, height: 32)
                                    .background(themeManager.primaryColor.opacity(0.1))
                                    .clipShape(Circle())

                                VStack(alignment: .leading, spacing: 2) {
                                    Text(userSession.currentOrganizationName)
                                        .font(.system(size: 15, weight: .medium))
                                        .foregroundColor(.primary)

                                    Text(userSession.currentMembership?.role.displayName ?? "Walker")
                                        .font(.system(size: 13))
                                        .foregroundColor(.secondary)
                                }

                                Spacer()

                                Image(systemName: "chevron.right")
                                    .font(.system(size: 14))
                                    .foregroundColor(.secondary)
                            }
                        }
                        .buttonStyle(PlainButtonStyle())
                        .accessibilityIdentifier("org-switcher")
                    }
                }

                Section("Account") {
                    NavigationLink {
                        WalkerProfileView()
                            .environmentObject(themeManager)
                    } label: {
                        Label("Profile", systemImage: "person.circle")
                    }
                    .accessibilityIdentifier("settings-profile-update")

                    NavigationLink {
                        WorkingHoursView()
                            .environmentObject(themeManager)
                    } label: {
                        Label("Working Hours", systemImage: "clock")
                    }
                    .accessibilityIdentifier("settings-working-hours")

                    NavigationLink {
                        ServiceAreasView()
                            .environmentObject(themeManager)
                    } label: {
                        Label("Service Areas", systemImage: "map")
                    }
                    .accessibilityIdentifier("settings-service-areas")
                }

                Section("Notifications") {
                    Toggle(isOn: $pushNotificationsEnabled) {
                        Label("Push Notifications", systemImage: "bell.badge")
                    }

                    Toggle(isOn: $emailNotificationsEnabled) {
                        Label("Email Notifications", systemImage: "envelope")
                    }
                }

                Section("Support") {
                    Link(destination: URL(string: "mailto:support@offleash.pro")!) {
                        Label("Contact Support", systemImage: "questionmark.circle")
                    }

                    Link(destination: URL(string: "https://offleash.pro/help")!) {
                        Label("Help Center", systemImage: "book")
                    }
                }

                Section {
                    Button(role: .destructive) {
                        Task {
                            await APIClient.shared.clearAuthToken()
                        }
                    } label: {
                        Label("Sign Out", systemImage: "rectangle.portrait.and.arrow.right")
                    }
                }
            }
            .navigationTitle("Settings")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
            .tint(themeManager.primaryColor)
            .sheet(isPresented: $showPersonaSwitcher) {
                PersonaSwitcherSheet()
            }
        }
    }
}

#Preview {
    WalkerDashboardView()
        .withThemeManager()
}
