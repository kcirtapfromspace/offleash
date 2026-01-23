-- Update Payment System Fields Migration
-- Adds missing columns for multi-party payments and enhanced provider tracking

-- ============================================
-- TRANSACTIONS TABLE UPDATES
-- ============================================

-- Add customer and provider user tracking for multi-party payments
ALTER TABLE transactions
    ADD COLUMN IF NOT EXISTS customer_user_id UUID REFERENCES users(id),
    ADD COLUMN IF NOT EXISTS provider_user_id UUID REFERENCES users(id);

-- Migrate existing user_id to customer_user_id
UPDATE transactions SET customer_user_id = user_id WHERE customer_user_id IS NULL;

-- Add tip and enhanced fee tracking
ALTER TABLE transactions
    ADD COLUMN IF NOT EXISTS tip_cents INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS platform_fee_cents INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS provider_payout_cents INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS external_payment_id VARCHAR(255);

-- Rename payment_provider_id to provider_id for consistency
ALTER TABLE transactions
    RENAME COLUMN payment_provider_id TO provider_id;

-- Create index for external payment ID lookups (webhook processing)
CREATE INDEX IF NOT EXISTS idx_transactions_external_id
    ON transactions(external_payment_id)
    WHERE external_payment_id IS NOT NULL;

-- Create index for customer lookups
CREATE INDEX IF NOT EXISTS idx_transactions_customer
    ON transactions(customer_user_id);

-- Create index for provider lookups
CREATE INDEX IF NOT EXISTS idx_transactions_provider
    ON transactions(provider_user_id);

-- ============================================
-- PAYMENT PROVIDERS TABLE UPDATES
-- ============================================

-- Add webhook and display fields
ALTER TABLE payment_providers
    ADD COLUMN IF NOT EXISTS webhook_secret TEXT,
    ADD COLUMN IF NOT EXISTS is_primary BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS account_name VARCHAR(255),
    ADD COLUMN IF NOT EXISTS merchant_id VARCHAR(255);

-- Create index for primary provider lookup
CREATE INDEX IF NOT EXISTS idx_payment_providers_primary
    ON payment_providers(organization_id, is_primary)
    WHERE is_primary = true;

-- Create index for webhook lookups by account ID
CREATE INDEX IF NOT EXISTS idx_payment_providers_stripe_account
    ON payment_providers(stripe_account_id)
    WHERE stripe_account_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_payment_providers_square_merchant
    ON payment_providers(square_merchant_id)
    WHERE square_merchant_id IS NOT NULL;

-- ============================================
-- PAYOUTS TABLE UPDATES
-- ============================================

-- Add transaction_ids array to track which transactions are in each payout
ALTER TABLE payouts
    ADD COLUMN IF NOT EXISTS transaction_ids UUID[] DEFAULT '{}';

-- Create GIN index for array lookups
CREATE INDEX IF NOT EXISTS idx_payouts_transaction_ids
    ON payouts USING GIN(transaction_ids);

-- ============================================
-- UPDATE COMMENTS
-- ============================================

COMMENT ON COLUMN transactions.customer_user_id IS 'The customer making the payment';
COMMENT ON COLUMN transactions.provider_user_id IS 'The service provider receiving payment (e.g., dog walker)';
COMMENT ON COLUMN transactions.tip_cents IS 'Optional tip amount in cents';
COMMENT ON COLUMN transactions.platform_fee_cents IS 'Platform fee charged (based on tenant tier)';
COMMENT ON COLUMN transactions.provider_payout_cents IS 'Net amount to be paid out to the provider';
COMMENT ON COLUMN transactions.external_payment_id IS 'Generic external payment reference ID';

COMMENT ON COLUMN payment_providers.webhook_secret IS 'Webhook signature verification secret';
COMMENT ON COLUMN payment_providers.is_primary IS 'Whether this is the primary payment provider for the org';
COMMENT ON COLUMN payment_providers.account_name IS 'Display name for the connected account';
COMMENT ON COLUMN payment_providers.merchant_id IS 'Generic merchant identifier';

COMMENT ON COLUMN payouts.transaction_ids IS 'Array of transaction IDs included in this payout';
