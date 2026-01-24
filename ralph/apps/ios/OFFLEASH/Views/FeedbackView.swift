//
//  FeedbackView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import UIKit

struct FeedbackView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss

    @State private var feedbackType: FeedbackType = .bug
    @State private var title = ""
    @State private var description = ""
    @State private var isSubmitting = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showSuccess = false

    // Validation
    private var titleError: String? {
        if title.isEmpty { return nil }
        if title.trimmingCharacters(in: .whitespaces).count < 5 {
            return "Title must be at least 5 characters"
        }
        return nil
    }

    private var descriptionError: String? {
        if description.isEmpty { return nil }
        if description.trimmingCharacters(in: .whitespaces).count < 20 {
            return "Description must be at least 20 characters"
        }
        return nil
    }

    private var isValid: Bool {
        title.trimmingCharacters(in: .whitespaces).count >= 5 &&
        description.trimmingCharacters(in: .whitespaces).count >= 20
    }

    var body: some View {
        NavigationStack {
            Form {
                // Feedback Type Section
                Section {
                    Picker("Type", selection: $feedbackType) {
                        ForEach(FeedbackType.allCases, id: \.self) { type in
                            Label(type.displayName, systemImage: type.icon)
                                .tag(type)
                        }
                    }
                    .pickerStyle(.segmented)
                } header: {
                    Text("What type of feedback?")
                } footer: {
                    Text(feedbackType == .bug
                         ? "Report an issue you've encountered"
                         : "Suggest a new feature or improvement")
                }

                // Title Section
                Section {
                    TextField(feedbackType.titlePlaceholder, text: $title)
                        .textInputAutocapitalization(.sentences)
                } header: {
                    Text("Title")
                } footer: {
                    if let error = titleError {
                        Text(error)
                            .foregroundColor(.red)
                    } else {
                        Text("A brief summary (min 5 characters)")
                    }
                }

                // Description Section
                Section {
                    TextField(feedbackType.placeholder, text: $description, axis: .vertical)
                        .lineLimit(5...10)
                        .textInputAutocapitalization(.sentences)
                } header: {
                    Text("Description")
                } footer: {
                    if let error = descriptionError {
                        Text(error)
                            .foregroundColor(.red)
                    } else {
                        Text("Provide details (min 20 characters). \(description.count)/20")
                    }
                }

                // Device Info (auto-filled)
                Section {
                    HStack {
                        Text("Device")
                        Spacer()
                        Text(deviceInfo)
                            .foregroundColor(.secondary)
                    }

                    HStack {
                        Text("App Version")
                        Spacer()
                        Text(appVersion)
                            .foregroundColor(.secondary)
                    }
                } header: {
                    Text("System Information")
                } footer: {
                    Text("This helps us diagnose issues")
                }
            }
            .navigationTitle("Send Feedback")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .confirmationAction) {
                    Button {
                        submitFeedback()
                    } label: {
                        if isSubmitting {
                            ProgressView()
                        } else {
                            Text("Submit")
                        }
                    }
                    .disabled(!isValid || isSubmitting)
                }
            }
            .onAppear {
                analyticsService.trackScreenView(screenName: "feedback")
            }
            .alert("Error", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage)
            }
            .alert("Thank You!", isPresented: $showSuccess) {
                Button("OK") {
                    dismiss()
                }
            } message: {
                Text("Your feedback has been submitted. We appreciate you taking the time to help us improve!")
            }
        }
    }

    // MARK: - Device Info

    private var deviceInfo: String {
        let device = UIDevice.current
        return "\(device.model) - iOS \(device.systemVersion)"
    }

    private var appVersion: String {
        let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "1.0"
        let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1"
        return "\(version) (\(build))"
    }

    // MARK: - Submit Feedback

    private func submitFeedback() {
        guard isValid else { return }

        isSubmitting = true

        Task {
            do {
                let request = CreateFeedbackRequest(
                    type: feedbackType.rawValue,
                    title: title.trimmingCharacters(in: .whitespaces),
                    description: description.trimmingCharacters(in: .whitespaces)
                )

                let _: CreateFeedbackResponse = try await APIClient.shared.post("/feedback", body: request)

                await MainActor.run {
                    isSubmitting = false
                    analyticsService.trackEvent(name: "feedback_submitted", params: ["type": feedbackType.rawValue])
                    showSuccess = true
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSubmitting = false
                    errorMessage = error.errorDescription ?? "Failed to submit feedback"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isSubmitting = false
                    errorMessage = "An unexpected error occurred. Please try again."
                    showError = true
                }
            }
        }
    }
}

#Preview {
    FeedbackView()
        .withThemeManager()
}
