use std::collections::HashMap;
use ggez::{audio::{self, SoundSource}, Context, GameResult};

/// Manages sound effects and background music for the game.
pub struct GameSounds {
    /// A map of sound effect names to their audio sources.
    sounds: HashMap<String, audio::Source>,
    /// The background music source, loaded once and reused.
    background_music: audio::Source,
    /// Flag indicating whether the background music is currently playing.
    pub background_playing: bool,
}

impl GameSounds {
    /// Creates a new `GameSounds` instance, loading all sound effects and background music.
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        // Use a HashMap to store sound effects for easy access and scalability.
        let mut sounds = HashMap::new();
        sounds.insert("move".to_string(), audio::Source::new(ctx, "/sounds/move.wav")?);
        sounds.insert("rotate".to_string(), audio::Source::new(ctx, "/sounds/rotate.wav")?);
        sounds.insert("drop".to_string(), audio::Source::new(ctx, "/sounds/drop.wav")?);
        sounds.insert("clear".to_string(), audio::Source::new(ctx, "/sounds/clear.wav")?);
        sounds.insert("tetris".to_string(), audio::Source::new(ctx, "/sounds/tetris.wav")?);
        sounds.insert("game_over".to_string(), audio::Source::new(ctx, "/sounds/game_over.wav")?);

        // Load the background music once and set it to repeat indefinitely.
        let mut background_music = audio::Source::new(ctx, "/sounds/background.wav")?;
        background_music.set_repeat(true);

        Ok(Self {
            sounds,
            background_music,
            background_playing: false,
        })
    }

    /// Plays a sound effect by name.
    /// 
    /// This method allows for a single function to handle all sound effects,
    /// reducing code duplication and making it easier to add new sounds.
    pub fn play_sound(&mut self, ctx: &mut Context, sound_name: &str) -> GameResult {
        if let Some(sound) = self.sounds.get_mut(sound_name) {
            // Play the sound detached so it doesn't block the main thread.
            sound.play_detached(ctx)?;
        }
        Ok(())
    }

    /// Starts the background music if it's not already playing.
    /// 
    /// Reuses the pre-loaded `background_music` source instead of reloading it,
    /// which improves performance by avoiding repeated file I/O operations.
    pub fn start_background_music(&mut self, ctx: &mut Context) -> GameResult {
        if !self.background_playing {
            // Play the pre-loaded background music.
            self.background_music.play(ctx)?;
            self.background_playing = true;
        }
        Ok(())
    }

    /// Stops the background music if it's currently playing.
    /// 
    /// Handles errors gracefully instead of panicking, improving robustness.
    pub fn stop_background_music(&mut self, ctx: &mut Context) {
        if self.background_playing {
            // Attempt to stop the music and log any errors without crashing.
            if let Err(e) = self.background_music.stop(ctx) {
                eprintln!("Failed to stop background music: {}", e);
            }
            self.background_playing = false;
        }
    }
}