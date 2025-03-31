# Tetris

A Rust implementation of the classic Tetris game using the ggez game framework.

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

## Project Structure

The project follows a modular organization:

```
tetris/
├── src/                  # Source code
│   ├── board/            # Game board implementation
│   ├── constants/        # Game constants
│   ├── score/            # Score tracking
│   ├── sound/            # Sound manager
│   ├── tetromino/        # Tetromino pieces
│   ├── ui/               # User interface rendering
│   ├── lib.rs            # Library exports
│   └── main.rs           # Entry point
├── resources/            # Game resources
│   ├── audio/            # Sound files
│   │   ├── background.wav  # Background music
│   │   ├── clear.wav       # Line clear sound
│   │   ├── drop.wav        # Piece drop sound
│   │   ├── game_over.wav   # Game over sound
│   │   ├── lock.wav        # Piece lock sound
│   │   ├── move.wav        # Piece movement sound
│   │   ├── rotate.wav      # Piece rotation sound
│   │   └── tetris.wav      # Tetris clear sound
│   └── images/           # Image files
│       └── icon_*.png      # Game icons
└── tests/                # Integration tests
```

## Resources Organization

All game resources are stored in the `resources` directory:

- **Audio files**: All sound effects and music are stored in `resources/audio/`
- **Image files**: All images and icons are stored in `resources/images/`

## Building and Running

```bash
# Build the project
cargo build

# Run the game
cargo run

# Run the tests
cargo test
```

## Sound Files

The game requires the following sound files:

- `background.wav` - Background music
- `clear.wav` - Sound when clearing lines
- `drop.wav` - Sound when dropping a piece
- `game_over.wav` - Sound when game over
- `lock.wav` - Sound when locking a piece
- `move.wav` - Sound when moving a piece
- `rotate.wav` - Sound when rotating a piece
- `tetris.wav` - Sound when clearing 4 lines (Tetris)

## Controls

- Left/Right Arrow: Move piece
- Up Arrow: Rotate piece
- Down Arrow: Soft drop
- Space: Hard drop
- M: Toggle music
- P: Pause game
- ESC: Quit game

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