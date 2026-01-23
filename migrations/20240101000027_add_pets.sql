-- Create pets table for customer dog profiles
CREATE TABLE pets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    species VARCHAR(50) NOT NULL DEFAULT 'dog',
    breed VARCHAR(100),
    date_of_birth DATE,
    weight_lbs DECIMAL(5, 1),
    gender VARCHAR(20),
    color VARCHAR(100),
    microchip_id VARCHAR(100),
    is_spayed_neutered BOOLEAN DEFAULT false,
    vaccination_status VARCHAR(50) DEFAULT 'unknown',
    temperament TEXT,
    special_needs TEXT,
    emergency_contact_name VARCHAR(200),
    emergency_contact_phone VARCHAR(50),
    vet_name VARCHAR(200),
    vet_phone VARCHAR(50),
    photo_url TEXT,
    notes TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_pets_organization ON pets(organization_id);
CREATE INDEX idx_pets_owner ON pets(owner_id);
CREATE INDEX idx_pets_organization_owner ON pets(organization_id, owner_id);

-- Add trigger for updated_at
CREATE TRIGGER update_pets_updated_at
    BEFORE UPDATE ON pets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
