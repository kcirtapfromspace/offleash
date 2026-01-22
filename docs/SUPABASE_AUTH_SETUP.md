# Supabase Auth Setup Guide

This guide walks through setting up Supabase Auth for OAuth (Google, Apple) and Passkey authentication.

## Prerequisites

- Supabase account (free tier works)
- Google Cloud Console access (for Google OAuth)
- Apple Developer account (for Apple Sign-In)

## Step 1: Create Supabase Project

1. Go to [supabase.com](https://supabase.com) and create a new project
2. Note down these values from Project Settings → API:
   - **Project URL**: `https://xxx.supabase.co`
   - **Anon public key**: `eyJ...` (safe for frontend)
   - **JWT Secret**: Found in Project Settings → API → JWT Settings

## Step 2: Configure Google OAuth

### In Google Cloud Console:

1. Go to [Google Cloud Console](https://console.cloud.google.com)
2. Create or select a project
3. Go to APIs & Services → Credentials
4. Create OAuth 2.0 Client ID:
   - Application type: Web application
   - Name: OFFLEASH
   - Authorized JavaScript origins:
     - `https://offleash-app.onrender.com`
     - `http://localhost:5173` (for local dev)
   - Authorized redirect URIs:
     - `https://xxx.supabase.co/auth/v1/callback`
5. Copy the Client ID and Client Secret

### In Supabase Dashboard:

1. Go to Authentication → Providers → Google
2. Enable Google provider
3. Paste Client ID and Client Secret
4. Save

## Step 3: Configure Apple Sign-In

### In Apple Developer Portal:

1. Go to [developer.apple.com](https://developer.apple.com)
2. Certificates, Identifiers & Profiles → Identifiers
3. Create a new App ID:
   - Enable "Sign in with Apple"
4. Create a Services ID:
   - Identifier: `com.offleash.web`
   - Enable "Sign in with Apple"
   - Configure domains:
     - Domain: `offleash-app.onrender.com`
     - Return URL: `https://xxx.supabase.co/auth/v1/callback`
5. Create a Key:
   - Enable "Sign in with Apple"
   - Download the `.p8` key file

### In Supabase Dashboard:

1. Go to Authentication → Providers → Apple
2. Enable Apple provider
3. Enter:
   - Service ID (Bundle ID): Your Services ID
   - Team ID: From Apple Developer account
   - Key ID: From the key you created
   - Private Key: Contents of the `.p8` file
4. Save

## Step 4: Configure Environment Variables

### Customer Web (`apps/customer-web/.env`):

```env
PUBLIC_SUPABASE_URL=https://xxx.supabase.co
PUBLIC_SUPABASE_ANON_KEY=eyJ...your-anon-key
```

### API (Render Dashboard or `.env`):

```env
SUPABASE_JWT_SECRET=your-jwt-secret-from-supabase
```

### In Render Dashboard:

1. Go to offleash-app service → Environment
2. Add:
   - `PUBLIC_SUPABASE_URL` = your Supabase project URL
   - `PUBLIC_SUPABASE_ANON_KEY` = your anon key

3. Go to offleash-api service → Environment
4. Add:
   - `SUPABASE_JWT_SECRET` = your JWT secret from Supabase

## Step 5: Configure Redirect URLs in Supabase

1. Go to Authentication → URL Configuration
2. Set Site URL: `https://offleash-app.onrender.com`
3. Add Redirect URLs:
   - `https://offleash-app.onrender.com/auth/callback`
   - `http://localhost:5173/auth/callback` (for local dev)
   - `offleash://auth/callback` (for iOS app deep link)

## Step 6: Test the Integration

1. Deploy the updated code to Render
2. Go to `https://offleash-app.onrender.com/login`
3. Click "Continue with Google"
4. Complete the OAuth flow
5. You should be redirected back and logged in

## iOS App Setup

For the iOS app, you'll need to:

1. Add Supabase Swift SDK via SPM:
   ```
   https://github.com/supabase/supabase-swift
   ```

2. Configure URL scheme for OAuth callback:
   - In Xcode: Target → Info → URL Types
   - Add: `offleash` (to handle `offleash://` deep links)

3. Add Associated Domains for Apple Sign-In (if using universal links)

## Passkey Setup (Future)

Passkeys require additional setup:

1. In Supabase, enable WebAuthn in Authentication settings
2. Add the relying party ID (your domain)
3. Implement the WebAuthn registration/authentication UI

## Troubleshooting

### "Invalid JWT" errors
- Ensure `SUPABASE_JWT_SECRET` matches the secret in Supabase dashboard
- Check that the token hasn't expired

### OAuth redirect fails
- Verify redirect URLs are correctly configured in both Supabase and OAuth provider
- Check browser console for CORS errors

### Apple Sign-In fails on web
- Apple Sign-In on web requires HTTPS
- Ensure Services ID is correctly configured with your domain

## Architecture

```
User clicks "Sign in with Google"
         ↓
SvelteKit redirects to Supabase Auth
         ↓
Supabase redirects to Google OAuth
         ↓
User authenticates with Google
         ↓
Google redirects back to Supabase
         ↓
Supabase creates session, redirects to /auth/callback
         ↓
SvelteKit exchanges code for session
         ↓
Supabase JWT stored in cookie
         ↓
API verifies Supabase JWT on requests
```
