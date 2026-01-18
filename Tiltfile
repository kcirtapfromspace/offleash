# -*- mode: Python -*-
# Dog Walker API - Tiltfile for local Kubernetes development
#
# Prerequisites:
#   - Docker
#   - k3d (or kind)
#   - tilt
#
# Quick start:
#   k3d cluster create dog-walker --port "8080:80@loadbalancer"
#   tilt up

# ============================================================================
# Configuration
# ============================================================================

# Allow building on k3d's registry
allow_k8s_contexts(['k3d-dog-walker'])

# Load extensions
load('ext://restart_process', 'docker_build_with_restart')
load('ext://namespace', 'namespace_create')

# ============================================================================
# Namespace
# ============================================================================
namespace_create('dog-walker')

# ============================================================================
# Kubernetes Resources
# ============================================================================

# Apply base resources (namespace created by namespace_create above)
k8s_yaml('k8s/base/postgres.yaml')
k8s_yaml('k8s/base/api.yaml')

# Apply dev overlays
k8s_yaml('k8s/dev/secrets.yaml')
k8s_yaml('k8s/dev/configmap.yaml')

# ============================================================================
# Docker Build - API
# ============================================================================

# Option 1: Full Docker build (slower, but production-like)
# docker_build(
#     'dog-walker-api',
#     '.',
#     dockerfile='Dockerfile',
#     only=['Cargo.toml', 'Cargo.lock', 'crates/', 'migrations/'],
# )

# Option 2: Live update with cargo-watch (faster iteration)
# This builds a dev image and syncs code changes
docker_build_with_restart(
    'dog-walker-api',
    '.',
    dockerfile='Dockerfile.dev',
    entrypoint=['/app/run-dev.sh'],
    live_update=[
        # Sync source code
        sync('./crates', '/app/crates'),
        sync('./Cargo.toml', '/app/Cargo.toml'),
        sync('./Cargo.lock', '/app/Cargo.lock'),
        sync('./migrations', '/app/migrations'),
        # Trigger rebuild
        run('cd /app && cargo build --bin api', trigger=['./crates', './Cargo.toml']),
    ],
)

# ============================================================================
# Resource Configuration
# ============================================================================

# PostgreSQL
k8s_resource(
    'postgres',
    port_forwards=['5433:5432'],  # Local port 5433 -> postgres 5432
    labels=['database'],
)

# API
k8s_resource(
    'api',
    port_forwards=['8080:8080'],
    resource_deps=['postgres'],
    labels=['backend'],
)

# ============================================================================
# Local Resources (optional helpers)
# ============================================================================

# Run migrations manually
local_resource(
    'run-migrations',
    cmd='kubectl apply -f k8s/base/migrations-job.yaml && kubectl wait --for=condition=complete job/run-migrations -n dog-walker --timeout=60s',
    resource_deps=['postgres'],
    labels=['database'],
    auto_init=False,  # Don't run automatically
)

# Database shell
local_resource(
    'psql',
    cmd='kubectl exec -it -n dog-walker postgres-0 -- psql -U dogwalker -d dog_walker',
    resource_deps=['postgres'],
    labels=['database'],
    auto_init=False,
)

# ============================================================================
# UI Configuration
# ============================================================================

# Update UI settings
update_settings(max_parallel_updates=2, k8s_upsert_timeout_secs=120)
