//
//  PetService.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import Foundation

// MARK: - Pet Service

actor PetService {
    static let shared = PetService()

    private init() {}

    // MARK: - Pet CRUD Operations

    /// Fetch all pets for the current user
    func getPets() async throws -> [Pet] {
        let response: PetListResponse = try await APIClient.shared.get("/pets")
        return response.pets
    }

    /// Fetch a single pet by ID
    func getPet(id: String) async throws -> Pet {
        try await APIClient.shared.get("/pets/\(id)")
    }

    /// Create a new pet
    func createPet(_ request: CreatePetRequest) async throws -> Pet {
        try await APIClient.shared.post("/pets", body: request)
    }

    /// Update an existing pet
    func updatePet(id: String, request: UpdatePetRequest) async throws -> Pet {
        try await APIClient.shared.put("/pets/\(id)", body: request)
    }

    /// Delete a pet
    func deletePet(id: String) async throws {
        try await APIClient.shared.delete("/pets/\(id)")
    }
}

// MARK: - Helper Extensions

extension CreatePetRequest {
    /// Create a request from form data
    static func from(
        name: String,
        species: PetSpecies,
        breed: String?,
        dateOfBirth: Date?,
        weightLbs: Double?,
        gender: PetGender,
        color: String?,
        microchipId: String?,
        isSpayedNeutered: Bool,
        vaccinationStatus: VaccinationStatus,
        temperament: String?,
        specialNeeds: String?,
        vetName: String?,
        vetPhone: String?,
        emergencyContactName: String?,
        emergencyContactPhone: String?,
        notes: String?
    ) -> CreatePetRequest {
        // Use simple date format that matches API expectations (YYYY-MM-DD)
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone(identifier: "UTC")

        return CreatePetRequest(
            name: name,
            species: species.rawValue,
            breed: breed?.isEmpty == true ? nil : breed,
            dateOfBirth: dateOfBirth.map { formatter.string(from: $0) },
            weightLbs: weightLbs,
            gender: gender.rawValue,
            color: color?.isEmpty == true ? nil : color,
            microchipId: microchipId?.isEmpty == true ? nil : microchipId,
            isSpayedNeutered: isSpayedNeutered,
            vaccinationStatus: vaccinationStatus.rawValue,
            temperament: temperament?.isEmpty == true ? nil : temperament,
            specialNeeds: specialNeeds?.isEmpty == true ? nil : specialNeeds,
            vetName: vetName?.isEmpty == true ? nil : vetName,
            vetPhone: vetPhone?.isEmpty == true ? nil : vetPhone,
            emergencyContactName: emergencyContactName?.isEmpty == true ? nil : emergencyContactName,
            emergencyContactPhone: emergencyContactPhone?.isEmpty == true ? nil : emergencyContactPhone,
            notes: notes?.isEmpty == true ? nil : notes
        )
    }
}

extension UpdatePetRequest {
    /// Create an update request from form data
    static func from(
        name: String?,
        species: PetSpecies?,
        breed: String?,
        dateOfBirth: Date?,
        weightLbs: Double?,
        gender: PetGender?,
        color: String?,
        microchipId: String?,
        isSpayedNeutered: Bool?,
        vaccinationStatus: VaccinationStatus?,
        temperament: String?,
        specialNeeds: String?,
        vetName: String?,
        vetPhone: String?,
        emergencyContactName: String?,
        emergencyContactPhone: String?,
        notes: String?
    ) -> UpdatePetRequest {
        // Use simple date format that matches API expectations (YYYY-MM-DD)
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone(identifier: "UTC")

        return UpdatePetRequest(
            name: name,
            species: species?.rawValue,
            breed: breed,
            dateOfBirth: dateOfBirth.map { formatter.string(from: $0) },
            weightLbs: weightLbs,
            gender: gender?.rawValue,
            color: color,
            microchipId: microchipId,
            isSpayedNeutered: isSpayedNeutered,
            vaccinationStatus: vaccinationStatus?.rawValue,
            temperament: temperament,
            specialNeeds: specialNeeds,
            vetName: vetName,
            vetPhone: vetPhone,
            emergencyContactName: emergencyContactName,
            emergencyContactPhone: emergencyContactPhone,
            notes: notes
        )
    }
}
