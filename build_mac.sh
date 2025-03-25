#!/bin/bash

# Exit on error
set -e

# Function to show usage
show_usage() {
    echo "Usage: $0 [--clean] [--help]"
    echo "  --clean    Clean build artifacts before building"
    echo "  --help     Show this help message"
    exit 1
}

# Get version from VERSION file
VERSION=$(cat VERSION)
if [ -z "$VERSION" ]; then
    echo "Error: VERSION file is empty or not found"
    exit 1
fi

# Parse command line arguments
CLEAN=false
for arg in "$@"; do
    case $arg in
        --clean)
            CLEAN=true
            ;;
        --help|-h)
            show_usage
            ;;
        *)
            echo "Unknown argument: $arg"
            show_usage
            ;;
    esac
done

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo "Cleaning previous build artifacts..."
    ./clean.sh
fi

echo "Building Tetris v${VERSION} for macOS..."

# Check for required Python packages
echo "Checking Python dependencies..."
python3 -c "import PIL" 2>/dev/null || { echo "Pillow not found. Installing..."; pip install pillow; }
python3 -c "import scipy" 2>/dev/null || { echo "scipy not found. Installing..."; pip install scipy; }

# Build the release version
echo "Building release version..."
cargo build --release

# Create the app bundle structure
echo "Creating app bundle structure..."
mkdir -p TetrisApp.app/Contents/{MacOS,Resources}

# Copy the binary
echo "Copying binary..."
cp target/release/tetris TetrisApp.app/Contents/MacOS/

# Generate sound files
echo "Generating sound files..."
python3 generate_sounds.py

# Copy sound files to Resources
echo "Copying sound files..."
cp -r sounds/ TetrisApp.app/Contents/Resources/

# Generate icons
echo "Generating icons..."
python3 generate_icon.py

# Create ICNS file
echo "Creating ICNS file..."
./create_icns.sh

# Create Info.plist
echo "Creating Info.plist..."
cat > TetrisApp.app/Contents/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>tetris</string>
    <key>CFBundleIconFile</key>
    <string>TetrisIcon</string>
    <key>CFBundleIdentifier</key>
    <string>com.rust.tetris</string>
    <key>CFBundleName</key>
    <string>Tetris</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.10</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
        <string>MacOSX</string>
    </array>
</dict>
</plist>
EOF

# Create a zip archive of the app
echo "Creating distribution archive..."
zip -r "Tetris-v${VERSION}.zip" TetrisApp.app

echo "Build complete! Tetris v${VERSION} is ready."
echo "You can find the distribution archive at: Tetris-v${VERSION}.zip"
echo "You can now run the game by double-clicking TetrisApp.app in Finder." 