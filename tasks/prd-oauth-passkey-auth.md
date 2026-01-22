# PRD: OAuth and Passkey Authentication

## Introduction

Replace the current email/password-only authentication with a modern auth system supporting Google OAuth, Apple Sign-In, and Passkeys while maintaining email/password as a fallback. This improves signup conversion by reducing friction and increases security by enabling passwordless authentication.

## Goals

- Add Google OAuth sign-in to reduce signup friction
- Add Apple Sign-In (required for iOS App Store compliance)
- Add Passkey/WebAuthn support for passwordless authentication
- Maintain email/password as fallback option
- Share authentication across web and iOS apps
- Minimize ongoing costs (prioritize free/low-cost solutions)
- Implement within 1 week

## Options Analysis & Grading

### Evaluation Criteria

| Criteria | Weight | Description |
|----------|--------|-------------|
| Cost | 35% | Monthly cost at 1K, 10K, 100K MAU |
| Dev Time | 25% | Time to implement all features |
| Features | 20% | OAuth + Passkeys + mobile SDK support |
| Maintenance | 10% | Ongoing maintenance burden |
| Flexibility | 10% | Customization and migration ability |

### Option 1: Auth0

| Criteria | Score | Notes |
|----------|-------|-------|
| Cost | 2/10 | Free up to 7.5K MAU, then $23+/mo. Passkeys require paid plan ($240/mo) |
| Dev Time | 8/10 | Excellent SDKs, quick integration |
| Features | 9/10 | Full OAuth, Passkeys, MFA, everything |
| Maintenance | 9/10 | Fully managed |
| Flexibility | 5/10 | Vendor lock-in, complex migration |

**Weighted Score: 5.3/10**

**Pros:**
- Industry standard, battle-tested
- Excellent documentation
- Universal Login handles all UI

**Cons:**
- Expensive for passkeys ($240/mo minimum)
- Overkill for current needs
- Vendor lock-in

---

### Option 2: Clerk

| Criteria | Score | Notes |
|----------|-------|-------|
| Cost | 4/10 | Free up to 10K MAU, then $25/mo + $0.02/MAU |
| Dev Time | 9/10 | Best DX, pre-built components |
| Features | 9/10 | OAuth, Passkeys, great mobile SDKs |
| Maintenance | 9/10 | Fully managed |
| Flexibility | 6/10 | Good APIs but still vendor lock-in |

**Weighted Score: 6.4/10**

**Pros:**
- Best developer experience
- Pre-built UI components
- Passkeys included in free tier
- Excellent React/Swift SDKs

**Cons:**
- Costs grow with users ($0.02/MAU after 10K)
- Relatively new company
- Less flexibility than self-hosted

---

### Option 3: Supabase Auth

| Criteria | Score | Notes |
|----------|-------|-------|
| Cost | 9/10 | Free up to 50K MAU, self-host for $0 |
| Dev Time | 7/10 | Good SDKs, some manual setup |
| Features | 7/10 | OAuth built-in, Passkeys via extension |
| Maintenance | 7/10 | Managed or self-hosted option |
| Flexibility | 9/10 | Open source, no lock-in, can self-host |

**Weighted Score: 8.0/10**

**Pros:**
- Free tier up to 50K MAU
- Open source (can self-host if needed)
- Google/Apple OAuth built-in
- Good Swift SDK for iOS
- PostgreSQL-based (matches our stack)
- No vendor lock-in

**Cons:**
- Passkeys require manual WebAuthn setup
- Less polished than Clerk
- Some assembly required

---

### Option 4: DIY (Rust crates)

| Criteria | Score | Notes |
|----------|-------|-------|
| Cost | 10/10 | $0 - just infrastructure |
| Dev Time | 3/10 | 2-4 weeks for full implementation |
| Features | 6/10 | Full control but must build everything |
| Maintenance | 4/10 | Security updates, bug fixes on you |
| Flexibility | 10/10 | Complete control |

**Weighted Score: 6.5/10**

**Pros:**
- Zero additional cost
- Complete control and customization
- No vendor dependencies

**Cons:**
- 2-4 weeks dev time (conflicts with ASAP requirement)
- Security responsibility on you
- Must maintain OAuth app credentials, handle token refresh, etc.
- Passkey implementation is complex

**Crates needed:**
- `openidconnect` - OAuth2/OIDC
- `webauthn-rs` - Passkeys/WebAuthn
- `oauth2` - OAuth2 flows

---

### Final Ranking

| Rank | Option | Score | Recommendation |
|------|--------|-------|----------------|
| 1 | **Supabase Auth** | 8.0 | **RECOMMENDED** - Best balance of cost, features, and speed |
| 2 | DIY | 6.5 | Good long-term but too slow for ASAP |
| 3 | Clerk | 6.4 | Great DX but costs grow |
| 4 | Auth0 | 5.3 | Too expensive for passkeys |

## Recommended Approach: Supabase Auth

### Why Supabase Auth?

1. **Cost**: Free up to 50K MAU (covers growth runway)
2. **Speed**: Can integrate in 3-5 days with existing SDKs
3. **Features**: Google + Apple OAuth built-in, WebAuthn extensible
4. **iOS Support**: Official Swift SDK
5. **No Lock-in**: Open source, can migrate or self-host
6. **Stack Fit**: PostgreSQL-based, similar to our existing DB

### Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Customer Web  │     │    iOS App      │     │  Admin Dashboard│
│   (SvelteKit)   │     │    (Swift)      │     │   (SvelteKit)   │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │    Supabase Auth        │
                    │  (Hosted or Self-host)  │
                    │  - Google OAuth         │
                    │  - Apple Sign-In        │
                    │  - Email/Password       │
                    │  - Passkeys (WebAuthn)  │
                    └────────────┬────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │    OFFLEASH API         │
                    │    (Rust/Axum)          │
                    │  - Verify Supabase JWT  │
                    │  - Map to org/user      │
                    └────────────┬────────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   PostgreSQL (Render)   │
                    └─────────────────────────┘
```

## User Stories

### US-001: Set up Supabase Auth project
**Description:** As a developer, I need to configure Supabase Auth with OAuth providers so users can sign in.

**Acceptance Criteria:**
- [ ] Create Supabase project (or connect to existing)
- [ ] Configure Google OAuth provider with credentials
- [ ] Configure Apple Sign-In provider with credentials
- [ ] Enable email/password auth as fallback
- [ ] Set redirect URLs for web and iOS
- [ ] Document environment variables needed

### US-002: Add Supabase Auth to customer-web
**Description:** As a customer, I want to sign in with Google or Apple so I don't need to remember a password.

**Acceptance Criteria:**
- [ ] Install `@supabase/supabase-js` package
- [ ] Add "Sign in with Google" button on login page
- [ ] Add "Sign in with Apple" button on login page
- [ ] Keep existing email/password form as fallback
- [ ] Handle OAuth callback and session creation
- [ ] Store Supabase session token in cookie
- [ ] Redirect to /services after successful login
- [ ] Typecheck passes
- [ ] Verify in browser: Google sign-in flow works
- [ ] Verify in browser: Apple sign-in flow works

### US-003: Add Supabase Auth to iOS app
**Description:** As an iOS user, I want to sign in with Apple or Google so I have a seamless mobile experience.

**Acceptance Criteria:**
- [ ] Install Supabase Swift SDK via SPM
- [ ] Add "Sign in with Apple" button using ASAuthorizationAppleIDButton
- [ ] Add "Sign in with Google" button using Google Sign-In SDK
- [ ] Handle OAuth callback via deep link
- [ ] Store session token in Keychain
- [ ] Sync auth state with existing API token flow
- [ ] Test on physical device (Apple Sign-In requires it)

### US-004: Update API to verify Supabase JWTs
**Description:** As a developer, I need the API to accept Supabase JWTs so authenticated users can access protected routes.

**Acceptance Criteria:**
- [ ] Add Supabase JWT secret to environment variables
- [ ] Update `AuthUser` extractor to verify Supabase JWTs
- [ ] Map Supabase user ID to OFFLEASH user record
- [ ] Create OFFLEASH user on first OAuth login (just-in-time provisioning)
- [ ] Maintain backward compatibility with existing JWT tokens (migration period)
- [ ] Typecheck passes
- [ ] Test: API accepts Supabase JWT and returns user data

### US-005: Link OAuth accounts to existing users
**Description:** As an existing user, I want to link my Google/Apple account so I can use OAuth going forward.

**Acceptance Criteria:**
- [ ] Detect if OAuth email matches existing user
- [ ] Auto-link OAuth identity to existing user record
- [ ] Allow user to link additional OAuth providers in profile
- [ ] Handle email conflict (OAuth email differs from account email)
- [ ] Typecheck passes
- [ ] Verify in browser: existing user can link Google account

### US-006: Add Passkey registration
**Description:** As a user, I want to register a passkey so I can sign in without any password.

**Acceptance Criteria:**
- [ ] Add "Add Passkey" button in user profile/settings
- [ ] Trigger WebAuthn registration ceremony
- [ ] Store passkey credential in Supabase (or our DB)
- [ ] Show list of registered passkeys with delete option
- [ ] Typecheck passes
- [ ] Verify in browser: can register passkey on supported device

### US-007: Add Passkey authentication
**Description:** As a user with a registered passkey, I want to sign in using biometrics/device PIN.

**Acceptance Criteria:**
- [ ] Add "Sign in with Passkey" button on login page
- [ ] Trigger WebAuthn authentication ceremony
- [ ] Create session on successful passkey auth
- [ ] Fallback gracefully if passkey not supported
- [ ] Typecheck passes
- [ ] Verify in browser: can sign in with registered passkey

### US-008: Update logout to clear all sessions
**Description:** As a user, I want logout to work correctly with the new auth system.

**Acceptance Criteria:**
- [ ] Clear Supabase session on logout
- [ ] Clear local cookies/storage
- [ ] Redirect to login page
- [ ] Handle logout in iOS app (clear Keychain)
- [ ] Typecheck passes

## Functional Requirements

- FR-1: Support Google OAuth 2.0 sign-in on web and iOS
- FR-2: Support Apple Sign-In on web and iOS (required for App Store)
- FR-3: Maintain email/password authentication as fallback
- FR-4: Support Passkey (WebAuthn) registration and authentication
- FR-5: Auto-provision OFFLEASH user on first OAuth login
- FR-6: Link OAuth identities to existing users by email match
- FR-7: API must verify Supabase JWTs and map to organization context
- FR-8: Session tokens must work across web and iOS
- FR-9: Support account linking (add OAuth to existing account)

## Non-Goals

- No SMS/phone authentication (can add later)
- No social providers beyond Google/Apple (can add later)
- No enterprise SSO/SAML (future consideration)
- No MFA beyond passkeys in v1 (can add later)
- No migration of existing password hashes to Supabase (keep in our DB)

## Technical Considerations

### Environment Variables Needed
```
SUPABASE_URL=https://xxx.supabase.co
SUPABASE_ANON_KEY=eyJ...
SUPABASE_SERVICE_KEY=eyJ... (for API)
SUPABASE_JWT_SECRET=xxx (for JWT verification)
GOOGLE_CLIENT_ID=xxx
GOOGLE_CLIENT_SECRET=xxx
APPLE_CLIENT_ID=xxx
APPLE_TEAM_ID=xxx
APPLE_KEY_ID=xxx
APPLE_PRIVATE_KEY=xxx
```

### Migration Strategy
1. Deploy Supabase Auth alongside existing auth
2. Accept both old JWTs and Supabase JWTs (transition period)
3. Add OAuth buttons to login page
4. Encourage users to link OAuth accounts
5. Eventually deprecate password-only accounts (optional)

### iOS Considerations
- Apple Sign-In requires physical device testing
- Need to configure Associated Domains for deep links
- Google Sign-In requires URL scheme configuration

## Success Metrics

- 50%+ of new signups use OAuth within 30 days
- Reduce password reset requests by 40%
- Zero increase in auth-related support tickets
- Passkey adoption: 10%+ of active users within 90 days

## Open Questions

1. Should we migrate existing users to Supabase Auth or keep dual systems?
2. Do we need to support "Sign in with Apple" on web (recommended but complex)?
3. Should passkeys be opt-in or default for new users?
4. How do we handle users who signed up with OAuth but need admin access?

## Timeline Estimate

| Day | Tasks |
|-----|-------|
| 1 | Set up Supabase project, configure OAuth providers |
| 2 | Integrate Supabase Auth in customer-web (Google + Apple) |
| 3 | Update API to verify Supabase JWTs, user provisioning |
| 4 | Integrate Supabase Auth in iOS app |
| 5 | Add Passkey support, testing, polish |
| 6-7 | Buffer for issues, deployment, monitoring |

## Cost Projection

| MAU | Supabase | Auth0 | Clerk |
|-----|----------|-------|-------|
| 1,000 | $0 | $0 | $0 |
| 10,000 | $0 | $23/mo | $0 |
| 50,000 | $0 | $228/mo | $800/mo |
| 100,000 | $25/mo | $528/mo | $1,800/mo |

Supabase remains free up to 50K MAU, then $25/mo for Pro plan.
