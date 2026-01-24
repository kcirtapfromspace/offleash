# PRD: Persona Management System

## Introduction

Implement a consistent persona management system across all OFFLEASH platforms (Web apps, iOS) that allows users to seamlessly switch between their different roles (Customer, Walker, Admin/Owner) within and across business tenants.

## Business Context

**Current Market Strategy**: B2B SaaS for dog walking businesses with existing client bases.

**Customer Acquisition Flow**:
1. Business owner signs up and creates their tenant
2. Business imports/invites their existing customers
3. Customers access the platform through the business's branded subdomain
4. Customers book services from THAT specific business

**Future Enhancement**: Marketplace for customer discovery (not current priority).

## System Hierarchy

```
OFFLEASH (Enclave/SaaS Platform)
    └── User (OAuth identity, payment methods)
            └── Business Tenant (Organization)
                    └── Persona (role within that tenant)
                            ├── Customer - books services
                            ├── Walker - provides services
                            └── Admin/Owner - manages business
```

### Key Principles

1. **One User, Many Personas**: A single authenticated user can have multiple personas across multiple tenants
2. **Persona Examples**:
   - Own a dog walking business (Owner of "Happy Paws")
   - Work as a walker for another business (Walker at "City Dogs")
   - Book services for their own pets (Customer at "Pet Paradise")
3. **Context Isolation**: Each persona operates within its tenant context
4. **Seamless Switching**: Users can switch personas without re-authenticating

## Goals

- Provide consistent persona switching UI across web and iOS
- Route users to appropriate interface based on active persona
- Allow customers to browse/select business tenants (marketplace)
- Enable users to manage all their personas from a unified settings area
- Ensure backend APIs properly scope data to active persona/tenant

## User Stories

### US-001: View all my personas
**Description:** As a user, I want to see all my personas (roles across all businesses) so I can understand my relationships.

**Acceptance Criteria:**
- [ ] Settings page shows list of all memberships grouped by tenant
- [ ] Each membership shows: business name, role, status
- [ ] Available in both customer-web and admin-dashboard
- [ ] Typecheck passes

### US-002: Switch persona within customer-web
**Description:** As a user with customer personas in multiple businesses, I want to switch between them to book services from different providers.

**Acceptance Criteria:**
- [ ] Organization switcher shows only customer-type memberships
- [ ] Switching updates the active tenant context
- [ ] Services page shows services from the selected business
- [ ] Typecheck passes
- [ ] Verify in browser

### US-003: Switch to walker/admin persona
**Description:** As a user who is also a walker or admin, I want to switch to that persona and be taken to the appropriate dashboard.

**Acceptance Criteria:**
- [ ] Organization switcher shows all memberships with role indicator
- [ ] Selecting walker/owner/admin role redirects to admin-dashboard
- [ ] Token is updated with new tenant context
- [ ] Typecheck passes
- [ ] Verify in browser

### US-004: Customer marketplace (no tenant selected)
**Description:** As a new customer without a selected business, I want to browse available businesses so I can choose one to book from.

**Acceptance Criteria:**
- [ ] Marketplace page at `/marketplace` or `/browse`
- [ ] Shows list of active businesses with branding
- [ ] Clicking a business sets it as active context
- [ ] Can also access via direct subdomain (business.offleash.world)
- [ ] Typecheck passes
- [ ] Verify in browser

### US-005: iOS persona switcher
**Description:** As an iOS user, I want to switch between my personas so I can access different features based on my role.

**Acceptance Criteria:**
- [ ] Profile/Settings shows all memberships
- [ ] Tapping a membership switches context
- [ ] UI adapts based on active role (customer vs walker tabs)
- [ ] Build succeeds

### US-006: iOS role-based navigation
**Description:** As an iOS user, I want the app navigation to change based on my active persona.

**Acceptance Criteria:**
- [ ] Customer persona: Services, Bookings, Profile tabs
- [ ] Walker persona: Schedule, Appointments, Earnings, Profile tabs
- [ ] Owner persona: Dashboard, Services, Walkers, Settings tabs
- [ ] Build succeeds

### US-007: Backend persona validation
**Description:** As a developer, I need the API to validate that the user has the correct persona for the requested action.

**Acceptance Criteria:**
- [ ] Booking endpoints require customer role in tenant
- [ ] Walker endpoints require walker/owner/admin role
- [ ] Admin endpoints require owner/admin role
- [ ] Returns 403 with clear message if role insufficient
- [ ] Typecheck passes

### US-008: Add persona (become customer of a business)
**Description:** As a user, I want to become a customer of a new business so I can book their services.

**Acceptance Criteria:**
- [ ] "Join as Customer" action on business profile/marketplace
- [ ] Creates customer membership for that tenant
- [ ] User can now book services from that business
- [ ] Typecheck passes
- [ ] Verify in browser

## Functional Requirements

### Web Apps

- FR-1: Customer-web shows only customer-relevant UI (book, view bookings, manage pets)
- FR-2: Admin-dashboard shows only admin-relevant UI (manage services, walkers, customers, analytics)
- FR-3: Persona switcher available in both apps, redirects to appropriate app based on role
- FR-4: Marketplace page for customers to discover and join businesses
- FR-5: Settings page shows unified view of all user's personas across all tenants

### iOS App

- FR-6: Tab bar navigation changes based on active persona role
- FR-7: Profile section includes persona switcher
- FR-8: Customer view: browse services, book, view bookings, manage pets
- FR-9: Walker view: calendar, appointments, route map, availability, earnings
- FR-10: Owner view: dashboard metrics, manage services, manage team (or link to web admin)

### Backend

- FR-11: `/contexts` endpoint returns all user memberships with roles
- FR-12: `/contexts/switch` updates JWT with new tenant/role context
- FR-13: Role-based middleware validates persona for protected endpoints
- FR-14: `/marketplace/businesses` returns public business listings
- FR-15: `/contexts/join-as-customer/:org_slug` creates customer membership

## Non-Goals

- Full admin dashboard in iOS (owners use web for complex management)
- Real-time persona sync across devices (refresh required)
- Persona-specific push notification routing (future enhancement)

## Technical Considerations

### Current State
- Memberships table links users to orgs with roles ✓
- Context switching API exists ✓
- JWT includes org_id claim ✓
- Admin-dashboard and customer-web are separate apps ✓

### Changes Needed
- Add marketplace page to customer-web
- Update iOS to support role-based navigation
- Ensure consistent persona UI across all platforms
- Add role indicators to switcher UI

### Token Structure
```json
{
  "sub": "user_uuid",
  "org_id": "org_uuid (optional)",
  "membership_id": "membership_uuid (optional)",
  "role": "customer|walker|admin|owner (when org_id set)",
  "platform_admin": false
}
```

## Platform Matrix

| Feature | customer-web | admin-dashboard | iOS |
|---------|--------------|-----------------|-----|
| View all personas | Settings | Settings | Profile |
| Switch persona | Header dropdown | Header dropdown | Profile |
| Customer UI | ✓ Primary | ✗ | ✓ |
| Walker UI | ✗ | ✓ | ✓ |
| Admin UI | ✗ | ✓ Primary | Limited |
| Marketplace | ✓ | ✗ | ✓ |

## Success Metrics

- Users can switch between personas in < 2 taps/clicks
- No confusion about which persona is active (clear indicator)
- Walker/admin actions not accessible from customer UI
- Customer actions not blocked when user also has walker role

## Open Questions

1. Should iOS include full walker features or just essentials with "Open in Dashboard" for complex tasks?
2. How do we handle a user who has walker role in one business and customer in another viewing the same service?
3. Should marketplace show businesses the user is already a customer of?

## Implementation Sequence

1. Document current state and identify gaps
2. Update customer-web marketplace page
3. Ensure admin-dashboard persona switching works
4. Update iOS persona management
5. Update iOS navigation for role-based tabs
6. Add role validation middleware to API endpoints
7. End-to-end testing across all platforms
