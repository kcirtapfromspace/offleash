-- Invitation types
CREATE TYPE invitation_type AS ENUM ('walker', 'client');
CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'expired', 'revoked');

-- Invitations table
CREATE TABLE invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    invited_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invitation_type invitation_type NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    token VARCHAR(64) NOT NULL UNIQUE,
    token_hash VARCHAR(255) NOT NULL,
    status invitation_status NOT NULL DEFAULT 'pending',
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    accepted_at TIMESTAMP WITH TIME ZONE,
    accepted_by UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,

    -- Ensure at least one contact method is provided
    CONSTRAINT check_contact_method CHECK (email IS NOT NULL OR phone IS NOT NULL)
);

-- Indexes for efficient lookups
CREATE INDEX idx_invitations_organization_id ON invitations(organization_id);
CREATE INDEX idx_invitations_token ON invitations(token);
CREATE INDEX idx_invitations_token_hash ON invitations(token_hash);
CREATE INDEX idx_invitations_email ON invitations(email) WHERE email IS NOT NULL;
CREATE INDEX idx_invitations_phone ON invitations(phone) WHERE phone IS NOT NULL;
CREATE INDEX idx_invitations_status ON invitations(status);
CREATE INDEX idx_invitations_expires_at ON invitations(expires_at);

-- Compound index for finding valid invitations
CREATE INDEX idx_invitations_valid ON invitations(token, status, expires_at)
    WHERE status = 'pending';
