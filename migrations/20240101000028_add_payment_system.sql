-- Multi-tenant Payment System Migration
-- Supports Stripe Connect, Square, and Platform Default processing

-- Create provider type enum
CREATE TYPE payment_provider_type AS ENUM ('stripe', 'square', 'platform');

-- Create payment method type enum (extended)
CREATE TYPE payment_method_type AS ENUM (
    'card', 'apple_pay', 'google_pay', 'shop_pay', 'link', 'bank_account'
);

-- Create transaction status enum
CREATE TYPE transaction_status AS ENUM (
    'pending', 'processing', 'succeeded', 'failed', 'refunded', 'partially_refunded', 'disputed'
);

-- Create subscription status enum
CREATE TYPE subscription_status AS ENUM (
    'active', 'paused', 'canceled', 'past_due', 'trialing', 'incomplete'
);

-- Create payout status enum
CREATE TYPE payout_status AS ENUM (
    'pending', 'in_transit', 'paid', 'failed', 'canceled'
);

-- Create dispute status enum
CREATE TYPE dispute_status AS ENUM (
    'needs_response', 'under_review', 'won', 'lost'
);

-- Create plan tier enum for platform subscriptions
CREATE TYPE plan_tier AS ENUM ('free', 'professional', 'business', 'enterprise');

-- Payment Providers: Stores tenant payment processor configurations
CREATE TABLE payment_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider_type payment_provider_type NOT NULL,
    -- For Stripe Connect
    stripe_account_id VARCHAR(255),
    stripe_account_type VARCHAR(50), -- 'standard' or 'express'
    -- For Square
    square_merchant_id VARCHAR(255),
    -- Encrypted tokens (use application-level encryption)
    access_token_encrypted TEXT,
    refresh_token_encrypted TEXT,
    token_expires_at TIMESTAMPTZ,
    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_status VARCHAR(100),
    -- Capabilities
    charges_enabled BOOLEAN DEFAULT false,
    payouts_enabled BOOLEAN DEFAULT false,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(organization_id, provider_type)
);

-- Customer Payment Methods: Stored payment methods for customers
CREATE TABLE customer_payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_type payment_provider_type NOT NULL,
    method_type payment_method_type NOT NULL,
    -- Provider references
    stripe_payment_method_id VARCHAR(255),
    stripe_customer_id VARCHAR(255),
    square_card_id VARCHAR(255),
    square_customer_id VARCHAR(255),
    -- Card details (non-sensitive)
    last_four VARCHAR(4),
    brand VARCHAR(50), -- visa, mastercard, amex, etc.
    exp_month INTEGER,
    exp_year INTEGER,
    cardholder_name VARCHAR(255),
    -- For bank accounts
    bank_name VARCHAR(255),
    account_last_four VARCHAR(4),
    -- Wallet info
    wallet_type VARCHAR(50), -- apple_pay, google_pay, shop_pay, link
    -- Status
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    -- Metadata
    billing_address JSONB,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Platform Fee Tiers: Defines fee structure per plan
CREATE TABLE platform_fee_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_tier plan_tier NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    -- Customer-facing fee (added to total)
    customer_fee_percent DECIMAL(5,4) NOT NULL, -- e.g., 0.0300 = 3%
    -- Provider fee (taken from their earnings)
    provider_fee_percent DECIMAL(5,4) NOT NULL, -- e.g., 0.2000 = 20%
    -- Minimum fees
    min_customer_fee_cents INTEGER NOT NULL DEFAULT 0,
    min_provider_fee_cents INTEGER NOT NULL DEFAULT 50, -- $0.50 minimum
    -- Monthly platform subscription price
    monthly_price_cents INTEGER NOT NULL DEFAULT 0,
    annual_price_cents INTEGER NOT NULL DEFAULT 0,
    -- Features
    features JSONB DEFAULT '[]',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default fee tiers
INSERT INTO platform_fee_tiers (plan_tier, display_name, customer_fee_percent, provider_fee_percent, min_provider_fee_cents, monthly_price_cents, annual_price_cents, features) VALUES
    ('free', 'Free', 0.0300, 0.2000, 50, 0, 0, '["Basic scheduling", "Up to 10 bookings/month"]'),
    ('professional', 'Professional', 0.0200, 0.1500, 50, 2900, 29000, '["Unlimited bookings", "Customer subscriptions", "Priority support"]'),
    ('business', 'Business', 0.0100, 0.1000, 50, 7900, 79000, '["Everything in Pro", "Custom branding", "API access", "Dedicated support"]'),
    ('enterprise', 'Enterprise', 0.0050, 0.0500, 50, 0, 0, '["Custom pricing", "SLA", "Dedicated account manager"]');

-- Tenant Subscriptions: Platform SaaS billing for tenants
CREATE TABLE tenant_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE UNIQUE,
    plan_tier plan_tier NOT NULL DEFAULT 'free',
    -- Stripe subscription for platform billing
    stripe_subscription_id VARCHAR(255),
    stripe_customer_id VARCHAR(255),
    -- Status
    status subscription_status NOT NULL DEFAULT 'active',
    -- Billing period
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    -- Trial
    trial_start TIMESTAMPTZ,
    trial_end TIMESTAMPTZ,
    -- Cancellation
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    canceled_at TIMESTAMPTZ,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Transactions: Comprehensive payment transaction records
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    booking_id UUID REFERENCES bookings(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    payment_method_id UUID REFERENCES customer_payment_methods(id) ON DELETE SET NULL,
    payment_provider_id UUID REFERENCES payment_providers(id) ON DELETE SET NULL,

    -- Amounts (all in cents)
    subtotal_cents INTEGER NOT NULL,
    customer_fee_cents INTEGER NOT NULL DEFAULT 0,
    tax_cents INTEGER NOT NULL DEFAULT 0,
    total_cents INTEGER NOT NULL,
    provider_fee_cents INTEGER NOT NULL DEFAULT 0,
    processing_fee_cents INTEGER NOT NULL DEFAULT 0,
    net_amount_cents INTEGER NOT NULL, -- Amount to provider after all fees

    -- Currency
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',

    -- Status
    status transaction_status NOT NULL DEFAULT 'pending',

    -- Provider references
    stripe_payment_intent_id VARCHAR(255),
    stripe_charge_id VARCHAR(255),
    stripe_transfer_id VARCHAR(255),
    square_payment_id VARCHAR(255),
    square_order_id VARCHAR(255),

    -- Tax details
    tax_rate_percent DECIMAL(5,4),
    tax_jurisdiction VARCHAR(255),
    tax_calculation_id VARCHAR(255), -- TaxJar/Avalara reference

    -- Refund tracking
    refunded_amount_cents INTEGER NOT NULL DEFAULT 0,

    -- Failure info
    failure_code VARCHAR(100),
    failure_message TEXT,

    -- Metadata
    description TEXT,
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Customer Subscriptions: Recurring service packages for customers
CREATE TABLE customer_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    service_id UUID REFERENCES services(id) ON DELETE SET NULL,

    -- Plan details
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price_cents INTEGER NOT NULL,
    interval VARCHAR(20) NOT NULL DEFAULT 'month', -- week, month, year
    interval_count INTEGER NOT NULL DEFAULT 1,

    -- Provider references
    stripe_subscription_id VARCHAR(255),
    stripe_price_id VARCHAR(255),
    square_subscription_id VARCHAR(255),

    -- Status
    status subscription_status NOT NULL DEFAULT 'active',

    -- Billing period
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,

    -- Cancellation
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    canceled_at TIMESTAMPTZ,

    -- Auto-booking settings
    auto_create_bookings BOOLEAN NOT NULL DEFAULT true,
    preferred_day_of_week INTEGER, -- 0=Sunday, 6=Saturday
    preferred_time TIME,

    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Payout Settings: How tenants receive their money
CREATE TABLE payout_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE UNIQUE,

    -- Payout method
    payout_method VARCHAR(20) NOT NULL DEFAULT 'bank', -- bank, debit

    -- Bank account (tokenized)
    stripe_bank_account_id VARCHAR(255),
    square_bank_account_id VARCHAR(255),
    bank_name VARCHAR(255),
    bank_account_last_four VARCHAR(4),
    bank_routing_last_four VARCHAR(4),

    -- Schedule
    payout_schedule VARCHAR(20) NOT NULL DEFAULT 'daily', -- daily, weekly, monthly
    payout_day_of_week INTEGER, -- For weekly: 0=Sunday
    payout_day_of_month INTEGER, -- For monthly: 1-28

    -- Thresholds
    minimum_payout_cents INTEGER NOT NULL DEFAULT 100, -- $1 minimum

    -- Status
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_status VARCHAR(100),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Payouts: Track payouts to tenants
CREATE TABLE payouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Amount
    amount_cents INTEGER NOT NULL,
    fee_cents INTEGER NOT NULL DEFAULT 0,
    net_amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',

    -- Period covered
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,

    -- Provider references
    stripe_payout_id VARCHAR(255),
    stripe_transfer_id VARCHAR(255),
    square_payout_id VARCHAR(255),

    -- Status
    status payout_status NOT NULL DEFAULT 'pending',

    -- Timing
    initiated_at TIMESTAMPTZ,
    arrival_date TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Failure info
    failure_code VARCHAR(100),
    failure_message TEXT,

    -- Transaction summary
    transaction_count INTEGER NOT NULL DEFAULT 0,

    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Disputes: Track payment disputes/chargebacks
CREATE TABLE disputes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    transaction_id UUID NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,

    -- Amount
    amount_cents INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',

    -- Provider references
    stripe_dispute_id VARCHAR(255),
    square_dispute_id VARCHAR(255),

    -- Dispute details
    reason VARCHAR(100) NOT NULL,
    status dispute_status NOT NULL DEFAULT 'needs_response',

    -- Evidence
    evidence_submitted BOOLEAN NOT NULL DEFAULT false,
    evidence_due_by TIMESTAMPTZ,

    -- Resolution
    resolved_at TIMESTAMPTZ,
    outcome VARCHAR(50), -- won, lost

    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Webhook Events: Log all webhook events for audit
CREATE TABLE payment_webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider payment_provider_type NOT NULL,
    event_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT false,
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(provider, event_id)
);

-- Indexes for performance
CREATE INDEX idx_payment_providers_org ON payment_providers(organization_id);
CREATE INDEX idx_payment_providers_active ON payment_providers(organization_id, is_active) WHERE is_active = true;

CREATE INDEX idx_customer_payment_methods_user ON customer_payment_methods(user_id);
CREATE INDEX idx_customer_payment_methods_org_user ON customer_payment_methods(organization_id, user_id);
CREATE INDEX idx_customer_payment_methods_default ON customer_payment_methods(user_id, is_default) WHERE is_default = true;

CREATE INDEX idx_transactions_org ON transactions(organization_id);
CREATE INDEX idx_transactions_user ON transactions(user_id);
CREATE INDEX idx_transactions_booking ON transactions(booking_id);
CREATE INDEX idx_transactions_status ON transactions(organization_id, status);
CREATE INDEX idx_transactions_created ON transactions(organization_id, created_at DESC);
CREATE INDEX idx_transactions_stripe_pi ON transactions(stripe_payment_intent_id) WHERE stripe_payment_intent_id IS NOT NULL;
CREATE INDEX idx_transactions_square ON transactions(square_payment_id) WHERE square_payment_id IS NOT NULL;

CREATE INDEX idx_customer_subscriptions_org ON customer_subscriptions(organization_id);
CREATE INDEX idx_customer_subscriptions_user ON customer_subscriptions(user_id);
CREATE INDEX idx_customer_subscriptions_status ON customer_subscriptions(organization_id, status);

CREATE INDEX idx_tenant_subscriptions_status ON tenant_subscriptions(status);

CREATE INDEX idx_payouts_org ON payouts(organization_id);
CREATE INDEX idx_payouts_status ON payouts(organization_id, status);
CREATE INDEX idx_payouts_period ON payouts(organization_id, period_start, period_end);

CREATE INDEX idx_disputes_org ON disputes(organization_id);
CREATE INDEX idx_disputes_status ON disputes(organization_id, status);
CREATE INDEX idx_disputes_transaction ON disputes(transaction_id);

CREATE INDEX idx_webhook_events_unprocessed ON payment_webhook_events(provider, processed, created_at) WHERE processed = false;

-- Add plan_tier column to organizations for quick access
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS plan_tier plan_tier NOT NULL DEFAULT 'free';

-- Update trigger for updated_at
CREATE OR REPLACE FUNCTION update_payment_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_payment_providers_updated_at BEFORE UPDATE ON payment_providers FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
CREATE TRIGGER update_customer_payment_methods_updated_at BEFORE UPDATE ON customer_payment_methods FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
CREATE TRIGGER update_transactions_updated_at BEFORE UPDATE ON transactions FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
CREATE TRIGGER update_customer_subscriptions_updated_at BEFORE UPDATE ON customer_subscriptions FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
CREATE TRIGGER update_tenant_subscriptions_updated_at BEFORE UPDATE ON tenant_subscriptions FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
CREATE TRIGGER update_payout_settings_updated_at BEFORE UPDATE ON payout_settings FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
CREATE TRIGGER update_payouts_updated_at BEFORE UPDATE ON payouts FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
CREATE TRIGGER update_disputes_updated_at BEFORE UPDATE ON disputes FOR EACH ROW EXECUTE FUNCTION update_payment_updated_at();
