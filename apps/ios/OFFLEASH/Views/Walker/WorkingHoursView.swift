//
//  WorkingHoursView.swift
//  OFFLEASH
//
//  Working hours configuration for walkers
//

import SwiftUI

struct WorkingHoursView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    @StateObject private var viewModel = WorkingHoursViewModel()
    @State private var showError = false

    var body: some View {
        List {
            // Quick Setup Section
            Section {
                Button {
                    viewModel.applyPreset(.weekdays9to5)
                } label: {
                    HStack {
                        Image(systemName: "briefcase.fill")
                            .foregroundColor(.orange)
                        VStack(alignment: .leading) {
                            Text("Weekdays 9-5")
                                .foregroundColor(.primary)
                            Text("Mon-Fri, 9:00 AM - 5:00 PM")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                }

                Button {
                    viewModel.applyPreset(.weekdays8to6)
                } label: {
                    HStack {
                        Image(systemName: "clock.fill")
                            .foregroundColor(.blue)
                        VStack(alignment: .leading) {
                            Text("Extended Weekdays")
                                .foregroundColor(.primary)
                            Text("Mon-Fri, 8:00 AM - 6:00 PM")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                }

                Button {
                    viewModel.applyPreset(.everyday)
                } label: {
                    HStack {
                        Image(systemName: "calendar.circle.fill")
                            .foregroundColor(.green)
                        VStack(alignment: .leading) {
                            Text("Every Day")
                                .foregroundColor(.primary)
                            Text("All week, 8:00 AM - 6:00 PM")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                }
            } header: {
                Text("Quick Setup")
            } footer: {
                Text("Select a preset or customize each day below.")
            }

            // Individual Days
            Section("Custom Schedule") {
                ForEach($viewModel.weekSchedule) { $daySchedule in
                    DayScheduleRow(daySchedule: $daySchedule, primaryColor: themeManager.primaryColor)
                }
            }

            // Breaks
            Section {
                Toggle("Lunch Break", isOn: $viewModel.hasLunchBreak)

                if viewModel.hasLunchBreak {
                    HStack {
                        Text("Break Time")
                        Spacer()
                        DatePicker("", selection: $viewModel.lunchBreakStart, displayedComponents: .hourAndMinute)
                            .labelsHidden()
                        Text("-")
                        DatePicker("", selection: $viewModel.lunchBreakEnd, displayedComponents: .hourAndMinute)
                            .labelsHidden()
                    }
                }
            } header: {
                Text("Breaks")
            } footer: {
                Text("Set a daily break time when you're not available for bookings.")
            }

            // Buffer Time
            Section {
                Picker("Buffer Between Bookings", selection: $viewModel.bufferMinutes) {
                    Text("No buffer").tag(0)
                    Text("15 minutes").tag(15)
                    Text("30 minutes").tag(30)
                    Text("45 minutes").tag(45)
                    Text("1 hour").tag(60)
                }
            } footer: {
                Text("Time between appointments for travel and preparation.")
            }
        }
        .navigationTitle("Working Hours")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Save") {
                    Task {
                        do {
                            try await viewModel.saveSchedule()
                            dismiss()
                        } catch {
                            showError = true
                        }
                    }
                }
                .disabled(viewModel.isSaving)
            }
        }
        .task {
            await viewModel.loadSchedule()
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(viewModel.errorMessage ?? "Failed to save working hours")
        }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
            }
        }
    }
}

// MARK: - Day Schedule Row

struct DayScheduleRow: View {
    @Binding var daySchedule: DaySchedule
    let primaryColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Toggle(daySchedule.dayName, isOn: $daySchedule.isEnabled)
                    .tint(primaryColor)
            }

            if daySchedule.isEnabled {
                HStack {
                    DatePicker("Start", selection: $daySchedule.startTime, displayedComponents: .hourAndMinute)
                        .labelsHidden()

                    Text("to")
                        .foregroundColor(.secondary)

                    DatePicker("End", selection: $daySchedule.endTime, displayedComponents: .hourAndMinute)
                        .labelsHidden()

                    Spacer()

                    Text(daySchedule.durationString)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding(.leading, 16)
            }
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Day Schedule Model

struct DaySchedule: Identifiable {
    let id: Int // 0 = Sunday, 6 = Saturday
    var isEnabled: Bool
    var startTime: Date
    var endTime: Date

    var dayName: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "EEEE"
        let calendar = Calendar.current
        let date = calendar.date(from: DateComponents(weekday: id + 1))!
        return formatter.string(from: date)
    }

    var durationString: String {
        let hours = endTime.timeIntervalSince(startTime) / 3600
        if hours < 1 {
            return "\(Int(hours * 60))m"
        }
        let wholeHours = Int(hours)
        let minutes = Int((hours - Double(wholeHours)) * 60)
        if minutes > 0 {
            return "\(wholeHours)h \(minutes)m"
        }
        return "\(wholeHours)h"
    }
}

// MARK: - Schedule Presets

enum SchedulePreset {
    case weekdays9to5
    case weekdays8to6
    case everyday

    var schedule: [DaySchedule] {
        let calendar = Calendar.current
        let today = calendar.startOfDay(for: Date())

        func time(_ hour: Int, _ minute: Int = 0) -> Date {
            calendar.date(bySettingHour: hour, minute: minute, second: 0, of: today)!
        }

        switch self {
        case .weekdays9to5:
            return (0..<7).map { day in
                DaySchedule(
                    id: day,
                    isEnabled: day >= 1 && day <= 5, // Mon-Fri
                    startTime: time(9),
                    endTime: time(17)
                )
            }
        case .weekdays8to6:
            return (0..<7).map { day in
                DaySchedule(
                    id: day,
                    isEnabled: day >= 1 && day <= 5,
                    startTime: time(8),
                    endTime: time(18)
                )
            }
        case .everyday:
            return (0..<7).map { day in
                DaySchedule(
                    id: day,
                    isEnabled: true,
                    startTime: time(8),
                    endTime: time(18)
                )
            }
        }
    }
}

// MARK: - API Models

struct WorkingHoursResponse: Decodable {
    let id: String
    let walkerId: String
    let dayOfWeek: Int
    let dayName: String
    let startTime: String  // "HH:MM" format
    let endTime: String    // "HH:MM" format
    let isActive: Bool
}

struct DayScheduleInput: Encodable {
    let dayOfWeek: Int
    let startTime: String  // "HH:MM" format
    let endTime: String    // "HH:MM" format
    let isActive: Bool
}

struct UpdateScheduleRequest: Encodable {
    let schedule: [DayScheduleInput]
}

// MARK: - View Model

@MainActor
class WorkingHoursViewModel: ObservableObject {
    @Published var weekSchedule: [DaySchedule] = []
    @Published var hasLunchBreak = false
    @Published var lunchBreakStart = Date()
    @Published var lunchBreakEnd = Date()
    @Published var bufferMinutes = 15
    @Published var isSaving = false
    @Published var isLoading = false
    @Published var errorMessage: String?

    private let timeFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm"
        return formatter
    }()

    init() {
        // Initialize with default schedule
        applyPreset(.weekdays9to5)

        // Set default lunch break
        let calendar = Calendar.current
        let today = calendar.startOfDay(for: Date())
        lunchBreakStart = calendar.date(bySettingHour: 12, minute: 0, second: 0, of: today)!
        lunchBreakEnd = calendar.date(bySettingHour: 13, minute: 0, second: 0, of: today)!
    }

    func applyPreset(_ preset: SchedulePreset) {
        weekSchedule = preset.schedule
    }

    func loadSchedule() async {
        guard let walkerId = UserSession.shared.currentUser?.id else {
            errorMessage = "User not logged in"
            return
        }

        isLoading = true
        errorMessage = nil
        defer { isLoading = false }

        do {
            let hours: [WorkingHoursResponse] = try await APIClient.shared.get("/working-hours/\(walkerId)")

            // If we got data from the API, update the schedule
            if !hours.isEmpty {
                updateScheduleFromAPI(hours)
            }
            // Otherwise keep the default schedule
        } catch let error as APIError {
            // If 404, the walker has no schedule yet - use defaults
            if case .httpError(let statusCode, _) = error, statusCode == 404 {
                // Keep defaults, no error
                return
            }
            errorMessage = error.errorDescription
            print("Failed to load working hours: \(error)")
        } catch {
            errorMessage = "Failed to load schedule"
            print("Failed to load working hours: \(error)")
        }
    }

    func saveSchedule() async throws {
        guard let walkerId = UserSession.shared.currentUser?.id else {
            errorMessage = "User not logged in"
            throw APIError.unauthorized
        }

        isSaving = true
        errorMessage = nil
        defer { isSaving = false }

        // Convert UI schedule to API format
        let scheduleInput = weekSchedule.map { day -> DayScheduleInput in
            DayScheduleInput(
                dayOfWeek: day.id,
                startTime: timeFormatter.string(from: day.startTime),
                endTime: timeFormatter.string(from: day.endTime),
                isActive: day.isEnabled
            )
        }

        let request = UpdateScheduleRequest(schedule: scheduleInput)

        do {
            let _: [WorkingHoursResponse] = try await APIClient.shared.put("/working-hours/\(walkerId)", body: request)
        } catch let error as APIError {
            errorMessage = error.errorDescription
            print("Failed to save working hours: \(error)")
            throw error
        } catch {
            errorMessage = "Failed to save schedule"
            print("Failed to save working hours: \(error)")
            throw error
        }
    }

    // MARK: - Private Helpers

    private func updateScheduleFromAPI(_ hours: [WorkingHoursResponse]) {
        let calendar = Calendar.current
        let today = calendar.startOfDay(for: Date())

        // Create a lookup by day of week
        var hoursByDay: [Int: WorkingHoursResponse] = [:]
        for hour in hours {
            hoursByDay[hour.dayOfWeek] = hour
        }

        // Update or create schedule for each day
        weekSchedule = (0..<7).map { dayIndex in
            if let apiHour = hoursByDay[dayIndex] {
                // Parse time strings from API
                let startTime = parseTime(apiHour.startTime, on: today)
                let endTime = parseTime(apiHour.endTime, on: today)

                return DaySchedule(
                    id: dayIndex,
                    isEnabled: apiHour.isActive,
                    startTime: startTime,
                    endTime: endTime
                )
            } else {
                // No data for this day - default to disabled
                return DaySchedule(
                    id: dayIndex,
                    isEnabled: false,
                    startTime: calendar.date(bySettingHour: 9, minute: 0, second: 0, of: today)!,
                    endTime: calendar.date(bySettingHour: 17, minute: 0, second: 0, of: today)!
                )
            }
        }
    }

    private func parseTime(_ timeString: String, on date: Date) -> Date {
        let calendar = Calendar.current
        let components = timeString.split(separator: ":").compactMap { Int($0) }

        guard components.count >= 2 else {
            // Fallback to noon if parsing fails
            return calendar.date(bySettingHour: 12, minute: 0, second: 0, of: date)!
        }

        return calendar.date(bySettingHour: components[0], minute: components[1], second: 0, of: date)!
    }
}

#Preview {
    NavigationStack {
        WorkingHoursView()
    }
    .withThemeManager()
}
