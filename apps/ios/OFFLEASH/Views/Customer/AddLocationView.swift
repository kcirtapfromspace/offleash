//
//  AddLocationView.swift
//  OFFLEASH
//
//  Created by OFFLEASH Team
//

import SwiftUI
import CoreLocation

// MARK: - Create Location Request

struct CreateLocationRequest: Codable {
    let name: String
    let address: String
    let city: String
    let state: String
    let zipCode: String
    let latitude: Double
    let longitude: Double
    let notes: String?
    let isDefault: Bool?
    // Note: No CodingKeys needed - APIClient uses convertToSnakeCase
}

// MARK: - Add Location View

struct AddLocationView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.analyticsService) private var analyticsService
    @Environment(\.dismiss) private var dismiss

    var onLocationAdded: ((Location) -> Void)?

    @State private var name = ""
    @State private var address = ""
    @State private var city = ""
    @State private var state = ""
    @State private var zipCode = ""
    @State private var notes = ""
    @State private var isDefault = false

    @State private var isSubmitting = false
    @State private var showError = false
    @State private var errorMessage = ""

    // Validation
    @State private var nameError: String?
    @State private var addressError: String?
    @State private var cityError: String?
    @State private var stateError: String?
    @State private var zipError: String?

    private let geocoder = CLGeocoder()

    var body: some View {
        NavigationStack {
            Form {
                Section("Location Details") {
                    VStack(alignment: .leading, spacing: 4) {
                        TextField("Location Name (e.g., Home, Work)", text: $name)
                            .textContentType(.name)
                        if let error = nameError {
                            Text(error)
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                    }

                    VStack(alignment: .leading, spacing: 4) {
                        TextField("Street Address", text: $address)
                            .textContentType(.streetAddressLine1)
                        if let error = addressError {
                            Text(error)
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                    }

                    VStack(alignment: .leading, spacing: 4) {
                        TextField("City", text: $city)
                            .textContentType(.addressCity)
                        if let error = cityError {
                            Text(error)
                                .font(.caption)
                                .foregroundColor(.red)
                        }
                    }

                    HStack(spacing: 12) {
                        VStack(alignment: .leading, spacing: 4) {
                            TextField("State", text: $state)
                                .textContentType(.addressState)
                            if let error = stateError {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                        }
                        .frame(maxWidth: .infinity)

                        VStack(alignment: .leading, spacing: 4) {
                            TextField("ZIP", text: $zipCode)
                                .textContentType(.postalCode)
                                .keyboardType(.numberPad)
                            if let error = zipError {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                        }
                        .frame(maxWidth: .infinity)
                    }
                }

                Section("Additional Info") {
                    TextField("Notes (Optional)", text: $notes, axis: .vertical)
                        .lineLimit(3...6)

                    Toggle("Set as default location", isOn: $isDefault)
                }

                Section {
                    Button {
                        saveLocation()
                    } label: {
                        HStack {
                            Spacer()
                            if isSubmitting {
                                ProgressView()
                                    .tint(.white)
                            } else {
                                Text("Save Location")
                                    .fontWeight(.semibold)
                            }
                            Spacer()
                        }
                        .padding(.vertical, 4)
                    }
                    .listRowBackground(isFormValid ? themeManager.primaryColor : Color(.systemGray4))
                    .foregroundColor(.white)
                    .disabled(!isFormValid || isSubmitting)
                }
            }
            .navigationTitle("Add Location")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
            .onAppear {
                analyticsService.trackScreenView(screenName: "add_location")
            }
            .alert("Error", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage)
            }
        }
    }

    private var isFormValid: Bool {
        !name.trimmingCharacters(in: .whitespaces).isEmpty &&
        !address.trimmingCharacters(in: .whitespaces).isEmpty &&
        !city.trimmingCharacters(in: .whitespaces).isEmpty &&
        !state.trimmingCharacters(in: .whitespaces).isEmpty &&
        !zipCode.trimmingCharacters(in: .whitespaces).isEmpty
    }

    private func validateForm() -> Bool {
        var isValid = true

        // Reset errors
        nameError = nil
        addressError = nil
        cityError = nil
        stateError = nil
        zipError = nil

        if name.trimmingCharacters(in: .whitespaces).isEmpty {
            nameError = "Location name is required"
            isValid = false
        }

        if address.trimmingCharacters(in: .whitespaces).isEmpty {
            addressError = "Street address is required"
            isValid = false
        }

        if city.trimmingCharacters(in: .whitespaces).isEmpty {
            cityError = "City is required"
            isValid = false
        }

        if state.trimmingCharacters(in: .whitespaces).isEmpty {
            stateError = "State is required"
            isValid = false
        }

        let trimmedZip = zipCode.trimmingCharacters(in: .whitespaces)
        if trimmedZip.isEmpty {
            zipError = "ZIP code is required"
            isValid = false
        } else if !trimmedZip.allSatisfy({ $0.isNumber }) || trimmedZip.count < 5 {
            zipError = "Please enter a valid ZIP code"
            isValid = false
        }

        return isValid
    }

    private func saveLocation() {
        guard validateForm() else { return }

        isSubmitting = true

        // Capture values before async closure
        let trimmedName = name.trimmingCharacters(in: .whitespaces)
        let trimmedAddress = address.trimmingCharacters(in: .whitespaces)
        let trimmedCity = city.trimmingCharacters(in: .whitespaces)
        let trimmedState = state.trimmingCharacters(in: .whitespaces)
        let trimmedZipCode = zipCode.trimmingCharacters(in: .whitespaces)
        let trimmedNotes = notes.isEmpty ? nil : notes.trimmingCharacters(in: .whitespaces)
        let setAsDefault = isDefault

        // Build full address for geocoding
        let fullAddress = "\(trimmedAddress), \(trimmedCity), \(trimmedState) \(trimmedZipCode)"

        // Geocode the address to get coordinates - validates address is real
        geocoder.geocodeAddressString(fullAddress) { placemarks, error in
            guard let placemark = placemarks?.first,
                  let location = placemark.location else {
                Task { @MainActor in
                    isSubmitting = false
                    if let error = error {
                        errorMessage = "Could not verify address: \(error.localizedDescription)"
                    } else {
                        errorMessage = "Address not found. Please check the address and try again."
                    }
                    showError = true
                }
                return
            }

            let latitude = location.coordinate.latitude
            let longitude = location.coordinate.longitude

            // Create the request with coordinates
            let request = CreateLocationRequest(
                name: trimmedName,
                address: trimmedAddress,
                city: trimmedCity,
                state: trimmedState,
                zipCode: trimmedZipCode,
                latitude: latitude,
                longitude: longitude,
                notes: trimmedNotes,
                isDefault: setAsDefault
            )

            Task {
                do {
                    let savedLocation: Location = try await APIClient.shared.post("/locations", body: request)

                    // Invalidate locations cache
                    await LocationSelectionView.invalidateCache()

                    await MainActor.run {
                        isSubmitting = false
                        onLocationAdded?(savedLocation)
                        dismiss()
                    }
                } catch let error as APIError {
                    await MainActor.run {
                        isSubmitting = false
                        errorMessage = error.errorDescription ?? "Failed to save location"
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
}

#Preview {
    AddLocationView(onLocationAdded: { location in
        print("Added location: \(location.name)")
    })
    .withThemeManager()
}
