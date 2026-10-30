#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --offline
APP="dist/Gradius 2.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/gradius2-native "$APP/Contents/MacOS/gradius2-native"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>gradius2-native</string>
<key>CFBundleIdentifier</key><string>fr.ptro.gradius2.rust</string>
<key>CFBundleName</key><string>Gradius 2</string>
<key>CFBundleDisplayName</key><string>Gradius 2</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.2.0</string>
<key>CFBundleVersion</key><string>3</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
PLIST
printf 'APPL????' > "$APP/Contents/PkgInfo"
codesign --force --sign - "$APP"
codesign --verify --deep --strict "$APP"
ditto -c -k --keepParent "$APP" dist/Gradius2-Mac-AppleSilicon.zip
echo "$PWD/$APP"
