//
//  Feedback.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Feedback Type

enum FeedbackType: String, Codable, CaseIterable {
    case bug
    case feature

    var displayName: String {
        switch self {
        case .bug: return "Bug Report"
        case .feature: return "Feature Request"
        }
    }

    var icon: String {
        switch self {
        case .bug: return "ladybug.fill"
        case .feature: return "lightbulb.fill"
        }
    }

    var placeholder: String {
        switch self {
        case .bug: return "Describe the bug you encountered..."
        case .feature: return "Describe the feature you'd like to see..."
        }
    }

    var titlePlaceholder: String {
        switch self {
        case .bug: return "Brief description of the issue"
        case .feature: return "Feature name or summary"
        }
    }
}

// MARK: - Feedback Model

struct Feedback: Identifiable, Codable {
    let id: String
    let userId: String
    let type: FeedbackType
    let title: String
    let description: String
    let status: FeedbackStatus?
    let createdAt: Date?
}

// MARK: - Feedback Status

enum FeedbackStatus: String, Codable {
    case submitted
    case reviewed
    case inProgress = "in_progress"
    case resolved
    case closed

    var displayName: String {
        switch self {
        case .submitted: return "Submitted"
        case .reviewed: return "Under Review"
        case .inProgress: return "In Progress"
        case .resolved: return "Resolved"
        case .closed: return "Closed"
        }
    }
}

// MARK: - Create Feedback Request

struct CreateFeedbackRequest: Codable {
    let type: String
    let title: String
    let description: String
}

// MARK: - Create Feedback Response

struct CreateFeedbackResponse: Codable {
    let id: String
    let message: String?
}
