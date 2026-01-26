//
//  CalendarMonthView.swift
//  OFFLEASH
//
//  Month calendar grid view for walkers showing booking counts and busy indicators
//

import SwiftUI

struct CalendarMonthView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Binding var selectedDate: Date
    let bookings: [Booking]
    let blocks: [WalkerBlock]
    let onDateSelected: (Date) -> Void

    private let calendar = Calendar.current
    private let daysOfWeek = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]

    var body: some View {
        VStack(spacing: 0) {
            // Month navigation header
            monthNavigationHeader

            // Day of week headers
            dayOfWeekHeaders

            Divider()

            // Calendar grid
            calendarGrid
        }
        .gesture(
            DragGesture(minimumDistance: 50)
                .onEnded { value in
                    if value.translation.width < -50 {
                        moveMonth(by: 1)
                    } else if value.translation.width > 50 {
                        moveMonth(by: -1)
                    }
                }
        )
    }

    // MARK: - Month Navigation Header

    private var monthNavigationHeader: some View {
        HStack {
            Button(action: { moveMonth(by: -1) }) {
                Image(systemName: "chevron.left")
                    .font(.title3)
                    .foregroundColor(themeManager.primaryColor)
            }

            Spacer()

            Text(monthYearString)
                .font(.headline)

            Spacer()

            Button(action: { moveMonth(by: 1) }) {
                Image(systemName: "chevron.right")
                    .font(.title3)
                    .foregroundColor(themeManager.primaryColor)
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 12)
    }

    private var dayOfWeekHeaders: some View {
        HStack(spacing: 0) {
            ForEach(daysOfWeek, id: \.self) { day in
                Text(day)
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity)
            }
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 8)
    }

    // MARK: - Calendar Grid

    private var calendarGrid: some View {
        let dates = monthDates
        return LazyVGrid(
            columns: Array(repeating: GridItem(.flexible(), spacing: 0), count: 7),
            spacing: 4
        ) {
            ForEach(Array(dates.enumerated()), id: \.offset) { index, date in
                if let date = date {
                    MonthDayCell(
                        date: date,
                        isSelected: calendar.isDate(date, inSameDayAs: selectedDate),
                        isToday: calendar.isDateInToday(date),
                        isCurrentMonth: calendar.isDate(date, equalTo: selectedDate, toGranularity: .month),
                        bookingCount: bookingCount(for: date),
                        busyLevel: busyLevel(for: date),
                        primaryColor: themeManager.primaryColor
                    ) {
                        onDateSelected(date)
                    }
                } else {
                    Color.clear
                        .frame(height: 60)
                }
            }
        }
        .padding(.horizontal, 8)
        .padding(.top, 4)
    }

    // MARK: - Computed Properties

    private var monthYearString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "MMMM yyyy"
        return formatter.string(from: selectedDate)
    }

    private var monthDates: [Date?] {
        let firstOfMonth = calendar.date(from: calendar.dateComponents([.year, .month], from: selectedDate))!
        let firstWeekday = calendar.component(.weekday, from: firstOfMonth)
        let daysInMonth = calendar.range(of: .day, in: .month, for: selectedDate)!.count

        var dates: [Date?] = []

        // Add empty cells for days before the first of the month
        for _ in 1..<firstWeekday {
            dates.append(nil)
        }

        // Add days of the month
        for day in 1...daysInMonth {
            let date = calendar.date(byAdding: .day, value: day - 1, to: firstOfMonth)!
            dates.append(date)
        }

        // Add empty cells to complete the last week (up to 6 rows = 42 cells)
        while dates.count < 42 && dates.count % 7 != 0 {
            dates.append(nil)
        }

        return dates
    }

    // MARK: - Helper Functions

    private func moveMonth(by months: Int) {
        if let newDate = calendar.date(byAdding: .month, value: months, to: selectedDate) {
            selectedDate = newDate
        }
    }

    private func bookingCount(for date: Date) -> Int {
        bookings.filter { booking in
            calendar.isDate(booking.scheduledStart, inSameDayAs: date) && booking.status != .cancelled
        }.count
    }

    private func busyLevel(for date: Date) -> BusyLevel {
        let count = bookingCount(for: date)
        let hasBlocks = blocks.contains { block in
            let startOfDay = calendar.startOfDay(for: date)
            let endOfDay = calendar.date(byAdding: .day, value: 1, to: startOfDay)!
            return block.startTime < endOfDay && block.endTime > startOfDay
        }

        if count == 0 && !hasBlocks {
            return .free
        } else if count <= 2 {
            return .light
        } else if count <= 4 {
            return .moderate
        } else {
            return .busy
        }
    }
}

// MARK: - Busy Level

enum BusyLevel {
    case free
    case light
    case moderate
    case busy

    var color: Color {
        switch self {
        case .free: return .clear
        case .light: return .green.opacity(0.3)
        case .moderate: return .orange.opacity(0.3)
        case .busy: return .red.opacity(0.3)
        }
    }
}

// MARK: - Month Day Cell

struct MonthDayCell: View {
    let date: Date
    let isSelected: Bool
    let isToday: Bool
    let isCurrentMonth: Bool
    let bookingCount: Int
    let busyLevel: BusyLevel
    let primaryColor: Color
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 4) {
                // Date number
                ZStack {
                    if isToday {
                        Circle()
                            .fill(primaryColor)
                            .frame(width: 28, height: 28)
                    } else if isSelected {
                        Circle()
                            .stroke(primaryColor, lineWidth: 2)
                            .frame(width: 28, height: 28)
                    }

                    Text(dayString)
                        .font(.subheadline)
                        .fontWeight(isToday || isSelected ? .bold : .regular)
                        .foregroundColor(textColor)
                }

                // Booking count badge
                if bookingCount > 0 {
                    Text("\(bookingCount) walk\(bookingCount == 1 ? "" : "s")")
                        .font(.system(size: 9))
                        .foregroundColor(isCurrentMonth ? .secondary : .secondary.opacity(0.5))
                        .lineLimit(1)
                }

                Spacer(minLength: 0)
            }
            .frame(height: 60)
            .frame(maxWidth: .infinity)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(isCurrentMonth ? busyLevel.color : Color.clear)
            )
        }
        .buttonStyle(.plain)
        .opacity(isCurrentMonth ? 1.0 : 0.4)
    }

    private var dayString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "d"
        return formatter.string(from: date)
    }

    private var textColor: Color {
        if isToday {
            return .white
        } else if isSelected {
            return primaryColor
        } else if !isCurrentMonth {
            return .secondary
        } else {
            return .primary
        }
    }
}

#Preview {
    CalendarMonthView(
        selectedDate: .constant(Date()),
        bookings: [],
        blocks: [],
        onDateSelected: { _ in }
    )
    .withThemeManager()
}
