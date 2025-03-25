# Tetris

A classic Tetris game implemented in Rust using the GGEZ game engine.

## Features

- Classic Tetris gameplay
- Sound effects and background music
- Modern UI with smooth animations
- Native macOS application bundle
- High-quality sound effects
- Custom application icon

## Requirements

- Rust (latest stable version)
- Python 3.x (for asset generation)
- macOS 10.10 or later

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/tetris.git
cd tetris
```

2. Install Python dependencies:
```bash
pip install pillow scipy
```

## Building

### Development Build
```bash
cargo run
```

### Release Build (macOS)
```bash
./build_mac.sh
```

Options:
- `--clean`: Clean build artifacts before building
- `--help`: Show help message

### Cleaning Build Artifacts
```bash
./clean.sh
```

## Controls

- Left/Right Arrow: Move piece
- Up Arrow: Rotate piece
- Down Arrow: Soft drop
- Space: Hard drop
- M: Toggle music
- P: Pause game
- ESC: Quit game

## Project Structure

```
tetris/
├── src/
│   ├── main.rs          # Main game logic
│   ├── tetromino.rs     # Tetromino piece implementation
│   └── sound_tests.rs   # Sound system tests
├── sounds/              # Generated sound effects
├── icons/              # Generated application icons
├── build_mac.sh        # macOS build script
├── clean.sh           # Cleanup script
├── generate_sounds.py # Sound generation script
├── generate_icon.py   # Icon generation script
└── create_icns.sh     # macOS icon creation script
```

## Version History

- v1.0.0 (2024-03-24)
  - Initial release
  - Complete Tetris implementation
  - Sound effects and music
  - macOS application bundle

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- GGEZ game engine
- Rust programming language
- Classic Tetris game design 