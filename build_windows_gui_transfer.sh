#!/usr/bin/env bash
set -euo pipefail

PROJECT="/home/jared/Projects/FlossKeeper_Rust_v0_7"
TARGET="x86_64-pc-windows-gnu"
TRANSFER_DIR="$HOME/transfer-iso"
ISO="$HOME/VMs/flosskeeper-transfer.iso"
VM_CONF="$HOME/VMs/quickemu/windows-11.conf"
EXE="$PROJECT/target/$TARGET/release/flosskeeper.exe"

cd "$PROJECT"

echo "== Forcing Windows GUI subsystem =="

# Add Rust crate-level subsystem attribute to every Rust file with fn main()
while IFS= read -r file; do
    if ! grep -q 'windows_subsystem = "windows"' "$file"; then
        tmpfile="$(mktemp)"
        printf '#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]\n\n' > "$tmpfile"
        cat "$file" >> "$tmpfile"
        mv "$tmpfile" "$file"
        echo "Patched $file"
    else
        echo "Already patched $file"
    fi
done < <(grep -RIl 'fn main' src)

# Force the MinGW linker to build a GUI app, not a console app.
mkdir -p .cargo
cat > .cargo/config.toml <<'CFG'
[target.x86_64-pc-windows-gnu]
rustflags = ["-C", "link-args=-mwindows"]
CFG

echo "== Making sure eframe uses Glow/OpenGL =="
sed -i 's/"wgpu"/"glow"/g' Cargo.toml
sed -i 's/eframe::Renderer::Wgpu/eframe::Renderer::Glow/g' src/*.rs 2>/dev/null || true

echo "== Clean rebuild =="
cargo clean
cargo build --release --target "$TARGET"

echo "== Checking Windows subsystem =="
if command -v x86_64-w64-mingw32-objdump >/dev/null 2>&1; then
    x86_64-w64-mingw32-objdump -x "$EXE" | grep -i 'Subsystem' || true
elif command -v objdump >/dev/null 2>&1; then
    objdump -x "$EXE" | grep -i 'Subsystem' || true
else
    echo "objdump not found; skipping subsystem check."
fi

echo "== Creating transfer ISO with obvious new name =="
rm -rf "$TRANSFER_DIR"
mkdir -p "$TRANSFER_DIR"
cp "$EXE" "$TRANSFER_DIR/FK-GUI.exe"

if ! command -v xorriso >/dev/null 2>&1; then
    echo "ERROR: xorriso is not installed."
    echo "Install it with:"
    echo "sudo pacman -S libisoburn"
    exit 1
fi

xorriso -as mkisofs -J -R -V FKGUI -o "$ISO" "$TRANSFER_DIR"

echo "== Pointing Quickemu Windows VM at transfer ISO =="
if [ -f "$VM_CONF" ]; then
    cp "$VM_CONF" "$VM_CONF.bak"
    if grep -q '^fixed_iso=' "$VM_CONF"; then
        sed -i "s|^fixed_iso=.*|fixed_iso=\"$ISO\"|" "$VM_CONF"
    else
        echo "fixed_iso=\"$ISO\"" >> "$VM_CONF"
    fi
fi

echo
echo "Done."
echo "Launch Windows with:"
echo "cd ~/VMs/quickemu && quickemu --vm windows-11.conf"
echo
echo "In Windows, open This PC > CD drive FKGUI > copy FK-GUI.exe to Desktop."
echo "Run FK-GUI.exe, not the older FLOSSKEE.EXE."
