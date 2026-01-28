# PRD: MVP Completion & Stabilization

## Introduction

This PRD outlines the final steps required to bring the OFFLEASH platform to a stable MVP state. It bridges the gaps identified in current development, specifically focusing on completing the payment system, finalizing iOS app features, and resolving hardcoded TODOs.

## Goals

- **Complete Payment Infrastructure:** Finish the core Stripe/Square integrations defined in `prd-multi-tenant-payment-system.md`.
- **Stabilize iOS App:** Replace hardcoded configurations with dynamic values and finish "mocked" views.
- **Backend Completeness:** Ensure all frontend views have corresponding functional API endpoints.
- **Security Readiness:** Prepare for production with certificate pinning and secure configuration.

## 1. Payment System Completion

*Reference: `tasks/prd-multi-tenant-payment-system.md`*

Prioritize the following User Stories to ensure money can change hands:

### 1.1 Core Tenant Onboarding
- **[Backend]** Verify Platform Fee Tiers are seeded/configurable.
- **[Backend]** Complete `POST /api/payments/stripe/connect` and callback handling.
- **[Backend]** Complete `POST /api/payments/square/connect` and callback handling.

### 1.2 Customer Checkout
- **[iOS/Web]** Implement "Add Payment Method" screens for Customers (Stripe Elements / Square SDK).
- **[Backend]** Implement `POST /api/payments/charge` to handle the actual transaction split.

### 1.3 Tenant Settings
- **[iOS]** Build "Payment Settings" view for Walkers/Tenants to connect their Stripe/Square accounts.

## 2. iOS App Refinements

The iOS app currently contains several placeholder implementations that need to be realigned with the backend.

### 2.1 Dynamic Organization Configuration
*Status: Hardcoded "demo" slug in `LoginView.swift`, `RegisterView.swift`.*

- **Requirement:** App should dynamically determine the tenant/organization context, possibly via subdomain detection (if white-labeled) or user selection.
- **Task:** Remove `let orgSlug = ProcessInfo.processInfo.environment["ORG_SLUG"] ?? "demo"` and replace with a configuration service or user input flow.

### 2.2 Walker Working Hours
*Status: UI exists (`WorkingHoursView.swift`) but logic is mocked.*

- **Requirement:** Walkers must be able to persist their schedule to the database.
- **Task:**
    - [Backend] Create `GET /api/availability/schedule` and `PUT /api/availability/schedule`.
    - [iOS] Wire `WorkingHoursViewModel` to call these endpoints instead of using local defaults.

### 2.3 Walker Calendar Management
*Status: Drag-to-create exists but breaks cannot be edited/deleted (`WalkerCalendarView.swift`).*

- **Requirement:** Walkers need full CRUD on their time-off blocks.
- **Task:**
    - [Backend] Ensure `DELETE /api/blocks/:id` and `PUT /api/blocks/:id` exist.
    - [iOS] Add "Edit Break" sheet when tapping a mock block.
    - [iOS] Implement delete functionality.

### 2.4 Profile Management
*Status: `EditProfileView.swift` missing phone number logic.*

- **Requirement:** Users must be able to update all contact info.
- **Task:**
    - [iOS] Load phone number in `loadCurrentValues` in `EditProfileView`.
    - [iOS] Handle phone number updates in the save request.

### 2.5 Security: Certificate Pinning
*Status: `APIClient.swift` has placeholder hashes.*

- **Requirement:** Prevent MITM attacks in production.
- **Task:**
    - Generate real public key hashes for `api.offleash.world`.
    - Update `CertificatePinningDelegate` with real values.
    - Test failure cases.

## 3. Customer Web Refinements

### 3.1 Booking Details
*Status: `bookings/[id]/+page.server.ts` has `TODO: Fetch walker details`.*

- **Requirement:** Customers need to see who is walking their dog.
- **Task:**
    - [Backend] Ensure booking response includes `walker_id`.
    - [Web] Fetch walker profile (name, photo) using `walker_id` and display on the Booking Details page.

## 4. Execution Plan

1.  **Backend & DB**: Finalize Payment & Schedule models/routes.
2.  **iOS Integration**: Update APIClient and Views to use new routes.
3.  **End-to-End Testing**: Verify a full flow: Register -> Connect Payment -> Set Schedule -> Book Walk -> Charge.

