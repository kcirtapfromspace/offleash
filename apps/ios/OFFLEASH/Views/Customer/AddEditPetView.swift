//
//  AddEditPetView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

// MARK: - Edit Mode

enum PetEditMode: Identifiable {
    case add
    case edit(Pet)

    var id: String {
        switch self {
        case .add: return "add"
        case .edit(let pet): return pet.id
        }
    }

    var title: String {
        switch self {
        case .add: return "Add Pet"
        case .edit: return "Edit Pet"
        }
    }

    var pet: Pet? {
        switch self {
        case .add: return nil
        case .edit(let pet): return pet
        }
    }
}

// MARK: - Add/Edit Pet View

struct AddEditPetView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss

    let mode: PetEditMode
    var onSave: () -> Void

    // MARK: - Form State

    @State private var name = ""
    @State private var species: PetSpecies = .dog
    @State private var breed = ""
    @State private var dateOfBirth: Date?
    @State private var showDatePicker = false
    @State private var weightLbs = ""
    @State private var gender: PetGender = .unknown
    @State private var color = ""
    @State private var microchipId = ""
    @State private var isSpayedNeutered = false
    @State private var vaccinationStatus: VaccinationStatus = .unknown
    @State private var temperament = ""
    @State private var specialNeeds = ""
    @State private var vetName = ""
    @State private var vetPhone = ""
    @State private var emergencyContactName = ""
    @State private var emergencyContactPhone = ""
    @State private var notes = ""

    // MARK: - UI State

    @State private var isSaving = false
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showDeleteConfirmation = false

    // MARK: - Computed Properties

    private var isValid: Bool {
        !name.trimmingCharacters(in: .whitespaces).isEmpty
    }

    private var parsedWeight: Double? {
        Double(weightLbs)
    }

    // MARK: - Body

    var body: some View {
        Form {
            // Basic Info Section
            Section("Basic Information") {
                TextField("Pet Name", text: $name)
                    .textContentType(.name)

                Picker("Species", selection: $species) {
                    ForEach(PetSpecies.allCases, id: \.self) { species in
                        Label(species.displayName, systemImage: species.icon)
                            .tag(species)
                    }
                }

                TextField("Breed", text: $breed)

                Picker("Gender", selection: $gender) {
                    ForEach(PetGender.allCases, id: \.self) { gender in
                        Text(gender.displayName).tag(gender)
                    }
                }
            }

            // Physical Details Section
            Section("Physical Details") {
                // Date of Birth
                HStack {
                    Text("Date of Birth")
                    Spacer()
                    if let dob = dateOfBirth {
                        Text(formatDate(dob))
                            .foregroundColor(.secondary)
                    } else {
                        Text("Not Set")
                            .foregroundColor(.secondary)
                    }
                }
                .contentShape(Rectangle())
                .onTapGesture {
                    showDatePicker = true
                }

                TextField("Weight (lbs)", text: $weightLbs)
                    .keyboardType(.decimalPad)

                TextField("Color", text: $color)
            }

            // Health Section
            Section("Health") {
                Toggle("Spayed/Neutered", isOn: $isSpayedNeutered)

                Picker("Vaccination Status", selection: $vaccinationStatus) {
                    ForEach(VaccinationStatus.allCases, id: \.self) { status in
                        Text(status.displayName).tag(status)
                    }
                }

                TextField("Microchip ID", text: $microchipId)
            }

            // Behavior Section
            Section("Behavior") {
                TextField("Temperament", text: $temperament)

                TextField("Special Needs", text: $specialNeeds, axis: .vertical)
                    .lineLimit(2...4)
            }

            // Vet Information Section
            Section("Veterinarian") {
                TextField("Vet Name", text: $vetName)
                TextField("Vet Phone", text: $vetPhone)
                    .keyboardType(.phonePad)
            }

            // Emergency Contact Section
            Section("Emergency Contact") {
                TextField("Contact Name", text: $emergencyContactName)
                TextField("Contact Phone", text: $emergencyContactPhone)
                    .keyboardType(.phonePad)
            }

            // Notes Section
            Section("Notes") {
                TextField("Additional notes about your pet...", text: $notes, axis: .vertical)
                    .lineLimit(3...6)
            }

            // Delete Button (Edit mode only)
            if case .edit = mode {
                Section {
                    Button(role: .destructive) {
                        showDeleteConfirmation = true
                    } label: {
                        HStack {
                            Spacer()
                            Text("Delete Pet")
                            Spacer()
                        }
                    }
                }
            }
        }
        .navigationTitle(mode.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("Cancel") {
                    dismiss()
                }
            }

            ToolbarItem(placement: .confirmationAction) {
                Button {
                    savePet()
                } label: {
                    if isSaving {
                        ProgressView()
                    } else {
                        Text("Save")
                    }
                }
                .disabled(!isValid || isSaving)
            }
        }
        .onAppear {
            loadExistingPet()
            analyticsService.trackScreenView(screenName: mode.title.lowercased().replacingOccurrences(of: " ", with: "_"))
        }
        .sheet(isPresented: $showDatePicker) {
            datePickerSheet
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .alert("Delete Pet", isPresented: $showDeleteConfirmation) {
            Button("Cancel", role: .cancel) {}
            Button("Delete", role: .destructive) {
                deletePet()
            }
        } message: {
            Text("Are you sure you want to delete this pet? This action cannot be undone.")
        }
    }

    // MARK: - Date Picker Sheet

    private var datePickerSheet: some View {
        NavigationStack {
            VStack {
                DatePicker(
                    "Date of Birth",
                    selection: Binding(
                        get: { dateOfBirth ?? Date() },
                        set: { dateOfBirth = $0 }
                    ),
                    in: ...Date(),
                    displayedComponents: .date
                )
                .datePickerStyle(.graphical)
                .padding()

                Spacer()
            }
            .navigationTitle("Date of Birth")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Clear") {
                        dateOfBirth = nil
                        showDatePicker = false
                    }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        showDatePicker = false
                    }
                }
            }
        }
        .presentationDetents([.medium])
    }

    // MARK: - Load Existing Pet

    private func loadExistingPet() {
        guard let pet = mode.pet else { return }

        name = pet.name
        species = pet.species
        breed = pet.breed ?? ""
        dateOfBirth = pet.dateOfBirth
        weightLbs = pet.weightLbs.map { String($0) } ?? ""
        gender = pet.gender
        color = pet.color ?? ""
        microchipId = pet.microchipId ?? ""
        isSpayedNeutered = pet.isSpayedNeutered
        vaccinationStatus = pet.vaccinationStatus
        temperament = pet.temperament ?? ""
        specialNeeds = pet.specialNeeds ?? ""
        vetName = pet.vetName ?? ""
        vetPhone = pet.vetPhone ?? ""
        emergencyContactName = pet.emergencyContactName ?? ""
        emergencyContactPhone = pet.emergencyContactPhone ?? ""
        notes = pet.notes ?? ""
    }

    // MARK: - Save Pet

    private func savePet() {
        isSaving = true

        Task {
            do {
                switch mode {
                case .add:
                    let request = CreatePetRequest.from(
                        name: name.trimmingCharacters(in: .whitespaces),
                        species: species,
                        breed: breed.isEmpty ? nil : breed,
                        dateOfBirth: dateOfBirth,
                        weightLbs: parsedWeight,
                        gender: gender,
                        color: color.isEmpty ? nil : color,
                        microchipId: microchipId.isEmpty ? nil : microchipId,
                        isSpayedNeutered: isSpayedNeutered,
                        vaccinationStatus: vaccinationStatus,
                        temperament: temperament.isEmpty ? nil : temperament,
                        specialNeeds: specialNeeds.isEmpty ? nil : specialNeeds,
                        vetName: vetName.isEmpty ? nil : vetName,
                        vetPhone: vetPhone.isEmpty ? nil : vetPhone,
                        emergencyContactName: emergencyContactName.isEmpty ? nil : emergencyContactName,
                        emergencyContactPhone: emergencyContactPhone.isEmpty ? nil : emergencyContactPhone,
                        notes: notes.isEmpty ? nil : notes
                    )
                    _ = try await PetService.shared.createPet(request)
                    await MainActor.run {
                        analyticsService.trackEvent(name: "pet_created", params: ["species": species.rawValue])
                    }

                case .edit(let pet):
                    let request = UpdatePetRequest.from(
                        name: name.trimmingCharacters(in: .whitespaces),
                        species: species,
                        breed: breed.isEmpty ? nil : breed,
                        dateOfBirth: dateOfBirth,
                        weightLbs: parsedWeight,
                        gender: gender,
                        color: color.isEmpty ? nil : color,
                        microchipId: microchipId.isEmpty ? nil : microchipId,
                        isSpayedNeutered: isSpayedNeutered,
                        vaccinationStatus: vaccinationStatus,
                        temperament: temperament.isEmpty ? nil : temperament,
                        specialNeeds: specialNeeds.isEmpty ? nil : specialNeeds,
                        vetName: vetName.isEmpty ? nil : vetName,
                        vetPhone: vetPhone.isEmpty ? nil : vetPhone,
                        emergencyContactName: emergencyContactName.isEmpty ? nil : emergencyContactName,
                        emergencyContactPhone: emergencyContactPhone.isEmpty ? nil : emergencyContactPhone,
                        notes: notes.isEmpty ? nil : notes
                    )
                    _ = try await PetService.shared.updatePet(id: pet.id, request: request)
                    await MainActor.run {
                        analyticsService.trackEvent(name: "pet_updated", params: ["pet_id": pet.id])
                    }
                }

                await MainActor.run {
                    isSaving = false
                    onSave()
                    dismiss()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSaving = false
                    errorMessage = error.errorDescription ?? "Failed to save pet"
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

    // MARK: - Delete Pet

    private func deletePet() {
        guard case .edit(let pet) = mode else { return }

        isSaving = true

        Task {
            do {
                try await PetService.shared.deletePet(id: pet.id)
                await MainActor.run {
                    analyticsService.trackEvent(name: "pet_deleted", params: ["pet_id": pet.id])
                    isSaving = false
                    onSave()
                    dismiss()
                }
            } catch let error as APIError {
                await MainActor.run {
                    isSaving = false
                    errorMessage = error.errorDescription ?? "Failed to delete pet"
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

    // MARK: - Helpers

    private func formatDate(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        return formatter.string(from: date)
    }
}

#Preview("Add Pet") {
    NavigationStack {
        AddEditPetView(mode: .add, onSave: {})
    }
    .withThemeManager()
}
