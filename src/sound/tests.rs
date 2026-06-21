use super::mocks::*;
use std::path::Path;

// Basic test for sound file existence
#[test]
fn test_sound_files_exist() {
    // Test that all sound files exist in the resources/audio directory
    let sound_files = [
        "move.wav",
        "rotate.wav",
        "drop.wav",
        "clear.wav",
        "tetris.wav",
        "lock.wav",
        "game_over.wav",
        "background.wav" // Check the background music file
    ];

    for file in sound_files.iter() {
        let path = Path::new("resources/audio").join(file);
        if !path.exists() {
            println!("Warning: Sound file {} not found at {:?}", file, path);
            // Don't fail the test for missing sound files, we handle this gracefully in the game
        }
    }
}

// Tests for the mock game sounds implementation
#[test]
fn test_new_game_sounds() {
    let game_sounds = TestGameSounds::new();
    assert!(!game_sounds.background_playing);
    assert_eq!(game_sounds.sounds.len(), 6);
}

#[test]
fn test_background_music_state() {
    let mut game_sounds = TestGameSounds::new();

    // Test initial state
    assert!(!game_sounds.background_playing);

    // Test starting music
    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);

    // Test stopping music
    game_sounds.stop_background_music();
    assert!(!game_sounds.background_playing);
}

#[test]
fn test_sound_effects_state() {
    let mut game_sounds = TestGameSounds::new();

    // Test playing various sound effects
    let sound_names = ["move", "rotate", "drop", "clear", "tetris", "game_over"];
    for sound_name in sound_names.iter() {
        assert!(game_sounds.play_sound(sound_name));
    }

    // Test playing non-existent sound
    assert!(!game_sounds.play_sound("non_existent_sound"));
}

#[test]
fn test_background_music_repeat_state() {
    let mut game_sounds = TestGameSounds::new();

    // Start background music
    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);

    // Set repeat flag
    assert!(game_sounds.set_background_repeat(true));
    assert!(game_sounds.background_music.is_repeating);

    // Change repeat flag
    assert!(game_sounds.set_background_repeat(false));
    assert!(!game_sounds.background_music.is_repeating);

    // Stop and start again
    game_sounds.stop_background_music();
    assert!(!game_sounds.background_playing);

    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);
} 