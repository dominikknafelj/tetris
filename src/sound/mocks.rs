use std::collections::HashMap;

/// A mock audio source for testing sound functionality without ggez.
#[derive(Debug, Clone)]
pub struct MockAudioSource {
    /// Whether the audio is currently playing.
    pub is_playing: bool,
    /// Whether the audio is set to repeat.
    pub is_repeating: bool,
}

impl MockAudioSource {
    /// Creates a new mock audio source.
    pub fn new() -> Self {
        Self {
            is_playing: false,
            is_repeating: false,
        }
    }

    /// Simulates playing the audio.
    pub fn play(&mut self) -> bool {
        self.is_playing = true;
        true
    }

    /// Simulates stopping the audio.
    pub fn stop(&mut self) -> bool {
        self.is_playing = false;
        true
    }

    /// Simulates playing the audio detached (fire and forget).
    pub fn play_detached(&mut self) -> bool {
        self.is_playing = true;
        true
    }

    /// Simulates setting the repeat flag.
    pub fn set_repeat(&mut self, repeat: bool) -> bool {
        self.is_repeating = repeat;
        true
    }
}

/// A mock game sounds manager for testing.
#[derive(Debug)]
pub struct TestGameSounds {
    /// Map of sound names to mock sound sources.
    pub sounds: HashMap<String, MockAudioSource>,
    /// The background music source.
    pub background_music: MockAudioSource,
    /// Flag indicating whether background music is playing.
    pub background_playing: bool,
}

impl TestGameSounds {
    /// Creates a new test game sounds manager with mock sources.
    pub fn new() -> Self {
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

    /// Simulates playing a sound effect.
    pub fn play_sound(&mut self, sound_name: &str) -> bool {
        if let Some(sound) = self.sounds.get_mut(sound_name) {
            sound.play()
        } else {
            false // Non-existent sounds cause errors in our mock implementation
        }
    }

    /// Simulates starting the background music.
    pub fn start_background_music(&mut self) -> bool {
        if !self.background_playing {
            self.background_music.play();
            self.background_playing = true;
            true
        } else {
            false
        }
    }

    /// Simulates stopping the background music.
    pub fn stop_background_music(&mut self) {
        if self.background_playing {
            self.background_music.stop();
            self.background_playing = false;
        }
    }

    /// Simulates setting the background music repeat flag.
    pub fn set_background_repeat(&mut self, repeat: bool) -> bool {
        self.background_music.set_repeat(repeat)
    }
} 