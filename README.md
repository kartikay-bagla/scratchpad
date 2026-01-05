# Scratchpad

A minimal, borderless floating scratchpad for quick notes. Built with Rust and egui.

## Features

- Clean, minimal UI with no window decorations
- Autosaves to `~/.scratchpad.txt` every 3 seconds
- Draggable from the top title bar region
- Always-on-top toggle (pin button in top-left corner)
- Resizable from edges and corners
- Monospace font for easy reading

## Controls

- **Drag**: Click and drag the title bar to move the window
- **Pin (Always on Top)**: Click the circle button in the top-left corner to toggle always-on-top mode (filled = pinned)
- **Info**: Click the i button to view about dialog and GitHub link
- **Close**: Click the X button to save and exit
- **Resize**: Drag from any edge or corner

## Building

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- On Debian/Ubuntu, you may need:
  ```bash
  sudo apt install build-essential pkg-config libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
  ```

### Debug build

```bash
cargo build
cargo run
```

### Release build

```bash
cargo build --release
```

The optimized binary will be at `target/release/scratchpad`.

## Installation

### macOS

Build and install the app bundle:

```bash
# Install cargo-bundle (one-time)
cargo install cargo-bundle

# Build the app bundle
cargo bundle --release

# Copy to Applications
cp -r target/release/bundle/osx/Scratchpad.app /Applications/
```

#### Regenerating the app icon

If you modify `icon.png`, you'll need to regenerate the `.icns` file for macOS:

```bash
# Create iconset with required sizes
mkdir -p icon.iconset
sips -z 16 16 icon.png --out icon.iconset/icon_16x16.png
sips -z 32 32 icon.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32 icon.png --out icon.iconset/icon_32x32.png
sips -z 64 64 icon.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128 icon.png --out icon.iconset/icon_128x128.png
sips -z 256 256 icon.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256 icon.png --out icon.iconset/icon_256x256.png
sips -z 512 512 icon.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512 icon.png --out icon.iconset/icon_512x512.png
sips -z 1024 1024 icon.png --out icon.iconset/icon_512x512@2x.png

# Convert to .icns
iconutil -c icns icon.iconset -o icons/scratchpad.icns
rm -rf icon.iconset
```

The app will appear in Launchpad and can be launched from Finder.

**Note**: On first launch, macOS may show a security warning for unsigned apps. Right-click the app and select "Open" to bypass this.

### Linux

```bash
# Build release binary
cargo build --release

# Copy binary and icon
mkdir -p ~/.local/bin
cp target/release/scratchpad ~/.local/bin/
cp icon.png ~/.local/bin/scratchpad-icon.png

# Install desktop entry
cp scratchpad.desktop ~/.local/share/applications/
chmod +x ~/.local/share/applications/scratchpad.desktop
```

The app will appear in your application menu/launcher.

## Project Structure

```
scratchpad/
├── Cargo.toml           # Dependencies and project metadata
├── icon.png             # Application icon (source)
├── icons/
│   └── scratchpad.icns  # macOS app icon (generated from icon.png)
├── LICENSE              # GPL v3 license
├── README.md            # This file
├── scratchpad.desktop   # Desktop entry for Linux launchers
└── src/
    └── main.rs          # Application source code
```

## Data Storage

Your notes are stored in `~/.scratchpad.txt` and persist across sessions. The file is automatically created on first use.

## License

GPL v3 - See [LICENSE](LICENSE) file.
