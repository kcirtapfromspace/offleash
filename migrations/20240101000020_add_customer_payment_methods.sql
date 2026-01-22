-- Customer saved payment methods
-- Stores tokenized payment methods for reuse

-- Create payment method type enum (different from payment_method which is per-transaction)
CREATE TYPE payment_method_type AS ENUM ('card', 'apple_pay', 'bank_account');

-- Create card brand enum for display purposes
CREATE TYPE card_brand AS ENUM ('visa', 'mastercard', 'amex', 'discover', 'other');

-- Customer payment methods table
CREATE TABLE customer_payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    customer_id UUID NOT NULL REFERENCES users(id),
    method_type payment_method_type NOT NULL,

    -- For cards: last 4 digits and brand for display
    card_last_four VARCHAR(4),
    card_brand card_brand,
    card_exp_month INTEGER CHECK (card_exp_month >= 1 AND card_exp_month <= 12),
    card_exp_year INTEGER CHECK (card_exp_year >= 2024),

    -- Square token for processing payments
    square_card_id VARCHAR(255),

    -- Display name (e.g., "Personal Visa", "Work Card")
    nickname VARCHAR(100),

    -- Whether this is the default payment method
    is_default BOOLEAN NOT NULL DEFAULT false,

    -- Whether this method is still active/valid
    is_active BOOLEAN NOT NULL DEFAULT true,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_customer_payment_methods_customer ON customer_payment_methods(customer_id);
CREATE INDEX idx_customer_payment_methods_org ON customer_payment_methods(organization_id);
CREATE INDEX idx_customer_payment_methods_default ON customer_payment_methods(customer_id, is_default) WHERE is_default = true;

-- Ensure only one default per customer
CREATE UNIQUE INDEX idx_customer_payment_methods_single_default
    ON customer_payment_methods(customer_id)
    WHERE is_default = true AND is_active = true;
