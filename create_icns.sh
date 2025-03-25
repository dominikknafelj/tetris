#!/bin/bash

# Create iconset directory
mkdir -p TetrisIcon.iconset

# Copy and rename files according to Apple's iconset format
cp icons/icon_16x16.png TetrisIcon.iconset/icon_16x16.png
cp icons/icon_32x32.png TetrisIcon.iconset/icon_16x16@2x.png
cp icons/icon_32x32.png TetrisIcon.iconset/icon_32x32.png
cp icons/icon_64x64.png TetrisIcon.iconset/icon_32x32@2x.png
cp icons/icon_128x128.png TetrisIcon.iconset/icon_128x128.png
cp icons/icon_256x256.png TetrisIcon.iconset/icon_128x128@2x.png
cp icons/icon_256x256.png TetrisIcon.iconset/icon_256x256.png
cp icons/icon_512x512.png TetrisIcon.iconset/icon_256x256@2x.png
cp icons/icon_512x512.png TetrisIcon.iconset/icon_512x512.png
cp icons/icon_1024x1024.png TetrisIcon.iconset/icon_512x512@2x.png

# Create icns file
iconutil -c icns TetrisIcon.iconset

# Clean up
rm -rf TetrisIcon.iconset

# Move the icon to the app bundle
mv TetrisIcon.icns TetrisApp.app/Contents/Resources/ 