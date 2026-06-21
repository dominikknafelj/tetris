use std::fs::{self, File};
use std::io::{self, Write};
use serde::{Serialize, Deserialize};
use crate::constants::*;

/// High score entry with player name and score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighScoreEntry {
    pub name: String,
    pub score: u32,
}

/// Collection of high scores that can be loaded/saved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighScores {
    entries: Vec<HighScoreEntry>,
}

impl HighScores {
    /// Create a new empty high score list
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    /// Load high scores from file, with better error handling
    pub fn load() -> Self {
        fs::read_to_string(HIGH_SCORES_FILE)
            .map_err(|e| {
                log::warn!("Failed to read high scores file: {}", e);
                e
            })
            .and_then(|contents| {
                serde_json::from_str(&contents).map_err(|e| {
                    log::warn!("Failed to parse high scores: {}", e);
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })
            })
            .unwrap_or_else(|_| Self::new())
    }
    
    /// Save high scores to file with improved error handling
    pub fn save(&self) -> io::Result<()> {
        let json = serde_json::to_string(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut file = File::create(HIGH_SCORES_FILE)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    /// Add a new high score if it qualifies, return true if it was added
    pub fn add_score(&mut self, name: String, score: u32) -> bool {
        // Check if the score qualifies
        if !self.would_qualify(score) {
            return false;
        }
        
        // Add the new entry
        self.entries.push(HighScoreEntry { name, score });
        
        // Sort entries by score (descending)
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Truncate to max number of entries
        if self.entries.len() > MAX_HIGH_SCORES {
            self.entries.truncate(MAX_HIGH_SCORES);
        }
        
        // Save the updated high scores
        if let Err(e) = self.save() {
            log::error!("Failed to save high scores: {}", e);
        }
        
        true
    }
    
    /// Check if a score would qualify for the high score list
    pub fn would_qualify(&self, score: u32) -> bool {
        self.entries.len() < MAX_HIGH_SCORES || 
        self.entries.iter().any(|entry| entry.score < score)
    }
    
    /// Get a reference to the entries
    pub fn entries(&self) -> &[HighScoreEntry] {
        &self.entries
    }
}

/// Helper function to format a score with commas
pub fn format_score(score: u32) -> String {
    let score_str = score.to_string();
    let mut result = String::new();
    let len = score_str.len();
    
    for (i, c) in score_str.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_high_scores() {
        let mut high_scores = HighScores::new();
        
        // Test adding scores when list is not full
        assert!(high_scores.add_score("Player1".to_string(), 1000));
        assert!(high_scores.add_score("Player2".to_string(), 500));
        assert!(high_scores.add_score("Player3".to_string(), 750));
        
        // Test scores are sorted correctly
        assert_eq!(high_scores.entries()[0].score, 1000);
        assert_eq!(high_scores.entries()[1].score, 750);
        assert_eq!(high_scores.entries()[2].score, 500);
        
        // Test would_qualify function with non-full list
        assert!(high_scores.would_qualify(400));
        
        // Fill up the high scores list
        for i in 0..MAX_HIGH_SCORES {
            high_scores.add_score(format!("Player{}", i), (1000 + i) as u32);
        }
        
        // Test would_qualify function with full list
        assert!(high_scores.would_qualify(1500));
        assert!(!high_scores.would_qualify(500));
        
        // Test maximum number of scores
        assert_eq!(high_scores.entries().len(), MAX_HIGH_SCORES);
    }
    
    #[test]
    fn test_format_score() {
        assert_eq!(format_score(1000), "1,000");
        assert_eq!(format_score(1000000), "1,000,000");
        assert_eq!(format_score(123), "123");
    }
} 