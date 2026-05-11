#!/usr/bin/env bash
set -euo pipefail

PROJECT="/home/jared/Projects/FlossKeeper_Rust_v0_7"
VERSION="1.4.0"
TARGET_WIN="x86_64-pc-windows-gnu"
DIST="$PROJECT/dist"
LINUX_DIST="$DIST/linux"
WINDOWS_DIST="$DIST/windows"

cd "$PROJECT"

echo "== Building Linux release binary =="
cargo build --release

echo "== Creating Linux tar.gz package =="
rm -rf "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64"
mkdir -p "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64"

cp target/release/flosskeeper "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64/FlossKeeper"

cat > "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64/run.sh" <<'RUN'
#!/usr/bin/env bash
cd "$(dirname "$0")"
./FlossKeeper
RUN

chmod +x "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64/FlossKeeper"
chmod +x "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64/run.sh"

cd "$LINUX_DIST"
tar -czf "FlossKeeper-v${VERSION}-linux-x86_64.tar.gz" "FlossKeeper-v${VERSION}-linux-x86_64"

cd "$PROJECT"

echo "== Building Linux AppImage =="
if [ -f ./build_appimage.sh ]; then
    sed -i 's/VERSION="[^"]*"/VERSION="1.4.0"/' build_appimage.sh
    ./build_appimage.sh
else
    echo "ERROR: build_appimage.sh not found."
    echo "If you renamed it differently, run that script manually."
    exit 1
fi

echo "== Building Windows EXE and transfer ISO =="
if [ -f ./build_windows_gui_transfer.sh ]; then
    ./build_windows_gui_transfer.sh
else
    echo "ERROR: build_windows_gui_transfer.sh not found."
    exit 1
fi

mkdir -p "$WINDOWS_DIST"
cp "target/$TARGET_WIN/release/flosskeeper.exe" "$WINDOWS_DIST/FlossKeeper-v${VERSION}-x86_64.exe"

echo
echo "== Release files created =="
ls -lh "$WINDOWS_DIST/FlossKeeper-v${VERSION}-x86_64.exe"
ls -lh "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64.tar.gz"
ls -lh "$LINUX_DIST/FlossKeeper-v${VERSION}-x86_64.AppImage"

echo
echo "Done. Now test these:"
echo "$WINDOWS_DIST/FlossKeeper-v${VERSION}-x86_64.exe"
echo "$LINUX_DIST/FlossKeeper-v${VERSION}-linux-x86_64.tar.gz"
echo "$LINUX_DIST/FlossKeeper-v${VERSION}-x86_64.AppImage"
