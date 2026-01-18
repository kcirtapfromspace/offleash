#!/bin/bash
set -e

echo "=========================================="
echo "Dog Walker API - Local Dev Setup"
echo "=========================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check for required tools
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        echo "Please install $1 first"
        exit 1
    fi
    echo -e "${GREEN}✓${NC} $1 found"
}

echo ""
echo "Checking prerequisites..."
check_tool docker
check_tool kubectl
check_tool k3d
check_tool tilt

echo ""
echo "Creating k3d cluster..."
if k3d cluster list | grep -q "dog-walker"; then
    echo -e "${YELLOW}Cluster 'dog-walker' already exists${NC}"
    read -p "Delete and recreate? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        k3d cluster delete dog-walker
    else
        echo "Using existing cluster"
    fi
fi

if ! k3d cluster list | grep -q "dog-walker"; then
    echo "Creating new k3d cluster..."
    k3d cluster create dog-walker \
        --port "8080:80@loadbalancer" \
        --port "5433:5432@loadbalancer" \
        --wait
fi

echo ""
echo "Setting kubectl context..."
kubectl config use-context k3d-dog-walker

echo ""
echo "=========================================="
echo -e "${GREEN}Setup complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Run 'tilt up' to start the development environment"
echo "  2. Press 's' to open Tilt UI in browser"
echo "  3. API will be available at http://localhost:8080"
echo "  4. PostgreSQL at localhost:5433"
echo ""
echo "Useful commands:"
echo "  tilt down          - Stop all services"
echo "  k3d cluster delete dog-walker - Remove the cluster"
echo "=========================================="
