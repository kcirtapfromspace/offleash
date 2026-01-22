# OFFLEASH Deployment Guide

## Render.com Deployment

### Prerequisites
- Render.com account with workspace created
- GitHub repository connected to Render

### Architecture Overview

| Service | Type | Plan | Cost |
|---------|------|------|------|
| offleash-api | Rust (Docker) | Starter | $7/mo |
| offleash-db | PostgreSQL 16 | Starter | $7/mo (free 90 days) |
| offleash-app | Node.js (SvelteKit) | Free | $0 (auto-sleeps) |
| offleash-admin | Node.js (SvelteKit) | Free | $0 (auto-sleeps) |
| offleash-platform | Node.js (SvelteKit) | Free | $0 (auto-sleeps) |

**Total: ~$14/mo** after database free trial

### Deployment Steps

#### Option 1: Blueprint Deployment (Recommended)

1. **Connect Repository**
   - Go to Render Dashboard → Blueprints
   - Click "New Blueprint Instance"
   - Select your GitHub repository
   - Render will detect `render.yaml` automatically

2. **Review Services**
   - Verify all 5 services are listed
   - Check environment variables are correctly mapped
   - The `JWT_SECRET` and `DATABASE_URL` are auto-generated

3. **Deploy**
   - Click "Apply"
   - Wait for database to provision first (~2-3 minutes)
   - API will build and deploy (~5-10 minutes for first build)
   - Frontend apps deploy last (~2-3 minutes each)

#### Option 2: Manual Service Creation

If you prefer manual setup:

1. **Create Database**
   ```
   Name: offleash-db
   Region: Oregon
   PostgreSQL Version: 16
   Plan: Starter
   ```

2. **Create API Service**
   ```
   Name: offleash-api
   Environment: Docker
   Dockerfile Path: ./Dockerfile
   Health Check: /health
   Plan: Starter

   Environment Variables:
   - DATABASE_URL: (from database internal URL)
   - JWT_SECRET: (generate secure random string)
   - RUST_LOG: info,tower_http=info,sqlx=warn
   - HOST: 0.0.0.0
   - PORT: 8080
   - ENVIRONMENT: test
   - BASE_DOMAIN: offleash.world
   - CORS_ORIGINS: (comma-separated list of frontend URLs)
   ```

3. **Create Frontend Services**
   For each app (customer-web, admin-dashboard, platform-admin):
   ```
   Name: offleash-{app/admin/platform}
   Environment: Node
   Build Command: cd apps/{app-name} && npm ci && npm run build
   Start Command: cd apps/{app-name} && node build
   Plan: Free

   Environment Variables:
   - NODE_ENV: production
   - PORT: 3000
   - PUBLIC_API_URL: https://offleash-api.onrender.com
   - ORIGIN: https://offleash-{name}.onrender.com
   ```

### Post-Deployment

1. **Run Database Migrations**
   - Go to offleash-api service → Shell
   - Migrations run automatically on startup via the API

2. **Verify Health**
   - Check API health: `https://offleash-api.onrender.com/health`
   - Check frontend apps load correctly

3. **Custom Domains (Optional)**
   For production with custom domains:

   | Service | Custom Domain |
   |---------|---------------|
   | offleash-api | api.offleash.world |
   | offleash-app | app.offleash.world |
   | offleash-admin | admin.offleash.world |
   | offleash-platform | platform.offleash.world |

   Update `CORS_ORIGINS` in API to include custom domains.

### Environment Variables Reference

#### API (offleash-api)
| Variable | Description | Example |
|----------|-------------|---------|
| DATABASE_URL | PostgreSQL connection string | Auto-set from database |
| JWT_SECRET | Secret for JWT token signing | Auto-generated |
| RUST_LOG | Logging level configuration | info,tower_http=info |
| HOST | Server bind address | 0.0.0.0 |
| PORT | Server port | 8080 |
| ENVIRONMENT | Deployment environment | test / production |
| BASE_DOMAIN | Base domain for multi-tenant | offleash.world |
| CORS_ORIGINS | Allowed CORS origins | Comma-separated URLs |

#### Frontend Apps
| Variable | Description | Example |
|----------|-------------|---------|
| NODE_ENV | Node environment | production |
| PORT | Server port | 3000 |
| PUBLIC_API_URL | API base URL | https://offleash-api.onrender.com |
| ORIGIN | App origin for CSRF | https://offleash-app.onrender.com |

### Troubleshooting

#### API Build Fails
- Check Dockerfile syntax
- Ensure Cargo.lock is committed
- Review build logs for missing dependencies

#### Database Connection Errors
- Verify DATABASE_URL is set correctly
- Check database is fully provisioned
- Ensure internal connection string is used (not external)

#### Frontend 502 Errors
- Free tier services auto-sleep after 15 min inactivity
- First request after sleep takes ~30 seconds
- Consider upgrading to Starter plan for always-on

#### CORS Errors
- Verify CORS_ORIGINS includes all frontend URLs
- Check for trailing slashes (don't include them)
- Restart API after updating CORS settings

### Scaling for Production

When ready for production:

1. **Upgrade Frontend Plans**
   - Change from Free to Starter ($7/mo each)
   - Removes auto-sleep behavior

2. **Add Custom Domains**
   - Configure DNS CNAME records
   - Add domains in Render dashboard
   - Update CORS_ORIGINS

3. **Environment Configuration**
   - Change ENVIRONMENT to `production`
   - Update BASE_DOMAIN to `offleash.pro`
   - Update all frontend ORIGIN values

4. **Database Scaling**
   - Upgrade to Standard plan for more connections
   - Enable connection pooling if needed
