#!/bin/zsh

# Exit immediately if any command fails
set -e

echo "🚀 Starting EAS local Android build..."
eas build --platform android --local

echo "✅ Build finished. Searching for the latest generated AAB..."
# Get the most recent .aab file by modification time
AAB_FILE=$(ls -t ./*.aab 2>/dev/null | head -n 1)

if [ -z "$AAB_FILE" ]; then
    echo "❌ No .aab file found. Build may have failed."
    exit 1
fi

echo "📦 Found latest AAB: $AAB_FILE"

# Generate corresponding APKS file name
APKS_FILE="${AAB_FILE%.aab}.apks"

echo "⚙️ Creating APKS with bundletool..."
bundletool build-apks --bundle="$AAB_FILE" --output="$APKS_FILE"

echo "📲 Installing APKS on connected Android device/emulator..."
bundletool install-apks --apks="$APKS_FILE"

echo "🎉 Done!"
