//
//  PetsListView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI

struct PetsListView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @State private var pets: [Pet] = []
    @State private var isLoading = true
    @State private var showError = false
    @State private var errorMessage = ""
    @State private var showAddPet = false
    @State private var petToEdit: Pet?
    @State private var petToDelete: Pet?

    var body: some View {
        Group {
            if isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if pets.isEmpty {
                emptyState
            } else {
                petsList
            }
        }
        .navigationTitle("My Pets")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    showAddPet = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .onAppear {
            loadPets()
            analyticsService.trackScreenView(screenName: "pets_list")
        }
        .sheet(isPresented: $showAddPet) {
            NavigationStack {
                AddEditPetView(mode: .add, onSave: {
                    loadPets()
                })
                .environmentObject(themeManager)
            }
        }
        .sheet(item: $petToEdit) { pet in
            NavigationStack {
                AddEditPetView(mode: .edit(pet), onSave: {
                    loadPets()
                })
                .environmentObject(themeManager)
            }
        }
        .alert("Error", isPresented: $showError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(errorMessage)
        }
        .alert("Delete Pet", isPresented: .init(
            get: { petToDelete != nil },
            set: { if !$0 { petToDelete = nil } }
        )) {
            Button("Cancel", role: .cancel) {
                petToDelete = nil
            }
            Button("Delete", role: .destructive) {
                if let pet = petToDelete {
                    deletePet(pet)
                }
            }
        } message: {
            if let pet = petToDelete {
                Text("Are you sure you want to remove \(pet.name)? This action cannot be undone.")
            }
        }
    }

    // MARK: - Empty State

    private var emptyState: some View {
        VStack(spacing: 16) {
            Image(systemName: "pawprint.circle")
                .font(.system(size: 60))
                .foregroundColor(.secondary)

            Text("No Pets Yet")
                .font(.title2)
                .fontWeight(.semibold)

            Text("Add your furry friends to start booking walks.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal)

            Button {
                showAddPet = true
            } label: {
                Label("Add Pet", systemImage: "plus")
                    .fontWeight(.semibold)
                    .frame(maxWidth: .infinity)
                    .padding()
                    .background(themeManager.primaryColor)
                    .foregroundColor(.white)
                    .cornerRadius(12)
            }
            .padding(.horizontal, 40)
            .padding(.top, 8)
        }
        .padding()
    }

    // MARK: - Pets List

    private var petsList: some View {
        List {
            ForEach(pets) { pet in
                PetRow(pet: pet, themeManager: themeManager)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        petToEdit = pet
                    }
                    .swipeActions(edge: .trailing) {
                        Button(role: .destructive) {
                            petToDelete = pet
                        } label: {
                            Label("Delete", systemImage: "trash")
                        }

                        Button {
                            petToEdit = pet
                        } label: {
                            Label("Edit", systemImage: "pencil")
                        }
                        .tint(.blue)
                    }
            }

            Section {
                Button {
                    showAddPet = true
                } label: {
                    Label("Add Pet", systemImage: "plus.circle")
                        .foregroundColor(themeManager.primaryColor)
                }
            }
        }
    }

    // MARK: - Data Loading

    private func loadPets() {
        isLoading = true

        Task {
            do {
                let fetchedPets = try await PetService.shared.getPets()
                await MainActor.run {
                    pets = fetchedPets
                    isLoading = false
                }
            } catch let error as APIError {
                await MainActor.run {
                    isLoading = false
                    errorMessage = error.errorDescription ?? "Failed to load pets"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    isLoading = false
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }

    private func deletePet(_ pet: Pet) {
        Task {
            do {
                try await PetService.shared.deletePet(id: pet.id)
                await MainActor.run {
                    petToDelete = nil
                    loadPets()
                    analyticsService.trackEvent(name: "pet_deleted", params: ["pet_id": pet.id])
                }
            } catch let error as APIError {
                await MainActor.run {
                    petToDelete = nil
                    errorMessage = error.errorDescription ?? "Failed to delete pet"
                    showError = true
                }
            } catch {
                await MainActor.run {
                    petToDelete = nil
                    errorMessage = "An unexpected error occurred"
                    showError = true
                }
            }
        }
    }
}

// MARK: - Pet Row

struct PetRow: View {
    let pet: Pet
    let themeManager: ThemeManager

    var body: some View {
        HStack(spacing: 12) {
            // Pet Avatar
            ZStack {
                Circle()
                    .fill(themeManager.primaryColor.opacity(0.1))
                    .frame(width: 50, height: 50)

                Image(systemName: pet.speciesIcon)
                    .font(.system(size: 24))
                    .foregroundColor(themeManager.primaryColor)
            }

            // Pet Details
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(pet.name)
                        .font(.headline)

                    Image(systemName: pet.gender.icon)
                        .font(.caption)
                        .foregroundColor(pet.gender == .male ? .blue : pet.gender == .female ? .pink : .gray)
                }

                Text(pet.subtitle)
                    .font(.subheadline)
                    .foregroundColor(.secondary)

                // Vaccination badge
                HStack(spacing: 4) {
                    Circle()
                        .fill(vaccinationColor)
                        .frame(width: 8, height: 8)
                    Text(pet.vaccinationStatus.displayName)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            Image(systemName: "chevron.right")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 4)
    }

    private var vaccinationColor: Color {
        switch pet.vaccinationStatus {
        case .upToDate: return .green
        case .partial: return .orange
        case .expired: return .red
        case .unknown: return .gray
        }
    }
}

#Preview {
    NavigationStack {
        PetsListView()
    }
    .withThemeManager()
}
