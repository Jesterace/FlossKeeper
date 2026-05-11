#!/usr/bin/env bash
set -euo pipefail

APP="FlossKeeper"
BIN="flosskeeper"
VERSION="1.1.3"
ARCH="x86_64"

PROJECT="/home/jared/Projects/FlossKeeper_Rust_v0_7"
APPDIR="$PROJECT/FlossKeeper.AppDir"
DIST="$PROJECT/dist/linux"
APPIMAGE="$DIST/FlossKeeper-v${VERSION}-${ARCH}.AppImage"

cd "$PROJECT"

echo "== Building Linux release binary =="
cargo build --release

echo "== Creating AppDir =="
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$DIST"

cp "target/release/$BIN" "$APPDIR/usr/bin/$BIN"
chmod +x "$APPDIR/usr/bin/$BIN"

echo "== Creating AppRun =="
cat > "$APPDIR/AppRun" <<'APPRUN'
#!/usr/bin/env bash
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/flosskeeper" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

echo "== Creating desktop file =="
cat > "$APPDIR/flosskeeper.desktop" <<'DESKTOP'
[Desktop Entry]
Name=FlossKeeper
Comment=Track your cross-stitch floss stash
Exec=flosskeeper
Icon=flosskeeper
Terminal=false
Type=Application
Categories=Utility;
DESKTOP

cp "$APPDIR/flosskeeper.desktop" "$APPDIR/usr/share/applications/flosskeeper.desktop"

echo "== Creating simple SVG icon =="
cat > "$APPDIR/flosskeeper.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256" viewBox="0 0 256 256">
  <rect width="256" height="256" rx="48" fill="#2f4050"/>
  <rect x="54" y="56" width="148" height="144" rx="18" fill="#f5f7fa"/>
  <path d="M76 96h104M76 128h104M76 160h104" stroke="#2f4050" stroke-width="12" stroke-linecap="round"/>
  <circle cx="88" cy="96" r="8" fill="#4aa3df"/>
  <circle cx="88" cy="128" r="8" fill="#7ac943"/>
  <circle cx="88" cy="160" r="8" fill="#f15a5a"/>
</svg>
SVG

cp "$APPDIR/flosskeeper.svg" "$APPDIR/usr/share/icons/hicolor/scalable/apps/flosskeeper.svg"
ln -sf flosskeeper.svg "$APPDIR/.DirIcon"

echo "== Getting appimagetool if needed =="
if ! command -v appimagetool >/dev/null 2>&1; then
    mkdir -p "$HOME/.local/bin"
    wget -O "$HOME/.local/bin/appimagetool" \
      "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$HOME/.local/bin/appimagetool"
    export PATH="$HOME/.local/bin:$PATH"
fi

echo "== Building AppImage =="
rm -f "$APPIMAGE"
ARCH="$ARCH" appimagetool "$APPDIR" "$APPIMAGE"

chmod +x "$APPIMAGE"

echo
echo "== Built AppImage =="
ls -lh "$APPIMAGE"

echo
echo "== Test launch =="
"$APPIMAGE" || true

echo
echo "Done."
echo "AppImage is here:"
echo "$APPIMAGE"
