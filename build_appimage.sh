#!/usr/bin/env bash
set -e

APP_NAME="FlossKeeper"
APP_ID="com.jesterace.FlossKeeper"
BIN_NAME="flosskeeper"
VERSION="1.1.2"
ARCH="$(uname -m)"

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
APPDIR="$PROJECT_DIR/${APP_NAME}.AppDir"
DIST_DIR="$PROJECT_DIR/dist"
APPIMAGETOOL="$PROJECT_DIR/appimagetool-x86_64.AppImage"

ICON_SRC="$PROJECT_DIR/assets/icons/flosskeeper.png"

if [ "$ARCH" != "x86_64" ]; then
    echo "This script currently expects x86_64."
    echo "Detected: $ARCH"
    exit 1
fi

echo "Building release binary..."
cargo build --release

echo "Preparing AppDir..."
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$DIST_DIR"

cp "$PROJECT_DIR/target/release/$BIN_NAME" "$APPDIR/usr/bin/$BIN_NAME"

if [ ! -f "$ICON_SRC" ]; then
    echo "Project icon not found at:"
    echo "  $ICON_SRC"
    echo
    echo "Trying local installed icon..."
    if [ -f "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" ]; then
        mkdir -p "$PROJECT_DIR/assets/icons"
        cp "$HOME/.local/share/icons/hicolor/256x256/apps/flosskeeper.png" "$ICON_SRC"
    else
        echo "No icon found. Put flosskeeper.png in assets/icons/ first."
        exit 1
    fi
fi

cp "$ICON_SRC" "$APPDIR/usr/share/icons/hicolor/256x256/apps/flosskeeper.png"

cat > "$APPDIR/usr/share/applications/$APP_ID.desktop" <<EOF
[Desktop Entry]
Name=FlossKeeper
Comment=Cross-stitch floss stash tracker
Exec=flosskeeper
Icon=flosskeeper
Terminal=false
Type=Application
Categories=Utility;Graphics;
StartupNotify=true
StartupWMClass=$APP_ID
EOF

# appimagetool expects these at the AppDir root too.
cp "$APPDIR/usr/share/applications/$APP_ID.desktop" "$APPDIR/$APP_ID.desktop"
cp "$ICON_SRC" "$APPDIR/flosskeeper.png"
cp "$ICON_SRC" "$APPDIR/.DirIcon"

cat > "$APPDIR/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/flosskeeper" "$@"
EOF

chmod +x "$APPDIR/AppRun"
chmod +x "$APPDIR/usr/bin/$BIN_NAME"

if command -v strip >/dev/null 2>&1; then
    strip "$APPDIR/usr/bin/$BIN_NAME" || true
fi

if [ ! -f "$APPIMAGETOOL" ]; then
    echo "Downloading appimagetool..."
    curl -L \
        -o "$APPIMAGETOOL" \
        "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
fi

OUT="$DIST_DIR/FlossKeeper-v${VERSION}-x86_64.AppImage"

echo "Creating AppImage..."
ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$APPIMAGETOOL" "$APPDIR" "$OUT"

chmod +x "$OUT"

echo
echo "Done."
echo "Created:"
echo "  $OUT"
echo
echo "Run it with:"
echo "  $OUT"
