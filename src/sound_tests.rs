use std::path::Path;

#[test]
fn test_sound_files_exist() {
    // Test that all sound files exist in the assets directory
    let sound_files = [
        "sounds/move.wav",
        "sounds/rotate.wav",
        "sounds/drop.wav",
        "sounds/clear.wav",
        "sounds/tetris.wav",
        "sounds/game_over.wav",
    ];

    for file in sound_files.iter() {
        let path = Path::new("assets").join(file);
        assert!(path.exists(), "Sound file {} not found", file);
    }
} 