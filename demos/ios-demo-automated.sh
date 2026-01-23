#!/bin/bash
# Automated iOS Demo Runner
# Uses XCUITest to run automated demo flows with screenshots

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
IOS_PROJECT_DIR="$SCRIPT_DIR/../ralph/apps/ios"
SCREENSHOTS_DIR="$SCRIPT_DIR/ios-screenshots"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "🎬 OFFLEASH iOS App - Automated Demo"
echo "====================================="
echo ""

# Create output directory
mkdir -p "$SCREENSHOTS_DIR"

# Check for simulator
DEVICE_NAME="iPhone 17 Pro"
DEVICE_ID=$(xcrun simctl list devices available -j | jq -r ".devices | to_entries[] | .value[] | select(.name == \"$DEVICE_NAME\") | .udid" | head -1)

if [ -z "$DEVICE_ID" ]; then
    echo "❌ $DEVICE_NAME simulator not found"
    echo "Available simulators:"
    xcrun simctl list devices available | grep iPhone
    exit 1
fi

echo "📱 Using simulator: $DEVICE_NAME ($DEVICE_ID)"
echo ""

# Boot simulator if needed
BOOTED=$(xcrun simctl list devices booted -j | jq -r ".devices | to_entries[] | .value[] | select(.udid == \"$DEVICE_ID\") | .state")
if [ "$BOOTED" != "Booted" ]; then
    echo "🔄 Booting simulator..."
    xcrun simctl boot "$DEVICE_ID"
    sleep 5
fi

# Open Simulator app
open -a Simulator

# Start screen recording in background
RECORDING_FILE="$SCRIPT_DIR/ios-recordings/${TIMESTAMP}_automated_demo.mp4"
mkdir -p "$(dirname "$RECORDING_FILE")"
echo "🔴 Starting screen recording..."
xcrun simctl io "$DEVICE_ID" recordVideo "$RECORDING_FILE" &
RECORD_PID=$!

# Give recording time to start
sleep 2

# Build and run UI tests
echo ""
echo "🏗️ Building and running UI tests..."
echo ""

cd "$IOS_PROJECT_DIR"

# Run specific demo test
xcodebuild test \
    -project OFFLEASH.xcodeproj \
    -scheme OFFLEASH \
    -destination "platform=iOS Simulator,id=$DEVICE_ID" \
    -only-testing:OFFLEASHUITests/WalkerAppDemoTests/testWalkerAppDemo \
    -resultBundlePath "$SCRIPT_DIR/ios-test-results/${TIMESTAMP}" \
    2>&1 | xcbeautify || true

# Stop recording
echo ""
echo "⏹️ Stopping recording..."
kill -INT $RECORD_PID 2>/dev/null || true
wait $RECORD_PID 2>/dev/null || true

# Extract screenshots from test results
echo ""
echo "📸 Extracting screenshots..."
RESULT_BUNDLE="$SCRIPT_DIR/ios-test-results/${TIMESTAMP}.xcresult"
if [ -d "$RESULT_BUNDLE" ]; then
    xcrun xcresulttool get --path "$RESULT_BUNDLE" --format json > /tmp/results.json 2>/dev/null || true
    # Screenshots are embedded in the xcresult bundle
    echo "   Test results saved to: $RESULT_BUNDLE"
fi

echo ""
echo "✅ Demo complete!"
echo ""
echo "📁 Outputs:"
echo "   🎥 Recording: $RECORDING_FILE"
echo "   📊 Test Results: $RESULT_BUNDLE"
echo ""
echo "To view test results with screenshots:"
echo "   open $RESULT_BUNDLE"
