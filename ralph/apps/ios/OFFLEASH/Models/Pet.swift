//
//  Pet.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Pet Gender

enum PetGender: String, Codable, CaseIterable {
    case male
    case female
    case unknown

    var displayName: String {
        switch self {
        case .male: return "Male"
        case .female: return "Female"
        case .unknown: return "Unknown"
        }
    }

    var icon: String {
        switch self {
        case .male: return "m.circle.fill"
        case .female: return "f.circle.fill"
        case .unknown: return "questionmark.circle.fill"
        }
    }
}

// MARK: - Pet Species

enum PetSpecies: String, Codable, CaseIterable {
    case dog
    case cat
    case other

    var displayName: String {
        switch self {
        case .dog: return "Dog"
        case .cat: return "Cat"
        case .other: return "Other"
        }
    }

    var icon: String {
        switch self {
        case .dog: return "dog.fill"
        case .cat: return "cat.fill"
        case .other: return "pawprint.fill"
        }
    }
}

// MARK: - Vaccination Status

enum VaccinationStatus: String, Codable, CaseIterable {
    case upToDate = "up_to_date"
    case partial
    case expired
    case unknown

    var displayName: String {
        switch self {
        case .upToDate: return "Up to Date"
        case .partial: return "Partial"
        case .expired: return "Expired"
        case .unknown: return "Unknown"
        }
    }

    var color: String {
        switch self {
        case .upToDate: return "green"
        case .partial: return "orange"
        case .expired: return "red"
        case .unknown: return "gray"
        }
    }
}

// MARK: - Pet Model

struct Pet: Identifiable, Codable {
    let id: String
    let userId: String
    let name: String
    let species: PetSpecies
    let breed: String?
    let dateOfBirth: Date?
    let weightLbs: Double?
    let gender: PetGender
    let color: String?
    let microchipId: String?
    let isSpayedNeutered: Bool
    let vaccinationStatus: VaccinationStatus
    let temperament: String?
    let specialNeeds: String?
    let vetName: String?
    let vetPhone: String?
    let emergencyContactName: String?
    let emergencyContactPhone: String?
    let photoUrl: String?
    let notes: String?
    let createdAt: Date?
    let updatedAt: Date?

    // MARK: - Computed Properties

    var displayName: String {
        name
    }

    var age: String? {
        guard let birthDate = dateOfBirth else { return nil }

        let calendar = Calendar.current
        let now = Date()
        let components = calendar.dateComponents([.year, .month], from: birthDate, to: now)

        if let years = components.year, years > 0 {
            return years == 1 ? "1 year old" : "\(years) years old"
        } else if let months = components.month, months > 0 {
            return months == 1 ? "1 month old" : "\(months) months old"
        } else {
            return "Less than a month old"
        }
    }

    var displayWeight: String? {
        guard let weight = weightLbs else { return nil }
        return String(format: "%.1f lbs", weight)
    }

    var breedDisplay: String {
        breed ?? "Unknown breed"
    }

    var speciesIcon: String {
        species.icon
    }

    var subtitle: String {
        var parts: [String] = []
        if let breed = breed, !breed.isEmpty {
            parts.append(breed)
        }
        if let ageString = age {
            parts.append(ageString)
        }
        return parts.isEmpty ? species.displayName : parts.joined(separator: " - ")
    }
}

// MARK: - Pet List Response

struct PetListResponse: Codable {
    let pets: [Pet]
}

// MARK: - Create Pet Request

struct CreatePetRequest: Codable {
    let name: String
    let species: String
    let breed: String?
    let dateOfBirth: String?
    let weightLbs: Double?
    let gender: String
    let color: String?
    let microchipId: String?
    let isSpayedNeutered: Bool
    let vaccinationStatus: String
    let temperament: String?
    let specialNeeds: String?
    let vetName: String?
    let vetPhone: String?
    let emergencyContactName: String?
    let emergencyContactPhone: String?
    let notes: String?
}

// MARK: - Update Pet Request

struct UpdatePetRequest: Codable {
    let name: String?
    let species: String?
    let breed: String?
    let dateOfBirth: String?
    let weightLbs: Double?
    let gender: String?
    let color: String?
    let microchipId: String?
    let isSpayedNeutered: Bool?
    let vaccinationStatus: String?
    let temperament: String?
    let specialNeeds: String?
    let vetName: String?
    let vetPhone: String?
    let emergencyContactName: String?
    let emergencyContactPhone: String?
    let notes: String?
}
