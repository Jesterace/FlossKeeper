# FlossKeeper

FlossKeeper is a small DMC floss collection tracker.

It is separate from FlossFinder. FlossFinder finds substitute colours. FlossKeeper tracks what floss you own.

## Features

- Track DMC colours
- Separate counts for bobbins and skeins
- Search by DMC number, colour name, or notes
- Filter by all, owned, missing, bobbins only, skeins only, low stock, or notes
- Summary counts for owned, missing, bobbins, skeins, and total floss units
- Add notes per colour
- Save collection to a plain text TSV file
- Export missing-colour shopping list
- Export missing-colours TSV report
- Export owned inventory CSV
- Linux and Windows build scripts included


## White / Blanc Note

DMC White and DMC Blanc are treated as the same colour and displayed as:

```text
Blanc / White
```

B5200 Snow White stays separate.

## Collection File

FlossKeeper saves your collection here on Linux:

```text
~/.config/flosskeeper/flosskeeper_collection.tsv
```

On Windows:

```text
%APPDATA%\FlossKeeper\flosskeeper_collection.tsv
```

The file format is plain text:

```text
# code    bobbins    skeins    notes
310       2          1         black
B5200     0          3         white skeins
```

## Run on Arch / EndeavourOS

```bash
sudo pacman -S --needed base-devel git rustup
rustup toolchain install stable
rustup default stable
./run_flosskeeper.sh
```

## Run on Linux Mint

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev libgl1-mesa-dev libfontconfig1-dev curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
./run_flosskeeper.sh
```

## Build Linux Release

```bash
./build_linux_release.sh
```

## Build Windows Release from Linux

On Arch / EndeavourOS:

```bash
sudo pacman -S --needed mingw-w64-gcc zip
./build_windows_on_linux.sh
```

## Android Later

The save file is intentionally simple so an Android version can reuse the same collection data later.

## License

MIT License.

DMC is a thread/floss brand owned by its respective owner. This project is not affiliated with or endorsed by DMC.

## Export Files

Export files are saved beside the collection file.

On Linux, that is usually:

```text
~/.config/flosskeeper/
```

On Windows, that is usually:

```text
%APPDATA%\FlossKeeper\
```

Export buttons create:

```text
flosskeeper_shopping_list.txt
flosskeeper_missing_colors.tsv
flosskeeper_inventory_export.csv
```

## Changes in v0.4

- Added Export shopping list.
- Added Export missing report.
- Added Export owned CSV.
- Export files save beside the main collection file.

## Changes in v0.3

- Added filter buttons: all, owned, missing, bobbins only, skeins only, low stock, and with notes.
- Search now checks DMC number, colour name, and notes.
- Added summary counts for missing colours, bobbins-only colours, skeins-only colours, colours with both, low stock, and notes.
- Added notes preview column to the main list.

## Changes in v0.2

- Added DMC colours 01 through 35.
- Made the colour list scrollable in both directions.
- Added a scroll hint above the list.
