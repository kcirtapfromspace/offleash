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

    private var isAuthMockMode: Bool {
        TestAuthMode.isMock
    }

    // MARK: - Pet CRUD Operations

    /// Fetch all pets for the current user
    func getPets() async throws -> [Pet] {
        if isAuthMockMode {
            return await MockDataStore.shared.getPets()
        }
        let response: PetListResponse = try await APIClient.shared.get("/pets")
        return response.pets
    }

    /// Fetch a single pet by ID
    func getPet(id: String) async throws -> Pet {
        try await APIClient.shared.get("/pets/\(id)")
    }

    /// Create a new pet
    func createPet(_ request: CreatePetRequest) async throws -> Pet {
        if isAuthMockMode {
            return await MockDataStore.shared.createPet(request)
        }
        let created: Pet = try await APIClient.shared.post("/pets", body: request)
        return created
    }

    /// Update an existing pet
    func updatePet(id: String, request: UpdatePetRequest) async throws -> Pet {
        if isAuthMockMode {
            if let updated = await MockDataStore.shared.updatePet(id: id, request: request) {
                return updated
            }
        }
        let updated: Pet = try await APIClient.shared.put("/pets/\(id)", body: request)
        return updated
    }

    /// Delete a pet
    func deletePet(id: String) async throws {
        if isAuthMockMode {
            await MockDataStore.shared.deletePet(id: id)
            return
        }
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

// MARK: - Mock Data Store (UI Tests)

actor MockDataStore {
    static let shared = MockDataStore()

    private var pets: [Pet]
    private var locations: [Location]

    private init() {
        let now = Date()
        pets = [
            Pet(
                id: "mock-pet-1",
                userId: "test-user",
                name: "Test Dog",
                species: .dog,
                breed: "Labrador",
                dateOfBirth: nil,
                weightLbs: 45,
                gender: .unknown,
                color: "Black",
                microchipId: nil,
                isSpayedNeutered: true,
                vaccinationStatus: .upToDate,
                temperament: "Friendly",
                specialNeeds: nil,
                vetName: nil,
                vetPhone: nil,
                emergencyContactName: nil,
                emergencyContactPhone: nil,
                photoUrl: nil,
                notes: nil,
                createdAt: now,
                updatedAt: now
            )
        ]

        locations = [
            Location(
                id: "mock-location-1",
                name: "Home",
                address: "123 Test St",
                city: "San Francisco",
                state: "CA",
                zipCode: "94102",
                latitude: 37.7749,
                longitude: -122.4194,
                notes: nil,
                isDefault: true
            ),
            Location(
                id: "mock-location-2",
                name: "Park",
                address: "456 Sample Ave",
                city: "San Francisco",
                state: "CA",
                zipCode: "94102",
                latitude: 37.7694,
                longitude: -122.4862,
                notes: nil,
                isDefault: false
            )
        ]
    }

    // MARK: - Pets

    func getPets() -> [Pet] {
        pets
    }

    func createPet(_ request: CreatePetRequest) -> Pet {
        let now = Date()
        let pet = Pet(
            id: UUID().uuidString,
            userId: "test-user",
            name: request.name,
            species: PetSpecies(rawValue: request.species) ?? .dog,
            breed: request.breed,
            dateOfBirth: nil,
            weightLbs: request.weightLbs,
            gender: PetGender(rawValue: request.gender) ?? .unknown,
            color: request.color,
            microchipId: request.microchipId,
            isSpayedNeutered: request.isSpayedNeutered,
            vaccinationStatus: VaccinationStatus(rawValue: request.vaccinationStatus) ?? .unknown,
            temperament: request.temperament,
            specialNeeds: request.specialNeeds,
            vetName: request.vetName,
            vetPhone: request.vetPhone,
            emergencyContactName: request.emergencyContactName,
            emergencyContactPhone: request.emergencyContactPhone,
            photoUrl: nil,
            notes: request.notes,
            createdAt: now,
            updatedAt: now
        )
        pets.insert(pet, at: 0)
        return pet
    }

    func updatePet(id: String, request: UpdatePetRequest) -> Pet? {
        guard let index = pets.firstIndex(where: { $0.id == id }) else {
            return nil
        }
        let existing = pets[index]
        let updated = Pet(
            id: existing.id,
            userId: existing.userId,
            name: request.name ?? existing.name,
            species: request.species.flatMap { PetSpecies(rawValue: $0) } ?? existing.species,
            breed: request.breed ?? existing.breed,
            dateOfBirth: existing.dateOfBirth,
            weightLbs: request.weightLbs ?? existing.weightLbs,
            gender: request.gender.flatMap { PetGender(rawValue: $0) } ?? existing.gender,
            color: request.color ?? existing.color,
            microchipId: request.microchipId ?? existing.microchipId,
            isSpayedNeutered: request.isSpayedNeutered ?? existing.isSpayedNeutered,
            vaccinationStatus: request.vaccinationStatus.flatMap { VaccinationStatus(rawValue: $0) } ?? existing.vaccinationStatus,
            temperament: request.temperament ?? existing.temperament,
            specialNeeds: request.specialNeeds ?? existing.specialNeeds,
            vetName: request.vetName ?? existing.vetName,
            vetPhone: request.vetPhone ?? existing.vetPhone,
            emergencyContactName: request.emergencyContactName ?? existing.emergencyContactName,
            emergencyContactPhone: request.emergencyContactPhone ?? existing.emergencyContactPhone,
            photoUrl: existing.photoUrl,
            notes: request.notes ?? existing.notes,
            createdAt: existing.createdAt,
            updatedAt: Date()
        )
        pets[index] = updated
        return updated
    }

    func deletePet(id: String) {
        pets.removeAll { $0.id == id }
    }

    // MARK: - Locations

    func getLocations() -> [Location] {
        locations
    }

    func createLocation(_ request: CreateLocationRequest) -> Location {
        let location = Location(
            id: UUID().uuidString,
            name: request.name,
            address: request.address,
            city: request.city,
            state: request.state,
            zipCode: request.zipCode,
            latitude: request.latitude,
            longitude: request.longitude,
            notes: request.notes,
            isDefault: request.isDefault ?? false
        )
        if location.isDefault {
            locations = locations.map { existing in
                Location(
                    id: existing.id,
                    name: existing.name,
                    address: existing.address,
                    city: existing.city,
                    state: existing.state,
                    zipCode: existing.zipCode,
                    latitude: existing.latitude,
                    longitude: existing.longitude,
                    notes: existing.notes,
                    isDefault: false
                )
            }
        }
        locations.insert(location, at: 0)
        return location
    }

    func setDefaultLocation(id: String) -> Location? {
        guard let index = locations.firstIndex(where: { $0.id == id }) else {
            return nil
        }
        locations = locations.map { existing in
            Location(
                id: existing.id,
                name: existing.name,
                address: existing.address,
                city: existing.city,
                state: existing.state,
                zipCode: existing.zipCode,
                latitude: existing.latitude,
                longitude: existing.longitude,
                notes: existing.notes,
                isDefault: existing.id == id
            )
        }
        return locations[index]
    }

    func deleteLocation(id: String) {
        locations.removeAll { $0.id == id }
    }
}
