-- Multi-provider authentication schema
-- Supports: email, phone, google, apple, wallet

-- Auth provider enum
CREATE TYPE auth_provider AS ENUM ('email', 'phone', 'google', 'apple', 'wallet');

-- User identities - allows multiple auth methods per user
CREATE TABLE user_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider auth_provider NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,  -- Google sub, Apple sub, phone number, wallet address
    provider_email VARCHAR(255),              -- Email from OAuth provider (if available)
    provider_data JSONB DEFAULT '{}',         -- Additional provider-specific data
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(provider, provider_user_id)
);

-- Phone verification codes for SMS OTP
CREATE TABLE phone_verifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number VARCHAR(20) NOT NULL,
    code_hash VARCHAR(255) NOT NULL,          -- bcrypt hashed 6-digit code
    expires_at TIMESTAMPTZ NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,       -- Lock after 5 failed attempts
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Wallet authentication challenges (sign-in nonces)
CREATE TABLE wallet_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) NOT NULL,       -- Checksummed Ethereum address
    nonce VARCHAR(64) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add phone number to users table
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone_number VARCHAR(20) UNIQUE;

-- Indexes for efficient lookups
CREATE INDEX idx_user_identities_user ON user_identities(user_id);
CREATE INDEX idx_user_identities_provider ON user_identities(provider, provider_user_id);
CREATE INDEX idx_phone_verifications_phone ON phone_verifications(phone_number);
CREATE INDEX idx_phone_verifications_expires ON phone_verifications(expires_at);
CREATE INDEX idx_wallet_challenges_address ON wallet_challenges(wallet_address);
CREATE INDEX idx_wallet_challenges_expires ON wallet_challenges(expires_at);

-- Migrate existing email users to user_identities
-- This ensures existing email/password users have an identity record
INSERT INTO user_identities (user_id, provider, provider_user_id, provider_email)
SELECT id, 'email'::auth_provider, email, email
FROM users
WHERE email IS NOT NULL
ON CONFLICT (provider, provider_user_id) DO NOTHING;

-- Cleanup job: Delete expired phone verifications and wallet challenges
-- Run periodically via cron or scheduled task
COMMENT ON TABLE phone_verifications IS 'Cleanup: DELETE FROM phone_verifications WHERE expires_at < NOW()';
COMMENT ON TABLE wallet_challenges IS 'Cleanup: DELETE FROM wallet_challenges WHERE expires_at < NOW()';
