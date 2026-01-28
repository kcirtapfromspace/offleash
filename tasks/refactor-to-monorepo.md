# Refactoring Plan: Unified Monorepo Structure

## Problem Analysis
The current repository structure contains confusion and duplication:
1.  **Duplicate "apps" directories:** `apps/` exists in the root, but `ralph/apps/` also exists.
2.  **Split Mobile/Web:** The iOS app is hidden inside `ralph/apps/ios`, while web apps are in the root `apps/`.
3.  **No Workspace Management:** The root `package.json` does not define `workspaces`, leading to inefficient dependency management for the JS/TS apps.

## Proposed Strategy
Consolidate all applications into a single `apps/` directory and formalize the monorepo structure for both Rust (already done via Cargo workspace) and JavaScript (via NPM Workspaces).

## Target Structure
```text
/
├── apps/                    # ALL frontend/mobile applications
│   ├── admin-dashboard/     # React/Svelte Admin Web
│   ├── customer-web/        # SvelteKit Customer Web
│   ├── platform-admin/      # Platform Admin Web
│   └── ios/                 # iOS Native App (Moved from ralph/)
├── crates/                  # Rust Backend Microservices (Existing)
│   ├── api/
│   ├── db/
│   └── ...
├── packages/                # (Future) Shared JS/TS libraries
├── scripts/                 # DevOps & Utility scripts
├── Cargo.toml               # Rust Workspace Root
└── package.json             # JS Workspace Root
```

## detailed Implementation Steps

### 1. File Relocation
- **Move iOS App:** Move `ralph/apps/ios` to `apps/ios` to place it alongside web apps.
- **Archive/Cleanup:** Delete the `ralph/` directory. The `customer-web` and `admin-dashboard` inside it are stale duplicates of the root versions.
    - *Note: `ralph/prd.json` indicates completed work, suggesting `ralph` was a temporary workspace. Its contents are safe to merge/delete.*

### 2. Workspace Configuration
- **Update `package.json`:** Add `"workspaces": ["apps/*", "packages/*"]` to the root `package.json`. This allows running `npm install` once in the root to install dependencies for all web apps.

### 3. Cleanup
- Remove references to `ralph` from documentation.
- Update `tasks/` to reflect the new structure.

## Benefits
- **Single Source of Truth:** No wondering if code is in `apps` or `ralph`.
- **Unified Tooling:** Run tests/builds for all apps from the root.
- **Shared Dependencies:** NPM workspaces reduce disk usage and ensure consistent versioning.
