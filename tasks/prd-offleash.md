# PRD: OFFLEASH - White-Label Dog Walking Booking Platform

## Introduction

OFFLEASH transforms the existing single-tenant Dog Walker Booking API into a white-label, multi-tenant SaaS platform that pet service businesses can purchase and customize with their own branding. The platform enables dog walking businesses (1-50 walkers) to manage bookings, customers, and payments under their own brand.

**Brand Inspiration:** OFFLEASH - wordplay on Off-White (iconic streetwear brand), captures the aspiration of dog walking - freedom, trust, the ideal walk. Short, memorable, brandable.

**Current State:** Rust backend API with travel-aware scheduling, Square payments, JWT auth. No frontend. Single-tenant.

**Target State:** Complete multi-tenant platform with Admin Dashboard, Customer Booking UI, and Native iOS app - all white-labelable per tenant.

## Goals

- Enable multiple pet service businesses to use the platform under their own branding
- Provide complete tenant isolation with separate databases per tenant
- Deliver a full product suite: API + Admin Dashboard + Customer Web UI + iOS App
- Support custom services and pricing per tenant
- Establish a sustainable SaaS billing model
- Launch MVP that can onboard paying customers

## User Personas

1. **Platform Admin (You):** Manages the overall SaaS, onboards tenants, handles billing
2. **Tenant Admin:** Business owner who configures their branded instance, manages walkers
3. **Walker:** Service provider who manages availability and completes bookings
4. **Customer:** End-user who books dog walking services via web or iOS app

---

## User Stories

### Phase 1: Multi-Tenant Foundation

#### US-001: Create tenant/organization data model
**Description:** As a platform admin, I need a database structure that supports multiple tenants so each business has isolated data.

**Acceptance Criteria:**
- [ ] Create `organizations` table with: id, name, slug (unique), subdomain, custom_domain, created_at, updated_at
- [ ] Create `organization_settings` table with: branding config (JSONB), feature flags, subscription_tier
- [ ] Add `organization_id` foreign key to: users, services, locations, bookings, payments, blocks, working_hours
- [ ] Create migration files for all schema changes
- [ ] All existing tables have NOT NULL constraint on organization_id (no orphan data)
- [ ] Typecheck passes

#### US-002: Implement tenant-aware database connections
**Description:** As a platform admin, I need each tenant to have their own database so data is fully isolated.

**Acceptance Criteria:**
- [ ] Create `tenant_databases` table in master DB: org_id, connection_string (encrypted), status
- [ ] Implement connection pool manager that routes requests to correct tenant DB
- [ ] Add tenant database provisioning logic (create new PostgreSQL database)
- [ ] Add tenant context extraction from JWT claims
- [ ] All repository methods receive tenant context and use correct connection
- [ ] Typecheck passes

#### US-003: Update JWT auth to include tenant context
**Description:** As a developer, I need JWTs to include organization_id so API requests route to the correct tenant.

**Acceptance Criteria:**
- [ ] JWT claims include `org_id` field
- [ ] Login response includes organization info
- [ ] Auth extractor validates user belongs to claimed organization
- [ ] Reject requests where user's org_id doesn't match JWT org_id
- [ ] Typecheck passes

#### US-004: Create tenant onboarding API
**Description:** As a platform admin, I need API endpoints to create and configure new tenants.

**Acceptance Criteria:**
- [ ] `POST /admin/tenants` - Create new tenant (provisions database, creates admin user)
- [ ] `GET /admin/tenants` - List all tenants (platform admin only)
- [ ] `GET /admin/tenants/:id` - Get tenant details
- [ ] `PATCH /admin/tenants/:id` - Update tenant settings
- [ ] `DELETE /admin/tenants/:id` - Soft-delete tenant (mark inactive, don't destroy data)
- [ ] Platform admin auth separate from tenant auth
- [ ] Typecheck passes

---

### Phase 2: Branding & Customization

#### US-005: Implement branding configuration storage
**Description:** As a tenant admin, I need to store my business branding so customers see my brand, not the platform's.

**Acceptance Criteria:**
- [ ] Branding config stored in `organization_settings.branding` JSONB column
- [ ] Schema supports: company_name, logo_url, favicon_url, primary_color, secondary_color, accent_color, support_email, support_phone
- [ ] Branding API endpoint: `GET /api/branding` returns tenant's branding (public, no auth)
- [ ] Branding determined by subdomain or custom domain in request
- [ ] Typecheck passes

#### US-006: Create branding management API
**Description:** As a tenant admin, I need to update my branding through the API.

**Acceptance Criteria:**
- [ ] `GET /admin/branding` - Get current branding config
- [ ] `PUT /admin/branding` - Update branding config
- [ ] `POST /admin/branding/logo` - Upload logo (returns CDN URL)
- [ ] Validate color formats (hex codes)
- [ ] Validate image uploads (size limits, file types)
- [ ] Typecheck passes

#### US-007: Implement custom services per tenant
**Description:** As a tenant admin, I need to define my own services and pricing so I can offer what my business provides.

**Acceptance Criteria:**
- [ ] Services are scoped to organization_id
- [ ] Tenant can CRUD their own services
- [ ] Services not visible across tenants
- [ ] Default services can be cloned from templates on tenant creation
- [ ] `GET /services` only returns current tenant's services
- [ ] Typecheck passes

#### US-008: Implement subdomain and custom domain routing
**Description:** As a tenant admin, I want my customers to access the platform via my own subdomain or domain.

**Acceptance Criteria:**
- [ ] Subdomain routing: `{slug}.offleash.app` routes to correct tenant
- [ ] Custom domain support: `bookings.acmepets.com` routes to tenant
- [ ] Tenant lookup by host header
- [ ] SSL certificate provisioning for custom domains (via Let's Encrypt or Cloudflare)
- [ ] Fallback to default branding if tenant not found
- [ ] Typecheck passes

---

### Phase 3: Admin Dashboard (Web)

#### US-009: Set up Admin Dashboard frontend project
**Description:** As a developer, I need a frontend project structure for the admin dashboard.

**Acceptance Criteria:**
- [ ] Create `/apps/admin-dashboard` with React + TypeScript + Vite
- [ ] Configure Tailwind CSS for styling
- [ ] Set up React Router for navigation
- [ ] Configure API client with auth token handling
- [ ] Set up environment variables for API URL
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-010: Implement admin authentication flow
**Description:** As a tenant admin, I need to log in to manage my business.

**Acceptance Criteria:**
- [ ] Login page with email/password
- [ ] JWT stored securely (httpOnly cookie or secure storage)
- [ ] Auto-redirect to dashboard on successful login
- [ ] Redirect to login on 401 responses
- [ ] Logout functionality clears token
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-011: Build walker management UI
**Description:** As a tenant admin, I need to manage my team of walkers.

**Acceptance Criteria:**
- [ ] List all walkers with status indicators
- [ ] Add new walker (invite via email)
- [ ] Edit walker profile (name, phone, photo)
- [ ] Activate/deactivate walker
- [ ] View walker's upcoming bookings
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-012: Build service management UI
**Description:** As a tenant admin, I need to configure the services my business offers.

**Acceptance Criteria:**
- [ ] List all services with name, duration, price
- [ ] Create new service with all fields
- [ ] Edit existing service
- [ ] Toggle service active/inactive
- [ ] Reorder services (display order)
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-013: Build booking management UI
**Description:** As a tenant admin, I need to view and manage all bookings.

**Acceptance Criteria:**
- [ ] Calendar view showing all bookings by day/week
- [ ] List view with filters (status, walker, date range)
- [ ] View booking details (customer, walker, service, location, status)
- [ ] Manually create booking on behalf of customer
- [ ] Update booking status (confirm, cancel, mark complete)
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-014: Build customer management UI
**Description:** As a tenant admin, I need to view and manage my customers.

**Acceptance Criteria:**
- [ ] List all customers with search
- [ ] View customer profile (name, email, phone, locations)
- [ ] View customer's booking history
- [ ] Add notes to customer profile
- [ ] Manually add new customer
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-015: Build branding settings UI
**Description:** As a tenant admin, I need to customize my business branding.

**Acceptance Criteria:**
- [ ] Upload logo with preview
- [ ] Color pickers for primary/secondary/accent colors
- [ ] Live preview of branding changes
- [ ] Company name and contact info fields
- [ ] Save and apply branding
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-016: Build dashboard analytics
**Description:** As a tenant admin, I need to see key business metrics at a glance.

**Acceptance Criteria:**
- [ ] Today's bookings count and list
- [ ] This week's revenue
- [ ] Upcoming bookings requiring confirmation
- [ ] Walker availability overview
- [ ] Recent customer signups
- [ ] Typecheck passes
- [ ] Verify in browser

---

### Phase 4: Customer Booking UI (Web)

#### US-017: Set up Customer Booking frontend project
**Description:** As a developer, I need a frontend project for customers to book services.

**Acceptance Criteria:**
- [ ] Create `/apps/customer-web` with React + TypeScript + Vite
- [ ] Configure Tailwind CSS
- [ ] Dynamic theming system that applies tenant branding
- [ ] API client configured for tenant-specific requests
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-018: Build customer registration and login
**Description:** As a customer, I need to create an account and log in to book services.

**Acceptance Criteria:**
- [ ] Registration form (name, email, password, phone)
- [ ] Login form
- [ ] Password reset flow via email
- [ ] JWT token management
- [ ] Branded with tenant's colors/logo
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-019: Build service selection UI
**Description:** As a customer, I need to browse and select a service to book.

**Acceptance Criteria:**
- [ ] Display all active services with name, description, duration, price
- [ ] Service cards styled with tenant branding
- [ ] Select service to proceed to booking
- [ ] Show service details on click/tap
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-020: Build location management for customers
**Description:** As a customer, I need to add and manage my locations (where walks happen).

**Acceptance Criteria:**
- [ ] Add new location with address autocomplete (Google Places)
- [ ] List saved locations
- [ ] Set default location
- [ ] Edit/delete locations
- [ ] Add special instructions (gate code, dog info)
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-021: Build availability and slot selection UI
**Description:** As a customer, I need to see available time slots and pick one.

**Acceptance Criteria:**
- [ ] Date picker showing next 14 days
- [ ] Display available slots for selected date
- [ ] Slots account for travel time (core feature)
- [ ] Visual indication of slot duration
- [ ] Select slot to proceed to confirmation
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-022: Build booking confirmation and payment
**Description:** As a customer, I need to confirm my booking and pay.

**Acceptance Criteria:**
- [ ] Booking summary (service, date, time, location, price)
- [ ] Square payment form integration
- [ ] Support card payments
- [ ] Booking created on successful payment
- [ ] Confirmation screen with booking details
- [ ] Email confirmation sent
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-023: Build customer booking history
**Description:** As a customer, I need to view my past and upcoming bookings.

**Acceptance Criteria:**
- [ ] List upcoming bookings with details
- [ ] List past bookings with status
- [ ] Cancel upcoming booking (if cancellation policy allows)
- [ ] Rebook from past booking
- [ ] Typecheck passes
- [ ] Verify in browser

---

### Phase 5: Native iOS App

#### US-024: Set up iOS project with Swift/SwiftUI
**Description:** As a developer, I need an iOS project structure for the customer app.

**Acceptance Criteria:**
- [ ] Create `/apps/ios` Xcode project with SwiftUI
- [ ] Minimum iOS version: 16.0
- [ ] Configure API client with URLSession
- [ ] Set up environment configurations (dev, staging, prod)
- [ ] Configure app icons and launch screen
- [ ] App builds successfully

#### US-025: Implement dynamic branding in iOS
**Description:** As a customer, I want the app to show my dog walking business's branding.

**Acceptance Criteria:**
- [ ] Fetch branding config on app launch
- [ ] Apply colors to UI components dynamically
- [ ] Display tenant logo in header/navigation
- [ ] Cache branding for offline use
- [ ] App builds successfully
- [ ] Verify on simulator/device

#### US-026: Build iOS authentication screens
**Description:** As a customer, I need to sign up and log in via the iOS app.

**Acceptance Criteria:**
- [ ] Login screen with email/password
- [ ] Registration screen with required fields
- [ ] Secure token storage in Keychain
- [ ] Biometric authentication option (Face ID/Touch ID)
- [ ] Password reset via deep link
- [ ] App builds successfully
- [ ] Verify on simulator/device

#### US-027: Build iOS service browsing
**Description:** As a customer, I need to browse available services in the app.

**Acceptance Criteria:**
- [ ] List services with SwiftUI List/LazyVStack
- [ ] Service detail view
- [ ] Pull-to-refresh
- [ ] Loading and error states
- [ ] App builds successfully
- [ ] Verify on simulator/device

#### US-028: Build iOS booking flow
**Description:** As a customer, I need to complete the full booking flow in the app.

**Acceptance Criteria:**
- [ ] Location selection/addition
- [ ] Date picker with available dates
- [ ] Time slot selection showing availability
- [ ] Booking summary screen
- [ ] Square payment SDK integration
- [ ] Booking confirmation with local notification
- [ ] App builds successfully
- [ ] Verify on simulator/device

#### US-029: Build iOS booking management
**Description:** As a customer, I need to view and manage my bookings in the app.

**Acceptance Criteria:**
- [ ] Upcoming bookings tab
- [ ] Past bookings tab
- [ ] Booking detail view
- [ ] Cancel booking with confirmation
- [ ] Add booking to calendar (EventKit)
- [ ] Push notifications for booking reminders
- [ ] App builds successfully
- [ ] Verify on simulator/device

#### US-030: Implement push notifications
**Description:** As a customer, I want to receive push notifications about my bookings.

**Acceptance Criteria:**
- [ ] Register for push notifications (APNs)
- [ ] Store device token in backend
- [ ] Backend sends notifications for: booking confirmed, reminder (1hr before), walker on the way
- [ ] Handle notification taps (deep link to booking)
- [ ] App builds successfully
- [ ] Verify on simulator/device

---

### Phase 6: Platform Admin & Billing

#### US-031: Create platform admin dashboard
**Description:** As a platform admin, I need a separate dashboard to manage all tenants.

**Acceptance Criteria:**
- [ ] Separate auth system for platform admins
- [ ] List all tenants with status, plan, created date
- [ ] View tenant details and usage metrics
- [ ] Impersonate tenant admin (for support)
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-032: Implement Stripe billing integration
**Description:** As a platform admin, I need to charge tenants for using the platform.

**Acceptance Criteria:**
- [ ] Stripe account connection
- [ ] Create Stripe customer on tenant creation
- [ ] Subscription plans: Starter ($49/mo), Growth ($99/mo), Scale ($199/mo)
- [ ] Webhook handling for subscription events
- [ ] Dunning management for failed payments
- [ ] Typecheck passes

#### US-033: Build tenant billing portal
**Description:** As a tenant admin, I need to manage my subscription and billing.

**Acceptance Criteria:**
- [ ] View current plan and usage
- [ ] Upgrade/downgrade plan
- [ ] Update payment method (Stripe Billing Portal)
- [ ] View invoice history
- [ ] Download invoices as PDF
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-034: Implement usage-based billing tracking
**Description:** As a platform admin, I need to track tenant usage for potential usage-based pricing.

**Acceptance Criteria:**
- [ ] Track monthly bookings per tenant
- [ ] Track monthly active customers per tenant
- [ ] Track API calls per tenant
- [ ] Usage visible in platform admin dashboard
- [ ] Alerts when approaching plan limits
- [ ] Typecheck passes

---

## Functional Requirements

### Multi-Tenancy
- FR-1: Every database table must have `organization_id` column with foreign key constraint
- FR-2: Each tenant must have isolated PostgreSQL database
- FR-3: All API queries must filter by tenant context (no cross-tenant data access)
- FR-4: Tenant context must be extracted from JWT `org_id` claim
- FR-5: Subdomains must route to correct tenant (e.g., `acme.offleash.app`)
- FR-6: Custom domains must be supported with SSL certificates

### Branding
- FR-7: Tenant branding must include: company name, logo, colors (primary, secondary, accent), contact info
- FR-8: Branding must be served via public API endpoint (no auth required)
- FR-9: All customer-facing UIs must dynamically apply tenant branding
- FR-10: iOS app must cache branding for offline display

### Booking Flow
- FR-11: Travel time between appointments must be calculated and enforced (existing feature)
- FR-12: Double-booking prevention must work across all frontends
- FR-13: Booking requires successful payment before confirmation
- FR-14: Cancellation policy configurable per tenant

### Payments
- FR-15: Square payment integration for customer payments
- FR-16: Stripe integration for tenant subscription billing
- FR-17: Each tenant can connect their own Square account for payouts
- FR-18: Platform takes no cut of customer payments (tenants pay subscription only)

### iOS App
- FR-19: Minimum iOS version 16.0
- FR-20: Support Face ID/Touch ID authentication
- FR-21: Push notifications for booking events
- FR-22: Offline mode for viewing existing bookings

---

## Non-Goals (Out of Scope for MVP)

- Android app (iOS only for MVP)
- Walker-facing mobile app (walkers use web dashboard)
- Real-time walker tracking/GPS
- In-app messaging between customers and walkers
- Automated recurring bookings
- Multi-language/i18n support
- Pet profiles and health records
- Integration with veterinary systems
- Marketplace for customers to find businesses (each tenant is independent)
- Revenue sharing or transaction fees (subscription-only model)
- White-label mobile app in App Store per tenant (single app, dynamic branding)

---

## Technical Considerations

### Architecture
- **Backend:** Keep existing Rust/Axum stack, extend for multi-tenancy
- **Admin Dashboard:** React + TypeScript + Vite + Tailwind
- **Customer Web:** React + TypeScript + Vite + Tailwind (separate app, shared component library)
- **iOS App:** Swift 5.9+ with SwiftUI, minimum iOS 16
- **Database:** PostgreSQL 16, one master DB for tenant registry, separate DB per tenant

### Database Strategy: Separate Databases per Tenant
**Rationale:** Stronger data isolation, easier compliance (GDPR data deletion), independent backups/restores, no risk of query bugs leaking data across tenants.

**Implementation:**
- Master database stores: tenants, tenant_databases, platform_admins, billing
- Tenant databases store: all business data (users, bookings, services, etc.)
- Connection pool manager routes queries to correct tenant DB based on JWT org_id

### Deployment
- Continue using existing Fly.io setup
- Tenant databases provisioned as Fly Postgres instances
- Consider managed PostgreSQL (Neon, Supabase) for easier scaling

### Existing Assets to Leverage
- Travel-time-aware availability engine (core differentiator)
- Square payment integration
- JWT authentication system
- Repository pattern (extend for multi-tenancy)
- SQLx compile-time query checking

---

## Billing Model

### Subscription Tiers

| Plan | Price | Walkers | Bookings/mo | Features |
|------|-------|---------|-------------|----------|
| **Starter** | $49/mo | Up to 3 | Unlimited | Core booking, basic branding |
| **Growth** | $99/mo | Up to 10 | Unlimited | Custom domain, priority support |
| **Scale** | $199/mo | Up to 50 | Unlimited | API access, advanced analytics, white-glove onboarding |

### Why Subscription Over Transaction Fees
- Predictable revenue for you
- Predictable costs for tenants
- No incentive for tenants to route bookings outside the system
- Simpler accounting

### Future Considerations
- Annual discount (2 months free)
- Usage-based add-ons (SMS notifications, additional storage)

---

## Success Metrics

- Onboard first 5 paying tenants within 30 days of launch
- 90% of tenant admins complete branding setup within first session
- Customer booking completion rate > 70% (start to payment)
- iOS app rating > 4.5 stars
- Tenant churn < 5% monthly
- Platform uptime > 99.9%

---

## Open Questions

1. **Square Connect:** Should each tenant connect their own Square account, or does platform handle all payments and pay out to tenants?
2. **App Store Strategy:** Single app with dynamic branding, or white-label builds per tenant? (Recommendation: Single app for MVP)
3. **Email Provider:** What transactional email service? (SendGrid, Postmark, AWS SES)
4. **SMS Notifications:** Include in MVP or add later?
5. **Data Retention:** How long to keep booking history for inactive tenants?
6. **Onboarding Flow:** Self-service signup or sales-assisted only?

---

## Implementation Order

Recommended sequence for parallel workstreams:

```
Week 1-2:  US-001, US-002, US-003 (Multi-tenant foundation)
Week 2-3:  US-004, US-005, US-006, US-007, US-008 (Tenant management + branding)
Week 3-5:  US-009 through US-016 (Admin Dashboard) [parallel with below]
Week 3-5:  US-017 through US-023 (Customer Web UI) [parallel]
Week 4-6:  US-024 through US-030 (iOS App) [parallel, can start after API stable]
Week 6-7:  US-031 through US-034 (Platform admin + billing)
Week 7-8:  Integration testing, bug fixes, polish
```

---

## Appendix: Current Codebase Summary

- **Language:** Rust 1.75+
- **Framework:** Axum 0.7
- **Database:** PostgreSQL 16, SQLx
- **Auth:** JWT (24hr expiry), Argon2 password hashing
- **Payments:** Square SDK
- **External APIs:** Google Maps Distance Matrix
- **Structure:** 5 workspace crates (api, db, domain, shared, integrations)
- **Existing Tables:** users, services, locations, bookings, payments, blocks, working_hours, travel_cache
- **Tests:** 32 passing tests (domain: 9, shared: 23)
