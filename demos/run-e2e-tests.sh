#!/bin/bash
# =====================================================
# OFFLEASH iOS E2E Test Runner
# =====================================================
# Runs comprehensive end-to-end tests for the iOS app
#
# Prerequisites:
# 1. Backend API running: docker compose up -d
# 2. Database seeded: docker compose exec db psql -U offleash -d offleash -f /demos/seed-test-data.sql
# =====================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$SCRIPT_DIR/../ralph/apps/ios"
RESULTS_DIR="$SCRIPT_DIR/e2e-test-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   OFFLEASH iOS E2E Test Runner           ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""

# Parse arguments
RUN_ALL=false
SPECIFIC_TEST=""
SEED_DATA=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            RUN_ALL=true
            shift
            ;;
        --seed)
            SEED_DATA=true
            shift
            ;;
        --test)
            SPECIFIC_TEST="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --all           Run all E2E tests"
            echo "  --seed          Seed test data before running tests"
            echo "  --test NAME     Run specific test (e.g., testCompleteUserJourney)"
            echo "  --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 --all                        # Run all tests"
            echo "  $0 --seed --all                 # Seed data and run all tests"
            echo "  $0 --test testLoginFlow         # Run specific test"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Default to running all tests if no specific test given
if [ -z "$SPECIFIC_TEST" ]; then
    RUN_ALL=true
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

# Check for simulator
DEVICE_NAME="iPhone 17 Pro"
DEVICE_ID=$(xcrun simctl list devices available -j | jq -r ".devices | to_entries[] | .value[] | select(.name == \"$DEVICE_NAME\") | .udid" | head -1)

if [ -z "$DEVICE_ID" ]; then
    # Try iPhone 16 Pro as fallback
    DEVICE_NAME="iPhone 16 Pro"
    DEVICE_ID=$(xcrun simctl list devices available -j | jq -r ".devices | to_entries[] | .value[] | select(.name == \"$DEVICE_NAME\") | .udid" | head -1)
fi

if [ -z "$DEVICE_ID" ]; then
    echo -e "${RED}❌ No suitable iPhone simulator found${NC}"
    echo "Available simulators:"
    xcrun simctl list devices available | grep iPhone | head -10
    exit 1
fi

echo -e "${GREEN}✓${NC} Using simulator: $DEVICE_NAME ($DEVICE_ID)"

# Check API health
echo ""
echo "Checking API health..."
API_URL="${API_BASE_URL:-http://localhost:8080}"
if curl -s -f "$API_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} API is healthy at $API_URL"
else
    echo -e "${YELLOW}⚠${NC} API not responding at $API_URL"
    echo "  Tests may fail if API is required."
    echo "  Start the API with: docker compose up -d"
fi

# Seed data if requested
if [ "$SEED_DATA" = true ]; then
    echo ""
    echo "Seeding test data..."
    if docker compose exec -T db psql -U offleash -d offleash -f /demos/seed-test-data.sql > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} Test data seeded successfully"
    else
        echo -e "${YELLOW}⚠${NC} Failed to seed data (database may not be running)"
    fi
fi

# Boot simulator if needed
BOOTED=$(xcrun simctl list devices booted -j | jq -r ".devices | to_entries[] | .value[] | select(.udid == \"$DEVICE_ID\") | .state")
if [ "$BOOTED" != "Booted" ]; then
    echo ""
    echo "Booting simulator..."
    xcrun simctl boot "$DEVICE_ID" 2>/dev/null || true
    sleep 3
fi

# Build test target
echo ""
echo "Building tests..."
cd "$PROJECT_DIR"

if ! xcodebuild build-for-testing \
    -project OFFLEASH.xcodeproj \
    -scheme OFFLEASH \
    -destination "platform=iOS Simulator,id=$DEVICE_ID" \
    -quiet 2>&1 | grep -v "warning:"; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓${NC} Build succeeded"

# Run tests
echo ""
echo -e "${BLUE}Running E2E tests...${NC}"
echo ""

TEST_FILTER=""
if [ -n "$SPECIFIC_TEST" ]; then
    TEST_FILTER="-only-testing:OFFLEASHUITests/E2ETests/$SPECIFIC_TEST"
elif [ "$RUN_ALL" = true ]; then
    TEST_FILTER="-only-testing:OFFLEASHUITests/E2ETests"
fi

RESULT_BUNDLE="$RESULTS_DIR/${TIMESTAMP}.xcresult"

# Run with xcbeautify if available, otherwise raw output
if command -v xcbeautify &> /dev/null; then
    xcodebuild test \
        -project OFFLEASH.xcodeproj \
        -scheme OFFLEASH \
        -destination "platform=iOS Simulator,id=$DEVICE_ID" \
        $TEST_FILTER \
        -resultBundlePath "$RESULT_BUNDLE" \
        2>&1 | xcbeautify
    TEST_EXIT_CODE=${PIPESTATUS[0]}
else
    xcodebuild test \
        -project OFFLEASH.xcodeproj \
        -scheme OFFLEASH \
        -destination "platform=iOS Simulator,id=$DEVICE_ID" \
        $TEST_FILTER \
        -resultBundlePath "$RESULT_BUNDLE" \
        2>&1 | grep -E "(Test Case|passed|failed|error:|✓|✗)"
    TEST_EXIT_CODE=${PIPESTATUS[0]}
fi

echo ""
echo "═══════════════════════════════════════════"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ ALL E2E TESTS PASSED${NC}"
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
fi

echo ""
echo "📁 Results: $RESULT_BUNDLE"
echo ""
echo "To view detailed results with screenshots:"
echo "  open $RESULT_BUNDLE"
echo ""

exit $TEST_EXIT_CODE
