#!/bin/bash
# WezBrowser Release Build Script

set -e

VERSION="${1:-0.1.0}"
APP_NAME="WezBrowser"
BUNDLE_ID="com.wezbrowser.app"

echo "Building WezBrowser v${VERSION}..."

# Build release binary
cargo build --release -p wezterm-gui

# Create app bundle directory structure
APP_DIR="target/release/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy binary
cp target/release/wezterm-gui "${MACOS_DIR}/"

# Copy icon if exists
if [ -f "assets/macos/WezTerm.app/Contents/Resources/terminal.icns" ]; then
    cp assets/macos/WezTerm.app/Contents/Resources/terminal.icns "${RESOURCES_DIR}/"
fi

# Create Info.plist
cat > "${CONTENTS_DIR}/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>wezterm-gui</string>
    <key>CFBundleIdentifier</key>
    <string>com.wezbrowser.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>WezBrowser</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>VERSION_PLACEHOLDER</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleIconFile</key>
    <string>terminal.icns</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleDisplayName</key>
    <string>WezBrowser</string>
    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsArbitraryLoads</key>
        <true/>
        <key>NSAllowsLocalNetworking</key>
        <true/>
        <key>NSAllowsArbitraryLoadsInWebContent</key>
        <true/>
    </dict>
</dict>
</plist>
EOF

# Replace version placeholder
sed -i '' "s/VERSION_PLACEHOLDER/${VERSION}/g" "${CONTENTS_DIR}/Info.plist"

# Ad-hoc sign the app
codesign --force --deep --sign - "${APP_DIR}"

echo "App bundle created at: ${APP_DIR}"

# Create DMG
DMG_NAME="${APP_NAME}-${VERSION}-macos.dmg"
echo "Creating DMG: ${DMG_NAME}..."

# Create temporary directory for DMG contents
DMG_TMP="target/release/dmg_tmp"
rm -rf "${DMG_TMP}"
mkdir -p "${DMG_TMP}"
cp -R "${APP_DIR}" "${DMG_TMP}/"

# Create symbolic link to Applications
ln -s /Applications "${DMG_TMP}/Applications"

# Create DMG
hdiutil create -volname "${APP_NAME}" -srcfolder "${DMG_TMP}" -ov -format UDZO "target/release/${DMG_NAME}"

# Cleanup
rm -rf "${DMG_TMP}"

echo ""
echo "=== Build Complete ==="
echo "App: ${APP_DIR}"
echo "DMG: target/release/${DMG_NAME}"
echo ""
echo "To distribute:"
echo "1. Upload DMG to GitHub Releases"
echo "2. Users can install by dragging app to Applications"
echo ""
echo "Note: Users will need to right-click and select 'Open' on first launch"
echo "      to bypass Gatekeeper (unsigned app warning)"
