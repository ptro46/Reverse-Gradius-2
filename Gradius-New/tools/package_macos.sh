#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p dist
python3 tools/make_icon.py
iconutil -c icns dist/Astra.iconset -o dist/Astra.icns
cargo build --release --offline
APP="dist/Astra Fracture.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/astra-fracture "$APP/Contents/MacOS/astra-fracture"
cp dist/Astra.icns "$APP/Contents/Resources/Astra.icns"
cp README.md VALIDATION.md "$APP/Contents/Resources/"
mkdir -p "$APP/Contents/Resources/Licenses"
cp assets/OFL-*.txt "$APP/Contents/Resources/Licenses/"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>astra-fracture</string>
<key>CFBundleIdentifier</key><string>fr.ptro.astrafracture</string>
<key>CFBundleName</key><string>Astra Fracture</string>
<key>CFBundleDisplayName</key><string>ASTRA FRACTURE</string>
<key>CFBundleIconFile</key><string>Astra.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.2</string>
<key>CFBundleVersion</key><string>3</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
</dict></plist>
PLIST
printf 'APPL????' > "$APP/Contents/PkgInfo"
codesign --force --sign - "$APP"
codesign --verify --deep --strict "$APP"
ditto -c -k --keepParent "$APP" dist/Astra-Fracture-Mac-AppleSilicon-v0.1.2.zip
echo "$PWD/$APP"
