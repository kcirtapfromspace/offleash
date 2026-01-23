# PRD: Hand-Rolled Multi-Provider Authentication

## Introduction

Implement a custom authentication system supporting multiple identity providers: email/password (existing), phone (SMS OTP), Web3 wallet, Apple Sign-In, and Google Sign-In. All providers link to a single user account, enabling users to authenticate via any method.

## Goals

- Support 5 authentication methods: email, phone, Web3 wallet, Apple, Google
- Allow users to link multiple auth methods to one account
- Maintain security best practices (secure token storage, rate limiting, etc.)
- Keep implementation costs minimal (avoid expensive third-party services)
- Support both web and iOS platforms

## User Stories

### US-001: Database schema for auth providers
**Description:** As a developer, I need to store multiple auth identities per user so users can log in via different methods.

**Acceptance Criteria:**
- [ ] Create `user_identities` table with columns: id, user_id, provider (enum), provider_user_id, provider_email, provider_data (JSONB), created_at
- [ ] Provider enum includes: email, phone, google, apple, wallet
- [ ] Unique constraint on (provider, provider_user_id)
- [ ] Foreign key to users table with CASCADE delete
- [ ] Migration runs successfully
- [ ] Typecheck passes

### US-002: Google OAuth - Backend
**Description:** As a user, I want to sign in with my Google account for quick access.

**Acceptance Criteria:**
- [ ] API endpoint `POST /auth/google` accepts Google ID token
- [ ] Verify token with Google's public keys (fetch from https://www.googleapis.com/oauth2/v3/certs)
- [ ] Extract email, sub (Google user ID), name from token
- [ ] If identity exists, return JWT for that user
- [ ] If identity doesn't exist but email matches existing user, link identity
- [ ] If no user exists, create new user + identity
- [ ] Return JWT token on success
- [ ] Typecheck passes

### US-003: Google OAuth - Frontend
**Description:** As a user, I want to click "Sign in with Google" and authenticate seamlessly.

**Acceptance Criteria:**
- [ ] Add Google Sign-In button to login page
- [ ] Use Google Identity Services library (accounts.google.com/gsi/client)
- [ ] On success, send ID token to backend `/auth/google`
- [ ] Store returned JWT in cookie
- [ ] Redirect to /services on success
- [ ] Show error message on failure
- [ ] Typecheck passes
- [ ] Verify in browser

### US-004: Apple Sign-In - Backend
**Description:** As a user, I want to sign in with my Apple ID for privacy-focused authentication.

**Acceptance Criteria:**
- [ ] API endpoint `POST /auth/apple` accepts Apple identity token
- [ ] Verify token with Apple's public keys (fetch from https://appleid.apple.com/auth/keys)
- [ ] Extract email, sub (Apple user ID), name from token
- [ ] Handle Apple's email relay (privaterelay.appleid.com)
- [ ] Same user matching logic as Google (link or create)
- [ ] Return JWT token on success
- [ ] Typecheck passes

### US-005: Apple Sign-In - Frontend
**Description:** As a user, I want to click "Sign in with Apple" on web.

**Acceptance Criteria:**
- [ ] Add Apple Sign-In button to login page
- [ ] Use Apple JS SDK (appleid.auth.init)
- [ ] Configure with Service ID and redirect URI
- [ ] On success, send identity token to backend `/auth/apple`
- [ ] Store returned JWT in cookie
- [ ] Redirect to /services on success
- [ ] Show error message on failure
- [ ] Typecheck passes
- [ ] Verify in browser

### US-006: Phone Auth - Database & Backend Setup
**Description:** As a developer, I need infrastructure for phone-based OTP authentication.

**Acceptance Criteria:**
- [ ] Create `phone_verifications` table: id, phone_number, code (hashed), expires_at, attempts, created_at
- [ ] Add phone_number column to users table (nullable, unique when not null)
- [ ] API endpoint `POST /auth/phone/send-code` accepts phone number
- [ ] Generate 6-digit OTP, hash it, store with 10-minute expiry
- [ ] Rate limit: max 3 codes per phone per hour
- [ ] Return success (don't reveal if phone exists)
- [ ] Typecheck passes

### US-007: Phone Auth - SMS Integration
**Description:** As a user, I want to receive an SMS code to verify my phone number.

**Acceptance Criteria:**
- [ ] Integrate Twilio or AWS SNS for SMS delivery
- [ ] Send OTP via SMS after generating code
- [ ] SMS message: "Your OFFLEASH code is: XXXXXX. Expires in 10 minutes."
- [ ] Handle SMS delivery failures gracefully
- [ ] Log delivery status (not the code)
- [ ] Typecheck passes

### US-008: Phone Auth - Verification & Login
**Description:** As a user, I want to enter my SMS code and log in.

**Acceptance Criteria:**
- [ ] API endpoint `POST /auth/phone/verify` accepts phone + code
- [ ] Verify code matches and hasn't expired
- [ ] Increment attempts counter, lock after 5 failed attempts
- [ ] If phone identity exists, return JWT for that user
- [ ] If phone matches existing user, link identity
- [ ] If no user exists, create new user + identity
- [ ] Delete verification record on success
- [ ] Return JWT token on success
- [ ] Typecheck passes

### US-009: Phone Auth - Frontend
**Description:** As a user, I want to enter my phone number and verify via SMS code.

**Acceptance Criteria:**
- [ ] Add "Continue with Phone" option to login page
- [ ] Phone number input with country code selector
- [ ] "Send Code" button calls `/auth/phone/send-code`
- [ ] Show code input field after sending
- [ ] 6-digit code input with auto-submit
- [ ] "Resend Code" link (with cooldown timer)
- [ ] On verification success, store JWT and redirect
- [ ] Show error messages for invalid code, expired, locked
- [ ] Typecheck passes
- [ ] Verify in browser

### US-010: Web3 Wallet Auth - Backend
**Description:** As a user, I want to sign in with my crypto wallet (MetaMask, etc.).

**Acceptance Criteria:**
- [ ] API endpoint `POST /auth/wallet/challenge` returns nonce for wallet address
- [ ] Store nonce with expiry (5 minutes) in `wallet_challenges` table
- [ ] API endpoint `POST /auth/wallet/verify` accepts address + signature
- [ ] Recover signer address from signature using secp256k1
- [ ] Verify recovered address matches claimed address
- [ ] If wallet identity exists, return JWT for that user
- [ ] If no user exists, create new user + identity (no email required)
- [ ] Return JWT token on success
- [ ] Typecheck passes

### US-011: Web3 Wallet Auth - Frontend
**Description:** As a user, I want to click "Connect Wallet" and sign a message to authenticate.

**Acceptance Criteria:**
- [ ] Add "Connect Wallet" button to login page
- [ ] Detect if MetaMask/wallet provider is available
- [ ] Request wallet connection (eth_requestAccounts)
- [ ] Fetch challenge from `/auth/wallet/challenge`
- [ ] Prompt user to sign message: "Sign in to OFFLEASH\n\nNonce: {nonce}"
- [ ] Send signature to `/auth/wallet/verify`
- [ ] Store returned JWT in cookie
- [ ] Redirect to /services on success
- [ ] Show "Install MetaMask" link if no wallet detected
- [ ] Typecheck passes
- [ ] Verify in browser

### US-012: Link Additional Auth Methods
**Description:** As a logged-in user, I want to link additional sign-in methods to my account.

**Acceptance Criteria:**
- [ ] Settings page section: "Connected Accounts"
- [ ] Show currently linked providers with unlink option
- [ ] Buttons to link: Google, Apple, Phone, Wallet
- [ ] Linking flow same as login, but attaches to current user
- [ ] Prevent unlinking last auth method
- [ ] API endpoints for link/unlink operations
- [ ] Typecheck passes
- [ ] Verify in browser

### US-013: iOS - Google Sign-In
**Description:** As an iOS user, I want to sign in with Google.

**Acceptance Criteria:**
- [ ] Add GoogleSignIn Swift package
- [ ] Configure with Google Client ID for iOS
- [ ] Add URL scheme for Google callback
- [ ] Implement sign-in button in LoginView
- [ ] Send ID token to backend `/auth/google`
- [ ] Store JWT in Keychain
- [ ] Navigate to main view on success

### US-014: iOS - Apple Sign-In
**Description:** As an iOS user, I want to sign in with Apple (required for App Store).

**Acceptance Criteria:**
- [ ] Use AuthenticationServices framework
- [ ] Implement ASAuthorizationControllerDelegate
- [ ] Add Sign in with Apple capability in Xcode
- [ ] Implement sign-in button in LoginView
- [ ] Send identity token to backend `/auth/apple`
- [ ] Handle first-time name/email (Apple only sends once)
- [ ] Store JWT in Keychain
- [ ] Navigate to main view on success

## Functional Requirements

- FR-1: All auth tokens (JWT) expire after 7 days with refresh capability
- FR-2: Users can have multiple identities linked to one account
- FR-3: Email from OAuth providers is used to match/link existing accounts
- FR-4: Phone numbers must be E.164 format (+1234567890)
- FR-5: Wallet addresses must be checksummed Ethereum addresses
- FR-6: Failed auth attempts are rate-limited (5 per minute per IP)
- FR-7: All sensitive tokens (Google/Apple) verified server-side, never trusted from client
- FR-8: OTP codes are hashed before storage (bcrypt)
- FR-9: OAuth state parameter used to prevent CSRF

## Non-Goals

- No support for Facebook, Twitter, or other OAuth providers (can add later)
- No passwordless email (magic links) - use OTP phone instead
- No biometric auth on web (handled natively on iOS)
- No multi-factor authentication (2FA) in initial implementation
- No account recovery flows beyond linked auth methods

## Technical Considerations

### Dependencies to Add

**Rust API:**
- `jsonwebtoken` - JWT verification (already have)
- `reqwest` - HTTP client for fetching OAuth public keys
- `secp256k1` or `ethers` - Wallet signature verification

**Customer Web:**
- Google Identity Services (script tag, no npm package)
- Apple JS SDK (script tag)
- `ethers` or `viem` - Wallet interaction

**iOS:**
- GoogleSignIn-iOS (SPM)
- AuthenticationServices (built-in)

### Environment Variables

```
# Google OAuth
GOOGLE_CLIENT_ID=xxx.apps.googleusercontent.com
GOOGLE_CLIENT_ID_IOS=xxx.apps.googleusercontent.com

# Apple Sign-In
APPLE_TEAM_ID=XXXXXXXXXX
APPLE_SERVICE_ID=com.offleash.web
APPLE_KEY_ID=XXXXXXXXXX
APPLE_PRIVATE_KEY=-----BEGIN PRIVATE KEY-----...

# Phone Auth (Twilio)
TWILIO_ACCOUNT_SID=ACxxx
TWILIO_AUTH_TOKEN=xxx
TWILIO_PHONE_NUMBER=+1234567890
```

### Database Schema

```sql
-- Auth provider enum
CREATE TYPE auth_provider AS ENUM ('email', 'phone', 'google', 'apple', 'wallet');

-- User identities (multiple per user)
CREATE TABLE user_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider auth_provider NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    provider_email VARCHAR(255),
    provider_data JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);

-- Phone verification codes
CREATE TABLE phone_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number VARCHAR(20) NOT NULL,
    code_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Wallet auth challenges
CREATE TABLE wallet_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,
    nonce VARCHAR(64) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add phone to users
ALTER TABLE users ADD COLUMN phone_number VARCHAR(20) UNIQUE;

-- Indexes
CREATE INDEX idx_user_identities_user ON user_identities(user_id);
CREATE INDEX idx_user_identities_provider ON user_identities(provider, provider_user_id);
CREATE INDEX idx_phone_verifications_phone ON phone_verifications(phone_number);
CREATE INDEX idx_wallet_challenges_address ON wallet_challenges(wallet_address);
```

## Success Metrics

- Users can sign in via any of the 5 methods
- OAuth sign-in completes in under 3 seconds
- Phone OTP delivery rate > 95%
- No security vulnerabilities in auth flow
- iOS app passes App Store review (requires Apple Sign-In)

## Open Questions

1. Should we require email for wallet-only users? (Currently: no)
2. Should phone auth be available in all regions or US-only initially?
3. What's the SMS budget per month? (Twilio costs ~$0.01/SMS)

## Implementation Sequence

1. **Database migration** - user_identities, phone_verifications, wallet_challenges tables
2. **Google OAuth backend** - Token verification, user matching/creation
3. **Google OAuth frontend** - Login button, token flow
4. **Apple Sign-In backend** - Token verification, user matching/creation
5. **Apple Sign-In frontend** - Login button, token flow
6. **Phone auth backend** - Send code, verify code endpoints
7. **Phone auth SMS** - Twilio integration
8. **Phone auth frontend** - Phone input, code input UI
9. **Wallet auth backend** - Challenge/verify endpoints
10. **Wallet auth frontend** - Connect wallet, sign message
11. **Account linking UI** - Settings page for managing connected accounts
12. **iOS Google Sign-In** - Native implementation
13. **iOS Apple Sign-In** - Native implementation
