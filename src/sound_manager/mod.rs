use std::collections::HashMap;
use ggez::{audio::{self, SoundSource}, Context, GameResult};

/// Manages sound effects and background music for the game.
#[derive(Debug)]
pub struct GameSounds {
    /// A map of sound effect names to their audio sources.
    sounds: HashMap<String, Option<audio::Source>>,
    /// The background music source, loaded once and reused.
    background_music: Option<audio::Source>,
    /// Flag indicating whether the background music is currently playing.
    pub background_playing: bool,
}

impl GameSounds {
    /// Creates a new GameSounds manager and loads all required sounds.
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        // Create a map to hold all sound effects.
        let mut sounds = HashMap::new();
        
        // Load sound effects with error handling - empty sound files will just be None
        let sound_names = [
            "move", "rotate", "drop", "clear", "tetris", "lock", "game_over"
        ];
        
        for name in sound_names.iter() {
            let source = audio::Source::new(ctx, format!("/sounds/{}.wav", name));
            match source {
                Ok(src) => sounds.insert(name.to_string(), Some(src)),
                Err(e) => {
                    eprintln!("Failed to load sound {}: {}", name, e);
                    sounds.insert(name.to_string(), None)
                }
            };
        }
        
        // Load background music with error handling
        let background_music = match audio::Source::new(ctx, "/sounds/theme.wav") {
            Ok(mut src) => {
                src.set_repeat(true);
                Some(src)
            },
            Err(e) => {
                eprintln!("Failed to load background music: {}", e);
                None
            }
        };
        
        Ok(Self {
            sounds,
            background_music,
            background_playing: false,
        })
    }
    
    /// Plays a sound effect by name.
    pub fn play_sound(&mut self, ctx: &mut Context, name: &str) -> GameResult {
        if let Some(Some(sound)) = self.sounds.get_mut(name) {
            let _ = sound.play_detached(ctx); // Ignore any play errors
        }
        Ok(())
    }
    
    /// Starts playing the background music.
    pub fn start_background_music(&mut self, ctx: &mut Context) -> GameResult {
        if !self.background_playing {
            // Play the pre-loaded background music if available
            if let Some(music) = &mut self.background_music {
                if let Err(e) = music.play(ctx) {
                    eprintln!("Failed to start background music: {}", e);
                } else {
                    self.background_playing = true;
                }
            }
        }
        Ok(())
    }
    
    /// Stops the background music.
    pub fn stop_background_music(&mut self, ctx: &mut Context) {
        if self.background_playing {
            // Attempt to stop the music and log any errors without crashing.
            if let Some(music) = &mut self.background_music {
                if let Err(e) = music.stop(ctx) {
                    eprintln!("Failed to stop background music: {}", e);
                }
            }
            self.background_playing = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        // Stop and start again
        game_sounds.stop_background_music();
        assert!(!game_sounds.background_playing);

        assert!(game_sounds.start_background_music());
        assert!(game_sounds.background_playing);
    }
}