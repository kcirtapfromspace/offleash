# PRD: Multi-Tenant Payment System

## Introduction

Build a comprehensive payment system for OFFLEASH that supports multiple payment processors (Stripe, Square), digital wallets (Apple Pay, Google Pay), and accelerated checkout options (Shop Pay, Link). The system must handle multi-tenant payment routing where each tenant (pet service business or individual provider) can connect their own payment accounts or use the platform's built-in processing. The platform will collect fees using a split model with tiered rates based on tenant subscription plans.

## Goals

- Enable customers to pay using credit cards, Apple Pay, Google Pay, Shop Pay, and Link
- Support both Stripe Connect and Square for tenant payment processing
- Provide a "platform default" option for tenants who don't want to manage their own payment accounts
- Implement split-fee model: small customer service fee + percentage from provider (tiered by plan)
- Handle automatic tax calculation via integration (TaxJar/Avalara)
- Support one-time payments, customer subscriptions (service packages), and tenant subscriptions (SaaS billing)
- Enable both B2B tenants (pet service businesses) and B2C tenants (individual providers)

## User Stories

### Phase 1: Core Payment Infrastructure

#### US-001: Payment processor database schema
**Description:** As a developer, I need database tables to store tenant payment configurations and transaction records.

**Acceptance Criteria:**
- [ ] Create `payment_providers` table (id, organization_id, provider_type ['stripe'|'square'|'platform'], account_id, access_token_encrypted, refresh_token_encrypted, is_active, created_at, updated_at)
- [ ] Create `payment_methods` table (id, organization_id, user_id, provider_type, provider_payment_method_id, type ['card'|'apple_pay'|'google_pay'|'shop_pay'|'link'], last_four, brand, exp_month, exp_year, is_default, created_at)
- [ ] Create `transactions` table (id, organization_id, booking_id, user_id, provider_id, amount_cents, platform_fee_cents, provider_fee_cents, tax_cents, net_amount_cents, currency, status ['pending'|'processing'|'succeeded'|'failed'|'refunded'], provider_transaction_id, metadata, created_at, updated_at)
- [ ] Create `platform_fee_tiers` table (id, plan_type, customer_fee_percent, provider_fee_percent, min_fee_cents, created_at)
- [ ] Generate and run migrations successfully
- [ ] Typecheck passes

#### US-002: Stripe Connect integration for tenant onboarding
**Description:** As a tenant (business or individual), I want to connect my Stripe account so I can receive payments directly.

**Acceptance Criteria:**
- [ ] Implement Stripe Connect OAuth flow (Standard accounts for businesses, Express for individuals)
- [ ] Create `/api/payments/stripe/connect` endpoint to initiate OAuth
- [ ] Create `/api/payments/stripe/callback` endpoint to handle OAuth callback
- [ ] Store encrypted access tokens in `payment_providers` table
- [ ] Handle account verification status from Stripe
- [ ] Typecheck passes

#### US-003: Square OAuth integration for tenant onboarding
**Description:** As a tenant, I want to connect my Square account as an alternative to Stripe.

**Acceptance Criteria:**
- [ ] Implement Square OAuth flow
- [ ] Create `/api/payments/square/connect` endpoint to initiate OAuth
- [ ] Create `/api/payments/square/callback` endpoint to handle OAuth callback
- [ ] Store encrypted access tokens in `payment_providers` table
- [ ] Handle token refresh for Square's expiring tokens
- [ ] Typecheck passes

#### US-004: Platform default payment processing
**Description:** As a tenant who doesn't want to manage payment accounts, I want to use the platform's built-in payment processing.

**Acceptance Criteria:**
- [ ] Create platform Stripe Connect account for collecting payments
- [ ] When tenant selects "Platform Default", mark their payment_provider as type 'platform'
- [ ] Route payments through platform account with appropriate fee structure
- [ ] Platform holds funds and handles payouts to tenants (manual or scheduled)
- [ ] Typecheck passes

### Phase 2: Customer Payment Methods

#### US-005: Add credit card payment method
**Description:** As a customer, I want to save my credit card for future payments.

**Acceptance Criteria:**
- [ ] Integrate Stripe Elements or Square Web Payments SDK for secure card input
- [ ] Create `/api/payments/methods` POST endpoint to save payment method
- [ ] Tokenize card with appropriate provider based on tenant's configuration
- [ ] Store payment method reference (not card details) in database
- [ ] Display masked card info (last 4, brand, expiry)
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-006: Add Apple Pay payment method
**Description:** As a customer with an Apple device, I want to pay using Apple Pay for faster checkout.

**Acceptance Criteria:**
- [ ] Implement Apple Pay button using Stripe Payment Request API or Square
- [ ] Handle domain verification for Apple Pay
- [ ] Create payment method record when Apple Pay is authorized
- [ ] Show Apple Pay option only on supported devices/browsers
- [ ] Typecheck passes
- [ ] Verify in browser (Safari on macOS/iOS)

#### US-007: Add Google Pay payment method
**Description:** As a customer, I want to pay using Google Pay for faster checkout.

**Acceptance Criteria:**
- [ ] Implement Google Pay button using Stripe Payment Request API or Square
- [ ] Configure Google Pay merchant settings
- [ ] Create payment method record when Google Pay is authorized
- [ ] Show Google Pay option only on supported browsers
- [ ] Typecheck passes
- [ ] Verify in browser (Chrome)

#### US-008: Shop Pay integration
**Description:** As a customer, I want to use Shop Pay for one-click checkout with saved shipping/billing info.

**Acceptance Criteria:**
- [ ] Integrate Stripe's Shop Pay (requires Stripe account)
- [ ] Display Shop Pay button on checkout
- [ ] Handle Shop Pay authentication flow
- [ ] Fall back gracefully if Shop Pay unavailable
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-009: Link (Stripe) integration
**Description:** As a customer, I want to use Link for fast, secure checkout with saved payment info.

**Acceptance Criteria:**
- [ ] Enable Link in Stripe Payment Element configuration
- [ ] Allow customers to save info to Link during checkout
- [ ] Display Link authentication when email recognized
- [ ] Handle Link payment flow
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-010: Manage saved payment methods
**Description:** As a customer, I want to view, set default, and delete my saved payment methods.

**Acceptance Criteria:**
- [ ] Create `/api/payments/methods` GET endpoint to list user's payment methods
- [ ] Create `/api/payments/methods/:id` DELETE endpoint to remove payment method
- [ ] Create `/api/payments/methods/:id/default` POST endpoint to set default
- [ ] UI shows all saved methods with card brand icons
- [ ] Confirm before deleting payment method
- [ ] Typecheck passes
- [ ] Verify in browser

### Phase 3: Checkout & Transactions

#### US-011: Calculate transaction fees and taxes
**Description:** As a platform, I need to calculate the correct fees and taxes for each transaction.

**Acceptance Criteria:**
- [ ] Look up tenant's plan tier to determine fee percentages
- [ ] Calculate customer service fee (percentage of subtotal)
- [ ] Calculate provider platform fee (percentage of subtotal)
- [ ] Integrate with TaxJar or Avalara API for tax calculation based on location
- [ ] Return breakdown: subtotal, customer_fee, tax, total, provider_fee, net_to_provider
- [ ] Typecheck passes

#### US-012: Process one-time payment
**Description:** As a customer, I want to pay for a booking using my saved payment method or new card.

**Acceptance Criteria:**
- [ ] Create `/api/payments/charge` POST endpoint
- [ ] Accept booking_id, payment_method_id (or new card token), amount
- [ ] Route payment to correct provider based on tenant configuration
- [ ] For Stripe Connect: use `transfer_data` or `destination` for split payments
- [ ] For Square: create payment and handle split via platform
- [ ] Record transaction with full fee breakdown
- [ ] Update booking status on successful payment
- [ ] Handle payment failures gracefully with clear error messages
- [ ] Typecheck passes

#### US-013: Checkout UI for booking payment
**Description:** As a customer, I want a clear checkout flow showing price breakdown and payment options.

**Acceptance Criteria:**
- [ ] Display service details, date/time, provider info
- [ ] Show price breakdown: subtotal, service fee, tax, total
- [ ] List saved payment methods with option to add new
- [ ] Show Apple Pay/Google Pay buttons when available
- [ ] Display Shop Pay/Link options when applicable
- [ ] Confirm button with loading state
- [ ] Success/failure feedback
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-014: Payment confirmation and receipt
**Description:** As a customer, I want to receive confirmation and a receipt after payment.

**Acceptance Criteria:**
- [ ] Show confirmation page with transaction details
- [ ] Send email receipt with breakdown (subtotal, fees, tax, total)
- [ ] Include booking details in receipt
- [ ] Provide link to view booking in app
- [ ] Typecheck passes
- [ ] Verify in browser

### Phase 4: Subscriptions

#### US-015: Customer service subscriptions (packages)
**Description:** As a customer, I want to subscribe to recurring service packages (e.g., weekly dog walking).

**Acceptance Criteria:**
- [ ] Create `subscriptions` table (id, organization_id, user_id, plan_id, provider_subscription_id, status, current_period_start, current_period_end, canceled_at)
- [ ] Create `/api/subscriptions` POST endpoint to create subscription
- [ ] Integrate with Stripe Subscriptions or Square Subscriptions API
- [ ] Handle subscription webhooks (payment_succeeded, payment_failed, canceled)
- [ ] Create bookings automatically based on subscription schedule
- [ ] Typecheck passes

#### US-016: Manage customer subscriptions
**Description:** As a customer, I want to view, pause, and cancel my service subscriptions.

**Acceptance Criteria:**
- [ ] Create `/api/subscriptions` GET endpoint to list user's subscriptions
- [ ] Create `/api/subscriptions/:id/cancel` POST endpoint
- [ ] Create `/api/subscriptions/:id/pause` POST endpoint (if supported)
- [ ] UI shows active subscriptions with next billing date
- [ ] Confirm before canceling with info about what they'll lose
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-017: Tenant platform subscriptions (SaaS billing)
**Description:** As a tenant, I want to subscribe to a platform plan that determines my fee tier.

**Acceptance Criteria:**
- [ ] Create `tenant_subscriptions` table for platform billing
- [ ] Define plan tiers: Free, Professional, Business (with different fee rates)
- [ ] Integrate Stripe billing for platform subscriptions (separate from Connect)
- [ ] Update tenant's fee tier when subscription changes
- [ ] Handle upgrade/downgrade proration
- [ ] Typecheck passes

#### US-018: Tenant subscription management UI
**Description:** As a tenant, I want to manage my platform subscription and view my current plan.

**Acceptance Criteria:**
- [ ] Show current plan with fee rates
- [ ] Display usage/transaction volume
- [ ] Allow plan upgrade/downgrade
- [ ] Show billing history
- [ ] Manage billing payment method
- [ ] Typecheck passes
- [ ] Verify in browser

### Phase 5: Tenant Payment Settings

#### US-019: Payment provider selection UI
**Description:** As a tenant, I want to choose and connect my preferred payment provider.

**Acceptance Criteria:**
- [ ] Settings page showing payment configuration options
- [ ] Options: "Use Platform Default", "Connect Stripe", "Connect Square"
- [ ] Show connection status for each provider
- [ ] Allow switching providers (with warning about impact)
- [ ] Display current fee structure based on plan
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-020: Tenant payout configuration
**Description:** As a tenant using platform default processing, I want to configure how I receive payouts.

**Acceptance Criteria:**
- [ ] Create `payout_settings` table (organization_id, method ['bank'|'debit'], bank_account_token, payout_schedule ['daily'|'weekly'|'monthly'])
- [ ] UI to add bank account for payouts (via Stripe/Square)
- [ ] Select payout frequency
- [ ] Show pending/completed payouts
- [ ] Typecheck passes
- [ ] Verify in browser

#### US-021: Transaction history for tenants
**Description:** As a tenant, I want to view all transactions and my earnings.

**Acceptance Criteria:**
- [ ] Create `/api/tenant/transactions` endpoint with filters
- [ ] Show transaction list: date, customer, service, amount, fees, net
- [ ] Filter by date range, status
- [ ] Export to CSV
- [ ] Show summary: total volume, total fees, net earnings
- [ ] Typecheck passes
- [ ] Verify in browser

### Phase 6: Refunds & Disputes

#### US-022: Process refunds
**Description:** As a tenant or platform admin, I want to issue full or partial refunds.

**Acceptance Criteria:**
- [ ] Create `/api/payments/refund` POST endpoint
- [ ] Support full and partial refunds
- [ ] Reverse platform fees proportionally on refunds
- [ ] Update transaction status to 'refunded' or 'partially_refunded'
- [ ] Notify customer of refund
- [ ] Typecheck passes

#### US-023: Handle payment disputes
**Description:** As a platform, I need to handle chargebacks and disputes from payment providers.

**Acceptance Criteria:**
- [ ] Set up webhooks for dispute events (Stripe: charge.dispute.created)
- [ ] Create `disputes` table to track dispute status
- [ ] Notify tenant when dispute opened
- [ ] Provide UI for tenant to submit dispute evidence
- [ ] Track dispute resolution
- [ ] Typecheck passes

### Phase 7: Webhooks & Background Jobs

#### US-024: Payment provider webhooks
**Description:** As a platform, I need to handle webhooks from payment providers for async events.

**Acceptance Criteria:**
- [ ] Create `/api/webhooks/stripe` endpoint with signature verification
- [ ] Create `/api/webhooks/square` endpoint with signature verification
- [ ] Handle events: payment_intent.succeeded, payment_intent.failed, charge.refunded, charge.dispute.created
- [ ] Handle subscription events: invoice.paid, invoice.payment_failed, customer.subscription.deleted
- [ ] Update local records based on webhook events
- [ ] Implement idempotency to handle duplicate webhooks
- [ ] Typecheck passes

#### US-025: Payout processing job
**Description:** As a platform, I need to automatically process payouts to tenants using platform default.

**Acceptance Criteria:**
- [ ] Create background job to process pending payouts
- [ ] Calculate payout amount (transactions - fees - refunds)
- [ ] Initiate payout via Stripe/Square
- [ ] Track payout status
- [ ] Notify tenant when payout sent
- [ ] Typecheck passes

## Functional Requirements

### Payment Processing
- FR-1: Support Stripe Connect (Standard + Express accounts) for tenant payment processing
- FR-2: Support Square OAuth for tenant payment processing
- FR-3: Provide "Platform Default" option using platform's Stripe account
- FR-4: Accept credit/debit cards via Stripe Elements or Square Web Payments SDK
- FR-5: Accept Apple Pay on supported devices via Payment Request API
- FR-6: Accept Google Pay on supported browsers via Payment Request API
- FR-7: Integrate Shop Pay for Stripe-connected tenants
- FR-8: Enable Link (Stripe) for faster checkout

### Fee Structure
- FR-9: Implement split-fee model: customer service fee + provider platform fee
- FR-10: Calculate fees based on tenant's subscription tier:
  - Free tier: 3% customer fee + 20% provider fee
  - Professional: 2% customer fee + 15% provider fee
  - Business: 1% customer fee + 10% provider fee
- FR-11: Enforce minimum platform fee of $0.50 per transaction
- FR-12: Calculate and collect applicable sales tax via TaxJar/Avalara

### Multi-Tenancy
- FR-13: Route payments to correct provider based on tenant configuration
- FR-14: Isolate payment methods by organization (customers can have different methods per org)
- FR-15: Support tenant switching for customers with multiple org memberships
- FR-16: Handle platform payouts for tenants using default processing

### Subscriptions
- FR-17: Support customer subscriptions to service packages (recurring bookings)
- FR-18: Support tenant subscriptions to platform plans (SaaS billing)
- FR-19: Automatically create bookings from active customer subscriptions
- FR-20: Handle subscription lifecycle: create, update, pause, cancel, resume

### Security & Compliance
- FR-21: Never store raw card numbers - use tokenization only
- FR-22: Encrypt OAuth tokens at rest
- FR-23: Verify webhook signatures from payment providers
- FR-24: Implement PCI DSS compliance via hosted payment forms
- FR-25: Log all payment events for audit trail

## Non-Goals (Out of Scope)

- Cryptocurrency or blockchain payments
- Buy Now Pay Later (Klarna, Affirm, Afterpay) - future consideration
- Cash or check payments
- International payment methods (iDEAL, SEPA, etc.) - US only for now
- Tipping/gratuity system - future feature
- Multi-currency support - USD only for MVP
- Custom invoicing system
- ACH/bank transfer payments for customers

## Technical Considerations

### Architecture
- Payment provider integrations in `/crates/integrations/` module
- Abstract payment provider interface to support Stripe/Square interchangeably
- Use Stripe Payment Element for unified card + wallet UI
- Square Web Payments SDK as alternative

### Security
- Store OAuth tokens encrypted using AES-256
- Use environment variables for API keys
- Implement webhook signature verification
- Use Stripe's hosted fields to stay out of PCI scope

### Database
- All amounts stored in cents (integers) to avoid floating point issues
- Use database transactions for payment + booking updates
- Index on organization_id for all payment tables

### External Services
- Stripe Connect for primary processing
- Square for alternative processor
- TaxJar or Avalara for tax calculation
- Consider Plaid for bank account verification

### Error Handling
- Graceful degradation if tax service unavailable
- Clear error messages for declined cards
- Retry logic for transient failures
- Circuit breaker for external API calls

## Success Metrics

- Payment success rate > 95%
- Checkout abandonment < 30%
- Average checkout completion time < 60 seconds
- Zero PCI compliance violations
- Tenant payout accuracy 100%
- Customer support tickets related to payments < 5% of transactions

## Open Questions

1. Should we support holding funds in escrow until service completion?
2. What's the grace period for failed subscription payments before cancellation?
3. Should customers be able to tip providers? If so, does platform take a fee on tips?
4. Do we need to support promotional codes/discounts at checkout?
5. Should we implement a wallet/credits system for customer balances?
6. What's the minimum payout threshold for tenants using platform default?
7. How do we handle refunds when tenant has already received payout?

## Appendix: Fee Calculation Example

**Scenario:** Customer books a $100 dog walking service from a Professional-tier tenant.

```
Subtotal:                    $100.00
Customer Service Fee (2%):   +  $2.00
Tax (8.5% - calculated):     +  $8.67
─────────────────────────────────────
Customer Pays:               $110.67

Provider Platform Fee (15%): - $15.00
Payment Processing (~2.9%):  -  $3.21
─────────────────────────────────────
Net to Provider:             $ 81.79
Platform Revenue:            $ 13.88 ($2.00 + $15.00 - $3.21 processing)
```

## Appendix: Database Schema Overview

```
┌─────────────────────┐     ┌─────────────────────┐
│  payment_providers  │     │   payment_methods   │
├─────────────────────┤     ├─────────────────────┤
│ id                  │     │ id                  │
│ organization_id     │     │ organization_id     │
│ provider_type       │     │ user_id             │
│ account_id          │     │ provider_type       │
│ access_token_enc    │     │ provider_method_id  │
│ refresh_token_enc   │     │ type                │
│ is_active           │     │ last_four           │
│ created_at          │     │ brand               │
│ updated_at          │     │ is_default          │
└─────────────────────┘     └─────────────────────┘

┌─────────────────────┐     ┌─────────────────────┐
│    transactions     │     │    subscriptions    │
├─────────────────────┤     ├─────────────────────┤
│ id                  │     │ id                  │
│ organization_id     │     │ organization_id     │
│ booking_id          │     │ user_id             │
│ user_id             │     │ plan_id             │
│ provider_id         │     │ provider_sub_id     │
│ amount_cents        │     │ status              │
│ platform_fee_cents  │     │ current_period_start│
│ provider_fee_cents  │     │ current_period_end  │
│ tax_cents           │     │ canceled_at         │
│ net_amount_cents    │     └─────────────────────┘
│ status              │
│ provider_tx_id      │     ┌─────────────────────┐
│ metadata            │     │ platform_fee_tiers  │
└─────────────────────┘     ├─────────────────────┤
                            │ id                  │
                            │ plan_type           │
                            │ customer_fee_pct    │
                            │ provider_fee_pct    │
                            │ min_fee_cents       │
                            └─────────────────────┘
```
