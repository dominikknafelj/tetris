#!/bin/bash

echo "Cleaning up build artifacts..."

# Remove build directories
rm -rf target/
rm -rf TetrisApp.app/

# Remove generated files
rm -rf sounds/
rm -rf icons/
rm -f TetrisIcon.icns
rm -rf TetrisIcon.iconset/

# Remove distribution archives
rm -f Tetris-v*.zip

# Remove Python cache files
find . -type d -name "__pycache__" -exec rm -rf {} +
find . -type f -name "*.pyc" -delete

echo "Cleanup complete!" 