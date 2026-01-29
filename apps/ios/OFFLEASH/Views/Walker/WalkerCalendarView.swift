//
//  WalkerCalendarView.swift
//  OFFLEASH
//
//  iOS Calendar-style day view for walkers with week and month views
//

import SwiftUI

// MARK: - Calendar View Mode

enum CalendarViewMode: String, CaseIterable {
    case day = "Day"
    case week = "Week"
    case month = "Month"
}

struct WalkerCalendarView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @StateObject private var viewModel = WalkerCalendarViewModel()
    @State private var selectedDate = Date()
    @State private var showAddBreak = false
    @State private var viewMode: CalendarViewMode = .day

    // Drag-to-create state
    @State private var isDragging = false
    @State private var dragStartY: CGFloat = 0
    @State private var dragEndY: CGFloat = 0
    @State private var showCreateActionSheet = false
    @State private var dragStartTime: Date?
    @State private var dragEndTime: Date?

    var body: some View {
        VStack(spacing: 0) {
            // View mode picker
            viewModePicker

            Divider()

            // Content based on view mode
            switch viewMode {
            case .day:
                dayView
            case .week:
                CalendarWeekView(
                    selectedDate: $selectedDate,
                    bookings: viewModel.allBookings,
                    blocks: viewModel.allBlocks,
                    onDateSelected: { date in
                        selectedDate = date
                        viewMode = .day
                    }
                )
            case .month:
                CalendarMonthView(
                    selectedDate: $selectedDate,
                    bookings: viewModel.allBookings,
                    blocks: viewModel.allBlocks,
                    onDateSelected: { date in
                        selectedDate = date
                        viewMode = .day
                    }
                )
            }
        }
        .accessibilityIdentifier("calendar-view")
        .navigationTitle("Schedule")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarLeading) {
                Button(action: { selectedDate = Date() }) {
                    Text("Today")
                        .font(.subheadline)
                }
            }
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: { showAddBreak = true }) {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showAddBreak, onDismiss: {
            // Clear prefilled times when sheet is dismissed
            viewModel.prefilledBreakStart = nil
            viewModel.prefilledBreakEnd = nil
        }) {
            AddBreakView(
                selectedDate: selectedDate,
                prefilledStart: viewModel.prefilledBreakStart,
                prefilledEnd: viewModel.prefilledBreakEnd
            ) {
                Task { await viewModel.loadData() }
            }
            .environmentObject(themeManager)
        }
        .sheet(item: $viewModel.selectedBooking) { booking in
            BookingDetailView(booking: booking) { action in
                Task {
                    await viewModel.handleBookingAction(booking: booking, action: action)
                }
            }
        }
        .task {
            await viewModel.loadData()
        }
        .onChange(of: selectedDate) { newDate in
            viewModel.selectDate(newDate)
        }
        .onAppear {
            analyticsService.trackScreenView(screenName: "walker_calendar")
        }
    }

    // MARK: - View Mode Picker

    private var viewModePicker: some View {
        Picker("View Mode", selection: $viewMode) {
            ForEach(CalendarViewMode.allCases, id: \.self) { mode in
                Text(mode.rawValue).tag(mode)
            }
        }
        .pickerStyle(.segmented)
        .accessibilityIdentifier("calendar-toggle-week")
        .padding(.horizontal)
        .padding(.vertical, 8)
    }

    // MARK: - Day View (existing timeline)

    private var dayView: some View {
        VStack(spacing: 0) {
            // Week selector
            weekSelector

            Divider()

            // Timeline day view
            ScrollViewReader { proxy in
                ScrollView {
                    timelineView
                        .id("timeline")
                }
                .onAppear {
                    // Scroll to current hour on appear
                    DispatchQueue.main.asyncAfter(deadline: .now() + 0.3) {
                        let currentHour = Calendar.current.component(.hour, from: Date())
                        let targetHour = max(6, min(currentHour - 1, 18))
                        proxy.scrollTo("hour-\(targetHour)", anchor: .top)
                    }
                }
            }
        }
    }

    // MARK: - Week Selector

    private var weekSelector: some View {
        VStack(spacing: 12) {
            HStack {
                Button(action: { moveWeek(by: -1) }) {
                    Image(systemName: "chevron.left")
                        .font(.title3)
                }

                Spacer()

                Text(monthYearString)
                    .font(.headline)

                Spacer()

                Button(action: { moveWeek(by: 1) }) {
                    Image(systemName: "chevron.right")
                        .font(.title3)
                }
            }
            .padding(.horizontal)

            HStack(spacing: 0) {
                ForEach(weekDates, id: \.self) { date in
                    DayButton(
                        date: date,
                        isSelected: Calendar.current.isDate(date, inSameDayAs: selectedDate),
                        isToday: Calendar.current.isDateInToday(date),
                        hasBookings: viewModel.hasBookings(on: date),
                        primaryColor: themeManager.primaryColor
                    ) {
                        selectedDate = date
                    }
                }
            }
        }
        .padding(.vertical, 12)
        .background(Color(.systemBackground))
    }

    private var weekDates: [Date] {
        let calendar = Calendar.current
        let startOfWeek = calendar.date(from: calendar.dateComponents([.yearForWeekOfYear, .weekOfYear], from: selectedDate))!
        return (0..<7).compactMap { calendar.date(byAdding: .day, value: $0, to: startOfWeek) }
    }

    private var monthYearString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "MMMM yyyy"
        return formatter.string(from: selectedDate)
    }

    private func moveWeek(by weeks: Int) {
        if let newDate = Calendar.current.date(byAdding: .weekOfYear, value: weeks, to: selectedDate) {
            selectedDate = newDate
        }
    }

    // MARK: - Timeline View (iOS Calendar style)

    private var timelineView: some View {
        ZStack(alignment: .topLeading) {
            // Hour grid lines
            VStack(spacing: 0) {
                ForEach(6..<22) { hour in
                    HourRow(hour: hour)
                        .id("hour-\(hour)")
                }
            }

            // Events overlay
            GeometryReader { geometry in
                let hourHeight: CGFloat = 60
                let leftMargin: CGFloat = 60

                // Drag-to-create overlay
                if isDragging {
                    DragPreviewOverlay(
                        startY: min(dragStartY, dragEndY),
                        endY: max(dragStartY, dragEndY),
                        leftMargin: leftMargin,
                        width: geometry.size.width - leftMargin - 16,
                        startTime: dragStartTime,
                        endTime: dragEndTime,
                        primaryColor: themeManager.primaryColor
                    )
                }

                // Breaks (gray blocks)
                ForEach(viewModel.blocksForSelectedDate) { block in
                    BreakEventBlock(
                        block: block,
                        hourHeight: hourHeight,
                        leftMargin: leftMargin,
                        width: geometry.size.width - leftMargin - 16
                    ) {
                        // TODO: Allow editing/deleting breaks
                    }
                }

                // Bookings (colored blocks)
                ForEach(viewModel.bookingsForSelectedDate) { booking in
                    BookingEventBlock(
                        booking: booking,
                        hourHeight: hourHeight,
                        leftMargin: leftMargin,
                        width: geometry.size.width - leftMargin - 16,
                        primaryColor: themeManager.primaryColor,
                        driveTime: viewModel.getDriveTimeBefore(booking)
                    ) {
                        viewModel.selectedBooking = booking
                    }
                }

                // Current time indicator (red line)
                if Calendar.current.isDateInToday(selectedDate) {
                    CurrentTimeIndicator(hourHeight: hourHeight, leftMargin: leftMargin)
                }

                // Drag gesture layer (invisible, captures taps)
                Color.clear
                    .contentShape(Rectangle())
                    .gesture(
                        DragGesture(minimumDistance: 20)
                            .onChanged { value in
                                handleDragChanged(value: value, hourHeight: hourHeight, geometry: geometry)
                            }
                            .onEnded { _ in
                                handleDragEnded()
                            }
                    )
            }
        }
        .padding(.trailing, 8)
        .confirmationDialog(
            "Create Event",
            isPresented: $showCreateActionSheet,
            titleVisibility: .visible
        ) {
            if let start = dragStartTime, let end = dragEndTime {
                Button("Block Time Off") {
                    showAddBreakWithTimes(start: start, end: end)
                }
                Button("Cancel", role: .cancel) {
                    resetDragState()
                }
            }
        } message: {
            if let start = dragStartTime, let end = dragEndTime {
                Text(timeRangeString(start: start, end: end))
            }
        }
    }

    // MARK: - Drag Gesture Handling

    private func handleDragChanged(value: DragGesture.Value, hourHeight: CGFloat, geometry: GeometryProxy) {
        let leftMargin: CGFloat = 60

        // Only process drags that start in the timeline area (not the hour labels)
        guard value.startLocation.x > leftMargin else { return }

        if !isDragging {
            isDragging = true
            dragStartY = value.startLocation.y
        }
        dragEndY = value.location.y

        // Calculate times from Y positions
        dragStartTime = yPositionToTime(dragStartY, hourHeight: hourHeight)
        dragEndTime = yPositionToTime(dragEndY, hourHeight: hourHeight)

        // Ensure start is before end
        if let start = dragStartTime, let end = dragEndTime, start > end {
            swap(&dragStartTime, &dragEndTime)
        }
    }

    private func handleDragEnded() {
        guard isDragging,
              let start = dragStartTime,
              let end = dragEndTime,
              end.timeIntervalSince(start) >= 15 * 60 else { // Minimum 15 minutes
            resetDragState()
            return
        }

        showCreateActionSheet = true
        isDragging = false
    }

    private func resetDragState() {
        isDragging = false
        dragStartY = 0
        dragEndY = 0
        dragStartTime = nil
        dragEndTime = nil
    }

    private func yPositionToTime(_ y: CGFloat, hourHeight: CGFloat) -> Date {
        let calendar = Calendar.current
        let startHour = 6
        let hoursFromStart = y / hourHeight
        let totalMinutes = Int(hoursFromStart * 60)

        // Snap to 15-minute increments
        let snappedMinutes = (totalMinutes / 15) * 15

        let hour = startHour + (snappedMinutes / 60)
        let minute = snappedMinutes % 60

        return calendar.date(bySettingHour: hour, minute: minute, second: 0, of: selectedDate) ?? selectedDate
    }

    private func timeRangeString(start: Date, end: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "h:mm a"
        return "\(formatter.string(from: start)) - \(formatter.string(from: end))"
    }

    private func showAddBreakWithTimes(start: Date, end: Date) {
        // Set the break times and show the sheet
        viewModel.prefilledBreakStart = start
        viewModel.prefilledBreakEnd = end
        showAddBreak = true
        resetDragState()
    }
}

// MARK: - Drag Preview Overlay

struct DragPreviewOverlay: View {
    let startY: CGFloat
    let endY: CGFloat
    let leftMargin: CGFloat
    let width: CGFloat
    let startTime: Date?
    let endTime: Date?
    let primaryColor: Color

    var body: some View {
        VStack(spacing: 0) {
            // Time range label
            if let start = startTime, let end = endTime {
                Text(timeRangeString(start: start, end: end))
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundColor(.white)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(primaryColor)
                    .cornerRadius(4)
            }

            // Preview rectangle
            Rectangle()
                .fill(primaryColor.opacity(0.3))
                .overlay(
                    Rectangle()
                        .strokeBorder(primaryColor, style: StrokeStyle(lineWidth: 2, dash: [5]))
                )
        }
        .frame(width: width, height: max(endY - startY, 30))
        .position(x: leftMargin + width / 2, y: startY + (endY - startY) / 2)
    }

    private func timeRangeString(start: Date, end: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "h:mm a"
        return "\(formatter.string(from: start)) - \(formatter.string(from: end))"
    }
}

// MARK: - Hour Row

struct HourRow: View {
    let hour: Int

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Text(hourString)
                .font(.caption)
                .foregroundColor(.secondary)
                .frame(width: 50, alignment: .trailing)

            VStack {
                Divider()
                Spacer()
            }
        }
        .frame(height: 60)
    }

    private var hourString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "h a"
        let date = Calendar.current.date(bySettingHour: hour, minute: 0, second: 0, of: Date())!
        return formatter.string(from: date)
    }
}

// MARK: - Booking Event Block

struct BookingEventBlock: View {
    let booking: Booking
    let hourHeight: CGFloat
    let leftMargin: CGFloat
    let width: CGFloat
    let primaryColor: Color
    let driveTime: Int? // Drive time in minutes before this booking
    let onTap: () -> Void

    var body: some View {
        VStack(spacing: 2) {
            // Drive time indicator above booking
            if let minutes = driveTime, minutes > 0 {
                HStack(spacing: 4) {
                    Image(systemName: "car.fill")
                        .font(.system(size: 10))
                    Text("\(minutes) min drive")
                        .font(.system(size: 10))
                }
                .foregroundColor(.secondary)
                .padding(.horizontal, 6)
                .padding(.vertical, 2)
                .background(Color(.systemGray5))
                .cornerRadius(4)
                .offset(y: -4)
            }

            // Booking block
            Button(action: onTap) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(booking.customerName ?? "Customer")
                        .font(.caption)
                        .fontWeight(.semibold)
                        .lineLimit(1)

                    Text(booking.serviceName ?? "Service")
                        .font(.caption2)
                        .lineLimit(1)

                    if let address = booking.locationAddress {
                        Text(address)
                            .font(.system(size: 9))
                            .lineLimit(1)
                            .opacity(0.8)
                    }
                }
                .foregroundColor(.white)
                .padding(.horizontal, 8)
                .padding(.vertical, 6)
                .frame(maxWidth: .infinity, alignment: .leading)
                .frame(height: max(blockHeight - 4, 30))
                .background(statusColor)
                .cornerRadius(6)
            }
            .buttonStyle(.plain)
        }
        .position(x: leftMargin + width / 2, y: topOffset + blockHeight / 2)
    }

    private var topOffset: CGFloat {
        let calendar = Calendar.current
        let startHour = calendar.component(.hour, from: booking.scheduledStart)
        let startMinute = calendar.component(.minute, from: booking.scheduledStart)
        return CGFloat(startHour - 6) * hourHeight + CGFloat(startMinute) * hourHeight / 60
    }

    private var blockHeight: CGFloat {
        let duration = booking.scheduledEnd.timeIntervalSince(booking.scheduledStart)
        return max(CGFloat(duration / 3600) * hourHeight, 30)
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

// MARK: - Break Event Block

struct BreakEventBlock: View {
    let block: WalkerBlock
    let hourHeight: CGFloat
    let leftMargin: CGFloat
    let width: CGFloat
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 4) {
                    Image(systemName: "pause.circle.fill")
                        .font(.caption2)
                    Text(block.reason)
                        .font(.caption)
                        .fontWeight(.medium)
                }
                .lineLimit(1)
            }
            .foregroundColor(.secondary)
            .padding(.horizontal, 8)
            .padding(.vertical, 6)
            .frame(maxWidth: .infinity, alignment: .leading)
            .frame(height: max(blockHeight - 4, 24))
            .background(Color(.systemGray5))
            .cornerRadius(6)
            .overlay(
                RoundedRectangle(cornerRadius: 6)
                    .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [4]))
                    .foregroundColor(.gray)
            )
        }
        .buttonStyle(.plain)
        .position(x: leftMargin + width / 2, y: topOffset + blockHeight / 2)
    }

    private var topOffset: CGFloat {
        let calendar = Calendar.current
        let startHour = calendar.component(.hour, from: block.startTime)
        let startMinute = calendar.component(.minute, from: block.startTime)
        return CGFloat(startHour - 6) * hourHeight + CGFloat(startMinute) * hourHeight / 60
    }

    private var blockHeight: CGFloat {
        let duration = block.endTime.timeIntervalSince(block.startTime)
        return max(CGFloat(duration / 3600) * hourHeight, 24)
    }
}

// MARK: - Current Time Indicator

struct CurrentTimeIndicator: View {
    let hourHeight: CGFloat
    let leftMargin: CGFloat

    var body: some View {
        HStack(spacing: 0) {
            Circle()
                .fill(Color.red)
                .frame(width: 10, height: 10)

            Rectangle()
                .fill(Color.red)
                .frame(height: 1)
        }
        .offset(x: leftMargin - 5, y: currentTimeOffset)
    }

    private var currentTimeOffset: CGFloat {
        let calendar = Calendar.current
        let hour = calendar.component(.hour, from: Date())
        let minute = calendar.component(.minute, from: Date())
        return CGFloat(hour - 6) * hourHeight + CGFloat(minute) * hourHeight / 60
    }
}

// MARK: - Day Button

struct DayButton: View {
    let date: Date
    let isSelected: Bool
    let isToday: Bool
    let hasBookings: Bool
    let primaryColor: Color
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            VStack(spacing: 4) {
                Text(dayOfWeekString)
                    .font(.caption2)
                    .foregroundColor(isSelected ? .white : .secondary)

                Text(dayString)
                    .font(.headline)
                    .foregroundColor(isSelected ? .white : (isToday ? primaryColor : .primary))

                Circle()
                    .fill(hasBookings ? (isSelected ? .white : primaryColor) : .clear)
                    .frame(width: 6, height: 6)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(isSelected ? primaryColor : Color.clear)
            )
        }
        .accessibilityIdentifier("calendar-day-\(isoDateString)")
        .buttonStyle(.plain)
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

    private var isoDateString: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        return formatter.string(from: date)
    }
}

// MARK: - Walker Block Model

struct WalkerBlock: Codable, Identifiable {
    let id: String
    let walkerId: String
    let reason: String
    let startTime: Date
    let endTime: Date
    let isRecurring: Bool
    let recurrenceRule: String?

    enum CodingKeys: String, CodingKey {
        case id
        case walkerId
        case reason
        case startTime
        case endTime
        case isRecurring
        case recurrenceRule
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decode(String.self, forKey: .id)
        walkerId = try container.decode(String.self, forKey: .walkerId)
        reason = try container.decode(String.self, forKey: .reason)
        isRecurring = try container.decode(Bool.self, forKey: .isRecurring)
        recurrenceRule = try container.decodeIfPresent(String.self, forKey: .recurrenceRule)

        // Parse ISO8601 dates
        let startTimeString = try container.decode(String.self, forKey: .startTime)
        let endTimeString = try container.decode(String.self, forKey: .endTime)

        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]

        if let start = formatter.date(from: startTimeString) {
            startTime = start
        } else {
            formatter.formatOptions = [.withInternetDateTime]
            startTime = formatter.date(from: startTimeString) ?? Date()
        }

        if let end = formatter.date(from: endTimeString) {
            endTime = end
        } else {
            formatter.formatOptions = [.withInternetDateTime]
            endTime = formatter.date(from: endTimeString) ?? Date()
        }
    }
}

// MARK: - View Model

@MainActor
class WalkerCalendarViewModel: ObservableObject {
    @Published var allBookings: [Booking] = []
    @Published var allBlocks: [WalkerBlock] = []
    @Published var bookingsForSelectedDate: [Booking] = []
    @Published var blocksForSelectedDate: [WalkerBlock] = []
    @Published var isLoading = false
    @Published var selectedBooking: Booking?

    // Prefilled times for drag-to-create
    var prefilledBreakStart: Date?
    var prefilledBreakEnd: Date?

    private var selectedDate = Date()

    func loadData() async {
        isLoading = true
        defer { isLoading = false }

        do {
            async let bookingsTask: [Booking] = APIClient.shared.get("/bookings/walker")
            async let blocksTask: [WalkerBlock] = APIClient.shared.get("/blocks")

            let (bookings, blocks) = try await (bookingsTask, blocksTask)
            allBookings = bookings
            allBlocks = blocks
            selectDate(selectedDate)
        } catch {
            print("Error loading calendar data: \(error)")
        }
    }

    func selectDate(_ date: Date) {
        selectedDate = date
        let calendar = Calendar.current
        let startOfDay = calendar.startOfDay(for: date)
        let endOfDay = calendar.date(byAdding: .day, value: 1, to: startOfDay)!

        bookingsForSelectedDate = allBookings.filter { booking in
            let bookingDate = calendar.startOfDay(for: booking.scheduledStart)
            return bookingDate == startOfDay && booking.status != .cancelled
        }.sorted { $0.scheduledStart < $1.scheduledStart }

        blocksForSelectedDate = allBlocks.filter { block in
            block.startTime < endOfDay && block.endTime > startOfDay
        }.sorted { $0.startTime < $1.startTime }
    }

    func hasBookings(on date: Date) -> Bool {
        let calendar = Calendar.current
        let startOfDay = calendar.startOfDay(for: date)

        return allBookings.contains { booking in
            let bookingDate = calendar.startOfDay(for: booking.scheduledStart)
            return bookingDate == startOfDay && booking.status != .cancelled
        }
    }

    func getDriveTimeBefore(_ booking: Booking) -> Int? {
        // Find the previous booking
        guard let index = bookingsForSelectedDate.firstIndex(where: { $0.id == booking.id }),
              index > 0 else {
            return nil
        }

        let previousBooking = bookingsForSelectedDate[index - 1]
        let gapMinutes = Int(booking.scheduledStart.timeIntervalSince(previousBooking.scheduledEnd) / 60)

        // Only show drive time if there's a gap and it's reasonable (5-60 min)
        if gapMinutes >= 5 && gapMinutes <= 60 {
            // Estimate drive time as roughly 70% of the gap
            return min(gapMinutes - 5, 30)
        }
        return nil
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
            case .startNavigation, .call:
                return
            }
            selectedBooking = nil
            await loadData()
        } catch {
            print("Error performing action: \(error)")
        }
    }
}

// MARK: - Create Block Request

struct CreateBlockRequest: Encodable {
    let reason: String
    let start_time: String
    let end_time: String
}

// MARK: - Add Break View

struct AddBreakView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss

    let selectedDate: Date
    let prefilledStart: Date?
    let prefilledEnd: Date?
    let onSave: () -> Void

    @State private var reason = "Break"
    @State private var startTime = Date()
    @State private var endTime = Date()
    @State private var isSaving = false
    @State private var showError = false
    @State private var errorMessage = ""

    init(selectedDate: Date, prefilledStart: Date? = nil, prefilledEnd: Date? = nil, onSave: @escaping () -> Void) {
        self.selectedDate = selectedDate
        self.prefilledStart = prefilledStart
        self.prefilledEnd = prefilledEnd
        self.onSave = onSave
    }

    var body: some View {
        NavigationStack {
            Form {
                Section("Reason") {
                    TextField("Break reason", text: $reason)
                }

                Section("Time") {
                    DatePicker("Start", selection: $startTime, displayedComponents: [.date, .hourAndMinute])
                    DatePicker("End", selection: $endTime, displayedComponents: [.date, .hourAndMinute])
                }

                Section {
                    Button {
                        saveBreak()
                    } label: {
                        HStack {
                            Spacer()
                            if isSaving {
                                ProgressView()
                                    .tint(.white)
                            } else {
                                Text("Add Break")
                                    .fontWeight(.semibold)
                            }
                            Spacer()
                        }
                    }
                    .listRowBackground(isValid ? themeManager.primaryColor : Color(.systemGray4))
                    .foregroundColor(.white)
                    .disabled(!isValid || isSaving)
                }
            }
            .navigationTitle("Add Break")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
            }
            .onAppear {
                // Use prefilled times if available, otherwise default to noon for 1 hour
                if let start = prefilledStart, let end = prefilledEnd {
                    startTime = start
                    endTime = end
                } else {
                    let calendar = Calendar.current
                    startTime = calendar.date(bySettingHour: 12, minute: 0, second: 0, of: selectedDate) ?? selectedDate
                    endTime = calendar.date(byAdding: .hour, value: 1, to: startTime) ?? startTime
                }
            }
            .alert("Error", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage)
            }
        }
    }

    private var isValid: Bool {
        !reason.isEmpty && endTime > startTime
    }

    private func saveBreak() {
        isSaving = true

        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]

        let request = CreateBlockRequest(
            reason: reason,
            start_time: formatter.string(from: startTime),
            end_time: formatter.string(from: endTime)
        )

        Task {
            do {
                let _: WalkerBlock = try await APIClient.shared.post("/blocks", body: request)
                await MainActor.run {
                    isSaving = false
                    onSave()
                    dismiss()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSaving = false
                    errorMessage = error.errorDescription ?? "Failed to add break"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isSaving = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }
}

#Preview {
    NavigationStack {
        WalkerCalendarView()
    }
    .withThemeManager()
}
