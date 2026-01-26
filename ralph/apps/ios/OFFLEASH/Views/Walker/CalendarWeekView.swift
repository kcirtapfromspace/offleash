//
//  CalendarWeekView.swift
//  OFFLEASH
//
//  Week calendar grid view for walkers showing 7 days with event indicators
//

import SwiftUI

struct CalendarWeekView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Binding var selectedDate: Date
    let bookings: [Booking]
    let blocks: [WalkerBlock]
    let onDateSelected: (Date) -> Void

    private let calendar = Calendar.current

    var body: some View {
        VStack(spacing: 0) {
            // Week navigation header
            weekNavigationHeader

            // Day columns
            HStack(spacing: 0) {
                ForEach(weekDates, id: \.self) { date in
                    DayColumn(
                        date: date,
                        isSelected: calendar.isDate(date, inSameDayAs: selectedDate),
                        isToday: calendar.isDateInToday(date),
                        bookings: bookingsFor(date),
                        blocks: blocksFor(date),
                        primaryColor: themeManager.primaryColor
                    ) {
                        onDateSelected(date)
                    }
                }
            }
            .frame(maxHeight: .infinity)
        }
    }

    // MARK: - Week Navigation Header

    private var weekNavigationHeader: some View {
        HStack {
            Button(action: { moveWeek(by: -1) }) {
                Image(systemName: "chevron.left")
                    .font(.title3)
                    .foregroundColor(themeManager.primaryColor)
            }

            Spacer()

            Text(weekRangeString)
                .font(.headline)

            Spacer()

            Button(action: { moveWeek(by: 1) }) {
                Image(systemName: "chevron.right")
                    .font(.title3)
                    .foregroundColor(themeManager.primaryColor)
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 12)
    }

    // MARK: - Computed Properties

    private var weekDates: [Date] {
        let startOfWeek = calendar.date(from: calendar.dateComponents([.yearForWeekOfYear, .weekOfYear], from: selectedDate))!
        return (0..<7).compactMap { calendar.date(byAdding: .day, value: $0, to: startOfWeek) }
    }

    private var weekRangeString: String {
        let dates = weekDates
        guard let first = dates.first, let last = dates.last else { return "" }

        let formatter = DateFormatter()

        // Check if week spans two months
        let firstMonth = calendar.component(.month, from: first)
        let lastMonth = calendar.component(.month, from: last)

        if firstMonth == lastMonth {
            formatter.dateFormat = "MMM d"
            let startStr = formatter.string(from: first)
            formatter.dateFormat = "d, yyyy"
            let endStr = formatter.string(from: last)
            return "\(startStr) - \(endStr)"
        } else {
            formatter.dateFormat = "MMM d"
            let startStr = formatter.string(from: first)
            formatter.dateFormat = "MMM d, yyyy"
            let endStr = formatter.string(from: last)
            return "\(startStr) - \(endStr)"
        }
    }

    // MARK: - Helper Functions

    private func moveWeek(by weeks: Int) {
        if let newDate = calendar.date(byAdding: .weekOfYear, value: weeks, to: selectedDate) {
            selectedDate = newDate
        }
    }

    private func bookingsFor(_ date: Date) -> [Booking] {
        let startOfDay = calendar.startOfDay(for: date)
        return bookings.filter { booking in
            calendar.isDate(booking.scheduledStart, inSameDayAs: startOfDay) && booking.status != .cancelled
        }
    }

    private func blocksFor(_ date: Date) -> [WalkerBlock] {
        let startOfDay = calendar.startOfDay(for: date)
        let endOfDay = calendar.date(byAdding: .day, value: 1, to: startOfDay)!
        return blocks.filter { block in
            block.startTime < endOfDay && block.endTime > startOfDay
        }
    }
}

// MARK: - Day Column

struct DayColumn: View {
    let date: Date
    let isSelected: Bool
    let isToday: Bool
    let bookings: [Booking]
    let blocks: [WalkerBlock]
    let primaryColor: Color
    let onTap: () -> Void

    private let calendar = Calendar.current

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 4) {
                // Day header
                dayHeader

                // Mini timeline
                miniTimeline
                    .frame(maxHeight: .infinity)
            }
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(isSelected ? primaryColor.opacity(0.1) : Color.clear)
            )
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(isSelected ? primaryColor : Color.clear, lineWidth: 2)
            )
        }
        .buttonStyle(.plain)
    }

    private var dayHeader: some View {
        VStack(spacing: 2) {
            Text(dayOfWeekString)
                .font(.caption2)
                .fontWeight(.medium)
                .foregroundColor(isSelected ? primaryColor : .secondary)

            ZStack {
                if isToday {
                    Circle()
                        .fill(primaryColor)
                        .frame(width: 28, height: 28)
                }

                Text(dayString)
                    .font(.subheadline)
                    .fontWeight(isToday ? .bold : .regular)
                    .foregroundColor(isToday ? .white : (isSelected ? primaryColor : .primary))
            }

            // Booking count badge
            if !bookings.isEmpty {
                Text("\(bookings.count)")
                    .font(.system(size: 10, weight: .bold))
                    .foregroundColor(.white)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(primaryColor)
                    .cornerRadius(8)
            }
        }
    }

    private var miniTimeline: some View {
        GeometryReader { geometry in
            let hourHeight = geometry.size.height / 16 // 6 AM to 10 PM
            let startHour = 6

            ZStack(alignment: .top) {
                // Hour lines
                VStack(spacing: 0) {
                    ForEach(0..<16, id: \.self) { _ in
                        Rectangle()
                            .fill(Color(.systemGray5))
                            .frame(height: 1)
                        Spacer()
                    }
                }

                // Block indicators (gray)
                ForEach(blocks) { block in
                    let startOffset = hourOffset(for: block.startTime, startHour: startHour, hourHeight: hourHeight)
                    let height = hourHeight * CGFloat(block.endTime.timeIntervalSince(block.startTime) / 3600)

                    Rectangle()
                        .fill(Color(.systemGray4))
                        .frame(width: geometry.size.width - 4, height: max(height, 2))
                        .cornerRadius(2)
                        .offset(y: startOffset)
                }

                // Booking indicators (colored)
                ForEach(bookings) { booking in
                    let startOffset = hourOffset(for: booking.scheduledStart, startHour: startHour, hourHeight: hourHeight)
                    let height = hourHeight * CGFloat(booking.scheduledEnd.timeIntervalSince(booking.scheduledStart) / 3600)

                    Rectangle()
                        .fill(statusColor(for: booking))
                        .frame(width: geometry.size.width - 4, height: max(height, 4))
                        .cornerRadius(2)
                        .offset(y: startOffset)
                }

                // Current time indicator (only for today)
                if isToday {
                    let currentOffset = currentTimeOffset(startHour: startHour, hourHeight: hourHeight)
                    Rectangle()
                        .fill(Color.red)
                        .frame(height: 2)
                        .offset(y: currentOffset)
                }
            }
            .padding(.horizontal, 2)
        }
    }

    private func hourOffset(for time: Date, startHour: Int, hourHeight: CGFloat) -> CGFloat {
        let hour = calendar.component(.hour, from: time)
        let minute = calendar.component(.minute, from: time)
        return CGFloat(hour - startHour) * hourHeight + CGFloat(minute) / 60 * hourHeight
    }

    private func currentTimeOffset(startHour: Int, hourHeight: CGFloat) -> CGFloat {
        let hour = calendar.component(.hour, from: Date())
        let minute = calendar.component(.minute, from: Date())
        return CGFloat(hour - startHour) * hourHeight + CGFloat(minute) / 60 * hourHeight
    }

    private func statusColor(for booking: Booking) -> Color {
        switch booking.status {
        case .pending: return .orange
        case .confirmed: return primaryColor
        case .inProgress: return .green
        case .completed: return .gray
        case .cancelled, .noShow: return .red
        }
    }

    private var dayOfWeekString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "EEE"
        return formatter.string(from: date).uppercased()
    }

    private var dayString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "d"
        return formatter.string(from: date)
    }
}

#Preview {
    CalendarWeekView(
        selectedDate: .constant(Date()),
        bookings: [],
        blocks: [],
        onDateSelected: { _ in }
    )
    .withThemeManager()
}
