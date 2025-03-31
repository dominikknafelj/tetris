// Integration tests for sound functionality
use std::collections::HashMap;

// Mock audio source for integration tests
#[derive(Debug, Clone)]
struct MockAudioSource {
    is_playing: bool,
    is_repeating: bool,
}

impl MockAudioSource {
    fn new() -> Self {
        Self {
            is_playing: false,
            is_repeating: false,
        }
    }

    fn play(&mut self) -> bool {
        self.is_playing = true;
        true
    }

    fn stop(&mut self) -> bool {
        self.is_playing = false;
        true
    }

    fn play_detached(&mut self) -> bool {
        self.is_playing = true;
        true
    }

    fn set_repeat(&mut self, repeat: bool) -> bool {
        self.is_repeating = repeat;
        true
    }
}

// Mock game sounds for integration tests
struct TestGameSounds {
    sounds: HashMap<String, MockAudioSource>,
    background_music: MockAudioSource,
    background_playing: bool,
}

impl TestGameSounds {
    fn new() -> Self {
        let mut sounds = HashMap::new();
        let sound_names = ["move", "rotate", "drop", "clear", "tetris", "game_over"];
        
        for name in sound_names.iter() {
            sounds.insert(name.to_string(), MockAudioSource::new());
        }
        
        Self {
            sounds,
            background_music: MockAudioSource::new(),
            background_playing: false,
        }
    }

    fn play_sound(&mut self, sound_name: &str) -> bool {
        if let Some(sound) = self.sounds.get_mut(sound_name) {
            sound.play()
        } else {
            false
        }
    }

    fn start_background_music(&mut self) -> bool {
        if !self.background_playing {
            self.background_music.play();
            self.background_playing = true;
            true
        } else {
            false
        }
    }

    fn stop_background_music(&mut self) {
        if self.background_playing {
            self.background_music.stop();
            self.background_playing = false;
        }
    }

    fn set_background_repeat(&mut self, repeat: bool) -> bool {
        self.background_music.set_repeat(repeat)
    }
}

// Test creation of game sounds
#[test]
fn test_sound_loading() {
    let game_sounds = TestGameSounds::new();
    
    // Verify that all expected sounds are loaded
    let expected_sounds = ["move", "rotate", "drop", "clear", "tetris", "game_over"];
    for sound_name in expected_sounds.iter() {
        assert!(game_sounds.sounds.contains_key(*sound_name), "Sound {} should be loaded", sound_name);
    }
}

// Test background music functionality
#[test]
fn test_background_music_integration() {
    let mut game_sounds = TestGameSounds::new();
    
    // Test initial state
    assert!(!game_sounds.background_playing);
    
    // Start music
    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);
    
    // Stop music
    game_sounds.stop_background_music();
    assert!(!game_sounds.background_playing);
    
    // Start again
    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);
    
    // Set repeat
    assert!(game_sounds.set_background_repeat(true));
    assert!(game_sounds.background_music.is_repeating);
}

// Test sound effects functionality
#[test]
fn test_sound_effects_integration() {
    let mut game_sounds = TestGameSounds::new();
    
    // Test playing multiple sounds in sequence
    let sound_sequence = ["move", "rotate", "drop", "clear"];
    for sound_name in sound_sequence.iter() {
        assert!(game_sounds.play_sound(sound_name));
        assert!(game_sounds.sounds.get(*sound_name).unwrap().is_playing);
    }
    
    // Test playing a non-existent sound
    assert!(!game_sounds.play_sound("non_existent_sound"));
}

// Test interaction between music and sound effects
#[test]
fn test_sound_and_music_interaction() {
    let mut game_sounds = TestGameSounds::new();
    
    // Start background music
    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);
    
    // Play sound effects while music is playing
    let sound_effects = ["move", "rotate", "drop"];
    for sound_name in sound_effects.iter() {
        assert!(game_sounds.play_sound(sound_name));
        assert!(game_sounds.sounds.get(*sound_name).unwrap().is_playing);
    }
    
    // Music should still be playing
    assert!(game_sounds.background_playing);
    assert!(game_sounds.background_music.is_playing);
    
    // Stop music
    game_sounds.stop_background_music();
    assert!(!game_sounds.background_playing);
    assert!(!game_sounds.background_music.is_playing);
    
    // Play more sound effects after music is stopped
    assert!(game_sounds.play_sound("clear"));
    assert!(game_sounds.sounds.get("clear").unwrap().is_playing);
} 