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
        
        println!("Note: Game will attempt to load sound files from resources/audio directory");
        
        for name in sound_names.iter() {
            let source = audio::Source::new(ctx, format!("/resources/audio/{}.wav", name));
            match source {
                Ok(src) => sounds.insert(name.to_string(), Some(src)),
                Err(e) => {
                    eprintln!("Failed to load sound {}: {}", name, e);
                    sounds.insert(name.to_string(), None)
                }
            };
        }
        
        // Load background music with error handling
        let background_music = match audio::Source::new(ctx, "/resources/audio/background.wav") {
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