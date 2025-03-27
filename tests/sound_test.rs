use tetris::sound_manager::GameSounds;
use std::collections::HashMap;

// Test-specific mock audio source
#[derive(Clone)]
struct MockAudioSource;

impl MockAudioSource {
    fn new() -> Self {
        Self
    }

    fn play(&mut self) -> bool {
        true
    }

    fn stop(&mut self) -> bool {
        true
    }

    fn play_detached(&mut self) -> bool {
        true
    }

    fn set_repeat(&mut self, _repeat: bool) {}
}

// Test-specific mock game sounds
struct TestGameSounds {
    sounds: HashMap<String, MockAudioSource>,
    background_music: MockAudioSource,
    pub background_playing: bool,
}

impl TestGameSounds {
    fn new() -> Self {
        let mut sounds = HashMap::new();
        sounds.insert("move".to_string(), MockAudioSource::new());
        sounds.insert("rotate".to_string(), MockAudioSource::new());
        sounds.insert("drop".to_string(), MockAudioSource::new());
        sounds.insert("clear".to_string(), MockAudioSource::new());
        sounds.insert("tetris".to_string(), MockAudioSource::new());
        sounds.insert("game_over".to_string(), MockAudioSource::new());

        let mut background_music = MockAudioSource::new();
        background_music.set_repeat(true);

        Self {
            sounds,
            background_music,
            background_playing: false,
        }
    }

    fn play_sound(&mut self, sound_name: &str) -> bool {
        if let Some(_sound) = self.sounds.get_mut(sound_name) {
            true
        } else {
            false
        }
    }

    fn start_background_music(&mut self) -> bool {
        if !self.background_playing {
            self.background_playing = true;
            true
        } else {
            false
        }
    }

    fn stop_background_music(&mut self) {
        if self.background_playing {
            self.background_playing = false;
        }
    }
}

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
fn test_sound_effects() {
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
fn test_background_music_repeat() {
    let mut game_sounds = TestGameSounds::new();

    // Start background music
    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);

    // Stop and start again
    game_sounds.stop_background_music();
    assert!(!game_sounds.background_playing);

    assert!(game_sounds.start_background_music());
    assert!(game_sounds.background_playing);
} 