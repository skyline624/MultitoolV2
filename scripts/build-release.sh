#!/bin/bash
# Build script for MultitoolV2 on Linux
# Usage: ./scripts/build-release.sh [--type standard|portable|all] [--clean]

set -e

TYPE="standard"
CLEAN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --type)
            TYPE="$2"
            shift 2
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--type standard|portable|all] [--clean]"
            exit 1
            ;;
    esac
done

echo "========================================"
echo "MultitoolV2 - Linux Build Script"
echo "========================================"
echo ""

# Check prerequisites
echo "Checking prerequisites..."

command -v node >/dev/null 2>&1 || { echo "Error: Node.js is required but not installed."; exit 1; }
command -v pnpm >/dev/null 2>&1 || { echo "Error: pnpm is required but not installed. Run: npm install -g pnpm"; exit 1; }
command -v rustc >/dev/null 2>&1 || { echo "Error: Rust is required but not installed. Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: Cargo is required but not installed."; exit 1; }

echo "✓ Node.js: $(node --version)"
echo "✓ pnpm: $(pnpm --version)"
echo "✓ Rust: $(rustc --version)"
echo "✓ Cargo: $(cargo --version)"
echo ""

# Check Linux dependencies
echo "Checking Linux dependencies..."
MISSING_DEPS=""

if ! pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
    echo "⚠ webkit2gtk-4.1 not found"
    MISSING_DEPS="$MISSING_DEPS libwebkit2gtk-4.1-dev"
fi

if ! pkg-config --exists gtk+-3.0 2>/dev/null; then
    echo "⚠ gtk+-3.0 not found"
    MISSING_DEPS="$MISSING_DEPS libgtk-3-dev"
fi

if ! pkg-config --exists openssl 2>/dev/null; then
    echo "⚠ openssl not found"
    MISSING_DEPS="$MISSING_DEPS libssl-dev"
fi

if [ -n "$MISSING_DEPS" ]; then
    echo ""
    echo "Missing dependencies detected. Install with:"
    echo "sudo apt install build-essential libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo "Cleaning previous builds..."
    rm -rf src-tauri/target
    rm -rf dist
    echo "✓ Clean complete"
    echo ""
fi

# Install dependencies
echo "Installing dependencies..."
pnpm install

# Determine target architecture
TARGET=""
case "$(uname -m)" in
    x86_64)
        TARGET="x86_64-unknown-linux-gnu"
        ;;
    aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        ;;
    *)
        echo "Error: Unsupported architecture: $(uname -m)"
        exit 1
        ;;
esac

echo "Building for $TARGET..."
echo ""

# Build function
build_standard() {
    echo "Building standard release..."
    pnpm tauri build --target "$TARGET"

    # Create builds directory
    mkdir -p builds/linux

    # Copy artifacts
    if [ -d "src-tauri/target/$TARGET/release/bundle/deb" ]; then
        cp -r src-tauri/target/$TARGET/release/bundle/deb/* builds/linux/
        echo "✓ DEB package copied to builds/linux/"
    fi

    if [ -d "src-tauri/target/$TARGET/release/bundle/appimage" ]; then
        cp -r src-tauri/target/$TARGET/release/bundle/appimage/* builds/linux/
        echo "✓ AppImage copied to builds/linux/"
    fi
}

build_portable() {
    echo "Building portable release..."
    TAURI_ENV_PORTABLE="true" pnpm tauri build --target "$TARGET"

    # Copy portable binary
    mkdir -p builds/linux
    if [ -f "src-tauri/target/$TARGET/release/multitool" ]; then
        cp src-tauri/target/$TARGET/release/multitool builds/linux/MultitoolV2-Portable
        echo "✓ Portable binary copied to builds/linux/MultitoolV2-Portable"
    fi
}

# Execute build based on type
case $TYPE in
    standard)
        build_standard
        ;;
    portable)
        build_portable
        ;;
    all)
        build_standard
        build_portable
        ;;
    *)
        echo "Error: Unknown build type: $TYPE"
        echo "Valid types: standard, portable, all"
        exit 1
        ;;
esac

echo ""
echo "========================================"
echo "Build complete!"
echo "Output: builds/linux/"
echo "========================================"