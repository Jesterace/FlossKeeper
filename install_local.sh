#!/usr/bin/env bash
set -e

APP_NAME="flosskeeper"
APP_ID="com.jesterace.FlossKeeper"
PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Building FlossKeeper release..."
cargo build --release

echo "Installing binary..."
mkdir -p "$HOME/.local/bin"
cp "$PROJECT_DIR/target/release/flosskeeper" "$HOME/.local/bin/flosskeeper"
chmod +x "$HOME/.local/bin/flosskeeper"

echo "Installing icons..."
mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps"
mkdir -p "$HOME/.local/share/icons/hicolor/64x64/apps"
mkdir -p "$HOME/.local/share/icons/hicolor/48x48/apps"

cp "$PROJECT_DIR/assets/icons/flosskeeper.png" "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png"

if command -v magick >/dev/null 2>&1; then
    magick "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" -resize 128x128 "$HOME/.local/share/icons/hicolor/128x128/apps/flosskeeper.png"
    magick "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" -resize 64x64 "$HOME/.local/share/icons/hicolor/64x64/apps/flosskeeper.png"
    magick "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" -resize 48x48 "$HOME/.local/share/icons/hicolor/48x48/apps/flosskeeper.png"
elif command -v convert >/dev/null 2>&1; then
    convert "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" -resize 128x128 "$HOME/.local/share/icons/hicolor/128x128/apps/flosskeeper.png"
    convert "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" -resize 64x64 "$HOME/.local/share/icons/hicolor/64x64/apps/flosskeeper.png"
    convert "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" -resize 48x48 "$HOME/.local/share/icons/hicolor/48x48/apps/flosskeeper.png"
fi

echo "Installing desktop launcher..."
mkdir -p "$HOME/.local/share/applications"

cat > "$HOME/.local/share/applications/${APP_ID}.desktop" <<EOF
[Desktop Entry]
Name=FlossKeeper
Comment=Cross-stitch floss stash tracker
Exec=$HOME/.local/bin/flosskeeper
Icon=flosskeeper
Terminal=false
Type=Application
Categories=Utility;Graphics;
StartupNotify=true
StartupWMClass=$APP_ID
EOF

echo "Refreshing desktop database..."
gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
kbuildsycoca6 --noincremental 2>/dev/null || kbuildsycoca5 --noincremental 2>/dev/null || true

echo
echo "FlossKeeper installed locally."
echo "You should now find it in your app menu."
