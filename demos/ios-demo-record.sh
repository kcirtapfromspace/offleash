#!/bin/bash
# iOS App Demo Recording Script
# Records the iOS simulator while you manually demo the app

DEMO_DIR="$(dirname "$0")"
SCREENSHOTS_DIR="$DEMO_DIR/ios-screenshots"
RECORDINGS_DIR="$DEMO_DIR/ios-recordings"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$SCREENSHOTS_DIR"
mkdir -p "$RECORDINGS_DIR"

# Find a booted simulator or boot one
DEVICE_ID=$(xcrun simctl list devices booted -j | jq -r '.devices | to_entries[] | .value[] | select(.state == "Booted") | .udid' | head -1)

if [ -z "$DEVICE_ID" ]; then
    echo "🔄 No simulator running. Booting iPhone 17 Pro..."
    DEVICE_ID=$(xcrun simctl list devices available -j | jq -r '.devices | to_entries[] | .value[] | select(.name == "iPhone 17 Pro") | .udid' | head -1)

    if [ -z "$DEVICE_ID" ]; then
        echo "❌ iPhone 17 Pro simulator not found. Available devices:"
        xcrun simctl list devices available
        exit 1
    fi

    xcrun simctl boot "$DEVICE_ID"
    sleep 3
fi

echo "📱 Using simulator: $DEVICE_ID"
echo ""

# Open Simulator app
open -a Simulator

echo "🎬 iOS Demo Recording"
echo "===================="
echo ""
echo "Commands:"
echo "  s - Take screenshot"
echo "  r - Start/stop recording"
echo "  q - Quit"
echo ""

RECORDING_PID=""
RECORDING_FILE=""

take_screenshot() {
    local name="${1:-screenshot_$(date +%H%M%S)}"
    local file="$SCREENSHOTS_DIR/${TIMESTAMP}_${name}.png"
    xcrun simctl io "$DEVICE_ID" screenshot "$file"
    echo "📸 Screenshot saved: $file"
}

start_recording() {
    if [ -n "$RECORDING_PID" ]; then
        echo "⚠️  Already recording!"
        return
    fi
    RECORDING_FILE="$RECORDINGS_DIR/${TIMESTAMP}_demo.mp4"
    xcrun simctl io "$DEVICE_ID" recordVideo "$RECORDING_FILE" &
    RECORDING_PID=$!
    echo "🔴 Recording started: $RECORDING_FILE"
}

stop_recording() {
    if [ -z "$RECORDING_PID" ]; then
        echo "⚠️  Not recording!"
        return
    fi
    kill -INT $RECORDING_PID 2>/dev/null
    wait $RECORDING_PID 2>/dev/null
    echo "⏹️  Recording stopped: $RECORDING_FILE"
    RECORDING_PID=""
    RECORDING_FILE=""
}

cleanup() {
    if [ -n "$RECORDING_PID" ]; then
        stop_recording
    fi
    echo ""
    echo "✅ Demo complete!"
    echo "📸 Screenshots: $SCREENSHOTS_DIR"
    echo "🎥 Recordings: $RECORDINGS_DIR"
}

trap cleanup EXIT

# Main loop
while true; do
    read -n 1 -s -p "" key
    case $key in
        s|S)
            read -p "Screenshot name (or Enter for auto): " name
            take_screenshot "$name"
            ;;
        r|R)
            if [ -z "$RECORDING_PID" ]; then
                start_recording
            else
                stop_recording
            fi
            ;;
        q|Q)
            break
            ;;
        *)
            ;;
    esac
done
