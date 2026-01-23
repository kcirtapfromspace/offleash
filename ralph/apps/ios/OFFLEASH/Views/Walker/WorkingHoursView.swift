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
                        await viewModel.saveSchedule()
                        dismiss()
                    }
                }
                .disabled(viewModel.isSaving)
            }
        }
        .task {
            await viewModel.loadSchedule()
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
        isLoading = true
        defer { isLoading = false }

        // TODO: Load from API when backend supports it
        // For now, use defaults
    }

    func saveSchedule() async {
        isSaving = true
        defer { isSaving = false }

        // TODO: Save to API when backend supports it
        // For now, just simulate a save
        try? await Task.sleep(nanoseconds: 500_000_000)
    }
}

#Preview {
    NavigationStack {
        WorkingHoursView()
    }
    .withThemeManager()
}
