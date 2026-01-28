//
//  WalkerProfileView.swift
//  OFFLEASH
//
//  Walker profile configuration view
//

import SwiftUI

struct WalkerProfileView: View {
    @EnvironmentObject private var themeManager: ThemeManager
    @Environment(\.dismiss) private var dismiss
    @StateObject private var viewModel = WalkerProfileViewModel()

    var body: some View {
        Form {
            // Profile Photo
            Section {
                HStack {
                    Spacer()
                    VStack(spacing: 12) {
                        ZStack {
                            Circle()
                                .fill(themeManager.primaryColor.opacity(0.1))
                                .frame(width: 100, height: 100)

                            if let initials = viewModel.initials {
                                Text(initials)
                                    .font(.largeTitle)
                                    .fontWeight(.bold)
                                    .foregroundColor(themeManager.primaryColor)
                            } else {
                                Image(systemName: "person.fill")
                                    .font(.largeTitle)
                                    .foregroundColor(themeManager.primaryColor)
                            }
                        }

                        Button("Change Photo") {
                            // Photo picker would go here
                        }
                        .font(.subheadline)
                        .foregroundColor(themeManager.primaryColor)
                    }
                    Spacer()
                }
                .listRowBackground(Color.clear)
            }

            // Personal Information
            Section("Personal Information") {
                TextField("First Name", text: $viewModel.firstName)
                    .textContentType(.givenName)

                TextField("Last Name", text: $viewModel.lastName)
                    .textContentType(.familyName)

                TextField("Email", text: $viewModel.email)
                    .textContentType(.emailAddress)
                    .keyboardType(.emailAddress)
                    .autocapitalization(.none)
                    .disabled(true)
                    .foregroundColor(.secondary)

                TextField("Phone", text: $viewModel.phone)
                    .textContentType(.telephoneNumber)
                    .keyboardType(.phonePad)
            }

            // Bio
            Section("About You") {
                TextEditor(text: $viewModel.bio)
                    .frame(minHeight: 100)

                Text("Tell customers about yourself, your experience with pets, and what makes you a great walker.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            // Specializations
            Section("Specializations") {
                ForEach(WalkerSpecialization.allCases, id: \.self) { specialization in
                    Toggle(specialization.displayName, isOn: Binding(
                        get: { viewModel.specializations.contains(specialization) },
                        set: { isOn in
                            if isOn {
                                viewModel.specializations.insert(specialization)
                            } else {
                                viewModel.specializations.remove(specialization)
                            }
                        }
                    ))
                }

                Text("Select any special skills or certifications you have.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            // Emergency Contact
            Section("Emergency Contact") {
                TextField("Contact Name", text: $viewModel.emergencyContactName)
                TextField("Contact Phone", text: $viewModel.emergencyContactPhone)
                    .keyboardType(.phonePad)
            }
        }
        .navigationTitle("Profile")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .confirmationAction) {
                Button("Save") {
                    Task {
                        await viewModel.saveProfile()
                        dismiss()
                    }
                }
                .disabled(!viewModel.hasChanges || viewModel.isSaving)
            }
        }
        .task {
            await viewModel.loadProfile()
        }
        .overlay {
            if viewModel.isSaving {
                ProgressView()
                    .scaleEffect(1.5)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(Color.black.opacity(0.2))
            }
        }
    }
}

// MARK: - Specializations

enum WalkerSpecialization: String, CaseIterable {
    case puppies
    case seniors
    case largeBreeds
    case smallBreeds
    case anxiousDogs
    case multipleDogsHandler = "multiple_dogs"
    case petFirstAid = "pet_first_aid"
    case dogTraining = "dog_training"

    var displayName: String {
        switch self {
        case .puppies: return "Puppies"
        case .seniors: return "Senior Dogs"
        case .largeBreeds: return "Large Breeds"
        case .smallBreeds: return "Small Breeds"
        case .anxiousDogs: return "Anxious/Reactive Dogs"
        case .multipleDogsHandler: return "Multiple Dogs at Once"
        case .petFirstAid: return "Pet First Aid Certified"
        case .dogTraining: return "Dog Training Experience"
        }
    }
}

// MARK: - View Model

@MainActor
class WalkerProfileViewModel: ObservableObject {
    @Published var firstName = ""
    @Published var lastName = ""
    @Published var email = ""
    @Published var phone = ""
    @Published var bio = ""
    @Published var specializations: Set<WalkerSpecialization> = []
    @Published var emergencyContactName = ""
    @Published var emergencyContactPhone = ""
    @Published var isSaving = false
    @Published var isLoading = false

    private var originalFirstName = ""
    private var originalLastName = ""
    private var originalPhone = ""
    private var originalBio = ""
    private var originalSpecializations: Set<WalkerSpecialization> = []
    private var originalEmergencyContactName = ""
    private var originalEmergencyContactPhone = ""

    var initials: String? {
        let first = firstName.first.map(String.init) ?? ""
        let last = lastName.first.map(String.init) ?? ""
        let result = first + last
        return result.isEmpty ? nil : result.uppercased()
    }

    var hasChanges: Bool {
        firstName != originalFirstName ||
        lastName != originalLastName ||
        phone != originalPhone ||
        bio != originalBio ||
        specializations != originalSpecializations ||
        emergencyContactName != originalEmergencyContactName ||
        emergencyContactPhone != originalEmergencyContactPhone
    }

    func loadProfile() async {
        isLoading = true
        defer { isLoading = false }

        do {
            // Load user info
            let user: User = try await APIClient.shared.get("/users/me")
            firstName = user.firstName ?? ""
            lastName = user.lastName ?? ""
            email = user.email

            // Load extended profile from walker profile endpoint
            let profile: WalkerProfileResponse = try await APIClient.shared.get("/walker/profile")
            bio = profile.bio ?? ""
            emergencyContactName = profile.emergencyContactName ?? ""
            emergencyContactPhone = profile.emergencyContactPhone ?? ""

            // Parse specializations
            specializations = Set(profile.specializations.compactMap { specResponse in
                WalkerSpecialization.allCases.first { $0.rawValue == specResponse.specialization }
            })

            // Store originals
            originalFirstName = firstName
            originalLastName = lastName
            originalPhone = phone
            originalBio = bio
            originalSpecializations = specializations
            originalEmergencyContactName = emergencyContactName
            originalEmergencyContactPhone = emergencyContactPhone
        } catch {
            print("Error loading profile: \(error)")
        }
    }

    func saveProfile() async {
        isSaving = true
        defer { isSaving = false }

        do {
            // Update user basic info
            let userRequest = WalkerUpdateProfileRequest(
                firstName: firstName,
                lastName: lastName,
                phone: phone.isEmpty ? nil : phone
            )
            let _: User = try await APIClient.shared.put("/users/me", body: userRequest)

            // Update extended walker profile
            let profileRequest = WalkerExtendedProfileRequest(
                bio: bio.isEmpty ? nil : bio,
                emergencyContactName: emergencyContactName.isEmpty ? nil : emergencyContactName,
                emergencyContactPhone: emergencyContactPhone.isEmpty ? nil : emergencyContactPhone,
                emergencyContactRelationship: nil,
                yearsExperience: nil,
                specializations: Array(specializations.map { $0.rawValue })
            )
            let _: WalkerProfileResponse = try await APIClient.shared.put("/walker/profile", body: profileRequest)

            // Update originals
            originalFirstName = firstName
            originalLastName = lastName
            originalPhone = phone
            originalBio = bio
            originalSpecializations = specializations
            originalEmergencyContactName = emergencyContactName
            originalEmergencyContactPhone = emergencyContactPhone
        } catch {
            print("Error saving profile: \(error)")
        }
    }
}

// MARK: - Request/Response Models

struct WalkerUpdateProfileRequest: Codable {
    let firstName: String
    let lastName: String
    let phone: String?

    enum CodingKeys: String, CodingKey {
        case firstName = "first_name"
        case lastName = "last_name"
        case phone
    }
}

struct WalkerExtendedProfileRequest: Codable {
    let bio: String?
    let emergencyContactName: String?
    let emergencyContactPhone: String?
    let emergencyContactRelationship: String?
    let yearsExperience: Int?
    let specializations: [String]?

    enum CodingKeys: String, CodingKey {
        case bio
        case emergencyContactName = "emergency_contact_name"
        case emergencyContactPhone = "emergency_contact_phone"
        case emergencyContactRelationship = "emergency_contact_relationship"
        case yearsExperience = "years_experience"
        case specializations
    }
}

struct WalkerProfileResponse: Codable {
    let id: String
    let userId: String
    let bio: String?
    let profilePhotoUrl: String?
    let emergencyContactName: String?
    let emergencyContactPhone: String?
    let emergencyContactRelationship: String?
    let yearsExperience: Int
    let specializations: [SpecializationResponse]
    let createdAt: String
    let updatedAt: String

    enum CodingKeys: String, CodingKey {
        case id
        case userId = "user_id"
        case bio
        case profilePhotoUrl = "profile_photo_url"
        case emergencyContactName = "emergency_contact_name"
        case emergencyContactPhone = "emergency_contact_phone"
        case emergencyContactRelationship = "emergency_contact_relationship"
        case yearsExperience = "years_experience"
        case specializations
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

struct SpecializationResponse: Codable {
    let specialization: String
    let displayName: String
    let certified: Bool
    let certificationDate: String?
    let certificationExpiry: String?

    enum CodingKeys: String, CodingKey {
        case specialization
        case displayName = "display_name"
        case certified
        case certificationDate = "certification_date"
        case certificationExpiry = "certification_expiry"
    }
}

#Preview {
    NavigationStack {
        WalkerProfileView()
    }
    .withThemeManager()
}
