-- Create platform_admins table for platform-level admin users
-- These are separate from tenant users and have global platform access
CREATE TABLE platform_admins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on email for login lookups
CREATE INDEX idx_platform_admins_email ON platform_admins(email);
