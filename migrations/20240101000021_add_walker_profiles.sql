-- Walker Profile Extension
-- Adds bio, specializations, emergency contact, and profile photo for walkers

-- Walker profiles table (extends users for walker-specific data)
CREATE TABLE IF NOT EXISTS walker_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Bio/About
    bio TEXT,

    -- Profile photo
    profile_photo_url TEXT,

    -- Emergency contact
    emergency_contact_name VARCHAR(255),
    emergency_contact_phone VARCHAR(50),
    emergency_contact_relationship VARCHAR(100),

    -- Experience
    years_experience INTEGER DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure one profile per user per org
    UNIQUE(user_id, organization_id)
);

-- Walker specializations enum
CREATE TYPE walker_specialization AS ENUM (
    'puppies',
    'senior_dogs',
    'large_breeds',
    'small_breeds',
    'anxious_reactive',
    'multiple_dogs',
    'pet_first_aid',
    'dog_training',
    'cat_care',
    'medication_administration'
);

-- Walker specializations junction table
CREATE TABLE IF NOT EXISTS walker_specializations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    walker_profile_id UUID NOT NULL REFERENCES walker_profiles(id) ON DELETE CASCADE,
    specialization walker_specialization NOT NULL,
    certified BOOLEAN DEFAULT FALSE,
    certification_date DATE,
    certification_expiry DATE,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Each specialization only once per walker
    UNIQUE(walker_profile_id, specialization)
);

-- Indexes
CREATE INDEX idx_walker_profiles_user ON walker_profiles(user_id);
CREATE INDEX idx_walker_profiles_org ON walker_profiles(organization_id);
CREATE INDEX idx_walker_specializations_profile ON walker_specializations(walker_profile_id);
CREATE INDEX idx_walker_specializations_type ON walker_specializations(specialization);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_walker_profile_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER walker_profiles_updated_at
    BEFORE UPDATE ON walker_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_walker_profile_updated_at();
